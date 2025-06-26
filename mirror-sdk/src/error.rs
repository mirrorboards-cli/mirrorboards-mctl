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

/// Errors that can occur during SSH operations
#[derive(Error, Debug)]
pub enum SshError {
    #[error("Failed to initialize SSH session: {message}")]
    SessionInitError { message: String },
    
    #[error("SSH agent connection failed: {message}")]
    AgentConnectionError { message: String },
    
    #[error("SSH agent has no keys loaded")]
    AgentEmptyError,
    
    #[error("SSH key file not found: {path}")]
    KeyFileNotFound { path: std::path::PathBuf },
    
    #[error("Invalid SSH key format: {path}")]
    InvalidKeyFormat { path: std::path::PathBuf },
    
    #[error("SSH key authentication failed for key: {path}")]
    KeyAuthenticationFailed { path: std::path::PathBuf },
    
    #[error("SSH agent authentication failed")]
    AgentAuthenticationFailed,
    
    #[error("No usable SSH keys found (checked agent and filesystem)")]
    NoUsableKeysError,
    
    #[error("SSH timeout: operation took too long")]
    TimeoutError,
    
    #[error("SSH I/O error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

/// Errors that can occur during Git operations
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git repository not found: {path}")]
    RepositoryNotFound { path: PathBuf },
    
    #[error("Git operation failed: {message}")]
    OperationFailed { message: String },
    
    #[error("Authentication failed after {attempts} attempts")]
    AuthenticationFailed { attempts: usize },
    
    #[error("Clone operation failed: {url} -> {path}: {message}")]
    CloneFailed {
        url: String,
        path: PathBuf,
        message: String
    },
    
    #[error("Pull operation failed: {path}: {message}")]
    PullFailed {
        path: PathBuf,
        message: String
    },
    
    #[error("Remote operation failed: {message}")]
    RemoteFailed { message: String },
    
    #[error("Invalid repository state: {message}")]
    InvalidState { message: String },
    
    #[error("Git credential callback error: {message}")]
    CredentialError { message: String },
    
    #[error("Max authentication attempts exceeded ({max_attempts})")]
    MaxAttemptsExceeded { max_attempts: usize },
    
    #[error("Progress reporting error: {message}")]
    ProgressError { message: String },
    
    #[error("SSH authentication error: {source}")]
    SshError {
        #[from]
        source: SshError,
    },
    
    #[error("Git2 library error: {source}")]
    Git2Error {
        #[source]
        source: git2::Error,
    },
    
    #[error("I/O error during git operation: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
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
    
    #[error(transparent)]
    Ssh(#[from] SshError),
    
    #[error(transparent)]
    Git(#[from] GitError),
}

pub type Result<T> = std::result::Result<T, MirrorSdkError>;
pub type ConfigResult<T> = std::result::Result<T, ConfigError>;
pub type RepositoryResult<T> = std::result::Result<T, RepositoryError>;
pub type HashResult<T> = std::result::Result<T, HashError>;
pub type SshResult<T> = std::result::Result<T, SshError>;
pub type GitResult<T> = std::result::Result<T, GitError>;