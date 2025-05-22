//! Utility functions module
//!
//! This module provides utility functions used throughout the CLI.

mod error;
mod config;

pub use error::*;
pub use config::*;

use std::path::{Path, PathBuf};

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

/// Generates a unique ID for a repository
///
/// # Returns
///
/// A unique ID string (first segment of a UUID v4)
pub fn generate_id() -> String {
    use mirror_sdk::uuid::Uuid;
    Uuid::new_v4().to_string().split('-').next().unwrap().to_string()
}