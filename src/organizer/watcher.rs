//! Watcher logic

use anyhow::{Context, Result};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{Receiver, channel};
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

use crate::config::Config;
use crate::file_ops::{remove_empty_dirs, Session};
use crate::organizer::organize;

/// Global lock to prevent the watcher from reacting to our own file movements
static IS_ORGANIZING: AtomicBool = AtomicBool::new(false);

pub fn watch_mode(config: &Config, dry_run: bool) -> Result<()> {
    let source_dir = &config.source_dir;

    let (tx, rx) = channel();

    let mut watcher =
        RecommendedWatcher::new(tx, NotifyConfig::default()).context("Failed to create watcher")?;

    watcher
        .watch(source_dir, RecursiveMode::Recursive)
        .context(format!("Failed to watch directory: {:?}", source_dir))?;

    info!("👀 Watching for changes in: {:?}", source_dir);

    run_event_loop(rx, config, dry_run)
}

fn run_event_loop(
    rx: Receiver<Result<Event, notify::Error>>,
    config: &Config,
    dry_run: bool,
) -> Result<()> {
    let mut last_event_time = None;
    let debounce_duration = Duration::from_secs(config.debounce_seconds);

    loop {
        match rx.recv_timeout(Duration::from_millis(config.polling_interval_ms)) {
            Ok(Ok(event)) => {
                debug!(
                    "Event detected: {:?} for paths: {:?}",
                    event.kind, event.paths
                );

                // Check if we are currently organizing (Atomic Lock) and Filter out unnecessary events (Path Exclusion)
                if IS_ORGANIZING.load(Ordering::SeqCst) || should_ignore(&event, config) {
                    continue;
                }

                // Check if the event is a file modification or creation
                if event.kind.is_modify() || event.kind.is_create() {
                    if event.paths.iter().any(|p| p.is_file()) {
                        debug!("Event detected: {:?}", event.kind);
                        last_event_time = Some(Instant::now());
                    }
                }
            }
            Ok(Err(e)) => warn!("Watcher error: {:?}", e),
            Err(_) => {
                if let Some(last_time) = last_event_time {
                    if last_time.elapsed() >= debounce_duration {
                        process_stabilized_events(config, dry_run);
                        last_event_time = None;
                    }
                }
            }
        }
    }
}

/// Determines if an event should be ignored.
/// An event is ignored ONLY if ALL paths in it are considered noise (trash, destinations, etc.)
pub fn should_ignore(event: &Event, config: &Config) -> bool {
    let abs_source = std::fs::canonicalize(&config.source_dir).unwrap_or(config.source_dir.clone());
    let abs_trash = abs_source.join(&config.trash_dir);
    let abs_unknown = abs_source.join(&config.unknown_dir);
    let abs_history = abs_source.join(&config.history_file);

    // If all paths in the event match ignore criteria, then we ignore the whole event.
    // If even one path is "valid" (should be organized), we return false to trigger organize.
    event.paths.iter().all(|path| {
        let abs_path = std::fs::canonicalize(path).unwrap_or(path.to_path_buf());

        // 1. Ignore source directory itself
        if abs_path == abs_source {
            return true;
        }

        // 2. Ignore internal managed directories (trash, unknown, history)
        if abs_path.starts_with(&abs_trash)
            || abs_path.starts_with(&abs_unknown)
            || abs_path == abs_history
        {
            return true;
        }

        // 3. Ignore hidden files and user-defined ignore patterns
        if let Some(name) = abs_path.file_name().and_then(|n| n.to_str()) {
            if config.ignore_patterns.iter().any(|p| name == *p) {
                return true;
            }
            if config.exclude_hidden && name.starts_with('.') {
                return true;
            }
        }

        // 4. Ignore destination paths (to prevent trigger recursion when moving files)
        for rule in &config.rules {
            let dest_path = abs_source.join(&rule.destination);
            if abs_path.starts_with(&dest_path) {
                return true;
            }
        }

        false
    })
}

fn process_stabilized_events(config: &Config, dry_run: bool) {
    info!("🚀 Change detected and stabilized. Running organization...");

    // Set lock before starting
    IS_ORGANIZING.store(true, Ordering::SeqCst);

    match organize(config, dry_run) {
        Ok(history) if !history.is_empty() => {
            let mut session =
                Session::load(&config.source_dir, &config.history_file).unwrap_or_default();

            session.moves.extend(history);

            if let Err(e) = session.save(&config.source_dir, &config.history_file) {
                error!("Failed to save history: {}", e);
            }
            info!("History updated in watch mode.");

            // Clean up empty directories
            if let Err(e) = remove_empty_dirs(&config.source_dir, dry_run, &session.moves) {
                error!("Failed to clean up empty directories: {}", e);
            }
        }
        Err(e) => error!("Organization failed during watch mode: {:?}", e),
        _ => {}
    }

    // Release lock after finishing
    IS_ORGANIZING.store(false, Ordering::SeqCst);
}
