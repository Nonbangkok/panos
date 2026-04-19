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

    pub fn sanitize(&mut self) {
        // extensions sanitize
        self.extensions = self
            .extensions
            .iter()
            .map(|ext| {
                let mut ext = ext.clone().to_lowercase();
                if ext.starts_with('.') {
                    ext = ext[1..].to_string();
                }
                ext
            })
            .collect();

        // patterns compile
        for pattern in &self.patterns {
            let pattern_lower = pattern.to_lowercase();
            if let Ok(p) = glob::Pattern::new(&pattern_lower) {
                self.compiled_patterns.push(p);
            } else {
                tracing::warn!("Invalid pattern: {:?}", pattern_lower);
            }
        }
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

impl Config {
    pub fn load(path: &std::path::Path) -> Result<Self> {
        let content: String = std::fs::read_to_string(path)?;
        let mut config: Config = toml::from_str(&content)?;

        if !config.source_dir.exists() {
            return Err(anyhow::anyhow!(
                "Source directory does not exist: {:?}",
                config.source_dir
            ));
        }

        config.sanitize();

        Ok(config)
    }

    pub fn sanitize(&mut self) {
        // temp_extensions sanitize
        self.temp_extensions = self
            .temp_extensions
            .iter()
            .map(|ext| {
                let mut ext = ext.clone().to_lowercase();
                if ext.starts_with('.') {
                    ext = ext[1..].to_string();
                }
                ext
            })
            .collect();

        // rules sanitize
        for rule in &mut self.rules {
            rule.sanitize();
        }
    }
}
