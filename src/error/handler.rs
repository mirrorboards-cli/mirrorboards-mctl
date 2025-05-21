//! Error handler for MCTL
//!
//! This module provides error handling utilities for the MCTL application.

use crate::error::types::{MctlError, MctlErrorTrait};
use log::error;

/// Error handler for MCTL errors
pub struct ErrorHandler {
    // Configuration for error handling
    verbose: bool,
}

impl ErrorHandler {
    /// Create a new error handler
    pub fn new() -> Self {
        // Default to non-verbose error messages
        Self { verbose: false }
    }

    /// Create a new error handler with verbose output
    pub fn with_verbose(verbose: bool) -> Self {
        Self { verbose }
    }

    /// Handle an error and return a user-friendly message
    pub fn handle_error(&self, err: &MctlError) -> String {
        // Log the error
        self.log_error(err);

        // Get the user message and recovery hint
        let user_message = match err {
            MctlError::CliError(e) => e.user_message(),
            MctlError::ConfigError(e) => e.user_message(),
            // Add other error types as needed
        };

        let recovery_hint = match err {
            MctlError::CliError(e) => e.recovery_hint(),
            MctlError::ConfigError(e) => e.recovery_hint(),
            // Add other error types as needed
        };

        // Format the error message
        let mut message = format!("Error: {}", user_message);

        // Add recovery hint if available
        if let Some(hint) = recovery_hint {
            message = format!("{}\nHint: {}", message, hint);
        }

        // Add verbose information if enabled
        if self.verbose {
            message = format!("{}\nDetails: {:?}", message, err);
        }

        message
    }

    /// Log an error
    pub fn log_error(&self, err: &MctlError) {
        match err {
            MctlError::CliError(e) => {
                error!("CLI Error [{}]: {}", e.code, e.message);
                if let Some(context) = &e.context {
                    error!("Context: {}", context);
                }
                if let Some(source) = &e.source {
                    error!("Caused by: {}", source);
                }
            }
            MctlError::ConfigError(e) => {
                error!("Config Error [{}]: {}", e.code, e.message);
                error!("Context: {}", e.context);
                if let Some(source) = &e.source {
                    error!("Caused by: {}", source);
                }
            }
            // Add other error types as needed
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::types::{CliError, ErrorCode};

    #[test]
    fn test_handle_cli_error() {
        let handler = ErrorHandler::new();
        let error = CliError::new(
            ErrorCode::InvalidArgument,
            "Invalid argument: --foo".to_string(),
        );
        let mctl_error = MctlError::CliError(error);

        let message = handler.handle_error(&mctl_error);
        assert!(message.contains("Error: Invalid argument"));
        assert!(message.contains("Check the command syntax"));
    }

    #[test]
    fn test_verbose_error_handling() {
        let handler = ErrorHandler::with_verbose(true);
        let error = CliError::new(
            ErrorCode::MissingRequiredOption,
            "Missing required option: --git-url".to_string(),
        );
        let mctl_error = MctlError::CliError(error);

        let message = handler.handle_error(&mctl_error);
        assert!(message.contains("Error: Missing required option"));
        assert!(message.contains("Details:"));
    }
}
