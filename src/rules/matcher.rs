//! Rule matching logic

use crate::config::Rule;
use std::path::Path;

/// Find the first rule that matches the given file
pub fn find_rule_for_file<'a>(path: &std::path::Path, rules: &'a [Rule]) -> Option<&'a Rule> {
    let filename: String = path.file_name()?.to_str()?.to_lowercase();

    let mut extension = path
        .extension()
        .and_then(|e| e.to_str())
        .map(|s| s.to_lowercase())
        .unwrap_or_default();

    // Check hidden file with name matching the extension (e.g., .tmp -> tmp)
    if extension.is_empty() && filename.starts_with('.') && filename.len() > 1 {
        extension = filename[1..].to_string();
    }

    rules
        .iter()
        .find(|rule| rule.matches(&filename, &extension))
}

/// Check if file is a temporary file
pub fn is_temp_file(path: &Path, temp_extensions: &[String]) -> bool {
    // 1. Check standard extension (e.g., data.tmp)
    if let Some(ext) = path.extension().and_then(|e| e.to_str()) {
        if temp_extensions.contains(&ext.to_lowercase()) {
            return true;
        }
    }

    // 2. Check hidden file with name matching the extension (e.g., .tmp -> tmp)
    if let Some(filename) = path.file_name().and_then(|n| n.to_str()) {
        if filename.starts_with('.') && filename.len() > 1 {
            let name_after_dot = &filename[1..].to_lowercase();
            if temp_extensions.contains(name_after_dot) {
                return true;
            }
        }
    }

    false
}
