//! Configuration utilities
//!
//! This module provides utilities for working with mirror.toml files.

use std::path::{Path, PathBuf};
use std::env;
use mirror_sdk::{MirrorConfig, DEFAULT_FILENAME, ENV_MIRROR_PATH};
use crate::commands::{CommandResult, CommandError};

/// Resolves the path to the mirror.toml file
///
/// This function will try to resolve the path in the following order:
/// 1. The path specified in the command-line argument
/// 2. The path specified in the MIRROR_CONFIG_PATH environment variable
/// 3. The default location (./mirror.toml)
///
/// # Arguments
///
/// * `cli_path` - Optional path specified in the command-line argument
///
/// # Returns
///
/// A `Result` containing the resolved path or an error
pub fn resolve_config_path(cli_path: Option<String>) -> CommandResult<PathBuf> {
    // Try to get the path from the command-line argument
    if let Some(path) = cli_path {
        return Ok(PathBuf::from(path));
    }
    
    // Try to get the path from the environment variable
    if let Ok(path) = env::var(ENV_MIRROR_PATH) {
        return Ok(PathBuf::from(path));
    }
    
    // Fall back to the default location
    Ok(PathBuf::from(DEFAULT_FILENAME))
}

/// Loads a mirror configuration from the specified path or default
///
/// # Arguments
///
/// * `cli_path` - Optional path specified in the command-line argument
///
/// # Returns
///
/// A `Result` containing the loaded configuration or an error
pub fn load_config(cli_path: Option<String>) -> CommandResult<MirrorConfig> {
    let path = resolve_config_path(cli_path)?;
    
    MirrorConfig::load_from(&path)
        .map_err(|e| CommandError::Sdk(e))
}

/// Checks if a mirror.toml file exists at the specified path or default
///
/// # Arguments
///
/// * `cli_path` - Optional path specified in the command-line argument
///
/// # Returns
///
/// `true` if the file exists, `false` otherwise
pub fn config_exists(cli_path: Option<String>) -> bool {
    if let Ok(path) = resolve_config_path(cli_path) {
        path.exists()
    } else {
        false
    }
}

/// Creates a backup of the mirror.toml file
///
/// # Arguments
///
/// * `cli_path` - Optional path specified in the command-line argument
///
/// # Returns
///
/// A `Result` containing the path to the backup file or an error
pub fn backup_config(cli_path: Option<String>) -> CommandResult<PathBuf> {
    let path = resolve_config_path(cli_path)?;
    
    // Check if the file exists
    if !path.exists() {
        return Err(CommandError::File(format!("File not found: {}", path.display())));
    }
    
    // Create a backup filename with a timestamp
    let timestamp = chrono::Local::now().format("%Y%m%d%H%M%S");
    let backup_path = path.with_extension(format!("toml.{}.bak", timestamp));
    
    // Copy the file
    std::fs::copy(&path, &backup_path)
        .map_err(|e| CommandError::File(format!("Failed to create backup: {}", e)))?;
    
    Ok(backup_path)
}