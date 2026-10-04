use crate::hardware::models::*;
use std::process::Command;
use sysinfo::{CpuRefreshKind, Disks, MemoryRefreshKind, RefreshKind, System};
use winreg::enums::HKEY_LOCAL_MACHINE;
use winreg::RegKey;

pub struct HardwareProfiler;

impl HardwareProfiler {
    pub fn collect_profile() -> HardwareProfile {
        let mut sys = System::new_with_specifics(
            RefreshKind::new()
                .with_cpu(CpuRefreshKind::everything())
                .with_memory(MemoryRefreshKind::everything()),
        );
        sys.refresh_all();

        let os = Self::detect_os();
        let cpu = Self::detect_cpu(&sys);
        let ram = Self::detect_ram(&sys);
        let drives = Self::detect_drives();
        let (chassis_type, power_state) = Self::detect_chassis_and_power();

        HardwareProfile {
            chassis_type,
            power_state,
            cpu,
            ram,
            drives,
            os,
        }
    }

    fn detect_os() -> OsInfo {
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        let cv_key = hklm.open_subkey("SOFTWARE\\Microsoft\\Windows NT\\CurrentVersion");

        let (product_name, current_build, display_version) = if let Ok(key) = cv_key {
            let p_name: String = key.get_value("ProductName").unwrap_or_else(|_| "Windows".into());
            let c_build: String = key.get_value("CurrentBuild").unwrap_or_else(|_| "0".into());
            let d_ver: String = key.get_value("DisplayVersion").unwrap_or_else(|_| "".into());
            (p_name, c_build, d_ver)
        } else {
            ("Windows".into(), "0".into(), "".into())
        };

        let build_num: u32 = current_build.parse().unwrap_or(0);
        let is_windows_11 = build_num >= 22000;
        let effective_name = if is_windows_11 && product_name.contains("Windows 10") {
            product_name.replace("Windows 10", "Windows 11")
        } else {
            product_name
        };

        OsInfo {
            name: effective_name,
            build: current_build,
            version: display_version,
            arch: std::env::consts::ARCH.to_string(),
            is_windows_11,
        }
    }

    fn detect_cpu(sys: &System) -> CpuInfo {
        let cpus = sys.cpus();
        let name = cpus.first().map(|c| c.brand().trim().to_string()).unwrap_or_else(|| "Unknown CPU".into());
        let vendor = cpus.first().map(|c| c.vendor_id().trim().to_string()).unwrap_or_else(|| "Unknown".into());
        let logical_cores = cpus.len();
        let physical_cores = sys.physical_core_count().unwrap_or(logical_cores);
        let base_frequency_mhz = cpus.first().map(|c| c.frequency()).unwrap_or(0);

        // Check if Intel hybrid architecture or AMD 3D V-Cache / big.LITTLE
        let is_hybrid = (name.contains("Intel") && (name.contains("12th") || name.contains("13th") || name.contains("14th") || name.contains("Core Ultra") || name.contains("i7-12") || name.contains("i9-12") || name.contains("i7-13") || name.contains("i9-13") || name.contains("i7-14") || name.contains("i9-14")))
            || (physical_cores > 0 && logical_cores > physical_cores && logical_cores != physical_cores * 2);

        CpuInfo {
            name,
            vendor,
            physical_cores,
            logical_cores,
            base_frequency_mhz,
            is_hybrid,
        }
    }

    fn detect_ram(sys: &System) -> RamInfo {
        let total_bytes = sys.total_memory();
        let available_bytes = sys.available_memory();
        let total_mb = total_bytes / (1024 * 1024);
        let available_mb = available_bytes / (1024 * 1024);

        let category = if total_mb < 8192 {
            RamCategory::Low
        } else if total_mb <= 32768 {
            RamCategory::Standard
        } else {
            RamCategory::High
        };

        RamInfo {
            total_mb,
            available_mb,
            category,
        }
    }

    fn detect_drives() -> Vec<DiskDriveInfo> {
        let disks = Disks::new_with_refreshed_list();
        let mut results = Vec::new();
        let trim_enabled = Self::check_trim_status();

        for disk in disks.list() {
            let mount_point = disk.mount_point().to_string_lossy().to_string();
            let is_system_drive = mount_point.starts_with("C:") || mount_point.starts_with("c:");
            let total_gb = disk.total_space() / (1024 * 1024 * 1024);
            let free_gb = disk.available_space() / (1024 * 1024 * 1024);
            let name = disk.name().to_string_lossy().to_string();

            // Detect storage type using medium heuristics and sysinfo disk kind
            let storage_type = match disk.kind() {
                sysinfo::DiskKind::SSD => {
                    if name.to_uppercase().contains("NVME") || disk.file_system().to_string_lossy().contains("NVMe") {
                        StorageType::Nvme
                    } else {
                        StorageType::Ssd
                    }
                }
                sysinfo::DiskKind::HDD => StorageType::Hdd,
                _ => {
                    if name.to_uppercase().contains("NVME") {
                        StorageType::Nvme
                    } else if name.to_uppercase().contains("SSD") {
                        StorageType::Ssd
                    } else {
                        StorageType::Ssd // Default modern expectation, verified non-destructive
                    }
                }
            };

            results.push(DiskDriveInfo {
                drive_letter: mount_point,
                label: name,
                total_gb,
                free_gb,
                storage_type,
                is_system_drive,
                trim_enabled,
            });
        }

        results
    }

    fn check_trim_status() -> bool {
        // Query TRIM using fsutil
        let output = Command::new("fsutil")
            .args(["behavior", "query", "DisableDeleteNotify"])
            .output();

        if let Ok(out) = output {
            let text = String::from_utf8_lossy(&out.stdout);
            // If DisableDeleteNotify = 0, TRIM is enabled
            text.contains("= 0")
        } else {
            true
        }
    }

    fn detect_chassis_and_power() -> (ChassisType, PowerState) {
        // Detect battery presence via WMI / PowerShell
        let script = r#"
        $battery = Get-CimInstance -ClassName Win32_Battery -ErrorAction SilentlyContinue
        if ($battery) {
            $status = $battery.BatteryStatus
            if ($status -eq 1) { "Battery|Discharging" } else { "Battery|AC" }
        } else {
            "Desktop|AC"
        }
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-NonInteractive", "-Command", script])
            .output();

        if let Ok(out) = output {
            let res = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if res.contains("Battery|Discharging") {
                (ChassisType::Laptop, PowerState::Battery)
            } else if res.contains("Battery|AC") {
                (ChassisType::Laptop, PowerState::AcPower)
            } else {
                (ChassisType::Desktop, PowerState::AcPower)
            }
        } else {
            (ChassisType::Desktop, PowerState::AcPower)
        }
    }
}
