//! Command implementation module
//!
//! This module implements the actual functionality of the CLI commands,
//! handling the business logic and integration with the mirror-sdk.

mod init;
mod repo;
mod tag;
mod config;

pub use init::*;
pub use repo::*;
pub use tag::*;
pub use config::*;

use crate::cli;
use crate::output::OutputFormatter;
use mirror_sdk::Error as SdkError;
use thiserror::Error;

/// Error type for command execution
#[derive(Debug, Error)]
pub enum CommandError {
    /// Error from the mirror-sdk
    #[error("SDK error: {0}")]
    Sdk(#[from] SdkError),

    /// Error from user input
    #[error("Input error: {0}")]
    Input(String),

    /// Error from file operations
    #[error("File error: {0}")]
    File(String),

    /// Error from configuration
    #[error("Configuration error: {0}")]
    Config(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Result type for command execution
pub type CommandResult<T> = Result<T, CommandError>;

/// Execute the CLI command
pub fn execute(cli: cli::Cli, formatter: &mut dyn OutputFormatter) -> CommandResult<()> {
    // Set up logging based on verbosity
    if cli.verbose {
        std::env::set_var("RUST_LOG", "debug");
    } else {
        std::env::set_var("RUST_LOG", "info");
    }
    env_logger::init();

    // Execute the appropriate command
    match cli.command {
        cli::Commands::Init(args) => init::execute(args, formatter, cli.config.clone()),
        cli::Commands::Repo(args) => repo::execute(args, formatter, cli.config.clone()),
        cli::Commands::Tag(args) => tag::execute(args, formatter, cli.config.clone()),
        cli::Commands::Config(args) => config::execute(args, formatter, cli.config.clone()),
    }
}