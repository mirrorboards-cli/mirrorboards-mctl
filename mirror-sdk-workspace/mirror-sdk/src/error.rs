//! Error types for the Mirror SDK.

use std::io;
use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with the Mirror SDK.
#[derive(Error, Debug)]
pub enum MirrorError {
    /// Error when a repository with the given ID already exists.
    #[error("Repository with ID '{0}' already exists")]
    DuplicateRepositoryId(String),

    /// Error when a repository with the given path already exists.
    #[error("Repository with path '{0}' already exists")]
    DuplicateRepositoryPath(String),

    /// Error when a repository with the given ID is not found.
    #[error("Repository with ID '{0}' not found")]
    RepositoryNotFound(String),

    /// Error when the origin is missing from a repository.
    #[error("Repository origin is required")]
    MissingOrigin,

    /// Error when the path is missing from a repository.
    #[error("Repository path is required")]
    MissingPath,

    /// Error when the configuration file is not found.
    #[error("Configuration file not found at '{0}'")]
    ConfigFileNotFound(PathBuf),

    /// Error when the configuration file cannot be parsed.
    #[error("Failed to parse configuration file: {0}")]
    ConfigParseError(#[from] toml::de::Error),

    /// Error when the configuration file cannot be serialized.
    #[error("Failed to serialize configuration: {0}")]
    ConfigSerializeError(#[from] toml::ser::Error),

    /// Error when an IO operation fails.
    #[error("IO error: {0}")]
    IoError(#[from] io::Error),

    /// Error when an environment variable is not set.
    #[error("Environment variable '{0}' not set")]
    EnvVarNotSet(String),

    /// Error when an environment variable cannot be parsed.
    #[error("Failed to parse environment variable '{0}': {1}")]
    EnvVarParseError(String, String),

    /// Error when a path is invalid.
    #[error("Invalid path: {0}")]
    InvalidPath(String),

    /// Other errors.
    #[error("Other error: {0}")]
    Other(String),
}