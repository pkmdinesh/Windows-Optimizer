use crate::engine::tweak::*;
use crate::hardware::models::HardwareProfile;
use crate::modules::*;
use crate::safety::{RestorePointManager, RollbackManager};
use colored::*;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TweakInfo {
    pub id: String,
    pub name: String,
    pub description: String,
    pub category: String,
    pub risk_level: String,
    pub is_applicable: bool,
    pub is_applied: bool,
}

pub struct OptimizationOrchestrator {
    pub profile: HardwareProfile,
    pub tweaks: Vec<Tweak>,
    pub rollback: RollbackManager,
}

impl OptimizationOrchestrator {
    pub fn new(profile: HardwareProfile) -> Self {
        let mut tweaks = Vec::new();
        tweaks.extend(get_ad_removal_tweaks());
        tweaks.extend(get_telemetry_tweaks());
        tweaks.extend(get_startup_tweaks());
        tweaks.extend(get_performance_tweaks());

        OptimizationOrchestrator {
            profile,
            tweaks,
            rollback: RollbackManager::new(),
        }
    }

    /// Retrieve status metadata of all registered tweaks
    pub fn get_tweak_infos(&self) -> Vec<TweakInfo> {
        self.tweaks
            .iter()
            .map(|t| {
                let applicable = (t.is_applicable)(&self.profile);
                let applied = (t.check_status)();
                TweakInfo {
                    id: t.id.to_string(),
                    name: t.name.to_string(),
                    description: t.description.to_string(),
                    category: t.category.to_string(),
                    risk_level: format!("{:?}", t.risk_level),
                    is_applicable: applicable,
                    is_applied: applied,
                }
            })
            .collect()
    }

    /// Apply all safe, hardware-applicable tweaks in a single batch
    pub fn apply_safe_preset(&self) -> Result<usize, String> {
        let session_id = format!("batch_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));

        println!("{}", "=========================================================".cyan());
        println!("{}", "  Applying Hardware-Aware Safe Optimization Preset".bold().cyan());
        println!("{}", "=========================================================".cyan());

        // 1. Create Windows Restore Point first
        let _ = RestorePointManager::create_restore_point("WinOpt_PreOptimization");

        let mut applied_count = 0;

        for tweak in &self.tweaks {
            if tweak.risk_level != RiskLevel::Safe {
                continue;
            }

            if !(tweak.is_applicable)(&self.profile) {
                println!("    [~] Skipping {} (Not applicable to current hardware)", tweak.name);
                continue;
            }

            if (tweak.check_status)() {
                println!("    [=] Already optimized: {}", tweak.name);
                continue;
            }

            print!("    [+] Applying: {}... ", tweak.name);
            match (tweak.apply)(&self.rollback, &session_id) {
                Ok(_) => {
                    println!("{}", "DONE".green());
                    applied_count += 1;
                }
                Err(e) => {
                    println!("{}", format!("FAILED ({})", e).red());
                }
            }
        }

        println!("{}", format!("\n==> Successfully applied {} optimization tweaks.", applied_count).green().bold());
        Ok(applied_count)
    }

    /// Apply a specific tweak by ID
    pub fn apply_tweak_by_id(&self, id: &str) -> Result<(), String> {
        let tweak = self.tweaks.iter().find(|t| t.id == id).ok_or_else(|| format!("Tweak '{}' not found", id))?;
        let session_id = format!("single_{}", chrono::Local::now().format("%Y%m%d_%H%M%S"));

        (tweak.apply)(&self.rollback, &session_id)?;
        Ok(())
    }

    /// Revert a specific tweak by ID
    pub fn revert_tweak_by_id(&self, id: &str) -> Result<(), String> {
        let tweak = self.tweaks.iter().find(|t| t.id == id).ok_or_else(|| format!("Tweak '{}' not found", id))?;
        (tweak.revert)(&self.rollback)?;
        Ok(())
    }

    /// Roll back all optimizations
    pub fn rollback_all(&self) -> Result<usize, String> {
        self.rollback.rollback_all()
    }
}
