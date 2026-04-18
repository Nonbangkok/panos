//! Watcher logic

use anyhow::{Context, Result};
use notify::{Config as NotifyConfig, Event, RecommendedWatcher, RecursiveMode, Watcher};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{channel, Receiver};
use std::time::{Duration, Instant};
use tracing::{error, info, warn, debug};

use crate::config::Config;
use crate::organizer::organize;

/// Global lock to prevent the watcher from reacting to our own file movements
static IS_ORGANIZING: AtomicBool = AtomicBool::new(false);

pub fn watch_mode(config: &Config, dry_run: bool) -> Result<()> {
    let source_dir = &config.source_dir;
    
    let (tx, rx) = channel();

    let mut watcher = RecommendedWatcher::new(tx, NotifyConfig::default())
        .context("Failed to create watcher")?;

    watcher.watch(source_dir, RecursiveMode::Recursive)
        .context(format!("Failed to watch directory: {:?}", source_dir))?;

    info!("👀 Watching for changes in: {:?}", source_dir);

    run_event_loop(rx, config, dry_run)
}

fn run_event_loop(rx: Receiver<Result<Event, notify::Error>>, config: &Config, dry_run: bool) -> Result<()> {
    let mut last_event_time = None;
    let debounce_duration = Duration::from_secs(2);

    loop {
        match rx.recv_timeout(Duration::from_millis(500)) {
            Ok(Ok(event)) => {
                // 1. Check if we are currently organizing (Atomic Lock)
                if IS_ORGANIZING.load(Ordering::SeqCst) {
                    continue;
                }

                // 2. Filter out unnecessary events (Path Exclusion)
                if should_ignore(&event, config) {
                    continue;
                }

                if event.kind.is_modify() || event.kind.is_create() {
                    debug!("Event detected: {:?}", event.kind);
                    last_event_time = Some(Instant::now());
                }
            }
            Ok(Err(e)) => warn!("Watcher error: {:?}", e),
            Err(_) => {
                if let Some(last_time) = last_event_time {
                    if last_time.elapsed() >= debounce_duration {
                        info!("🚀 Change detected and stabilized. Running organization...");
                        
                        // Set lock before starting
                        IS_ORGANIZING.store(true, Ordering::SeqCst);
                        
                        if let Err(e) = organize(config, dry_run) {
                            error!("Organization failed during watch mode: {:?}", e);
                        }
                        
                        // Release lock after finishing
                        IS_ORGANIZING.store(false, Ordering::SeqCst);
                        
                        last_event_time = None;
                    }
                }
            }
        }
    }
}

/// Determines if an event should be ignored to prevent loops or noise
fn should_ignore(event: &Event, config: &Config) -> bool {
    static IGNORE_PATTERNS: &[&str] = &[
        ".DS_Store", ".git", ".svn", ".hg",
        "Thumbs.db","desktop.ini",
        "node_modules", "target", ".vscode"
    ];

    for path in &event.paths {
        if let Some(name) = path.file_name().and_then(|n| n.to_str()) {
            // 1. Ignore specific patterns AND any hidden files/folders (dotfiles)
            if name.starts_with('.') || IGNORE_PATTERNS.iter().any(|pattern| name == *pattern) {
                return true;
            }
        }

        // 2. Ignore destination paths (if they are inside source_dir)
        for rule in &config.rules {
            let dest_path = config.source_dir.join(&rule.destination);
            if path.starts_with(&dest_path) {
                return true;
            }
        }
    }
    false
}
