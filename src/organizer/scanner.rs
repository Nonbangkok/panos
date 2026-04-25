//! Directory scanning and organization logic

use anyhow::Result;
use rayon::prelude::*;
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::info;
use walkdir::WalkDir;

use crate::config::Config;
use crate::file_ops::hashing::*;
use crate::file_ops::history::MoveRecord;
use crate::file_ops::move_file;
use crate::rules::ai::PanosAI;
use crate::rules::{find_rule_for_file, is_temp_file};
use crate::ui::ProgressReporter;

/// Organize files in the source directory according to rules
pub fn organize(
    config: &Config,
    dry_run: bool,
    reporter: &dyn ProgressReporter,
    ai: &mut Option<PanosAI>,
) -> Result<Vec<MoveRecord>> {
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
    let entries = collect_files(config, reporter);

    // 2. Identify duplicates
    let (unique_entries, duplicates) = identify_duplicates(entries, reporter);

    let mut history = Vec::new();

    // 3. Process all entries in parallel using Rayon (Rule-based)
    let results = process_files_in_parallel(unique_entries, config, dry_run, reporter)?;

    // 4. Separate processed files and pending files for AI
    let mut pending_files = Vec::new();
    for (path, res) in results {
        if let Some(record) = res {
            history.push(record);
        } else {
            pending_files.push(path);
        }
    }

    // 5. Process pending files with AI
    let (ai_history, still_pending) =
        process_files_with_ai(pending_files, config, dry_run, reporter, ai)?;
    history.extend(ai_history);

    // 6. Move any remaining files to Unknown directory
    let unknown_history = process_unknown_files(still_pending, config, dry_run, reporter)?;
    history.extend(unknown_history);

    let duplicate_results = process_duplicates(duplicates, config, dry_run, reporter)?;
    history.extend(duplicate_results.into_iter().flatten());

    Ok(history)
}

fn identify_duplicates(
    entries: Vec<PathBuf>,
    reporter: &dyn ProgressReporter,
) -> (Vec<PathBuf>, Vec<(PathBuf, PathBuf)>) {
    reporter.start(None, "Identifying duplicate files...".to_string());

    // ด่านที่ 1: จัดกลุ่มตามขนาด (ประหยัดพลังงานที่สุด)
    let mut by_size: HashMap<u64, Vec<PathBuf>> = HashMap::new();
    for path in &entries {
        if let Ok(size) = get_file_size(&path) {
            by_size.entry(size).or_default().push(path.clone());
        }
    }

    let mut unique_entries = Vec::new();
    let mut duplicates = Vec::new();

    for (_size, paths) in by_size {
        // Unique Check (Size)
        if paths.len() == 1 {
            unique_entries.push(paths[0].clone());
            continue;
        }

        // Partial Check (First 4KB)
        let mut by_partial: HashMap<String, Vec<PathBuf>> = HashMap::new();
        for path in &paths {
            reporter.update(
                0,
                format!(
                    "⚡️ Analyzing: {}",
                    path.file_name()
                        .unwrap_or_default()
                        .to_str()
                        .unwrap_or("unknown")
                ),
            );
            if let Ok(h) = calculate_partial_hash(&path) {
                by_partial.entry(h).or_default().push(path.clone());
            } else {
                unique_entries.push(path.clone());
            }
        }

        // Final Check (Full Hash)
        for (_p_hash, p_paths) in by_partial {
            if p_paths.len() == 1 {
                unique_entries.push(p_paths[0].clone());
                continue;
            }

            let mut by_full: HashMap<String, Vec<PathBuf>> = HashMap::new();
            for path in &p_paths {
                reporter.update(
                    0,
                    format!(
                        "⚡️ Analyzing: {}",
                        path.file_name()
                            .unwrap_or_default()
                            .to_str()
                            .unwrap_or("unknown")
                    ),
                );
                if let Ok(f_hash) = calculate_full_hash(&path) {
                    by_full.entry(f_hash).or_default().push(path.clone());
                } else {
                    unique_entries.push(path.clone());
                }
            }

            for (_f_hash, f_paths) in by_full {
                // Unique file
                unique_entries.push(f_paths[0].clone());
                // Duplicate files
                for dup in f_paths.iter().skip(1) {
                    reporter.update(
                        0,
                        format!(
                            "Found duplicate: {}",
                            dup.file_name()
                                .unwrap_or_default()
                                .to_str()
                                .unwrap_or("unknown")
                        ),
                    );
                    duplicates.push((dup.clone(), f_paths[0].clone()));
                }
            }
        }
    }

    reporter.finish(format!(
        "Found {} unique files, {} duplicates",
        unique_entries.len(),
        duplicates.len()
    ));
    (unique_entries, duplicates)
}

fn collect_files(config: &Config, reporter: &dyn ProgressReporter) -> Vec<PathBuf> {
    reporter.start(None, "Scanning directory...".to_string());

    let entries: Vec<PathBuf> = WalkDir::new(&config.source_dir)
        .min_depth(1)
        .into_iter()
        .filter_entry(|e| {
            // Optimization: Skip scanning into internal directories, hidden folders, or destinations
            let name = e.file_name().to_str().unwrap_or("");

            reporter.update(0, format!("Scanning: {}", name));

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
    reporter.finish(format!(
        "Scan complete. Found {} candidates.",
        entries.len()
    ));
    entries
}

fn process_files_in_parallel(
    entries: Vec<PathBuf>,
    config: &Config,
    dry_run: bool,
    reporter: &dyn ProgressReporter,
) -> Result<Vec<(PathBuf, Option<MoveRecord>)>> {
    let total_files = entries.len() as u64;
    reporter.start(Some(total_files), "Organizing files...".to_string());

    let history_results: Result<Vec<(PathBuf, Option<MoveRecord>)>> = entries
        .into_par_iter()
        .map(|path| {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("No filename"))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename encoding"))?;

            let result = if is_temp_file(&path, &config.temp_extensions) {
                let trash_path: PathBuf = config.source_dir.join(&config.trash_dir).join(file_name);
                move_file(&path, &trash_path, dry_run)
            } else if let Some(rule) = find_rule_for_file(&path, &config.rules) {
                let dest_path: PathBuf = config.source_dir.join(&rule.destination).join(file_name);
                move_file(&path, &dest_path, dry_run)
            } else {
                Ok(None)
            };

            reporter.update(1, "".to_string());
            result.map(|res| (path, res))
        })
        .collect();

    reporter.finish("Processing complete!".to_string());
    history_results
}

fn process_files_with_ai(
    pending_files: Vec<PathBuf>,
    config: &Config,
    dry_run: bool,
    reporter: &dyn ProgressReporter,
    ai: &mut Option<PanosAI>,
) -> Result<(Vec<MoveRecord>, Vec<PathBuf>)> {
    let mut history = Vec::new();
    let mut still_pending = Vec::new();

    if pending_files.is_empty() {
        return Ok((history, still_pending));
    }

    if let Some(ai_engine) = ai {
        info!("Processing {} files with AI...", pending_files.len());
        for path in pending_files {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("No filename"))?
                .to_str()
                .ok_or_else(|| anyhow::anyhow!("Invalid filename encoding"))?;

            if let Some(rule) = ai_engine.determine_rule(file_name, config, &config.rules) {
                let dest_path = config.source_dir.join(&rule.destination).join(file_name);
                if let Ok(Some(record)) = move_file(&path, &dest_path, dry_run) {
                    history.push(record);
                }
                reporter.update(1, "".to_string());
            } else {
                still_pending.push(path);
            }
        }
    } else {
        still_pending = pending_files;
    }

    Ok((history, still_pending))
}

fn process_unknown_files(
    pending_files: Vec<PathBuf>,
    config: &Config,
    dry_run: bool,
    reporter: &dyn ProgressReporter,
) -> Result<Vec<MoveRecord>> {
    let mut history = Vec::new();

    if pending_files.is_empty() {
        return Ok(history);
    }

    info!("Moving {} unknown files...", pending_files.len());
    for path in pending_files {
        let file_name = path
            .file_name()
            .ok_or_else(|| anyhow::anyhow!("No filename"))?
            .to_str()
            .ok_or_else(|| anyhow::anyhow!("Invalid filename encoding"))?;

        let unknown_path = config.source_dir.join(&config.unknown_dir).join(file_name);
        if let Ok(Some(record)) = move_file(&path, &unknown_path, dry_run) {
            history.push(record);
        }
        reporter.update(1, "".to_string());
    }

    Ok(history)
}

fn process_duplicates(
    duplicates: Vec<(PathBuf, PathBuf)>,
    config: &Config,
    dry_run: bool,
    reporter: &dyn ProgressReporter,
) -> Result<Vec<Option<MoveRecord>>> {
    if duplicates.is_empty() {
        return Ok(vec![]);
    }

    reporter.start(
        Some(duplicates.len() as u64),
        "Moving duplicates...".to_string(),
    );

    let results: Result<Vec<Option<MoveRecord>>> = duplicates
        .into_par_iter()
        .map(|(path, _original)| {
            let file_name = path
                .file_name()
                .ok_or_else(|| anyhow::anyhow!("No filename"))?;

            let dest_path = config
                .source_dir
                .join(&config.duplicates_dir)
                .join(file_name);

            let res = move_file(&path, &dest_path, dry_run);
            reporter.update(1, "".to_string());
            res
        })
        .collect();

    reporter.finish("Duplicates processed!".to_string());
    results
}
