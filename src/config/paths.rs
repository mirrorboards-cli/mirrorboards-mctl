//! Configuration paths module for MCTL
//!
//! This module handles configuration file paths and directory resolution.

use dirs;
use log::debug;
use std::env;
use std::path::{Path, PathBuf};

/// Configuration paths for MCTL
pub struct ConfigPaths {
    /// Path to the mirror.toml configuration file
    pub config_file: PathBuf,

    /// Path to the directory containing the mirror.toml file
    pub config_dir: PathBuf,

    /// Path to the directory where repositories are cloned by default
    pub repos_dir: PathBuf,
}

impl ConfigPaths {
    /// Create a new ConfigPaths instance with default paths
    pub fn new() -> Self {
        let config_file = Self::default_config_file();
        let config_dir = config_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let repos_dir = config_dir.clone();

        Self {
            config_file,
            config_dir,
            repos_dir,
        }
    }

    /// Create a new ConfigPaths instance with a custom configuration file path
    pub fn with_config_file<P: AsRef<Path>>(config_file: P) -> Self {
        let config_file = config_file.as_ref().to_path_buf();
        let config_dir = config_file
            .parent()
            .unwrap_or_else(|| Path::new("."))
            .to_path_buf();
        let repos_dir = config_dir.clone();

        Self {
            config_file,
            config_dir,
            repos_dir,
        }
    }

    /// Set a custom repositories directory
    pub fn with_repos_dir<P: AsRef<Path>>(mut self, repos_dir: P) -> Self {
        self.repos_dir = repos_dir.as_ref().to_path_buf();
        self
    }

    /// Get the default configuration file path
    pub fn default_config_file() -> PathBuf {
        // Check for MCTL_CONFIG environment variable
        if let Ok(path) = env::var("MCTL_CONFIG") {
            let path = PathBuf::from(path);
            debug!(
                "Using configuration file from MCTL_CONFIG: {}",
                path.display()
            );
            return path;
        }

        // Check for config in current directory
        let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        let local_config = current_dir.join("mirror.toml");
        if local_config.exists() {
            debug!(
                "Using configuration file from current directory: {}",
                local_config.display()
            );
            return local_config;
        }

        // Check for config in XDG_CONFIG_HOME or ~/.config
        if let Some(config_dir) = dirs::config_dir() {
            let xdg_config = config_dir.join("mctl").join("mirror.toml");
            if xdg_config.exists() {
                debug!(
                    "Using configuration file from XDG config directory: {}",
                    xdg_config.display()
                );
                return xdg_config;
            }
        }

        // Default to mirror.toml in current directory
        debug!(
            "Using default configuration file: {}",
            local_config.display()
        );
        local_config
    }

    /// Resolve a relative path to an absolute path
    ///
    /// If the path is already absolute, it is returned as is.
    /// If the path is relative, it is resolved relative to the configuration directory.
    pub fn resolve_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.config_dir.join(path)
        }
    }

    /// Resolve a repository path to an absolute path
    ///
    /// If the path is already absolute, it is returned as is.
    /// If the path is relative, it is resolved relative to the repositories directory.
    pub fn resolve_repo_path<P: AsRef<Path>>(&self, path: P) -> PathBuf {
        let path = path.as_ref();
        if path.is_absolute() {
            path.to_path_buf()
        } else {
            self.repos_dir.join(path)
        }
    }
}

impl Default for ConfigPaths {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::tempdir;

    #[test]
    fn test_default_config_paths() {
        let paths = ConfigPaths::default();
        assert_eq!(paths.config_file, ConfigPaths::default_config_file());
        assert_eq!(
            paths.config_dir,
            paths
                .config_file
                .parent()
                .unwrap_or_else(|| Path::new("."))
                .to_path_buf()
        );
        assert_eq!(paths.repos_dir, paths.config_dir);
    }

    #[test]
    fn test_with_config_file() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("custom-mirror.toml");

        let paths = ConfigPaths::with_config_file(&config_file);
        assert_eq!(paths.config_file, config_file);
        assert_eq!(paths.config_dir, temp_dir.path());
        assert_eq!(paths.repos_dir, temp_dir.path());
    }

    #[test]
    fn test_with_repos_dir() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("mirror.toml");
        let repos_dir = temp_dir.path().join("repos");

        let paths = ConfigPaths::with_config_file(&config_file).with_repos_dir(&repos_dir);
        assert_eq!(paths.config_file, config_file);
        assert_eq!(paths.config_dir, temp_dir.path());
        assert_eq!(paths.repos_dir, repos_dir);
    }

    #[test]
    fn test_resolve_path() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("mirror.toml");

        let paths = ConfigPaths::with_config_file(&config_file);

        // Absolute path
        let abs_path = PathBuf::from("/absolute/path");
        assert_eq!(paths.resolve_path(&abs_path), abs_path);

        // Relative path
        let rel_path = PathBuf::from("relative/path");
        assert_eq!(
            paths.resolve_path(&rel_path),
            temp_dir.path().join("relative/path")
        );
    }

    #[test]
    fn test_resolve_repo_path() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("mirror.toml");
        let repos_dir = temp_dir.path().join("repos");

        let paths = ConfigPaths::with_config_file(&config_file).with_repos_dir(&repos_dir);

        // Absolute path
        let abs_path = PathBuf::from("/absolute/path");
        assert_eq!(paths.resolve_repo_path(&abs_path), abs_path);

        // Relative path
        let rel_path = PathBuf::from("relative/path");
        assert_eq!(
            paths.resolve_repo_path(&rel_path),
            repos_dir.join("relative/path")
        );
    }

    #[test]
    fn test_default_config_file_env_var() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("env-mirror.toml");

        // Set environment variable
        env::set_var("MCTL_CONFIG", config_file.to_str().unwrap());

        let paths = ConfigPaths::default();
        assert_eq!(paths.config_file, config_file);

        // Clean up
        env::remove_var("MCTL_CONFIG");
    }

    #[test]
    fn test_default_config_file_current_dir() {
        let temp_dir = tempdir().unwrap();
        let config_file = temp_dir.path().join("mirror.toml");

        // Create config file
        fs::write(&config_file, "").unwrap();

        // Change current directory
        let old_dir = env::current_dir().unwrap();
        env::set_current_dir(temp_dir.path()).unwrap();

        let paths = ConfigPaths::default();
        assert_eq!(paths.config_file, config_file);

        // Clean up
        env::set_current_dir(old_dir).unwrap();
    }
}
