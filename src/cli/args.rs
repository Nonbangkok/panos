//! Command line arguments parsing

use clap::Parser;
use std::path::PathBuf;

#[derive(Parser, Debug)]
#[command(author, version, about)]
#[command(help_template = "\
{before-help}
name: {name}
description: {about}
version: {version}
author: {author-with-newline}
{usage-heading} {usage}

{all-args}
{after-help}
")]
// #[command(arg_required_else_help = true)]
pub struct Args {
    /// Path to the configuration file (panos.toml)
    #[arg(short, long, default_value = "panos.toml")]
    pub config: PathBuf,

    /// Override the source directory to organize
    #[arg(short, long)]
    pub source: Option<PathBuf>,

    /// Run without moving files (only show what would happen)
    #[arg(short, long)]
    pub dry_run: bool,

    /// Run in watch mode (background daemon)
    #[arg(short, long)]
    pub watch: bool,
}
