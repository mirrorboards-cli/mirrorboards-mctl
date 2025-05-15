//! # CLI Runner Module
//!
//! This module provides the high-level functionality for running the CLI commands.
//! It connects the CLI argument parsing with the command execution and output formatting.

use std::path::PathBuf;
use std::sync::Arc;
use anyhow::{Result, Context};
use log::{info, error, debug};

use crate::infrastructure::config::ConfigProvider;
use crate::infrastructure::logging::LoggingSetup;
use crate::infrastructure::git::GitOperations;
use crate::application::orchestrator::CommandOrchestrator;
use crate::domain::repository::RepositoryOperations;
use crate::presentation::output::{OutputFormatter, OutputFormat, ColorMode, TextFormatter};
use crate::presentation::cli::{Cli, Commands};

/// CLI Runner for the MCTL application
pub struct CliRunner {
    /// Command line arguments
    args: Cli,
    /// Logging setup
    logging: LoggingSetup,
}

impl CliRunner {
    /// Create a new CLI runner with the given arguments
    pub fn new(args: Cli) -> Self {
        Self {
            args,
            logging: LoggingSetup::new(),
        }
    }
    
    /// Run the CLI application
    pub fn run(&self) -> Result<()> {
        // Initialize logging
        self.logging.init_logging(&self.args.verbose)
            .context("Failed to initialize logging")?;
        
        info!("Starting MCTL version {}", env!("CARGO_PKG_VERSION"));
        debug!("Verbose level: {}", self.args.verbose);
        
        // Initialize output formatting
        let color_mode = self.args.color.unwrap_or(ColorMode::Auto);
        let output_format = self.args.format.unwrap_or(OutputFormat::Text);
        
        info!("Output format: {:?}, Color mode: {:?}", output_format, color_mode);
        
        // Load configuration
        let config_provider = ConfigProvider::new(self.args.config_path.clone());
        let config = self.load_configuration(&config_provider)?;
        
        // Execute command based on output format
        match output_format {
            OutputFormat::Text => {
                let formatter = Arc::new(TextFormatter::new(color_mode));
                self.execute_command(config, formatter)?;
            },
            // For other output formats, we fall back to text for now
            _ => {
                let formatter = Arc::new(TextFormatter::new(color_mode));
                self.execute_command(config, formatter)?;
            }
        }
        
        Ok(())
    }
    
    /// Load configuration from file or create default
    fn load_configuration(&self, config_provider: &ConfigProvider) -> Result<crate::domain::configuration::Config> {
        let config_path_str = self.args.config_path.as_ref()
            .map(|p| p.display().to_string())
            .unwrap_or_else(|| "default location".to_string());
        
        match config_provider.load_config() {
            Ok(config) => {
                info!("Configuration loaded successfully from {}", config_path_str);
                Ok(config)
            },
            Err(err) => {
                error!("Failed to load configuration: {}", err);
                // Only show configuration error message if not running the init command
                if !matches!(self.args.command, Commands::Init(_)) {
                    eprintln!("Configuration error: {}. Run 'mctl init' to create a new configuration file.", err);
                }
                Ok(config_provider.get_default_config())
            }
        }
    }
    
    /// Execute command with specified formatter type
    fn execute_command<F>(&self, config: crate::domain::configuration::Config, formatter: Arc<F>) -> Result<()>
    where
        F: OutputFormatter + Send + Sync + 'static,
    {
        // Create dependencies for commands
        let git_operations = Arc::new(GitOperations::new());
        
        // Convert command enum to string and args for the orchestrator
        let (command_name, command_args) = match &self.args.command {
            Commands::Sync(_) => ("sync".to_string(), self.args.args.clone()),
            Commands::Status(_) => ("status".to_string(), self.args.args.clone()),
            Commands::Save(_) => ("save".to_string(), self.args.args.clone()),
            Commands::Init(_) => ("init".to_string(), self.args.args.clone()),
        };
        
        // Create command orchestrator with GitOperations as the concrete type
        let orchestrator = CommandOrchestrator::<GitOperations, F>::new(
            config,
            git_operations,
            formatter
        );
        
        // Execute the specified command
        info!("Executing command: {} with args: {:?}", command_name, command_args);
        match orchestrator.execute_command(command_name, command_args) {
            Ok(_) => {
                info!("Command executed successfully");
                Ok(())
            },
            Err(err) => {
                error!("Command execution failed: {}", err);
                // Display detailed error chain for better diagnostics
                let mut source = err.source();
                let mut depth = 1;
                while let Some(error) = source {
                    error!("  Caused by ({}): {}", depth, error);
                    depth += 1;
                    source = error.source();
                }
                Err(err)
            }
        }
    }
    
    /// Display progress for a long-running operation
    pub fn show_progress(&self, message: &str, current: usize, total: usize) {
        // This would be implemented based on the output format
        // For now, we just log it
        info!("Progress: {} [{}/{}]", message, current, total);
    }
    
    /// Display completion message for an operation
    pub fn show_completion(&self, message: &str) {
        // This would be implemented based on the output format
        info!("Completed: {}", message);
    }
}