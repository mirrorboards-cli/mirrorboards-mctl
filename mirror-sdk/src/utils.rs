//! Utility functions for mirror-sdk
//!
//! This module provides utility functions used throughout the mirror-sdk library.

use uuid::Uuid;
use std::path::{Path, PathBuf};
use std::env;

use crate::error::Result;
use crate::config::ENV_MIRROR_PATH;

/// Generates a unique ID for a repository
///
/// # Returns
///
/// A unique ID string (first segment of a UUID v4)
pub fn generate_id() -> String {
    Uuid::new_v4().to_string().split('-').next().unwrap().to_string()
}

/// Resolves the path to the mirror.toml file
///
/// This function will try to resolve the path in the following order:
/// 1. The path specified in the MIRROR_CONFIG_PATH environment variable
/// 2. The default location (./mirror.toml)
///
/// # Returns
///
/// A `Result` containing the resolved path or an error
#[allow(dead_code)]
pub fn resolve_config_path() -> Result<PathBuf> {
    // Try to get the path from the environment variable
    if let Ok(path) = env::var(ENV_MIRROR_PATH) {
        return Ok(PathBuf::from(path));
    }
    
    // Fall back to the default location
    Ok(PathBuf::from("mirror.toml"))
}

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
#[allow(dead_code)]
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
#[allow(dead_code)]
pub fn normalize_path<P: AsRef<Path>>(path: P) -> String {
    let path_str = path.as_ref().to_string_lossy().to_string();
    
    // Remove leading "./"
    if path_str.starts_with("./") {
        path_str[2..].to_string()
    } else {
        path_str
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_generate_id() {
        let id1 = generate_id();
        let id2 = generate_id();
        
        assert!(!id1.is_empty());
        assert!(!id2.is_empty());
        assert_ne!(id1, id2);
    }
    
    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("./foo/bar"), "foo/bar");
        assert_eq!(normalize_path("foo/bar"), "foo/bar");
    }
    
    #[test]
    fn test_resolve_relative_path() {
        let config_path = Path::new("/tmp/mirror.toml");
        let relative_path = Path::new("foo/bar");
        
        let resolved = resolve_relative_path(config_path, relative_path);
        assert_eq!(resolved, Path::new("/tmp/foo/bar"));
    }
}