//! Utility functions for the Mirror SDK.

use std::env;
use std::path::{Path, PathBuf};

use crate::error::MirrorError;
use crate::{CONFIG_PATH_ENV_VAR, DEFAULT_CONFIG_FILENAME, Result};

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
/// * `Result<PathBuf>` - The resolved path or an error
pub fn resolve_config_path(path: Option<&Path>) -> Result<PathBuf> {
    if let Some(path) = path {
        return Ok(path.to_path_buf());
    }

    if let Ok(env_path) = env::var(CONFIG_PATH_ENV_VAR) {
        return Ok(PathBuf::from(env_path));
    }

    Ok(PathBuf::from(DEFAULT_CONFIG_FILENAME))
}

/// Normalizes a path string by resolving relative paths and removing redundant components.
///
/// # Arguments
///
/// * `path` - The path string to normalize
///
/// # Returns
///
/// * `Result<String>` - The normalized path or an error
pub fn normalize_path(path: &str) -> Result<String> {
    let path_buf = PathBuf::from(path);
    
    // Convert to absolute path if it's relative
    let absolute_path = if path_buf.is_relative() {
        // For relative paths, we just canonicalize them as best we can
        // without requiring the path to exist
        let mut normalized = PathBuf::new();
        for component in path_buf.components() {
            match component {
                std::path::Component::ParentDir => {
                    if !normalized.pop() {
                        // If we can't pop (at root), just keep the ..
                        normalized.push(component);
                    }
                },
                std::path::Component::CurDir => {
                    // Skip . components
                },
                _ => normalized.push(component),
            }
        }
        normalized
    } else {
        path_buf
    };

    // Convert back to string
    absolute_path
        .to_str()
        .map(|s| s.to_string())
        .ok_or_else(|| MirrorError::InvalidPath(path.to_string()))
}

/// Checks if a path exists and is a file.
///
/// # Arguments
///
/// * `path` - The path to check
///
/// # Returns
///
/// * `bool` - True if the path exists and is a file, false otherwise
pub fn path_exists_and_is_file(path: &Path) -> bool {
    path.exists() && path.is_file()
}

/// Creates parent directories for a file if they don't exist.
///
/// # Arguments
///
/// * `path` - The path to the file
///
/// # Returns
///
/// * `Result<()>` - Ok if the directories were created or already exist, error otherwise
pub fn create_parent_dirs(path: &Path) -> Result<()> {
    if let Some(parent) = path.parent() {
        if !parent.exists() {
            fs_err::create_dir_all(parent)?;
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;
    use tempfile::tempdir;

    #[test]
    fn test_resolve_config_path_with_provided_path() {
        let path = Path::new("/tmp/test/mirror.toml");
        let result = resolve_config_path(Some(path)).unwrap();
        assert_eq!(result, path);
    }

    #[test]
    fn test_resolve_config_path_with_env_var() {
        let temp_path = "/tmp/test/env/mirror.toml";
        env::set_var(CONFIG_PATH_ENV_VAR, temp_path);
        let result = resolve_config_path(None).unwrap();
        assert_eq!(result, PathBuf::from(temp_path));
        env::remove_var(CONFIG_PATH_ENV_VAR);
    }

    #[test]
    fn test_resolve_config_path_with_default() {
        env::remove_var(CONFIG_PATH_ENV_VAR);
        let result = resolve_config_path(None).unwrap();
        assert_eq!(result, PathBuf::from(DEFAULT_CONFIG_FILENAME));
    }

    #[test]
    fn test_normalize_path() {
        assert_eq!(normalize_path("./test").unwrap(), "test");
        assert_eq!(normalize_path("test/../test").unwrap(), "test");
        assert_eq!(normalize_path("./test/./subdir/..").unwrap(), "test");
    }

    #[test]
    fn test_create_parent_dirs() {
        let temp_dir = tempdir().unwrap();
        let path = temp_dir.path().join("subdir/file.txt");
        
        create_parent_dirs(&path).unwrap();
        
        assert!(path.parent().unwrap().exists());
        assert!(path.parent().unwrap().is_dir());
    }
}