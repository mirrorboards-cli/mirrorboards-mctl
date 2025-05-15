//! # Error Types Module
//!
//! This module defines the domain-specific error types for the application.
//! It uses thiserror for defining error types and provides context-rich error information.

use std::path::PathBuf;
use thiserror::Error;

/// Root error type for repository operations
#[derive(Debug, Error)]
pub enum RepositoryError {
    /// Error when a repository is not found
    #[error("Repository not found at path: {path}")]
    RepositoryNotFound {
        /// Path where the repository was expected
        path: PathBuf,
    },
    
    /// Error when a repository already exists
    #[error("Repository already exists at path: {path}")]
    RepositoryAlreadyExists {
        /// Path where the repository exists
        path: PathBuf,
    },
    
    /// Error with repository submodules
    #[error("Submodule error: {message}")]
    SubmoduleError {
        /// Error message
        message: String,
    },
    
    /// Error when operation times out
    #[error("Repository operation timed out after {seconds} seconds")]
    OperationTimeout {
        /// Timeout in seconds
        seconds: u64,
    },
}

/// Git operation errors
#[derive(Debug, Error)]
pub enum GitError {
    /// Error cloning a repository
    #[error("Failed to clone repository from {url} to {path}: {message}")]
    CloneError {
        /// Remote URL
        url: String,
        /// Local path
        path: PathBuf,
        /// Error message
        message: String,
    },
    
    /// Error pushing changes
    #[error("Failed to push changes to remote for repository at {path}: {message}")]
    PushError {
        /// Repository path
        path: PathBuf,
        /// Error message
        message: String,
    },
    
    /// Error committing changes
    #[error("Failed to commit changes in repository at {path}: {message}")]
    CommitError {
        /// Repository path
        path: PathBuf,
        /// Error message
        message: String,
    },
    
    /// Authentication error
    #[error("Git authentication failed: {message}")]
    AuthenticationError {
        /// Error message with troubleshooting guidance
        message: String,
        /// SSH key path if applicable
        key_path: Option<PathBuf>,
    },
    
    /// Error executing Git command
    #[error("Git command failed: {command}\nOutput: {output}")]
    CommandError {
        /// Git command that failed
        command: String,
        /// Command output (stdout/stderr)
        output: String,
    },
}

/// Configuration errors
#[derive(Debug, Error)]
pub enum ConfigError {
    /// Configuration file not found
    #[error("Configuration file not found. Searched in: {search_paths:?}")]
    ConfigFileNotFound {
        /// Paths where the config was searched for
        search_paths: Vec<PathBuf>,
    },
    
    /// Error parsing configuration
    #[error("Failed to parse configuration file at {path}: {message}")]
    ConfigParseError {
        /// Path to the config file
        path: PathBuf,
        /// Error message
        message: String,
    },
    
    /// Error validating configuration
    #[error("Configuration validation error: {message}")]
    ConfigValidationError {
        /// Validation error message
        message: String,
    },
}

/// Command-related errors
#[derive(Debug, Error)]
pub enum CommandError {
    /// Invalid command arguments
    #[error("Invalid argument: {message}")]
    InvalidArgumentError {
        /// Error message
        message: String,
    },
    
    /// Error executing a command
    #[error("Command execution error: {message}")]
    CommandExecutionError {
        /// Error message
        message: String,
        /// Command that failed
        command: String,
    },
    
    /// Command timeout
    #[error("Command timed out after {seconds} seconds")]
    CommandTimeoutError {
        /// Timeout in seconds
        seconds: u64,
    },
    
    /// Command not found
    #[error("Command not found: {command}")]
    CommandNotFoundError {
        /// Command name
        command: String,
    },
}