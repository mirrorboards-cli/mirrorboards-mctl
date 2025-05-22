//! Error type definitions for MCTL
//!
//! This module defines the error types and error codes used throughout the MCTL application.

use std::fmt;
use thiserror::Error;

/// Error codes for MCTL errors
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ErrorCode {
    // CLI errors
    InvalidArgument,
    MissingCommand,
    MissingRequiredOption,
    InvalidCommand,

    // Config errors
    ConfigNotFound,
    InvalidConfigFormat,
    ConfigWriteFailed,

    // Repository errors
    RepositoryNotFound,
    RepositoryAccessDenied,

    // Git errors
    GitCommandFailed,
    GitAuthenticationFailed,
    GitMergeConflict,

    // Security errors
    CredentialsNotFound,
    InvalidCredentials,
    CredentialStoreFailed,
}

impl fmt::Display for ErrorCode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ErrorCode::InvalidArgument => write!(f, "E001"),
            ErrorCode::MissingCommand => write!(f, "E002"),
            ErrorCode::MissingRequiredOption => write!(f, "E003"),
            ErrorCode::InvalidCommand => write!(f, "E004"),
            ErrorCode::ConfigNotFound => write!(f, "E101"),
            ErrorCode::InvalidConfigFormat => write!(f, "E102"),
            ErrorCode::ConfigWriteFailed => write!(f, "E103"),
            ErrorCode::RepositoryNotFound => write!(f, "E201"),
            ErrorCode::RepositoryAccessDenied => write!(f, "E202"),
            ErrorCode::GitCommandFailed => write!(f, "E301"),
            ErrorCode::GitAuthenticationFailed => write!(f, "E302"),
            ErrorCode::GitMergeConflict => write!(f, "E303"),
            ErrorCode::CredentialsNotFound => write!(f, "E401"),
            ErrorCode::InvalidCredentials => write!(f, "E402"),
            ErrorCode::CredentialStoreFailed => write!(f, "E403"),
        }
    }
}

/// Common trait for all MCTL errors
pub trait MctlErrorTrait: std::error::Error {
    fn error_code(&self) -> ErrorCode;
    fn user_message(&self) -> String;
    fn recovery_hint(&self) -> Option<String>;
}

/// CLI error type
#[derive(Error, Debug)]
pub struct CliError {
    pub code: ErrorCode,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub context: Option<String>,
}

impl CliError {
    pub fn new(code: ErrorCode, message: String) -> Self {
        Self {
            code,
            message,
            source: None,
            context: None,
        }
    }

    pub fn with_source(mut self, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.source = Some(source);
        self
    }

    pub fn with_context(mut self, context: String) -> Self {
        self.context = Some(context);
        self
    }
}

impl fmt::Display for CliError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;

        if let Some(context) = &self.context {
            write!(f, " ({})", context)?;
        }

        Ok(())
    }
}

impl MctlErrorTrait for CliError {
    fn error_code(&self) -> ErrorCode {
        self.code.clone()
    }

    fn user_message(&self) -> String {
        match self.code {
            ErrorCode::InvalidArgument => format!("Invalid argument: {}", self.message),
            ErrorCode::MissingCommand => format!("Missing command: {}", self.message),
            ErrorCode::MissingRequiredOption => {
                format!("Missing required option: {}", self.message)
            }
            ErrorCode::InvalidCommand => format!("Invalid command: {}", self.message),
            _ => self.message.clone(),
        }
    }

    fn recovery_hint(&self) -> Option<String> {
        match self.code {
            ErrorCode::InvalidArgument => {
                Some("Check the command syntax and try again".to_string())
            }
            ErrorCode::MissingCommand => {
                Some("Run 'mctl --help' to see available commands".to_string())
            }
            ErrorCode::MissingRequiredOption => {
                Some("Run 'mctl <command> --help' to see required options".to_string())
            }
            ErrorCode::InvalidCommand => {
                Some("The command type does not match the expected operation. This is likely an internal error.".to_string())
            }
            _ => None,
        }
    }
}

/// Config error type
#[derive(Error, Debug)]
pub struct ConfigError {
    pub code: ErrorCode,
    pub message: String,
    #[source]
    pub source: Option<Box<dyn std::error::Error + Send + Sync>>,
    pub context: String,
}

impl ConfigError {
    pub fn new(code: ErrorCode, message: String, context: String) -> Self {
        Self {
            code,
            message,
            source: None,
            context,
        }
    }

    pub fn with_source(mut self, source: Box<dyn std::error::Error + Send + Sync>) -> Self {
        self.source = Some(source);
        self
    }
}

impl fmt::Display for ConfigError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "[{}] {}", self.code, self.message)?;
        write!(f, " ({})", self.context)?;
        Ok(())
    }
}

impl MctlErrorTrait for ConfigError {
    fn error_code(&self) -> ErrorCode {
        self.code.clone()
    }

    fn user_message(&self) -> String {
        match self.code {
            ErrorCode::ConfigNotFound => format!("Configuration file not found: {}", self.context),
            ErrorCode::InvalidConfigFormat => {
                format!("Invalid configuration format: {}", self.message)
            }
            ErrorCode::ConfigWriteFailed => {
                format!("Failed to write configuration: {}", self.message)
            }
            _ => self.message.clone(),
        }
    }

    fn recovery_hint(&self) -> Option<String> {
        match self.code {
            ErrorCode::ConfigNotFound => Some(format!("Create a new configuration file with 'mctl add' or specify a different path with '--config'")),
            ErrorCode::InvalidConfigFormat => Some(format!("Check the syntax of your TOML file")),
            _ => None,
        }
    }
}

/// Wrapper enum for all MCTL errors
#[derive(Error, Debug)]
pub enum MctlError {
    #[error(transparent)]
    CliError(#[from] CliError),

    #[error(transparent)]
    ConfigError(#[from] ConfigError),
    // Add other error types as needed
    // #[error(transparent)]
    // RepoError(#[from] RepoError),

    // #[error(transparent)]
    // GitError(#[from] GitError),

    // #[error(transparent)]
    // StatusError(#[from] StatusError),

    // #[error(transparent)]
    // SecurityError(#[from] SecurityError),
}
