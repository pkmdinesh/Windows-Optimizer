pub mod ads;
pub mod cleaner;
pub mod debloat;
pub mod startup;
pub mod telemetry;
pub mod tuning;

pub use ads::get_ad_removal_tweaks;
pub use cleaner::CleanerManager;
pub use debloat::DebloatManager;
pub use startup::{get_startup_tweaks, StartupManager};
pub use telemetry::get_telemetry_tweaks;
pub use tuning::get_performance_tweaks;
