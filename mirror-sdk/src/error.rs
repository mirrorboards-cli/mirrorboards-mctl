//! Error types for the Mirror SDK.

use thiserror::Error;

/// Main error type for the Mirror SDK.
#[derive(Debug, Error)]
pub enum MirrorError {
    /// IO error.
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    /// TOML parsing error.
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    /// TOML serialization error.
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    /// Validation error.
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    /// Repository not found.
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    /// Repository already exists.
    #[error("Repository already exists: {0}")]
    RepositoryAlreadyExists(String),
    
    /// Path conflict.
    #[error("Path conflict: {0}")]
    PathConflict(String),
    
    /// Invalid configuration.
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    /// Environment error.
    #[error("Environment error: {0}")]
    Environment(String),
}

/// Validation-specific errors.
#[derive(Debug, Error)]
pub enum ValidationError {
    /// Invalid repository path.
    #[error("Invalid repository path: {0}")]
    InvalidPath(String),
    
    /// Invalid repository origin.
    #[error("Invalid repository origin: {0}")]
    InvalidOrigin(String),
    
    /// Missing required field.
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    /// Duplicate ID.
    #[error("Duplicate ID: {0}")]
    DuplicateId(String),
    
    /// Path conflict.
    #[error("Path conflict: {0} and {1}")]
    PathConflict(String, String),
}