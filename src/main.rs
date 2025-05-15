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
    
    /// Check git status of all repositories and only show those with changes
    #[command(name = "status")]
    Status,
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    
    // Try to find config file in several locations
    let config_paths = [
        "mirror.toml",                              // Current directory
        "config/mirror.toml",                       // Config subdirectory
        &format!("{}/.config/mctl/mirror.toml",     // User config directory
            std::env::var("HOME").unwrap_or_else(|_| String::from("."))),
    ];
    
    let mut config = None;
    let mut tried_paths = Vec::new();
    
    for path in &config_paths {
        match config::Config::from_file(path) {
            Ok(cfg) => {
                config = Some(cfg);
                break;
            }
            Err(_) => {
                tried_paths.push(path.to_string());
            }
        }
    }
    
    let config = match config {
        Some(cfg) => cfg,
        None => {
            eprintln!("{}", output::colorize("Error: Could not find config file", "red"));
            eprintln!("Looked in the following locations:");
            for path in tried_paths {
                eprintln!("  - {}", path);
            }
            eprintln!("Please create a mirror.toml file in one of these locations.");
            std::process::exit(1);
        }
    };

    match cli.command {
        Commands::Sync => commands::sync::sync_repositories(config),
        Commands::Status => commands::status::check_status(config),
    }
}
