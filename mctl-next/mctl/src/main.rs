mod cli;
mod commands;

use anyhow::Result;
use clap::Parser;

use cli::{Cli, Commands, ConfigCommands, SnapshotCommands};

fn main() -> Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Sync { workspace } => {
            commands::sync::run(&cli.config, workspace.as_deref())?;
        }
        Commands::List {
            workspace,
            by_workspace,
        } => {
            commands::list::run(&cli.config, workspace.as_deref(), by_workspace)?;
        }
        Commands::Save { workspace, message } => {
            commands::save::run(&cli.config, &workspace, message.as_deref())?;
        }
        Commands::Config { command } => match command {
            ConfigCommands::Save { message } => {
                commands::config::save::run(&cli.config, &message)?;
            }
            ConfigCommands::Pull => {
                commands::config::pull::run(&cli.config)?;
            }
        },
        Commands::Snapshot { command } => match command {
            SnapshotCommands::Create { name, description } => {
                commands::snapshot::create::run(&cli.config, &name, description.as_deref())?;
            }
            SnapshotCommands::List => {
                commands::snapshot::list::run(&cli.config)?;
            }
            SnapshotCommands::Restore { name } => {
                commands::snapshot::restore::run(&cli.config, &name)?;
            }
        },
    }

    Ok(())
}
