mod engine;
mod hardware;
mod modules;
mod safety;
mod ui;

use clap::Parser;
use engine::OptimizationOrchestrator;
use hardware::HardwareProfiler;
use ui::{start_web_server, CliInterface};

#[derive(Parser, Debug)]
#[command(name = "winopt")]
#[command(author = "WinOpt Team")]
#[command(version = "0.1.0")]
#[command(about = "Universal Hardware-Aware Windows OS Optimization & Debloater Tool", long_about = None)]
struct Args {
    /// Apply all safe hardware-aware optimizations immediately
    #[arg(short, long)]
    optimize: bool,

    /// Batch purge common UWP bloatware packages
    #[arg(short, long)]
    debloat: bool,

    /// Roll back previous optimizations to original state
    #[arg(short, long)]
    rollback: bool,

    /// Display current hardware profile and tweak statuses
    #[arg(short, long)]
    status: bool,

    /// Launch modern embedded web GUI in browser
    #[arg(short, long)]
    web: bool,
}

fn main() {
    let args = Args::parse();

    println!("Detecting system hardware configuration...");
    let profile = HardwareProfiler::collect_profile();
    let orchestrator = OptimizationOrchestrator::new(profile.clone());

    if args.status {
        CliInterface::print_banner(&profile);
        CliInterface::display_hardware_recommendations(&profile);
        CliInterface::list_tweaks(&orchestrator);
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

    if args.web {
        CliInterface::print_banner(&profile);
        println!("==> Launching web interface on http://127.0.0.1:8765 ...");
        let _ = std::process::Command::new("cmd").args(["/c", "start", "http://127.0.0.1:8765"]).spawn();
        start_web_server(profile);
        return;
    }

    // Default: Interactive menu
    CliInterface::print_banner(&profile);
    CliInterface::display_hardware_recommendations(&profile);
    CliInterface::run_interactive_menu(&orchestrator);
}
