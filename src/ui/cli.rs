use crate::engine::OptimizationOrchestrator;
use crate::hardware::models::*;
use crate::modules::{CleanerManager, DebloatManager, StartupManager};
use colored::*;
use std::io::{self, Write};

pub struct CliInterface;

impl CliInterface {
    pub fn print_banner(profile: &HardwareProfile) {
        println!("{}", "================================================================================".cyan());
        println!("{}", "            UNIVERSAL HARDWARE-AWARE WINDOWS OS OPTIMIZER v0.2.0                ".bold().cyan());
        println!("{}", "================================================================================".cyan());
        println!(
            "  OS:       {} (Build {}) [{}]",
            profile.os.name.green().bold(),
            profile.os.build.yellow(),
            profile.os.arch.cyan()
        );
        println!(
            "  CPU:      {} ({} Cores / {} Threads) - Hybrid Layout: {}",
            profile.cpu.name.green(),
            profile.cpu.physical_cores,
            profile.cpu.logical_cores,
            if profile.cpu.is_hybrid { "Yes (P/E Cores)".yellow() } else { "Symmetric".cyan() }
        );
        println!(
            "  Memory:   {:.1} GB Total / {:.1} GB Free [{:?}]",
            profile.ram.total_mb as f64 / 1024.0,
            profile.ram.available_mb as f64 / 1024.0,
            profile.ram.category
        );
        println!(
            "  Chassis:  {:?} | Power: {:?}",
            profile.chassis_type, profile.power_state
        );

        print!("  Storage:  ");
        for drive in &profile.drives {
            print!(
                "[{}: {:?} {:.0}GB (TRIM: {})] ",
                drive.drive_letter,
                drive.storage_type,
                drive.total_gb,
                if drive.trim_enabled { "ON".green() } else { "OFF".red() }
            );
        }
        println!("\n{}\n", "================================================================================".cyan());
    }

    pub fn display_hardware_recommendations(profile: &HardwareProfile) {
        println!("{}", "--- Hardware-Aware Strategy & Diagnostic Breakdown ---".bold().yellow());
        if profile.is_laptop_on_battery() {
            println!("  [!] Device is a LAPTOP on BATTERY. Aggressive core unparking & priority locks are restrained to save power.");
        } else {
            println!("  [✓] Device is on AC/DESKTOP power. High-responsiveness foreground CPU priority separation enabled.");
        }

        if profile.has_fast_storage() {
            println!("  [✓] Fast NVMe/SSD detected as system disk. SysMain redundant flash write prefetching will be restrained.");
            println!("  [✓] TRIM signaling optimization available for fast flash garbage collection.");
        } else {
            println!("  [!] Rotational HDD detected. SysMain will be kept active to accelerate disk seek times.");
        }

        match profile.ram.category {
            RamCategory::Low => println!("  [!] Low RAM (<8GB) detected. Background memory bloatware minimization prioritized."),
            RamCategory::Standard => println!("  [✓] Standard RAM (8-32GB) detected. Balanced memory footprint policy."),
            RamCategory::High => println!("  [✓] High-Capacity RAM (32GB+) detected. Paging file stress reduction enabled."),
        }
        println!();
    }

    pub fn list_tweaks(orchestrator: &OptimizationOrchestrator) {
        println!("\n{:<32} {:<22} {:<10} {:<10}", "TWEAK NAME", "CATEGORY", "STATUS", "RISK");
        println!("{}", "-".repeat(80));

        for info in orchestrator.get_tweak_infos() {
            let status_str = if !info.is_applicable {
                "N/A (HW)".yellow()
            } else if info.is_applied {
                "OPTIMIZED".green()
            } else {
                "DEFAULT".white()
            };

            println!(
                "{:<32} {:<22} {:<10} {:<10}",
                if info.name.len() > 30 { &info.name[..30] } else { &info.name },
                info.category,
                status_str,
                info.risk_level
            );
        }
        println!();
    }

    pub fn run_interactive_menu(orchestrator: &OptimizationOrchestrator) {
        loop {
            println!("{}", "Available Actions:".bold());
            println!("  1. Apply All Safe Hardware-Aware Defaults (One-Click)");
            println!("  2. View All Tweaks & Current System Status");
            println!("  3. Deep Clean Temp & Junk Files (%TEMP%, Prefetch, Caches)");
            println!("  4. Quick Flush RAM Working Sets & DNS Resolver Cache");
            println!("  5. Scan & Purge Installed UWP Bloatware");
            println!("  6. Inspect Registered Startup Programs");
            println!("  7. Rollback All Optimizations (Undo to Previous State)");
            println!("  8. Launch Modern Web GUI App (Dedicated Window)");
            println!("  0. Exit");
            print!("\nSelect an option [0-8]: ");
            let _ = io::stdout().flush();

            let mut input = String::new();
            if io::stdin().read_line(&mut input).is_err() {
                break;
            }

            match input.trim() {
                "1" => {
                    let _ = orchestrator.apply_safe_preset();
                }
                "2" => {
                    Self::list_tweaks(orchestrator);
                }
                "3" => {
                    println!("\n{}", "==> Scanning junk files...".cyan());
                    let junk = CleanerManager::scan_junk();
                    let total_bytes: u64 = junk.iter().map(|j| j.size_bytes).sum();
                    let total_files: usize = junk.iter().map(|j| j.file_count).sum();
                    println!("    Detected {:.2} MB across {} files cleanable.", total_bytes as f64 / (1024.0 * 1024.0), total_files);

                    print!("Proceed to purge all temporary & junk files? (y/N): ");
                    let _ = io::stdout().flush();
                    let mut confirm = String::new();
                    let _ = io::stdin().read_line(&mut confirm);
                    if confirm.trim().eq_ignore_ascii_case("y") {
                        let res = CleanerManager::clean_all_junk();
                        println!("{}", format!("    [✓] Cleaned {:.2} MB across {} files!", res.total_freed_bytes as f64 / (1024.0 * 1024.0), res.files_deleted).green());
                    }
                    println!();
                }
                "4" => {
                    println!("\n{}", "==> Flushing DNS resolver and trimming RAM working sets...".cyan());
                    let res = CleanerManager::boost_memory_and_network();
                    println!("{}", format!("    [✓] {}", res.message).green());
                    println!();
                }
                "5" => {
                    println!("\n{}", "==> Scanning for pre-installed bloatware packages...".cyan());
                    let bloat = DebloatManager::scan_installed_bloatware();
                    let installed: Vec<_> = bloat.iter().filter(|b| b.is_installed).collect();

                    if installed.is_empty() {
                        println!("{}", "    [✓] No common bloatware packages detected! System is clean.".green());
                    } else {
                        println!("    Detected {} bloatware packages:", installed.len());
                        for b in &installed {
                            println!("      - {} ({})", b.display_name.yellow(), b.description);
                        }

                        print!("\nDo you want to purge these packages now? (y/N): ");
                        let _ = io::stdout().flush();
                        let mut confirm = String::new();
                        let _ = io::stdin().read_line(&mut confirm);
                        if confirm.trim().eq_ignore_ascii_case("y") {
                            let results = DebloatManager::purge_all_safe_bloatware();
                            for r in results {
                                println!("      {}", r);
                            }
                        }
                    }
                    println!();
                }
                "6" => {
                    println!("\n{}", "==> Inspecting Registered Startup Items in Windows Registry...".cyan());
                    let items = StartupManager::get_registered_startup_programs();
                    if items.is_empty() {
                        println!("    No autostart programs found in registry run keys.");
                    } else {
                        for p in items {
                            println!("    [{}] {} -> {}", p.location.yellow(), p.name.green().bold(), p.command);
                        }
                    }
                    println!();
                }
                "7" => {
                    println!("\n{}", "==> Rolling back previous modifications...".yellow());
                    match orchestrator.rollback_all() {
                        Ok(count) => println!("{}", format!("    [✓] Reverted {} previous backups successfully.", count).green()),
                        Err(e) => println!("{}", format!("    [!] Error during rollback: {}", e).red()),
                    }
                    println!();
                }
                "8" => {
                    println!("\n{}", "==> Launching dedicated desktop GUI window...".green());
                    crate::ui::web::launch_gui_window(orchestrator.profile.clone());
                }
                "0" | "q" | "exit" => {
                    println!("Exiting WinOpt. Stay fast and optimized!");
                    break;
                }
                _ => println!("{}", "Invalid option. Please choose a number between 0 and 8.\n".red()),
            }
        }
    }
}
