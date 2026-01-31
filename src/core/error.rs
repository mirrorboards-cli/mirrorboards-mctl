//! Error types for the mctl crate.

use std::path::PathBuf;
use thiserror::Error;

/// Git operation errors
#[derive(Error, Debug)]
pub enum GitError {
    #[error("Git executable not found at '{path}'. Please ensure git is installed.")]
    GitNotFound { path: String },

    #[error("Git command failed with exit code {exit_code}: {stderr}\nCommand: {command}")]
    CommandFailed {
        exit_code: i32,
        stderr: String,
        command: String,
    },

    #[error("Repository not found at path: {path}")]
    RepositoryNotFound { path: PathBuf },

    #[error("Path is not a git repository: {path}")]
    NotGitRepository { path: PathBuf },

    #[error("Failed to clone repository from '{url}': {message}")]
    CloneFailed { url: String, message: String },

    #[error("Authentication failed: {message}")]
    AuthenticationFailed { message: String },

    #[error("Branch not found: {branch}")]
    BranchNotFound { branch: String },

    #[error("Tag not found: {tag}")]
    TagNotFound { tag: String },

    #[error("Revision not found: {rev}")]
    RevisionNotFound { rev: String },

    #[error("Merge conflict detected")]
    MergeConflict,

    #[error("No changes to commit")]
    NoChangesToCommit,

    #[error("Push rejected by remote")]
    PushRejected,

    #[error("Failed to parse git output: {message}")]
    ParseError { message: String },

    #[error("Git operation timed out after {seconds} seconds")]
    Timeout { seconds: u64 },

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },
}

/// Configuration errors
#[derive(Error, Debug)]
pub enum ConfigError {
    #[error("Configuration file not found: {path}")]
    NotFound { path: PathBuf },

    #[error("Failed to parse configuration: {message}")]
    ParseError { message: String },

    #[error("Invalid configuration: {message}")]
    ValidationError { message: String },

    #[error("Duplicate repository path: {path}")]
    DuplicatePath { path: String },

    #[error("Include cycle detected: {cycle}")]
    IncludeCycle { cycle: String },

    #[error("Include file not found: {path:?} (referenced from {referenced_from})")]
    IncludeNotFound { path: PathBuf, referenced_from: String },

    #[error("Repository has multiple version specifiers (branch, rev, tag). Only one is allowed.")]
    MultipleVersionSpecs,

    #[error("IO error: {source}")]
    IoError {
        #[from]
        source: std::io::Error,
    },

    #[error("TOML parse error: {source}")]
    TomlError {
        #[from]
        source: toml::de::Error,
    },

    #[error("TOML serialization error: {source}")]
    TomlSerError {
        #[from]
        source: toml::ser::Error,
    },
}

/// URL parsing errors
#[derive(Error, Debug)]
pub enum UrlError {
    #[error("Invalid git URL: {url}")]
    InvalidUrl { url: String },

    #[error("Unsupported protocol in URL: {url}")]
    UnsupportedProtocol { url: String },
}

/// Result types
pub type GitResult<T> = Result<T, GitError>;
pub type ConfigResult<T> = Result<T, ConfigError>;
pub type UrlResult<T> = Result<T, UrlError>;
