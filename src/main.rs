#![windows_subsystem = "windows"]

mod engine;
mod hardware;
mod modules;
mod safety;
mod ui;

use clap::Parser;
use engine::OptimizationOrchestrator;
use hardware::HardwareProfiler;
use modules::CleanerManager;
use ui::{launch_gui_window, CliInterface};

#[cfg(windows)]
extern "system" {
    fn AttachConsole(dwProcessId: u32) -> i32;
}

const ATTACH_PARENT_PROCESS: u32 = 0xFFFFFFFF;

fn attach_parent_console() {
    #[cfg(windows)]
    unsafe {
        let _ = AttachConsole(ATTACH_PARENT_PROCESS);
    }
}

#[derive(Parser, Debug)]
#[command(name = "winopt")]
#[command(author = "WinOpt Team")]
#[command(version = "0.2.0")]
#[command(about = "Universal Hardware-Aware Windows OS Optimizer & Deep Cleaner GUI", long_about = None)]
struct Args {
    /// Apply all safe hardware-aware optimizations immediately
    #[arg(short, long)]
    optimize: bool,

    /// Deep clean temporary files, %TEMP%, Prefetch, and system caches
    #[arg(short, long)]
    clean: bool,

    /// Flush DNS resolver cache and trim RAM process working sets
    #[arg(short, long)]
    boost: bool,

    /// Batch purge common UWP bloatware packages
    #[arg(short, long)]
    debloat: bool,

    /// Roll back previous optimizations to original state
    #[arg(short, long)]
    rollback: bool,

    /// Display current hardware profile and tweak statuses in terminal
    #[arg(short, long)]
    status: bool,

    /// Force launch interactive terminal CLI menu instead of GUI app
    #[arg(long)]
    cli: bool,

    /// Explicitly launch the GUI app window
    #[arg(short, long)]
    gui: bool,
}

fn main() {
    let args = Args::parse();

    // Check if any headless CLI flag was passed
    let has_cli_flag = args.status
        || args.rollback
        || args.debloat
        || args.optimize
        || args.clean
        || args.boost
        || args.cli;

    // Default behavior: Pure GUI window with NO console window
    if !has_cli_flag || args.gui {
        let profile = HardwareProfiler::collect_profile();
        launch_gui_window(profile);
        return;
    }

    // Attach to existing console if user explicitly ran from command prompt / PowerShell
    attach_parent_console();

    println!("Detecting system hardware configuration...");
    let profile = HardwareProfiler::collect_profile();
    let orchestrator = OptimizationOrchestrator::new(profile.clone());

    if args.status {
        CliInterface::print_banner(&profile);
        CliInterface::display_hardware_recommendations(&profile);
        CliInterface::list_tweaks(&orchestrator);
        return;
    }

    if args.clean {
        CliInterface::print_banner(&profile);
        println!("==> Scanning & cleaning temporary files...");
        let res = CleanerManager::clean_all_junk();
        println!(
            "Successfully cleaned {:.2} MB across {} files ({} locked skipped).",
            res.total_freed_bytes as f64 / (1024.0 * 1024.0),
            res.files_deleted,
            res.errors_encountered
        );
        return;
    }

    if args.boost {
        println!("==> Flushing DNS and trimming RAM working sets...");
        let res = CleanerManager::boost_memory_and_network();
        println!("{}", res.message);
        return;
    }

    if args.rollback {
        CliInterface::print_banner(&profile);
        println!("==> Rolling back previous modifications...");
        match orchestrator.rollback_all() {
            Ok(count) => println!("Successfully reverted {} state records.", count),
            Err(e) => eprintln!("Error during rollback: {}", e),
        }
        return;
    }

    if args.debloat {
        CliInterface::print_banner(&profile);
        println!("==> Scanning & purging pre-installed bloatware...");
        let results = modules::DebloatManager::purge_all_safe_bloatware();
        for r in results {
            println!("  {}", r);
        }
        return;
    }

    if args.optimize {
        CliInterface::print_banner(&profile);
        CliInterface::display_hardware_recommendations(&profile);
        let _ = orchestrator.apply_safe_preset();
        return;
    }

    if args.cli {
        CliInterface::print_banner(&profile);
        CliInterface::display_hardware_recommendations(&profile);
        CliInterface::run_interactive_menu(&orchestrator);
    }
}
