//! Directory scanning and organization logic

use anyhow::Result;
use rayon::prelude::*;
use std::path::PathBuf;
use tracing::info;
use walkdir::WalkDir;

use crate::config::Config;
use crate::file_ops::history::MoveRecord;
use crate::file_ops::move_file;
use crate::rules::{find_rule_for_file, is_temp_file};

/// Organize files in the source directory according to rules
pub fn organize(config: &Config, dry_run: bool) -> Result<Vec<MoveRecord>> {
    if !config.source_dir.exists() {
        return Err(anyhow::anyhow!(
            "Source directory does not exist: {:?}",
            config.source_dir
        ));
    }

    if dry_run {
        info!("[DRY RUN] Scanning {:?}...", config.source_dir);
    } else {
        info!("Scanning {:?}...", config.source_dir);
    }

    // 1. Collect all files that need to be processed (Sequential)
    // WalkDir is not easily parallelized, but collecting paths is fast.
    let entries = collect_files(config);

    // 2. Process all entries in parallel using Rayon
    let history_results = process_files_in_parallel(entries, config, dry_run);

    // Flatten Ok(Vec<Option>) -> Ok(Vec) by filtering out None values
    history_results.map(|opts| opts.into_iter().flatten().collect())
}

fn collect_files(config: &Config) -> Vec<PathBuf> {
    let entries: Vec<PathBuf> = WalkDir::new(&config.source_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // Optimization: Skip scanning into internal directories, hidden folders, or destinations
            let name = e.file_name().to_str().unwrap_or("");
            if config.exclude_hidden && name.starts_with('.') && name != "." && name != ".." {
                return false;
            }

            // Ignore specific internal/managed files
            if config
                .ignore_patterns
                .iter()
                .any(|pattern| name == *pattern)
                || name == config.trash_dir.to_str().unwrap_or("")
                || name == config.unknown_dir.to_str().unwrap_or("")
                || name == config.history_file
            {
                return false;
            }

            // Ignore destination directories of all rules
            for rule in &config.rules {
                if name == rule.destination.to_str().unwrap_or("") {
                    return false;
                }
            }
            true
        })
        .filter_map(|e| e.ok())
        .filter(|e| e.file_type().is_file())
        .map(|e| e.path().to_path_buf())
        .collect();
    entries
}

fn process_files_in_parallel(
    entries: Vec<PathBuf>,
    config: &Config,
    dry_run: bool,
) -> Result<Vec<Option<MoveRecord>>> {
    let history_results: Result<Vec<Option<MoveRecord>>> = entries
        .into_par_iter()
        .map(|path| {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("No filename"))?;

            if is_temp_file(&path, &config.temp_extensions) {
                let trash_path: PathBuf = config.source_dir.join(&config.trash_dir).join(file_name);
                move_file(&path, &trash_path, dry_run)
            } else if let Some(rule) = find_rule_for_file(&path, &config.rules) {
                let dest_path: PathBuf = config.source_dir.join(&rule.destination).join(file_name);
                move_file(&path, &dest_path, dry_run)
            } else {
                let unknown_path: PathBuf =
                    config.source_dir.join(&config.unknown_dir).join(file_name);
                move_file(&path, &unknown_path, dry_run)
            }
        })
        .collect();
    history_results
}
