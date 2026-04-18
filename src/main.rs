//! PANOS: Universal File Organizer OS

use anyhow::Result;
use clap::Parser;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use panos::{
    Args, Config, load_config, organize, remove_empty_dirs, watch_mode,
    file_ops::{MoveRecord, Session},
    organizer::run_undo,
};

fn main() -> Result<()> {
    // Initialize logging
    let subscriber: FmtSubscriber = FmtSubscriber::builder()
        .with_max_level(Level::INFO)
        .finish();
    tracing::subscriber::set_global_default(subscriber)?;

    let args: Args = Args::parse();

    info!("Starting PANOS...");
    info!("Config file: {:?}", args.config);

    // Load config
    let mut config: Config = load_config(&args.config)?;

    // CLI override for source directory
    if let Some(source) = args.source {
        config.source_dir = source;
    }
    info!("Source directory: {:?}", config.source_dir);

    // Undo operation
    if args.undo {
        run_undo(&config)?;
        remove_empty_dirs(&config.source_dir, args.dry_run)?;
        return Ok(());
    }

    if args.dry_run {
        info!("Dry run mode enabled. Files will not be moved.");
    }

    let history: Vec<MoveRecord> = organize(&config, args.dry_run)?;

    if !history.is_empty() {
        let session = Session { moves: history };
        session.save(&config.source_dir, &config.history_file)?;
        info!("History saved. You can undo this operation with --undo");
    }

    remove_empty_dirs(&config.source_dir, args.dry_run)?;

    if args.watch {
        watch_mode(&config, args.dry_run)?;
    }

    Ok(())
}
