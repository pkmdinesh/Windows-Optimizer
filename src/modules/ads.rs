use crate::engine::tweak::*;
use crate::safety::rollback::{OptimizationBackup, RollbackManager};
use winreg::enums::HKEY_CURRENT_USER;
use winreg::RegKey;

pub fn get_ad_removal_tweaks() -> Vec<Tweak> {
    vec![
        Tweak {
            id: "disable_start_suggestions",
            name: "Disable Start Menu Ads & Promoted Apps",
            description: "Stops Windows from displaying promoted apps and suggestions in the Start menu.",
            category: TweakCategory::AdRemoval,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager") {
                    if let Ok(val) = key.get_value::<u32, _>("SystemPaneSuggestionsEnabled") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager";
                let snap1 = RollbackManager::snapshot_registry_value("HKCU", subkey, "SystemPaneSuggestionsEnabled");
                let snap2 = RollbackManager::snapshot_registry_value("HKCU", subkey, "SubscribedContent-338388Enabled");
                let snap3 = RollbackManager::snapshot_registry_value("HKCU", subkey, "SubscribedContent-310093Enabled");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("SystemPaneSuggestionsEnabled", &0u32).map_err(|e| e.to_string())?;
                key.set_value("SubscribedContent-338388Enabled", &0u32).map_err(|e| e.to_string())?;
                key.set_value("SubscribedContent-310093Enabled", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_start_suggestions".to_string(),
                    description: "Disable Start Menu Ads & Promoted Apps".to_string(),
                    registry_snapshots: vec![snap1, snap2, snap3],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_start_suggestions") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_silent_app_install",
            name: "Disable Automatic Sponsored App Downloads",
            description: "Prevents Windows from silently downloading third-party promotional apps in the background.",
            category: TweakCategory::AdRemoval,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager") {
                    if let Ok(val) = key.get_value::<u32, _>("SilentInstalledAppsEnabled") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager";
                let snap = RollbackManager::snapshot_registry_value("HKCU", subkey, "SilentInstalledAppsEnabled");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("SilentInstalledAppsEnabled", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_silent_app_install".to_string(),
                    description: "Disable Automatic Sponsored App Downloads".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_silent_app_install") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_explorer_ads",
            name: "Disable File Explorer Sync Provider Ads",
            description: "Removes OneDrive and subscription banner advertisements inside Windows File Explorer.",
            category: TweakCategory::AdRemoval,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced") {
                    if let Ok(val) = key.get_value::<u32, _>("ShowSyncProviderNotifications") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced";
                let snap = RollbackManager::snapshot_registry_value("HKCU", subkey, "ShowSyncProviderNotifications");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("ShowSyncProviderNotifications", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_explorer_ads".to_string(),
                    description: "Disable File Explorer Sync Provider Ads".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_explorer_ads") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_bing_search",
            name: "Disable Bing Web Results in Windows Search",
            description: "Removes web search ads and restores fast, private offline local file and app search.",
            category: TweakCategory::AdRemoval,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Search") {
                    if let Ok(val) = key.get_value::<u32, _>("BingSearchEnabled") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Search";
                let snap1 = RollbackManager::snapshot_registry_value("HKCU", subkey, "BingSearchEnabled");
                let snap2 = RollbackManager::snapshot_registry_value("HKCU", subkey, "ConnectedSearchUseWeb");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("BingSearchEnabled", &0u32).map_err(|e| e.to_string())?;
                key.set_value("ConnectedSearchUseWeb", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_bing_search".to_string(),
                    description: "Disable Bing Web Results in Windows Search".to_string(),
                    registry_snapshots: vec![snap1, snap2],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_bing_search") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_windows_tips",
            name: "Disable Windows Tips & Suggestions",
            description: "Stops popups and suggestions recommending Microsoft products in Settings and Notifications.",
            category: TweakCategory::AdRemoval,
            risk_level: RiskLevel::Safe,
            is_applicable: |_| true,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\UserProfileEngagement") {
                    if let Ok(val) = key.get_value::<u32, _>("ScoobeSystemSettingEnabled") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey1 = "Software\\Microsoft\\Windows\\CurrentVersion\\UserProfileEngagement";
                let subkey2 = "Software\\Microsoft\\Windows\\CurrentVersion\\ContentDeliveryManager";
                let snap1 = RollbackManager::snapshot_registry_value("HKCU", subkey1, "ScoobeSystemSettingEnabled");
                let snap2 = RollbackManager::snapshot_registry_value("HKCU", subkey2, "SubscribedContent-338389Enabled");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key1, _) = hkcu.create_subkey(subkey1).map_err(|e| e.to_string())?;
                key1.set_value("ScoobeSystemSettingEnabled", &0u32).map_err(|e| e.to_string())?;

                let (key2, _) = hkcu.create_subkey(subkey2).map_err(|e| e.to_string())?;
                key2.set_value("SubscribedContent-338389Enabled", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_windows_tips".to_string(),
                    description: "Disable Windows Tips & Suggestions".to_string(),
                    registry_snapshots: vec![snap1, snap2],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_windows_tips") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
        Tweak {
            id: "disable_taskbar_widgets",
            name: "Disable Taskbar Widgets / News Feed",
            description: "Removes MSN News & Interests widget button from Windows taskbar, saving RAM and background CPU cycles.",
            category: TweakCategory::AdRemoval,
            risk_level: RiskLevel::Safe,
            is_applicable: |p| p.os.is_windows_11,
            check_status: || {
                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                if let Ok(key) = hkcu.open_subkey("Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced") {
                    if let Ok(val) = key.get_value::<u32, _>("TaskbarDa") {
                        return val == 0;
                    }
                }
                false
            },
            apply: |rb, session_id| {
                let subkey = "Software\\Microsoft\\Windows\\CurrentVersion\\Explorer\\Advanced";
                let snap = RollbackManager::snapshot_registry_value("HKCU", subkey, "TaskbarDa");

                let hkcu = RegKey::predef(HKEY_CURRENT_USER);
                let (key, _) = hkcu.create_subkey(subkey).map_err(|e| e.to_string())?;
                key.set_value("TaskbarDa", &0u32).map_err(|e| e.to_string())?;

                rb.save_backup(&OptimizationBackup {
                    session_id: session_id.to_string(),
                    timestamp: chrono::Local::now().to_rfc3339(),
                    tweak_id: "disable_taskbar_widgets".to_string(),
                    description: "Disable Taskbar Widgets / News Feed".to_string(),
                    registry_snapshots: vec![snap],
                    service_snapshots: vec![],
                })?;

                Ok(())
            },
            revert: |rb| {
                for b in rb.list_backups().into_iter().filter(|b| b.tweak_id == "disable_taskbar_widgets") {
                    rb.rollback_backup(&b)?;
                }
                Ok(())
            },
        },
    ]
}
