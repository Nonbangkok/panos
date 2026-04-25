//! PANOS: Universal File Organizer OS

use anyhow::Result;
use clap::Parser;
use tracing::info;
use tracing_subscriber::{EnvFilter, FmtSubscriber};

use panos::{
    Args, Config,
    file_ops::{MoveRecord, Session, check_integrity, remove_empty_dirs},
    organizer::{organize, run_undo, watch_mode},
    rules::PanosAI,
    ui::IndicatifReporter,
};

fn main() -> Result<()> {
    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("off"));

    let subscriber = FmtSubscriber::builder().with_env_filter(filter).finish();

    tracing::subscriber::set_global_default(subscriber)?;

    let reporter = IndicatifReporter::new();

    let args: Args = Args::parse();

    info!("Starting PANOS...");
    info!("Config file: {:?}", args.config);

    // Load config
    let mut config = Config::load(&args.config)?;

    let mut ai_engine = if config.ai_enabled {
        Some(PanosAI::new(
            config.model_dir.to_str().ok_or(anyhow::anyhow!(
                "Failed to convert model directory to string"
            ))?,
            &config.rules,
        )?)
    } else {
        None
    };

    // CLI override for source directory
    if let Some(source) = args.source {
        config.source_dir = source;
    }
    info!("Source directory: {:?}", config.source_dir);

    if args.dry_run {
        info!("Dry run mode enabled. Files will not be moved.");
    }

    // Undo operation
    if args.undo {
        run_undo(&config, args.dry_run, &reporter)?;
        remove_empty_dirs(&config.source_dir, args.dry_run, &[], &reporter)?;
        return Ok(());
    }

    let history: Vec<MoveRecord> = organize(&config, args.dry_run, &reporter, &mut ai_engine)?;

    check_integrity(&history, args.dry_run, &reporter)?;

    if !args.dry_run && !history.is_empty() {
        let mut session = Session::load(&config.source_dir, &config.history_file)?;
        session.moves.extend(history.clone());
        session.save(&config.source_dir, &config.history_file)?;
        info!("History saved. You can undo this operation with --undo");
    }

    remove_empty_dirs(&config.source_dir, args.dry_run, &history, &reporter)?;

    if args.watch {
        watch_mode(&config, args.dry_run, &mut ai_engine)?;
    }

    info!("PANOS completed successfully.");

    Ok(())
}
