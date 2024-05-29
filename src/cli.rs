use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None, disable_help_subcommand(true))]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check for updates
    Check(CheckArgs),
    /// Show cached results
    Cached(CachedArgs),
}

#[derive(Args)]
pub struct CheckArgs {
    /// File with thread links
    pub file: PathBuf,
    /// Force check
    #[arg(short, long)]
    pub force: bool,
}

#[derive(Args)]
pub struct CachedArgs {
    /// Select older cache by offset
    pub offset: Option<i64>,
}
