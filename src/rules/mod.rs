//! Rule matching and file classification module

pub mod ai;
pub mod matcher;

pub use ai::PanosAI;
pub use matcher::{find_rule_for_file, is_temp_file};
