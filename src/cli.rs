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
    /// Authentication token
    #[arg(long)]
    pub xf_user: Option<String>,
    /// Authentication token for 2FA accounts
    #[arg(long)]
    pub xf_tfa_trust: Option<String>,
}

#[derive(Args)]
pub struct CachedArgs {
    /// Select older cache by offset
    pub offset: Option<i64>,
}
