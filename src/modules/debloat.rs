use colored::*;
use serde::{Deserialize, Serialize};
use std::process::Command;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BloatwareApp {
    pub display_name: String,
    pub package_pattern: String,
    pub description: String,
    pub is_installed: bool,
}

pub struct DebloatManager;

impl DebloatManager {
    pub const PROTECTED_WHITELIST: &'static [&'static str] = &[
        "Microsoft.WindowsStore",
        "Microsoft.DesktopAppInstaller",
        "Microsoft.WindowsTerminal",
        "Microsoft.VCLibs",
        "Microsoft.UI.Xaml",
        "Microsoft.NET.Native",
        "Microsoft.WindowsCalculator",
        "Microsoft.WindowsCamera",
        "Microsoft.ScreenSketch",
    ];

    pub const CURATED_BLOATWARE_LIST: &'static [(&'static str, &'static str, &'static str)] = &[
        ("Cortana", "*Cortana*", "Microsoft Cortana Assistant (Deprecated)"),
        ("Clipchamp Video Editor", "*Clipchamp*", "Pre-installed cloud video editor"),
        ("MSN News", "*BingNews*", "MSN News and feeds app"),
        ("MSN Weather", "*BingWeather*", "MSN Weather app"),
        ("Get Help", "*GetHelp*", "Diagnostic help app"),
        ("Tips & Get Started", "*Getstarted*", "Windows introductory promotional tips"),
        ("Microsoft Solitaire Collection", "*SolitaireCollection*", "Solitaire game with ads"),
        ("Mixed Reality Portal", "*MixedReality.Portal*", "Windows Mixed Reality portal"),
        ("People", "*Microsoft.People*", "Windows contacts app"),
        ("Skype", "*SkypeApp*", "Legacy Skype client"),
        ("Feedback Hub", "*WindowsFeedbackHub*", "Microsoft user survey and feedback telemetry"),
        ("TikTok Preload", "*TikTok*", "Sponsored TikTok app"),
        ("Disney+ Preload", "*Disney*", "Sponsored Disney+ app"),
        ("Spotify Preload", "*Spotify*", "Sponsored Spotify desktop app"),
    ];

    /// Scans the system for installed bloatware packages
    pub fn scan_installed_bloatware() -> Vec<BloatwareApp> {
        let mut results = Vec::new();

        // Query installed appx packages
        let script = r#"
        Get-AppxPackage | Select-Object -Property Name, PackageFullName | ConvertTo-Json
        "#;

        let output = Command::new("powershell")
            .args(["-NoProfile", "-Command", script])
            .output();

        let installed_names = if let Ok(out) = output {
            String::from_utf8_lossy(&out.stdout).to_string()
        } else {
            String::new()
        };

        for (display_name, pattern, desc) in Self::CURATED_BLOATWARE_LIST {
            let clean_pat = pattern.replace('*', "");
            let is_installed = installed_names.contains(&clean_pat);

            results.push(BloatwareApp {
                display_name: display_name.to_string(),
                package_pattern: pattern.to_string(),
                description: desc.to_string(),
                is_installed,
            });
        }

        results
    }

    /// Removes a specific app package safely
    pub fn remove_package(pattern: &str) -> Result<String, String> {
        // Enforce whitelist protection
        for safe in Self::PROTECTED_WHITELIST {
            if pattern.contains(safe) {
                return Err(format!("Protection violation: '{}' is a protected system core package.", safe));
            }
        }

        let script = format!(
            r#"
            Get-AppxPackage -Name '{}' -ErrorAction SilentlyContinue | Remove-AppxPackage -ErrorAction SilentlyContinue
            Get-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue | Where-Object {{ $_.DisplayName -like '{}' }} | Remove-AppxProvisionedPackage -Online -ErrorAction SilentlyContinue
            "#,
            pattern, pattern
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .output()
            .map_err(|e| e.to_string())?;

        if output.status.success() {
            Ok(format!("Successfully removed package matching {}", pattern))
        } else {
            Err(String::from_utf8_lossy(&output.stderr).to_string())
        }
    }

    /// Batch purges all detected safe bloatware
    pub fn purge_all_safe_bloatware() -> Vec<String> {
        let mut actions = Vec::new();
        let scanned = Self::scan_installed_bloatware();

        for app in scanned.into_iter().filter(|a| a.is_installed) {
            println!("{}", format!("    [-] Removing {}...", app.display_name).cyan());
            match Self::remove_package(&app.package_pattern) {
                Ok(msg) => actions.push(format!("{}: {}", app.display_name, msg)),
                Err(e) => actions.push(format!("Failed to remove {}: {}", app.display_name, e)),
            }
        }

        actions
    }
}
