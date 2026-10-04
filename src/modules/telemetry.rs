use crate::engine::tweak::*;
use crate::safety::rollback::{OptimizationBackup, RollbackManager};
use std::process::Command;
use winreg::enums::{HKEY_CURRENT_USER, HKEY_LOCAL_MACHINE, KEY_READ};
use winreg::RegKey;

pub fn get_telemetry_tweaks() -> Vec<Tweak> {
    vec![
        Tweak {
            id: "telemetry_datacollection",
            name: "Disable Diagnostic Telemetry Collection",
            description: "Restricts Windows Diagnostic Data transmission to basic minimum or security level.",
            category: TweakCategory::Privacy,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                if let Ok(key) = hklm.open_subkey_with_flags("SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection", KEY_READ) {
                    if let Ok(val) = key.get_value::<u32, _>("AllowTelemetry") {
                        return val == 0 || val == 1;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "SOFTWARE\\Policies\\Microsoft\\Windows\\DataCollection";
                let snap1 = RollbackManager::snapshot_registry_value("HKLM", subkey, "AllowTelemetry");
                let snap2 = RollbackManager::snapshot_registry_value("HKLM", subkey, "DisableDiagnosticDataViewer");

                let hklm = RegKey::predef(HKEY_LOCAL_MACHINE);
                let (key, _) = hklm.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("AllowTelemetry", &0u32).map_err(|e| e.to_string())?;
                key.set_value("DisableDiagnosticDataViewer", &1u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "telemetry_datacollection".to_string(),
                    description: "Disable Diagnostic Telemetry Collection".to_string(),
                    registry_snapshots: vec![snap1, snap2],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "telemetry_datacollection") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "telemetry_services",
            name: "Disable Telemetry & Diagnostic Tracking Services",
            description: "Sets Connected User Experiences and Telemetry (DiagTrack) and WAP push services to Manual/Disabled.",
            category: TweakCategory::Privacy,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let out = Command::new("powershell")
                    .args(["-NoProfile", "-Command", "(Get-Service -Name 'DiagTrack' -ErrorAction SilentlyContinue).StartType"])
                    .output();
                if let Ok(o) = out {
                    let s = String::from_utf8_lossy(&o.stdout).trim().to_string();
                    return s == "Disabled" || s == "Manual";
                }
                false
            },
            apply: |rb, session_id| {
                let svc_snap1 = RollbackManager::snapshot_service("DiagTrack");
                let svc_snap2 = RollbackManager::snapshot_service("dmwappushservice");

                // Stop and disable DiagTrack
                let _ = Command::new("sc.exe").args(["stop", "DiagTrack"]).output();
                let _ = Command::new("sc.exe").args(["config", "DiagTrack", "start=disabled"]).output();

                // Stop and disable dmwappushservice
                let _ = Command::new("sc.exe").args(["stop", "dmwappushservice"]).output();
                let _ = Command::new("sc.exe").args(["config", "dmwappushservice", "start=disabled"]).output();

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "telemetry_services".to_string(),
                    description: "Disable Telemetry & Diagnostic Tracking Services".to_string(),
                    registry_snapshots: vec![],
                    service_snapshots: vec![svc_snap1, svc_snap2],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "telemetry_services") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_advertising_id",
            name: "Disable Windows Advertising ID",
            description: "Stops applications from tracking user activities for targeted advertising.",
            category: TweakCategory::Privacy,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\AdvertisingInfo") {
                    if let Ok(val) = key.get_value::<u32, _>("Enabled") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\AdvertisingInfo";
                let snap = RollbackManager::snapshot_registry_value("HKCU", subkey, "Enabled");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("Enabled", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_advertising_id".to_string(),
                    description: "Disable Windows Advertising ID".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_advertising_id") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_ceip_tasks",
            name: "Disable Customer Experience Improvement Tasks",
            description: "Disables scheduled background tasks for CEIP and compatibility data collection.",
            category: TweakCategory::Privacy,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let out = Command::new("schtasks")
                    .args(["/query", "/tn", "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator"])
                    .output();
                if let Ok(o) = out {
                    let s = String::from_utf8_lossy(&o.stdout);
                    return s.to_uppercase().contains("DISABLED");
                }
                false
            },
            apply: |_rb, _session_id| {
                let tasks = [
                    "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
                    "\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip",
                    "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
                    "\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater",
                    "\\Microsoft\\Windows\\Autochk\\Proxy",
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
                    "\\Microsoft\\Windows\\Customer Experience Improvement Program\\Consolidator",
                    "\\Microsoft\\Windows\\Customer Experience Improvement Program\\UsbCeip",
                    "\\Microsoft\\Windows\\Application Experience\\Microsoft Compatibility Appraiser",
                    "\\Microsoft\\Windows\\Application Experience\\ProgramDataUpdater",
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
