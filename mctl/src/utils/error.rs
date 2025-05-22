//! Error handling utilities
//!
//! This module provides error handling utilities for the CLI.

use std::fmt;
use mirror_sdk::Error as SdkError;
use colored::*;

/// Format an error message for display to the user
///
/// # Arguments
///
/// * `error` - The error to format
/// * `context` - Optional context for the error
/// * `use_color` - Whether to use color in the output
///
/// # Returns
///
/// A formatted error message
pub fn format_error<E: fmt::Display>(error: E, context: Option<&str>, use_color: bool) -> String {
    let mut message = String::new();
    
    if let Some(ctx) = context {
        let header = if use_color {
            format!("Error: {}", ctx).red().bold().to_string()
        } else {
            format!("Error: {}", ctx)
        };
        message.push_str(&header);
        message.push_str("\n");
    } else {
        let header = if use_color {
            "Error:".red().bold().to_string()
        } else {
            "Error:".to_string()
        };
        message.push_str(&header);
        message.push_str("\n");
    }
    
    let error_msg = if use_color {
        format!("  {}", error).red().to_string()
    } else {
        format!("  {}", error)
    };
    message.push_str(&error_msg);
    
    message
}

/// Format an SDK error with a suggested solution
///
/// # Arguments
///
/// * `error` - The SDK error to format
/// * `use_color` - Whether to use color in the output
///
/// # Returns
///
/// A formatted error message with a suggested solution
pub fn format_sdk_error(error: &SdkError, use_color: bool) -> String {
    let mut message = String::new();
    
    let header = if use_color {
        "Error:".red().bold().to_string()
    } else {
        "Error:".to_string()
    };
    message.push_str(&header);
    message.push_str("\n");
    
    let error_msg = if use_color {
        format!("  Problem: {}", error).red().to_string()
    } else {
        format!("  Problem: {}", error)
    };
    message.push_str(&error_msg);
    message.push_str("\n");
    
    // Add a suggested solution based on the error type
    let solution = match error {
        SdkError::Io(io_err) => {
            match io_err.kind() {
                std::io::ErrorKind::NotFound => "Check that the file exists and you have permission to access it".to_string(),
                std::io::ErrorKind::PermissionDenied => "Check that you have permission to access the file".to_string(),
                _ => "Check the file system and try again".to_string(),
            }
        },
        SdkError::TomlDe(_) => "Check the format of your mirror.toml file".to_string(),
        SdkError::TomlSer(_) => "There was an error serializing the configuration".to_string(),
        SdkError::MissingField(field) => format!("Provide a value for the required field '{}'", field),
        SdkError::DuplicateId(id) => format!("Use a different ID or update the existing repository with ID '{}'", id),
        SdkError::DuplicatePath(path) => format!("Use a different path or update the existing repository with path '{}'", path),
        SdkError::RepositoryNotFound(id) => format!("Check that a repository with ID '{}' exists", id),
        SdkError::ConfigNotFound(_) => "Initialize a new file or specify the correct path to mirror.toml".to_string(),
        SdkError::InvalidConfig(reason) => format!("Fix the configuration issue: {}", reason),
        SdkError::Other(msg) => msg.clone(),
    };
    
    let solution_msg = if use_color {
        format!("  Solution: {}", solution).yellow().to_string()
    } else {
        format!("  Solution: {}", solution)
    };
    message.push_str(&solution_msg);
    
    message
}