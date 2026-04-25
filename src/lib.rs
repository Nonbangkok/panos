//! PANOS: Universal File Organizer OS Library

pub mod cli;
pub mod config;
pub mod file_ops;
pub mod organizer;
pub mod rules;
pub mod ui;

// Re-export main types for convenience
pub use cli::Args;
pub use config::{Config, Rule};
pub use file_ops::{check_integrity, move_file, remove_empty_dirs};
pub use organizer::{organize, run_undo, watch_mode, watcher::should_ignore};
pub use rules::{PanosAI, find_rule_for_file, is_temp_file};
pub use ui::{IndicatifReporter, NoopReporter, ProgressReporter};
