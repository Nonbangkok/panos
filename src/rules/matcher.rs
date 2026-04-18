//! Rule matching logic

use crate::config::Rule;
use std::path::Path;

/// Find the first rule that matches the given file
pub fn find_rule_for_file<'a>(path: &std::path::Path, rules: &'a [Rule]) -> Option<&'a Rule> {
    let filename: String = path.file_name()?.to_str()?.to_lowercase();
    let extension: String = path.extension()?.to_str()?.to_lowercase();

    rules
        .iter()
        .find(|rule| rule.matches(&filename, &extension))
}

/// Check if file is a temporary file
pub fn is_temp_file(path: &Path, temp_extensions: &[String]) -> bool {
    path.extension()
        .and_then(|e| e.to_str())
        .map(|ext| temp_extensions.contains(&ext.to_lowercase()))
        .unwrap_or(false)
}
