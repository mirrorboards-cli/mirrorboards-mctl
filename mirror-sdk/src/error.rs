//! Error types for mirror-sdk
//!
//! This module provides the error types used throughout the mirror-sdk library.

use thiserror::Error;
use std::path::PathBuf;

/// Errors that can occur when working with mirror configurations
#[derive(Error, Debug)]
pub enum Error {
    /// Error occurred during I/O operations
    #[error("I/O error: {0}")]
    Io(#[from] std::io::Error),

    /// Error occurred during TOML deserialization
    #[error("TOML parsing error: {0}")]
    TomlDe(#[from] toml::de::Error),

    /// Error occurred during TOML serialization
    #[error("TOML serialization error: {0}")]
    TomlSer(#[from] toml::ser::Error),

    /// Required field is missing
    #[error("Missing required field: {0}")]
    MissingField(String),

    /// Repository with the given ID already exists
    #[error("Repository with ID '{0}' already exists")]
    DuplicateId(String),

    /// Repository with the given path already exists
    #[error("Repository with path '{0}' already exists")]
    DuplicatePath(String),

    /// Repository with the given ID was not found
    #[error("Repository with ID '{0}' not found")]
    RepositoryNotFound(String),

    /// Configuration file not found
    #[error("Configuration file not found at {0}")]
    ConfigNotFound(PathBuf),

    /// Invalid configuration file
    #[error("Invalid configuration file: {0}")]
    InvalidConfig(String),

    /// Other errors
    #[error("{0}")]
    Other(String),
}

/// Result type for mirror-sdk operations
pub type Result<T> = std::result::Result<T, Error>;