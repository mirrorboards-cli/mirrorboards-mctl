//! Error types for the Mirror CLI.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Result type for CLI operations
pub type CliResult<T> = Result<T, CliError>;

/// Errors that can occur when using the Mirror CLI
#[derive(Error, Debug)]
pub enum CliError {
    /// Error when the SDK returns an error
    #[error("SDK error: {0}")]
    SdkError(#[from] mirror_sdk::MirrorError),

    /// Error when a repository is not found
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),

    /// Error when a repository already exists
    #[error("Repository already exists: {0}")]
    RepositoryAlreadyExists(String),

    /// Error when a required argument is missing
    #[error("Missing required argument: {0}")]
    MissingArgument(String),

    /// Error when an invalid argument is provided
    #[error("Invalid argument: {0}")]
    InvalidArgument(String),

    /// Error when a configuration file is not found
    #[error("Configuration file not found at '{0}'")]
    ConfigFileNotFound(PathBuf),

    /// Error when an IO operation fails
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Error when a command fails
    #[error("Command failed: {0}")]
    CommandFailed(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Convert anyhow::Error to CliError
impl From<anyhow::Error> for CliError {
    fn from(err: anyhow::Error) -> Self {
        CliError::Other(err.to_string())
    }
}