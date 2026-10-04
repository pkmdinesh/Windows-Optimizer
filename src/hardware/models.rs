use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum ChassisType {
    Desktop,
    Laptop,
    Tablet,
    VirtualMachine,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum StorageType {
    Nvme,
    Ssd,
    Hdd,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum PowerState {
    AcPower,
    Battery,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub enum RamCategory {
    Low,      // < 8 GB
    Standard, // 8 - 32 GB
    High,     // > 32 GB
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CpuInfo {
    pub name: String,
    pub vendor: String,
    pub physical_cores: usize,
    pub logical_cores: usize,
    pub base_frequency_mhz: u64,
    pub is_hybrid: bool, // Heterogeneous P/E cores
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RamInfo {
    pub total_mb: u64,
    pub available_mb: u64,
    pub category: RamCategory,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiskDriveInfo {
    pub drive_letter: String,
    pub label: String,
    pub total_gb: u64,
    pub free_gb: u64,
    pub storage_type: StorageType,
    pub is_system_drive: bool,
    pub trim_enabled: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OsInfo {
    pub name: String,
    pub build: String,
    pub version: String,
    pub arch: String,
    pub is_windows_11: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HardwareProfile {
    pub chassis_type: ChassisType,
    pub power_state: PowerState,
    pub cpu: CpuInfo,
    pub ram: RamInfo,
    pub drives: Vec<DiskDriveInfo>,
    pub os: OsInfo,
}

impl HardwareProfile {
    pub fn is_laptop_on_battery(&self) -> bool {
        self.chassis_type == ChassisType::Laptop && self.power_state == PowerState::Battery
    }

    #[allow(dead_code)]
    pub fn primary_system_drive(&self) -> Option<&DiskDriveInfo> {
        self.drives.iter().find(|d| d.is_system_drive)
    }

    pub fn has_fast_storage(&self) -> bool {
        self.drives.iter().any(|d| d.is_system_drive && (d.storage_type == StorageType::Nvme || d.storage_type == StorageType::Ssd))
    }
}
