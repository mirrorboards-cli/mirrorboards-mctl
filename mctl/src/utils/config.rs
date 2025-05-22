//! Configuration utilities
//!
//! This module provides utilities for working with mirror.toml files and user configuration.

use std::path::{Path, PathBuf};
use std::env;
use std::fs;
use std::io::{Read, Write};
use std::collections::HashMap;
use serde::{Deserialize, Serialize};
use mirror_sdk::{MirrorConfig, DEFAULT_FILENAME, ENV_MIRROR_PATH};
use crate::commands::{CommandResult, CommandError};

/// User configuration file name
const USER_CONFIG_FILE: &str = ".mctl.json";

/// SSH key configuration key
const SSH_KEY_CONFIG_KEY: &str = "ssh_key";

/// User configuration structure
#[derive(Debug, Serialize, Deserialize, Default)]
struct UserConfig {
    /// Configuration options
    options: HashMap<String, String>,
}

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

/// Get the path to the user configuration file
fn get_user_config_path() -> CommandResult<PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| CommandError::Config("Could not determine home directory".to_string()))?;
    Ok(home_dir.join(USER_CONFIG_FILE))
}

/// Load the user configuration
fn load_user_config() -> CommandResult<UserConfig> {
    let config_path = get_user_config_path()?;
    
    // If the file doesn't exist, return a default configuration
    if !config_path.exists() {
        return Ok(UserConfig::default());
    }
    
    // Read the file
    let mut file = fs::File::open(&config_path)
        .map_err(|e| CommandError::File(format!("Failed to open user config file: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| CommandError::File(format!("Failed to read user config file: {}", e)))?;
    
    // Parse the JSON
    serde_json::from_str(&contents)
        .map_err(|e| CommandError::Config(format!("Failed to parse user config file: {}", e)))
}

/// Save the user configuration
fn save_user_config(config: &UserConfig) -> CommandResult<()> {
    let config_path = get_user_config_path()?;
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| CommandError::Config(format!("Failed to serialize user config: {}", e)))?;
    
    // Write to file
    let mut file = fs::File::create(&config_path)
        .map_err(|e| CommandError::File(format!("Failed to create user config file: {}", e)))?;
    file.write_all(json.as_bytes())
        .map_err(|e| CommandError::File(format!("Failed to write user config file: {}", e)))?;
    
    Ok(())
}

/// Get the saved SSH key path from user configuration
///
/// # Returns
///
/// An `Option<String>` containing the SSH key path if set, or `None` if not configured
pub fn get_ssh_key() -> CommandResult<Option<String>> {
    let config = load_user_config()?;
    Ok(config.options.get(SSH_KEY_CONFIG_KEY).cloned())
}

/// Set the SSH key path in user configuration
///
/// # Arguments
///
/// * `path` - SSH key path to save
///
/// # Returns
///
/// A `Result<()>` indicating success or failure
pub fn set_ssh_key(path: &str) -> CommandResult<()> {
    let mut config = load_user_config()?;
    config.options.insert(SSH_KEY_CONFIG_KEY.to_string(), path.to_string());
    save_user_config(&config)
}