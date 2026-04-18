//! Configuration loading and parsing

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::path::PathBuf;

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Rule {
    pub name: String,
    pub extensions: Vec<String>,
    pub patterns: Vec<String>,
    pub destination: PathBuf,

    #[serde(skip)]
    pub compiled_patterns: Vec<glob::Pattern>,
}

impl Rule {
    pub fn matches(&self, filename: &str, extension: &str) -> bool {
        // check pattern
        for pattern in &self.compiled_patterns {
            if pattern.matches(filename) {
                return true;
            }
        }

        // check extension
        for ext in &self.extensions {
            if ext.to_lowercase() == extension {
                return true;
            }
        }
        false
    }
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Config {
    pub source_dir: PathBuf,
    pub rules: Vec<Rule>,
    pub watch_mode: bool,
    pub debounce_seconds: u64,
    pub polling_interval_ms: u64,
    pub temp_extensions: Vec<String>,
    pub ignore_patterns: Vec<String>,
    pub trash_dir: PathBuf,
    pub unknown_dir: PathBuf,
    pub history_file: String,
    pub exclude_hidden: bool,
}

pub fn load_config(path: &std::path::Path) -> Result<Config> {
    let content: String = std::fs::read_to_string(path)?;
    let mut config: Config = toml::from_str(&content)?;

    for rule in &mut config.rules {
        for pattern in &rule.patterns {
            if let Ok(pattern) = glob::Pattern::new(pattern) {
                rule.compiled_patterns.push(pattern);
            } else {
                tracing::warn!("Invalid pattern: {:?}", pattern);
            }
        }
    }

    if !config.source_dir.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {:?}",
            config.source_dir
        ));
    }

    Ok(config)
}
