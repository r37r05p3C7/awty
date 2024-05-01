use std::path::PathBuf;

use clap::{Args, Parser, Subcommand};

#[derive(Parser)]
#[command(version, about, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Check for updates
    Check(CheckArgs),
}

#[derive(Args)]
pub struct CheckArgs {
    /// File with thread links
    pub file: PathBuf,
    /// Force check
    #[arg(short, long)]
    pub force: bool,
}
