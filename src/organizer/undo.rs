//! Undo logic with file history

use anyhow::{Context, Result};
use tracing::{info, warn};

use crate::config::Config;
use crate::file_ops::{Session, move_file};

pub fn run_undo(config: &Config, dry_run: bool) -> Result<()> {
    // 1. Load history log
    let session = Session::load(&config.source_dir, &config.history_file)
        .context("Failed to load history log")?;

    if session.moves.is_empty() {
        info!("No recent moves found to undo.");
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

    // 2. Loop in reverse (Important: Use .rev() to move the latest file back first)
    for record in session.moves.iter().rev() {
        if record.destination.exists() {
            if dry_run {
                info!(
                    "[DRY RUN] Would restore: {:?} -> {:?}",
                    record.destination, record.source
                );
            }

            if !dry_run {
                if let Some(original_dir) = record.source.parent() {
                    move_file(&record.destination, original_dir, dry_run)?;
                }
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

    info!("Undo operation completed.");
    Ok(())
}
