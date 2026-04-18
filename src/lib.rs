//! PANOS: Universal File Organizer OS Library

pub mod cli;
pub mod config;
pub mod file_ops;
pub mod organizer;
pub mod rules;

// Re-export main types for convenience
pub use cli::Args;
pub use config::{Config, Rule, load_config};
pub use file_ops::{move_file, remove_empty_dirs};
pub use organizer::{organize, watch_mode};
pub use rules::{find_rule_for_file, is_temp_file};
