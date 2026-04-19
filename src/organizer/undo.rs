//! Undo logic with file history

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::config::Config;
use crate::file_ops::{Session, move_file};
use crate::ui::reporter::ProgressReporter;

pub fn run_undo(config: &Config, dry_run: bool, reporter: &dyn ProgressReporter) -> Result<()> {
    // 1. Load history log
    let session = Session::load(&config.source_dir, &config.history_file)
        .context("Failed to load history log")?;

    if session.moves.is_empty() {
        info!("No recent moves found to undo.");
        let history_path = config.source_dir.join(&config.history_file);
        if !dry_run {
            if history_path.exists() {
                std::fs::remove_file(history_path)?;
            }
        }
        return Ok(());
    }

    if dry_run {
        info!(
            "[DRY RUN] Would undo {} file movements...",
            session.moves.len()
        );
    } else {
        info!("Undoing {} file movements...", session.moves.len());
    }

    reporter.start(
        Some(session.moves.len() as u64),
        "Undoing file movements...".to_string(),
    );

    // 2. Loop in reverse (Important: Use .rev() to move the latest file back first)
    for record in session.moves.iter().rev() {
        reporter.update(
            1,
            format!("Undoing: {:?} -> {:?}", record.destination, record.source),
        );
        if record.destination.exists() {
            if dry_run {
                info!(
                    "[DRY RUN] Would restore: {:?} -> {:?}",
                    record.destination, record.source
                );
            }

            if !dry_run {
                move_file(&record.destination, &record.source, dry_run)?;
            }
        } else {
            warn!(
                "Could not find file at {:?}, skipping...",
                record.destination
            );
        }
    }

    // 3. After Undo is completed, delete the history file to prevent duplication
    let history_path = config.source_dir.join(&config.history_file);
    if !dry_run {
        std::fs::remove_file(history_path)?;
    }

    reporter.finish("Undo operation completed.".to_string());

    info!("Undo operation completed.");
    Ok(())
}
