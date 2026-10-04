use crate::engine::tweak::*;
use crate::hardware::models::{ChassisType, PowerState};
use crate::safety::rollback::{OptimizationBackup, RollbackManager};
use std::process::Command;
use winreg::enums::{HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

pub fn get_performance_tweaks() -> Vec<Tweak> {
    vec![
        Tweak {
            id: "optimize_storage_trim",
            name: "Hardware-Aware Storage: Enforce TRIM for NVMe/SSD",
            description: "Verifies and enables TRIM command signaling to maximize flash storage endurance and read/write speeds.",
            category: TweakCategory::Performance,
            risk_level: RiskLevel::Safe,
            is_applicable: |p| p.has_fast_storage(),
            check_status: || {
                let out = Command::new("fsutil")
                    .args(["behavior", "query", "DisableDeleteNotify"])
                    .output();
                if let Ok(o) = out {
                    let s = String::from_utf8_lossy(&o.stdout);
                    return s.contains("= 0");
                }
                false
            },
            apply: |_rb, _session_id| {
                let _ = Command::new("fsutil")
                    .args(["behavior", "set", "DisableDeleteNotify", "0"])
                    .output();
                Ok(())
            },
            revert: |_rb| Ok(()),
        },
        Tweak {
            id: "hardware_aware_sysmain",
            name: "Hardware-Aware SysMain (SuperFetch) Tuning",
            description: "Adapts SysMain based on storage: sets to Manual on ultra-fast NVMe/SSD drives to reduce NAND write wear; keeps active on rotational HDDs.",
            category: TweakCategory::Performance,
            risk_level: RiskLevel::Safe,
            is_applicable: |p| p.has_fast_storage(),
            check_status: || {
                let out = Command::new("powershell")
                    .args(["-NoProfile", "-Command", "(Get-Service -Name 'SysMain' -ErrorAction SilentlyContinue).StartType"])
                    .output();
                if let Ok(o) = out {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    return s == "Manual" || s == "Disabled";
                }
                false
            },
            apply: |rb, session_id| {
                let svc_snap = RollbackManager::snapshot_service("SysMain");

                // Set SysMain to manual on SSDs
                let _ = Command::new("sc.exe").args(["config", "SysMain", "start=demand"]).output();

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "hardware_aware_sysmain".to_string(),
                    description: "Hardware-Aware SysMain Tuning".to_string(),
                    registry_snapshots: vec![],
                    service_snapshots: vec![svc_snap],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "hardware_aware_sysmain") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "hardware_aware_cpu_priority",
            name: "Hardware-Aware CPU Win32PrioritySeparation",
            description: "Optimizes foreground thread quantum for maximum responsiveness on Desktop PCs while maintaining balanced thermals on Laptops.",
            category: TweakCategory::Performance,
            risk_level: RiskLevel::Safe,
            is_applicable: |p| p.chassis_type == ChassisType::Desktop || p.power_state == PowerState::AcPower,
            check_status: || {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                if let Ok(key) = hklm.open_subkey_with_flags("SYSTEM\\CurrentControlSet\\Control\\PriorityControl", KEY_READ) {
                    if let Ok(val) = key.get_value::<u32, _>("Win32PrioritySeparation") {
                        return val == 0x26;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "SYSTEM\\CurrentControlSet\\Control\\PriorityControl";
                let snap = RollbackManager::snapshot_registry_value("HKLM", subkey, "Win32PrioritySeparation");

                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let (key, _) = hklm.create_subkey(subkey).map_err(|e| e.to_string())?;
                // 0x26 = 38 decimal (short, variable foreground priority boost)
                key.set_value("Win32PrioritySeparation", &0x26u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "hardware_aware_cpu_priority".to_string(),
                    description: "Hardware-Aware CPU Win32PrioritySeparation".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "hardware_aware_cpu_priority") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_delivery_optimization_p2p",
            name: "Disable Windows Update Public P2P Upload",
            description: "Restricts Windows Delivery Optimization to LAN only, preventing unwanted bandwidth and CPU usage uploading updates to strangers over the Internet.",
            category: TweakCategory::Performance,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                if let Ok(key) = hklm.open_subkey_with_flags("SOFTWARE\\Policies\\Microsoft\\Windows\\DeliveryOptimization", KEY_READ) {
                    if let Ok(val) = key.get_value::<u32, _>("DODownloadMode") {
                        return val == 1; // 1 = LAN only (no internet upload)
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "SOFTWARE\\Policies\\Microsoft\\Windows\\DeliveryOptimization";
                let snap = RollbackManager::snapshot_registry_value("HKLM", subkey, "DODownloadMode");

                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let (key, _) = hklm.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("DODownloadMode", &1u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_delivery_optimization_p2p".to_string(),
                    description: "Disable Windows Update Public P2P Upload".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_delivery_optimization_p2p") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_ntfs_last_access",
            name: "Disable NTFS Last Access Time Updates",
            description: "Reduces disk write I/O cycles by stopping NTFS from recording timestamps every time a file is accessed.",
            category: TweakCategory::Performance,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let out = Command::new("fsutil")
                    .args(["behavior", "query", "disablelastaccess"])
                    .output();
                if let Ok(o) = out {
                    let s = String::from_utf8_lossy(&o.stdout);
                    return s.contains("= 1") || s.contains("= 2");
                }
                false
            },
            apply: |_rb, _session_id| {
                let _ = Command::new("fsutil")
                    .args(["behavior", "set", "disablelastaccess", "1"])
                    .output();
                Ok(())
            },
            revert: |_rb| {
                let _ = Command::new("fsutil")
                    .args(["behavior", "set", "disablelastaccess", "0"])
                    .output();
                Ok(())
            },
        },
    ]
}
