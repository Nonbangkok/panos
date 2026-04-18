//! History logic

use anyhow::Result;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct MoveRecord {
    pub source: PathBuf,
    pub destination: PathBuf,
    pub timestamp: DateTime<Utc>,
}

#[derive(Debug, Serialize, Deserialize, Default)]
pub struct Session {
    pub moves: Vec<MoveRecord>,
}

impl Session {
    /// Save Session to JSON file
    pub fn save(&self, source_dir: &Path, history_file: &str) -> Result<()> {
        let history_path = source_dir.join(history_file);
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(history_path, content)?;
        Ok(())
    }

    /// Load Session from JSON file
    pub fn load(source_dir: &Path, history_file: &str) -> Result<Self> {
        let history_path = source_dir.join(history_file);
        if !history_path.exists() {
            return Ok(Session::default());
        }
        let content = std::fs::read_to_string(history_path)?;
        let session = serde_json::from_str(&content)?;
        Ok(session)
    }
}
