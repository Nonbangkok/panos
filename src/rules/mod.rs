//! Rule matching and file classification module

pub mod matcher;

pub use matcher::{find_rule_for_file, is_temp_file};
