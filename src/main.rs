use clap::Parser;
use color_eyre::eyre::Result;

use crate::cli::{Cli, Commands};

mod cli;
mod commands;
mod parsing;

fn main() -> Result<()> {
    color_eyre::install()?;
    let cli = Cli::parse();
    match &cli.command {
        Commands::Check(args) => commands::check(&args.file)?,
    }
    Ok(())
}
