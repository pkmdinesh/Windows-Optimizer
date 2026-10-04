use crate::hardware::models::HardwareProfile;
use crate::safety::RollbackManager;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TweakCategory {
    AdRemoval,
    Privacy,
    Debloat,
    Startup,
    Performance,
}

impl std::fmt::Display for TweakCategory {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TweakCategory::AdRemoval => write!(f, "Ad Removal"),
            TweakCategory::Privacy => write!(f, "Privacy & Telemetry"),
            TweakCategory::Debloat => write!(f, "UWP Debloater"),
            TweakCategory::Startup => write!(f, "Startup Optimization"),
            TweakCategory::Performance => write!(f, "Hardware Performance"),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RiskLevel {
    Safe,
    Moderate,
    Advanced,
}

#[allow(dead_code)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum TweakStatus {
    Applied,
    NotApplied,
    Inapplicable,
}

pub struct Tweak {
    pub id: &'static str,
    pub name: &'static str,
    pub description: &'static str,
    pub category: TweakCategory,
    pub risk_level: RiskLevel,
    pub is_applicable: fn(&HardwareProfile) -> bool,
    pub check_status: fn() -> bool,
    pub apply: fn(&RollbackManager, &str) -> Result<(), String>,
    pub revert: fn(&RollbackManager) -> Result<(), String>,
}
