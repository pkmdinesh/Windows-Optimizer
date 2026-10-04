use crate::engine::tweak::*;
use crate::safety::rollback::{OptimizationBackup, RollbackManager};
use serde::{Deserialize, Serialize};
use std::process::Command;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupProgram {
    pub name: String,
    pub command: String,
    pub location: String, // "HKLM Run", "HKCU Run", etc.
}

pub fn get_startup_tweaks() -> Vec<Tweak> {
    vec![
        Tweak {
            id: "disable_startup_delay",
            name: "Disable Windows Desktop Startup Delay",
            description: "Eliminates the artificial 5-10 second delay Windows introduces before launching startup apps.",
            category: TweakCategory::Startup,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Serialize") {
                    if let Ok(val) = key.get_value::<u32, _>("StartupDelayInMSec") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Serialize";
                let snap = RollbackManager::snapshot_registry_value("HKCU", subkey, "StartupDelayInMSec");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("StartupDelayInMSec", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_startup_delay".to_string(),
                    description: "Disable Windows Desktop Startup Delay".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_startup_delay") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "optimize_startup_scheduled_scans",
            name: "Disable Diagnostic Logon Tasks",
            description: "Stops non-critical diagnostic disk analyzers and telemetry schedulers from firing at user logon.",
            category: TweakCategory::Startup,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let out = Command::new("schtasks")
                    .args(["/query", "/tn", "\\Microsoft\\Windows\\DiskDiagnostic\\Microsoft-Windows-DiskDiagnosticDataCollector"])
                    .output();
                if let Ok(o) = out {
                    let s = String::from_utf8_lossy(&o.stdout);
                    return s.to_uppercase().contains("DISABLED");
                }
                false
            },
            apply: |_rb, _session_id| {
                let tasks = [
                    "\\Microsoft\\Windows\\DiskDiagnostic\\Microsoft-Windows-DiskDiagnosticDataCollector",
                    "\\Microsoft\\Windows\\DiskCleanup\\SilentCleanup",
                ];

                for task in tasks {
                    let _ = Command::new("schtasks")
                        .args(["/change", "/tn", task, "/disable"])
                        .output();
                }

                Ok(())
            },
            revert: |_rb| {
                let tasks = [
                    "\\Microsoft\\Windows\\DiskDiagnostic\\Microsoft-Windows-DiskDiagnosticDataCollector",
                ];

                for task in tasks {
                    let _ = Command::new("schtasks")
                        .args(["/change", "/tn", task, "/enable"])
                        .output();
                }

                Ok(())
            },
        },
    ]
}

pub struct StartupManager;

impl StartupManager {
    /// Enumerate all registered autostart programs in Windows Registry
    pub fn get_registered_startup_programs() -> Vec<StartupProgram> {
        let mut programs = Vec::new();

        // 1. HKLM Run
        let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
        if let Ok(key) = hklm.open_subkey_with_flags("SOFTWARE\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_READ) {
            for val in key.enum_values().flatten() {
                if let Ok(cmd) = key.get_value::<String, _>(&val.0) {
                    programs.push(StartupProgram {
                        name: val.0,
                        command: cmd,
                        location: "HKLM:Run".to_string(),
                    });
                }
            }
        }

        // 2. HKCU Run
        let hkcu = RegKey::predef(HKEY_CURRENT_USER);
        if let Ok(key) = hkcu.open_subkey_with_flags("Software\\Microsoft\\Windows\\CurrentVersion\\Run", KEY_READ) {
            for val in key.enum_values().flatten() {
                if let Ok(cmd) = key.get_value::<String, _>(&val.0) {
                    programs.push(StartupProgram {
                        name: val.0,
                        command: cmd,
                        location: "HKCU:Run".to_string(),
                    });
                }
            }
        }

        programs
    }
}
