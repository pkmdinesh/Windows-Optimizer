use colored::*;
use std::process::Command;

pub struct RestorePointManager;

impl RestorePointManager {
    /// Attempts to create a Windows System Restore Point prior to applying any optimizations
    pub fn create_restore_point(description: &str) -> Result<String, String> {
        println!("{}", "==> [Safety] Requesting Windows System Restore Point creation...".cyan());

        let script = format!(
            r#"
            try {{
                # Enable system restore on C: if not enabled
                Enable-ComputerRestore -Drive "C:\" -ErrorAction SilentlyContinue
                
                # Checkpoint-Computer
                $result = Checkpoint-Computer -Description "{}" -RestorePointType "MODIFY_SETTINGS" -ErrorAction Stop
                Write-Output "SUCCESS"
            }} catch {{
                Write-Output "ERROR: $($_.Exception.Message)"
            }}
            "#,
            description
        );

        let output = Command::new("powershell")
            .args(["-NoProfile", "-ExecutionPolicy", "Bypass", "-Command", &script])
            .output();

        match output {
            Ok(out) => {
                let stdout = String::from_utf8_lossy(&out.stdout).trim().to_string();
                let stderr = String::from_utf8_lossy(&out.stderr).trim().to_string();

                if stdout.contains("SUCCESS") {
                    let msg = format!("System Restore Point '{}' created successfully.", description);
                    println!("{}", format!("    [✓] {}", msg).green());
                    Ok(msg)
                } else {
                    let err_msg = if !stdout.is_empty() { stdout } else { stderr };
                    // On many modern Windows 11 installs, System Restore may be disabled by policy or rate-limited to 1 per 24 hours
                    println!("{}", format!("    [!] Note: System Restore Point creation returned: {}", err_msg).yellow());
                    println!("{}", "    [i] WinOpt will rely on its dedicated transactional Registry and Service rollback engine.".cyan());
                    Ok(format!("Restore point skipped or rate-limited: {}", err_msg))
                }
            }
            Err(e) => {
                let err_msg = format!("Failed to spawn PowerShell: {}", e);
                println!("{}", format!("    [!] {}", err_msg).yellow());
                Ok(err_msg)
            }
        }
    }
}
