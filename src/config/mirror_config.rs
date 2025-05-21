//! Mirror configuration module for MCTL
//!
//! This module defines the structure and operations for the mirror.toml configuration file.

use crate::error::types::{ConfigError, ErrorCode, MctlError};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

/// Repository configuration in mirror.toml
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Git URL of the repository
    pub url: String,

    /// Local path where the repository is cloned
    pub path: PathBuf,

    /// Specific branch to track (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,

    /// Authentication method (ssh or https)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub auth_method: Option<String>,

    /// Additional configuration options
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl Repository {
    /// Create a new repository configuration
    pub fn new(url: String, path: PathBuf, branch: Option<String>) -> Self {
        Self {
            url,
            path,
            branch,
            auth_method: None,
            extra: HashMap::new(),
        }
    }

    /// Validate the repository configuration
    pub fn validate(&self) -> Result<(), MctlError> {
        // Validate URL
        if self.url.is_empty() {
            return Err(ConfigError::new(
                ErrorCode::InvalidConfigFormat,
                "Repository URL cannot be empty".to_string(),
                format!("Repository: {:?}", self),
            )
            .into());
        }

        // Validate path
        if self.path.as_os_str().is_empty() {
            return Err(ConfigError::new(
                ErrorCode::InvalidConfigFormat,
                "Repository path cannot be empty".to_string(),
                format!("Repository: {:?}", self),
            )
            .into());
        }

        // Validate auth_method if present
        if let Some(auth) = &self.auth_method {
            if auth != "ssh" && auth != "https" {
                return Err(ConfigError::new(
                    ErrorCode::InvalidConfigFormat,
                    format!("Invalid authentication method: {}", auth),
                    format!("Repository: {:?}", self),
                )
                .into());
            }
        }

        Ok(())
    }

    /// Detect the authentication method from the URL
    pub fn detect_auth_method(&mut self) {
        if self.auth_method.is_none() {
            if self.url.starts_with("git@") || self.url.starts_with("ssh://") {
                self.auth_method = Some("ssh".to_string());
            } else if self.url.starts_with("https://") || self.url.starts_with("http://") {
                self.auth_method = Some("https".to_string());
            }
        }
    }
}

/// Mirror configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MirrorConfig {
    /// Global configuration options
    #[serde(default)]
    pub global: GlobalConfig,

    /// Repository configurations
    #[serde(default)]
    pub repositories: HashMap<String, Repository>,
}

/// Global configuration options
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct GlobalConfig {
    /// Default branch to use when not specified in repository
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_branch: Option<String>,

    /// Default authentication method
    #[serde(skip_serializing_if = "Option::is_none")]
    pub default_auth_method: Option<String>,

    /// Additional configuration options
    #[serde(flatten)]
    pub extra: HashMap<String, toml::Value>,
}

impl MirrorConfig {
    /// Create a new empty mirror configuration
    pub fn new() -> Self {
        Self {
            global: GlobalConfig::default(),
            repositories: HashMap::new(),
        }
    }

    /// Load mirror configuration from a file
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, MctlError> {
        let path = path.as_ref();
        debug!("Loading mirror configuration from {}", path.display());

        // Check if the file exists
        if !path.exists() {
            return Err(ConfigError::new(
                ErrorCode::ConfigNotFound,
                "Configuration file not found".to_string(),
                path.display().to_string(),
            )
            .into());
        }

        // Read the file
        let content = fs::read_to_string(path).map_err(|e| {
            ConfigError::new(
                ErrorCode::ConfigNotFound,
                "Failed to read configuration file".to_string(),
                path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Parse the TOML
        let config: MirrorConfig = toml::from_str(&content).map_err(|e| {
            ConfigError::new(
                ErrorCode::InvalidConfigFormat,
                "Failed to parse configuration file".to_string(),
                path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Validate the configuration
        config.validate()?;

        info!(
            "Successfully loaded mirror configuration with {} repositories",
            config.repositories.len()
        );
        Ok(config)
    }

    /// Save mirror configuration to a file
    pub fn save<P: AsRef<Path>>(&self, path: P) -> Result<(), MctlError> {
        let path = path.as_ref();
        debug!("Saving mirror configuration to {}", path.display());

        // Validate the configuration
        self.validate()?;

        // Convert to TOML
        let content = toml::to_string_pretty(self).map_err(|e| {
            ConfigError::new(
                ErrorCode::ConfigWriteFailed,
                "Failed to serialize configuration".to_string(),
                path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        // Create parent directories if they don't exist
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent).map_err(|e| {
                ConfigError::new(
                    ErrorCode::ConfigWriteFailed,
                    "Failed to create parent directories".to_string(),
                    parent.display().to_string(),
                )
                .with_source(Box::new(e))
            })?;
        }

        // Write the file
        fs::write(path, content).map_err(|e| {
            ConfigError::new(
                ErrorCode::ConfigWriteFailed,
                "Failed to write configuration file".to_string(),
                path.display().to_string(),
            )
            .with_source(Box::new(e))
        })?;

        info!(
            "Successfully saved mirror configuration to {}",
            path.display()
        );
        Ok(())
    }

    /// Add a repository to the configuration
    pub fn add_repository(
        &mut self,
        name: String,
        repository: Repository,
    ) -> Result<(), MctlError> {
        // Validate the repository
        repository.validate()?;

        // Check if the repository already exists
        if self.repositories.contains_key(&name) {
            return Err(ConfigError::new(
                ErrorCode::InvalidConfigFormat,
                format!("Repository '{}' already exists", name),
                format!("Repository: {:?}", repository),
            )
            .into());
        }

        // Add the repository
        self.repositories.insert(name, repository);

        Ok(())
    }

    /// Remove a repository from the configuration
    pub fn remove_repository(&mut self, name: &str) -> Result<Repository, MctlError> {
        // Check if the repository exists
        if !self.repositories.contains_key(name) {
            return Err(ConfigError::new(
                ErrorCode::InvalidConfigFormat,
                format!("Repository '{}' not found", name),
                "".to_string(),
            )
            .into());
        }

        // Remove the repository
        Ok(self.repositories.remove(name).unwrap())
    }

    /// Validate the configuration
    pub fn validate(&self) -> Result<(), MctlError> {
        // Validate global configuration
        if let Some(auth) = &self.global.default_auth_method {
            if auth != "ssh" && auth != "https" {
                return Err(ConfigError::new(
                    ErrorCode::InvalidConfigFormat,
                    format!("Invalid default authentication method: {}", auth),
                    "".to_string(),
                )
                .into());
            }
        }

        // Validate repositories
        for (name, repo) in &self.repositories {
            if let Err(e) = repo.validate() {
                return Err(ConfigError::new(
                    ErrorCode::InvalidConfigFormat,
                    format!("Invalid repository '{}': {}", name, e),
                    format!("Repository: {:?}", repo),
                )
                .into());
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_new_repository() {
        let repo = Repository::new(
            "git@github.com:example/repo.git".to_string(),
            PathBuf::from("example-repo"),
            Some("main".to_string()),
        );

        assert_eq!(repo.url, "git@github.com:example/repo.git");
        assert_eq!(repo.path, PathBuf::from("example-repo"));
        assert_eq!(repo.branch, Some("main".to_string()));
        assert_eq!(repo.auth_method, None);
        assert!(repo.extra.is_empty());
    }

    #[test]
    fn test_detect_auth_method() {
        let mut repo = Repository::new(
            "git@github.com:example/repo.git".to_string(),
            PathBuf::from("example-repo"),
            None,
        );

        repo.detect_auth_method();
        assert_eq!(repo.auth_method, Some("ssh".to_string()));

        let mut repo = Repository::new(
            "https://github.com/example/repo.git".to_string(),
            PathBuf::from("example-repo"),
            None,
        );

        repo.detect_auth_method();
        assert_eq!(repo.auth_method, Some("https".to_string()));
    }

    #[test]
    fn test_save_and_load_config() {
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("mirror.toml");

        let mut config = MirrorConfig::new();
        config.global.default_branch = Some("main".to_string());

        let repo = Repository::new(
            "git@github.com:example/repo.git".to_string(),
            PathBuf::from("example-repo"),
            Some("main".to_string()),
        );

        config.add_repository("example".to_string(), repo).unwrap();

        // Save the configuration
        config.save(&config_path).unwrap();

        // Load the configuration
        let loaded_config = MirrorConfig::load(&config_path).unwrap();

        assert_eq!(
            loaded_config.global.default_branch,
            Some("main".to_string())
        );
        assert_eq!(loaded_config.repositories.len(), 1);
        assert!(loaded_config.repositories.contains_key("example"));

        let loaded_repo = &loaded_config.repositories["example"];
        assert_eq!(loaded_repo.url, "git@github.com:example/repo.git");
        assert_eq!(loaded_repo.path, PathBuf::from("example-repo"));
        assert_eq!(loaded_repo.branch, Some("main".to_string()));
    }
}
