//! Utility functions module
//!
//! This module provides utility functions used throughout the CLI.

mod error;
mod config;

pub use error::*;
pub use config::*;

// Re-export SSH key functions specifically
pub use config::{get_ssh_key, set_ssh_key};

use std::path::{Path, PathBuf};
use std::fs;
use crate::commands::{CommandResult, CommandError};

/// Resolves a path relative to the mirror.toml file
///
/// # Arguments
///
/// * `config_path` - Path to the mirror.toml file
/// * `relative_path` - Path relative to the mirror.toml file
///
/// # Returns
///
/// A `PathBuf` containing the resolved path
pub fn resolve_relative_path<P: AsRef<Path>, R: AsRef<Path>>(config_path: P, relative_path: R) -> PathBuf {
    let config_path = config_path.as_ref();
    let relative_path = relative_path.as_ref();
    
    if let Some(parent) = config_path.parent() {
        parent.join(relative_path)
    } else {
        relative_path.to_path_buf()
    }
}

/// Normalizes a path by removing leading "./" and resolving relative components
///
/// # Arguments
///
/// * `path` - Path to normalize
///
/// # Returns
///
/// A `String` containing the normalized path
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let path_str = path.as_ref().to_string_lossy().to_string();
    
    // Remove leading "./"
    if path_str.starts_with("./") {
        path_str[2..].to_string()
    } else {
        path_str
    }
}

/// Resolves and validates an SSH key path
///
/// This function handles tilde expansion, relative path resolution, and validates
/// that the SSH key file exists and is readable.
///
/// # Arguments
///
/// * `ssh_key_path` - SSH key path that may contain tilde or be relative
///
/// # Returns
///
/// A `Result` containing the resolved absolute path or an error
pub fn resolve_ssh_key_path(ssh_key_path: &str) -> CommandResult<PathBuf> {
    let path = if ssh_key_path.starts_with("~/") {
        // Handle tilde expansion
        if let Some(home_dir) = dirs::home_dir() {
            home_dir.join(&ssh_key_path[2..])
        } else {
            return Err(CommandError::Config("Could not determine home directory for tilde expansion".to_string()));
        }
    } else {
        // Handle relative and absolute paths
        PathBuf::from(ssh_key_path)
    };

    // Canonicalize the path to resolve any relative components and get absolute path
    let resolved_path = path.canonicalize()
        .map_err(|e| CommandError::File(format!("SSH key path '{}' could not be resolved: {}", ssh_key_path, e)))?;

    // Validate that the file exists and is readable
    if !resolved_path.exists() {
        return Err(CommandError::File(format!("SSH key file not found: {}", resolved_path.display())));
    }

    if !resolved_path.is_file() {
        return Err(CommandError::File(format!("SSH key path is not a file: {}", resolved_path.display())));
    }

    // Check if the file is readable
    match fs::File::open(&resolved_path) {
        Ok(_) => Ok(resolved_path),
        Err(e) => Err(CommandError::File(format!("SSH key file is not readable: {} ({})", resolved_path.display(), e))),
    }
}

/// Generates a unique ID for a repository
///
/// # Returns
///
/// A unique ID string (first segment of a UUID v4)
pub fn generate_id() -> String {
    use mirror_sdk::uuid::Uuid;
    Uuid::new_v4().to_string().split('-').next().unwrap().to_string()
}