mod config;
mod commands;
mod git;
mod output;

use clap::{Parser, Subcommand};
use anyhow::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Sync repositories defined in mirror.toml with submodule support
    /// Clones repositories with their submodules and updates submodules in existing repositories
    Sync,
}

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync => {
            let config = config::Config::from_file("mirror.toml")?;
            commands::sync::sync_repositories(config)
        }
    }
}
