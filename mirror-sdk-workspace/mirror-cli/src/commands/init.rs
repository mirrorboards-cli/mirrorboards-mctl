//! Initialization command for the Mirror CLI.

use clap::Args;
use colored::Colorize;
use std::path::PathBuf;

use crate::error::CliResult;
use crate::utils::{print_success, resolve_config_path};
use mirror_sdk::MirrorConfig;

/// Arguments for the init command
#[derive(Args)]
#[command(about = "Initialize a new mirror.toml configuration file")]
pub struct InitCommand {
    /// Path to the mirror.toml file (optional)
    #[arg(short, long, value_name = "FILE", help = "Specify a custom path for the mirror.toml file")]
    config: Option<PathBuf>,
}

impl InitCommand {
    /// Execute the init command
    pub fn execute(&self) -> CliResult<()> {
        // Resolve the configuration path
        let config_path = resolve_config_path(self.config.as_deref())?;
        
        // Check if the file already exists
        if config_path.exists() {
            // Provide a more user-friendly error message
            return Err(crate::error::CliError::Other(
                format!("Configuration file already exists at '{}'. Use other commands to modify it.", config_path.display())
            ));
        }
        
        // Initialize a new configuration
        let config = MirrorConfig::init(Some(&config_path))?;
        
        print_success(&format!("Initialized empty mirror.toml at {}", config_path.display()));
        println!("You can now add repositories using 'mirror repo add <origin> <path>'");
        Ok(())
    }
}