//! Utility functions for the Mirror CLI.

use colored::Colorize;
use mirror_sdk::{MirrorConfig, CONFIG_PATH_ENV_VAR, DEFAULT_CONFIG_FILENAME};
use std::env;
use std::path::{Path, PathBuf};

use crate::error::{CliError, CliResult};

/// Resolves the configuration file path based on the provided path, environment variable, or default.
///
/// The resolution order is:
/// 1. Use the provided path if it's not None
/// 2. Check for the environment variable MIRROR_CONFIG_PATH
/// 3. Use the default path (./mirror.toml)
///
/// # Arguments
///
/// * `path` - An optional path to the configuration file
///
/// # Returns
///
/// * `CliResult<PathBuf>` - The resolved path or an error
pub fn resolve_config_path(path: Option<&Path>) -> CliResult<PathBuf> {
    if let Some(path) = path {
        return Ok(path.to_path_buf());
    }

    if let Ok(env_path) = env::var(CONFIG_PATH_ENV_VAR) {
        return Ok(PathBuf::from(env_path));
    }

    Ok(PathBuf::from(DEFAULT_CONFIG_FILENAME))
}

/// Loads a configuration from a file or creates a new one if it doesn't exist.
///
/// # Arguments
///
/// * `path` - An optional path to the configuration file
/// * `create_if_missing` - Whether to create a new configuration if the file doesn't exist
///
/// # Returns
///
/// * `CliResult<MirrorConfig>` - The loaded or created configuration or an error
pub fn load_or_create_config(path: Option<&Path>, create_if_missing: bool) -> CliResult<MirrorConfig> {
    let resolved_path = resolve_config_path(path)?;

    // Try to load the configuration
    let config = MirrorConfig::load_from_file(&resolved_path);

    match config {
        Ok(config) => Ok(config),
        Err(mirror_sdk::MirrorError::ConfigFileNotFound(_)) if create_if_missing => {
            // Create a new configuration if the file doesn't exist and create_if_missing is true
            println!("{} Creating new configuration at {}", "Info:".bright_blue(), resolved_path.display());
            Ok(MirrorConfig::init(Some(&resolved_path))?)
        }
        Err(err) => Err(CliError::SdkError(err)),
    }
}

/// Prints a success message.
///
/// # Arguments
///
/// * `message` - The message to print
pub fn print_success(message: &str) {
    println!("{} {}", "Success:".bright_green(), message);
}

/// Prints a warning message.
///
/// # Arguments
///
/// * `message` - The message to print
pub fn print_warning(message: &str) {
    println!("{} {}", "Warning:".bright_yellow(), message);
}

/// Prints an info message.
///
/// # Arguments
///
/// * `message` - The message to print
pub fn print_info(message: &str) {
    println!("{} {}", "Info:".bright_blue(), message);
}

/// Prints an error message.
///
/// # Arguments
///
/// * `message` - The message to print
pub fn print_error(message: &str) {
    eprintln!("{} {}", "Error:".bright_red(), message);
}