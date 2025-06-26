use std::path::PathBuf;
use thiserror::Error;

/// Errors that can occur when working with mirror configurations
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    FileNotFound { path: PathBuf },
    
    #[error("Failed to read configuration file: {source}")]
    ReadError {
        #[from]
        source: std::io::Error,
    },
    
    #[error("Invalid TOML format: {source}")]
    InvalidToml {
        #[from]
        source: toml::de::Error,
    },
    
    #[error("Failed to serialize configuration: {source}")]
    SerializationError {
        #[from]
        source: toml::ser::Error,
    },
    
    #[error("Repository with hash '{hash}' not found")]
    RepositoryNotFound { hash: String },
    
    #[error("Repository already exists: {git}")]
    DuplicateRepository { git: String },
    
    #[error("Configuration validation failed: {message}")]
    ValidationError { message: String },
}

/// Errors that can occur when working with repository definitions
#[derive(Error, Debug)]
pub enum RepositoryError {
    #[error("Invalid git URL format: {url}")]
    InvalidUrl { url: String },
    
    #[error("Unsupported URL scheme: {scheme}")]
    UnsupportedScheme { scheme: String },
    
    #[error("Could not extract organization/repository from URL: {url}")]
    PathExtractionFailed { url: String },
    
    #[error("Invalid branch name: {branch}")]
    InvalidBranch { branch: String },
    
    #[error("Invalid path: {path}")]
    InvalidPath { path: String },
}

/// Errors that can occur during hash generation
#[derive(Error, Debug)]
pub enum HashError {
    #[error("Hash collision detected for repository: {git}")]
    Collision { git: String },
    
    #[error("Invalid hash format: {hash}")]
    InvalidFormat { hash: String },
}

/// Combined error type for all SDK operations
#[derive(Error, Debug)]
pub enum MirrorSdkError {
    #[error(transparent)]
    Config(#[from] ConfigError),
    
    #[error(transparent)]
    Repository(#[from] RepositoryError),
    
    #[error(transparent)]
    Hash(#[from] HashError),
}

pub type Result<T> = std::result::Result<T, MirrorSdkError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type RepositoryResult<T> = std::result::Result<T, RepositoryError>;
pub type HashResult<T> = std::result::Result<T, HashError>;