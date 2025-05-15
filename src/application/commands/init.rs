//! # Init Command Module
//!
//! This module implements the init command, which creates a new configuration file.

use std::sync::Arc;
use anyhow::{Result, Context, anyhow};
use std::path::PathBuf;
use log::{info, error, debug, warn};

use crate::domain::repository::RepositoryOperations;
use crate::domain::configuration::Config;
use crate::presentation::output::OutputFormatter;
use crate::application::commands::Command;
use crate::presentation::cli::InitArgs;
use crate::infrastructure::filesystem::FilesystemProvider;

/// Init command implementation
pub struct InitCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Repository operations implementation
    repository_ops: Arc<O>,
    /// Output formatter
    output_formatter: Arc<F>,
    /// Application configuration
    config: Config,
    /// Command arguments
    args: InitArgs,
    /// Filesystem provider
    fs_provider: FilesystemProvider,
}

impl<O, F> InitCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    /// Create a new init command
    pub fn new(
        repository_ops: Arc<O>,
        output_formatter: Arc<F>,
        config: Config,
        args: InitArgs,
    ) -> Self {
        Self {
            repository_ops,
            output_formatter,
            config,
            args,
            fs_provider: FilesystemProvider::new(),
        }
    }
    
    /// Create a default configuration
    fn create_default_config(&self) -> Config {
        let mut config = Config::default();
        
        // Set custom SSH key path if provided
        if let Some(ssh_key) = &self.args.ssh_key {
            config.auth.ssh.key_path = Some(ssh_key.to_string_lossy().to_string());
        }
        
        config
    }
}

impl<O, F> Command for InitCommand<O, F>
where
    O: RepositoryOperations + Send + Sync + 'static,
    F: OutputFormatter + Send + Sync + 'static,
{
    fn execute(&self) -> Result<()> {
        // Check if file exists and handle force flag
        if self.fs_provider.exists(&self.args.output) && !self.args.force {
            return Err(anyhow!(
                "Configuration file already exists at {}. Use --force to overwrite.",
                self.args.output.display()
            ));
        }
        
        // Create default configuration
        let config = self.create_default_config();
        
        // Serialize to TOML
        let config_str = toml::to_string_pretty(&config)
            .context("Failed to serialize configuration to TOML")?;
        
        // Write to file
        self.fs_provider.write_file(&self.args.output, &config_str, self.args.force)
            .with_context(|| format!("Failed to write configuration to {}", self.args.output.display()))?;
        
        let formatter = &*self.output_formatter;
        let message = formatter.format_success(
            &format!("Created new configuration file at {}", self.args.output.display())
        );
        println!("{}", message);
        
        info!("Created new configuration file at {}", self.args.output.display());
        
        Ok(())
    }
    
    fn name(&self) -> &'static str {
        "init"
    }
    
    fn description(&self) -> &'static str {
        "Initialize a new configuration file"
    }
}