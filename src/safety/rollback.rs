use colored::*;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;
use std::process::Command;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE};
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegValueData {
    Dword(u32),
    String(String),
    None,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegistrySnapshot {
    pub hive: String, // "HKLM" or "HKCU"
    pub subkey: String,
    pub name: String,
    pub existed: bool,
    pub original_data: RegValueData,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceSnapshot {
    pub service_name: String,
    pub previous_start_type: String, // "Automatic", "Manual", "Disabled"
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OptimizationBackup {
    pub session_id: String,
    pub timestamp: String,
    pub tweak_id: String,
    pub description: String,
    pub registry_snapshots: Vec<RegistrySnapshot>,
    pub service_snapshots: Vec<ServiceSnapshot>,
}

pub struct RollbackManager {
    backup_dir: PathBuf,
}

impl RollbackManager {
    pub fn new() -> Self {
        let dir = PathBuf::from("backups");
        if !dir.exists() {
            let _ = fs::create_dir_all(&dir);
        }
        RollbackManager { backup_dir: dir }
    }

    /// Snapshot a registry value prior to changing it
    pub fn snapshot_registry_value(
        hive_str: &str,
        subkey_str: &str,
        value_name: &str,
    ) -> RegistrySnapshot {
        let hive = if hive_str == "HKLM" {
            RegKey::predef(HKEY_LOCAL_MACHINE)
        } else {
            RegKey::predef(HKEY_CURRENT_USER)
        };

        if let Ok(key) = hive.open_subkey(subkey_str) {
            if let Ok(val) = key.get_value::<u32, _>(value_name) {
                return RegistrySnapshot {
                    hive: hive_str.to_string(),
                    subkey: subkey_str.to_string(),
                    name: value_name.to_string(),
                    existed: true,
                    original_data: RegValueData::Dword(val),
                };
            } else if let Ok(val) = key.get_value::<String, _>(value_name) {
                return RegistrySnapshot {
                    hive: hive_str.to_string(),
                    subkey: subkey_str.to_string(),
                    name: value_name.to_string(),
                    existed: true,
                    original_data: RegValueData::String(val),
                };
            }
        }

        // If subkey or value did not exist
        RegistrySnapshot {
            hive: hive_str.to_string(),
            subkey: subkey_str.to_string(),
            name: value_name.to_string(),
            existed: false,
            original_data: RegValueData::None,
        }
    }

    /// Snapshot a Windows service's current start type
    pub fn snapshot_service(service_name: &str) -> ServiceSnapshot {
        let cmd = format!("(Get-Service -Name '{}' -ErrorAction SilentlyContinue).StartType", service_name);
        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", &cmd])
            .output();

        let start_type = if let Ok(out) = output {
            let s = String::from_utf8_lossy(&out.stdout).trim().to_string();
            if s.is_empty() { "Manual".to_string() } else { s }
        } else {
            "Manual".to_string()
        };

        ServiceSnapshot {
            service_name: service_name.to_string(),
            previous_start_type: start_type,
        }
    }

    /// Persist backup to disk
    pub fn save_backup(&self, backup: &OptimizationBackup) -> Result<(), String> {
        let file_name = format!("backup_{}_{}.json", backup.session_id, backup.tweak_id);
        let path = self.backup_dir.join(file_name);
        let json = serde_json::to_string_pretty(backup).map_err(|e| e.to_string())?;

        let mut f = File::create(&path).map_err(|e| e.to_string())?;
        f.write_all(json.as_bytes()).map_err(|e| e.to_string())?;
        Ok(())
    }

    /// List all existing backups
    pub fn list_backups(&self) -> Vec<OptimizationBackup> {
        let mut backups = Vec::new();
        if let Ok(entries) = fs::read_dir(&self.backup_dir) {
            for entry in entries.flatten() {
                let path = entry.path();
                if path.extension().and_then(|s| s.to_str()) == Some("json") {
                    if let Ok(mut file) = File::open(&path) {
                        let mut contents = String::new();
                        if file.read_to_string(&mut contents).is_ok() {
                            if let Ok(backup) = serde_json::from_str::<OptimizationBackup>(&contents) {
                                backups.push(backup);
                            }
                        }
                    }
                }
            }
        }
        // Sort descending by timestamp
        backups.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
        backups
    }

    /// Rollback a specific backup session
    pub fn rollback_backup(&self, backup: &OptimizationBackup) -> Result<(), String> {
        println!("{}", format!("==> Restoring state for tweak: {}", backup.description).yellow());

        // Revert Registry changes
        for reg in &backup.registry_snapshots {
            let hive = if reg.hive == "HKLM" {
                RegKey::predef(HKEY_LOCAL_MACHINE)
            } else {
                RegKey::predef(HKEY_CURRENT_USER)
            };

            if reg.existed {
                let (key, _) = hive.create_subkey(&reg.subkey).map_err(|e| e.to_string())?;
                match &reg.original_data {
                    RegValueData::Dword(val) => {
                        key.set_value(&reg.name, val).map_err(|e| e.to_string())?;
                        println!("    [✓] Restored {}:{} = {}", reg.subkey, reg.name, val);
                    }
                    RegValueData::String(val) => {
                        key.set_value(&reg.name, val).map_err(|e| e.to_string())?;
                        println!("    [✓] Restored {}:{} = \"{}\"", reg.subkey, reg.name, val);
                    }
                    RegValueData::None => {}
                }
            } else {
                // If it didn't exist before, delete the added value
                if let Ok(key) = hive.open_subkey_with_flags(&reg.subkey, winreg::enums::KEY_WRITE) {
                    let _ = key.delete_value(&reg.name);
                    println!("    [✓] Removed injected value {}:{}", reg.subkey, reg.name);
                }
            }
        }

        // Revert Service changes
        for svc in &backup.service_snapshots {
            let start_mode = match svc.previous_start_type.as_str() {
                "Automatic" => "auto",
                "Disabled" => "disabled",
                _ => "demand", // Manual
            };

            let _ = Command::new("sc.exe")
                .args(["config", &svc.service_name, &format!("start={}", start_mode)])
                .output();

            println!("    [✓] Restored service '{}' startup mode to {}", svc.service_name, start_mode);
        }

        Ok(())
    }

    /// Rollback all known backups
    pub fn rollback_all(&self) -> Result<usize, String> {
        let backups = self.list_backups();
        let count = backups.len();
        if count == 0 {
            return Ok(0);
        }

        for backup in &backups {
            let _ = self.rollback_backup(backup);
        }

        Ok(count)
    }
}
