use std::path::PathBuf;
use thiserror::Error;

/// Custom error types for MCTL
#[derive(Error, Debug)]
pub enum MctlError {
    #[error("Configuration error: {0}")]
    ConfigError(String),

    #[error("Git error: {0}")]
    GitError(String),

    #[error("Repository not found: {0}")]
    RepositoryNotFound(PathBuf),

    #[error("Invalid repository URL: {0}")]
    InvalidRepositoryUrl(String),

    #[error("Invalid path: {0}")]
    InvalidPath(PathBuf),

    #[error("Authentication failed: {0}")]
    AuthenticationFailed(String),

    #[error("Network error: {0}")]
    NetworkError(String),

    #[error("Merge conflict in repository: {0}")]
    MergeConflict(PathBuf),

    #[error("Uncommitted changes in repository: {0}")]
    UncommittedChanges(PathBuf),

    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    #[error("TOML parsing error: {0}")]
    TomlError(#[from] toml::de::Error),

    #[error("TOML serialization error: {0}")]
    TomlSerError(#[from] toml::ser::Error),

    #[error("Missing required parameter: {0}")]
    MissingParameter(String),

    #[error("Operation not permitted: {0}")]
    OperationNotPermitted(String),
}

/// Result type for MCTL operations
pub type MctlResult<T> = Result<T, MctlError>;
