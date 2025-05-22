//! Mirror CLI
//!
//! A command-line interface for managing mirror.toml configuration files.
//! This CLI leverages the mirror-sdk to provide functionality for repository management.

mod commands;
mod error;
mod utils;

use clap::{Parser, Subcommand};
use std::process;

use crate::utils::print_error;

use crate::commands::init::InitCommand;
use crate::commands::repo::RepoCommand;

/// Mirror CLI - A tool for managing repository mirrors
#[derive(Parser)]
#[command(name = "mirror", about = "CLI for managing mirror.toml files")]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The subcommand to execute
    #[command(subcommand)]
    command: Commands,
}

/// Subcommands for the Mirror CLI
#[derive(Subcommand)]
enum Commands {
    /// Initialize a new mirror.toml file
    #[command(about = "Initialize a new mirror.toml configuration file")]
    Init(InitCommand),
    
    /// Repository management commands
    #[command(about = "Manage repositories in mirror.toml")]
    Repo(RepoCommand),
    // Additional modules can be added here in the future
}

fn main() {
    // Initialize logger
    env_logger::init();

    // Parse command-line arguments
    let cli = Cli::parse();

    // Execute the appropriate command
    let result = match &cli.command {
        Commands::Init(cmd) => cmd.execute(),
        Commands::Repo(cmd) => cmd.execute(),
        // Additional modules can be handled here in the future
    };

    // Handle the result
    if let Err(err) = result {
        print_error(&err.to_string());
        process::exit(1);
    }
}