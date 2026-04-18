//! PANOS: Universal File Organizer OS

use anyhow::Result;
use clap::Parser;
use tracing::{Level, info};
use tracing_subscriber::FmtSubscriber;

use panos::{Args, Config, load_config, organize, remove_empty_dirs};

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

    if args.dry_run {
        info!("Dry run mode enabled. Files will not be moved.");
    }

    organize(&config, args.dry_run)?;
    remove_empty_dirs(&config.source_dir, args.dry_run)?;

    Ok(())
}
