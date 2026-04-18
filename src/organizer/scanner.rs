//! Directory scanning and organization logic

use anyhow::Result;
use std::path::PathBuf;
use tracing::info;
use walkdir::WalkDir;

use crate::config::Config;
use crate::file_ops::move_file;
use crate::rules::{find_rule_for_file, is_temp_file};

/// Organize files in the source directory according to rules
pub fn organize(config: &Config, dry_run: bool) -> Result<()> {
    if !config.source_dir.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {:?}",
            config.source_dir
        ));
    }

    info!("Scanning {:?}...", config.source_dir);

    for entry in WalkDir::new(&config.source_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // Optimization: Skip scanning into destination directories or hidden folders
            let name = e.file_name().to_str().unwrap_or("");
            if name.starts_with('.') && name != "." && name != ".." {
                return false;
            }
            
            for rule in &config.rules {
                if name == rule.destination.to_str().unwrap_or("") {
                    return false;
                }
            }
            true
        })
        .filter_map(|e: std::result::Result<walkdir::DirEntry, walkdir::Error>| e.ok())
    {
        if entry.file_type().is_file() {
            let path: &std::path::Path = entry.path();

            // cleanup temp file
            if is_temp_file(path) {
                let trash_dir: PathBuf = config.source_dir.join(".panos_trash");
                move_file(path, &trash_dir, dry_run)?;
                continue;
            }

            if let Some(rule) = find_rule_for_file(path, &config.rules) {
                let dest_dir: PathBuf = config.source_dir.join(rule.destination.clone());
                move_file(path, &dest_dir, dry_run)?;
            }
        }
    }

    Ok(())
}
