use crate::error::{MctlError, MctlResult};
use log::{debug, info};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use url::Url;

/// Repository configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Git URL of the repository
    #[serde(rename = "git-url")]
    pub git_url: String,

    /// Local path where the repository will be cloned
    pub path: String,

    /// Branch to clone (optional)
    #[serde(skip_serializing_if = "Option::is_none")]
    pub branch: Option<String>,
}

impl Repository {
    /// Create a new repository configuration
    pub fn new(git_url: String, path: String, branch: Option<String>) -> Self {
        Self {
            git_url,
            path,
            branch,
        }
    }

    /// Get the absolute path of the repository
    pub fn absolute_path(&self, base_dir: &Path) -> PathBuf {
        if Path::new(&self.path).is_absolute() {
            Path::new(&self.path).to_path_buf()
        } else {
            base_dir.join(&self.path)
        }
    }

    /// Check if the repository exists locally
    pub fn exists_locally(&self, base_dir: &Path) -> bool {
        let path = self.absolute_path(base_dir);
        path.exists() && path.join(".git").exists()
    }
}

/// MCTL configuration
#[derive(Debug, Serialize, Deserialize)]
pub struct Config {
    /// List of repositories to mirror
    pub repositories: Vec<Repository>,
}

impl Config {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self {
            repositories: Vec::new(),
        }
    }

    /// Add a repository to the configuration
    pub fn add_repository(&mut self, repository: Repository) {
        self.repositories.push(repository);
    }

    /// Save the configuration to a file
    pub fn save(&self, path: &Path) -> MctlResult<()> {
        let toml = toml::to_string_pretty(self)?;
        fs::write(path, toml)?;
        Ok(())
    }
}

/// Load configuration from a file
pub fn load_config(path: &Path) -> MctlResult<Config> {
    // Check if the file exists
    if !path.exists() {
        debug!("Configuration file not found, creating a new one");
        let config = Config::new();
        config.save(path)?;
        return Ok(config);
    }

    // Read the file
    let content = fs::read_to_string(path)?;

    // Parse the TOML
    let config: Config = toml::from_str(&content)
        .map_err(|e| MctlError::ConfigError(format!("Failed to parse configuration: {}", e)))?;

    info!(
        "Loaded configuration with {} repositories",
        config.repositories.len()
    );

    Ok(config)
}

/// Validate a repository configuration
pub fn validate_repository(git_url: &str, path: &str) -> MctlResult<()> {
    // Validate git URL
    if git_url.is_empty() {
        return Err(MctlError::InvalidRepositoryUrl(
            "Git URL cannot be empty".to_string(),
        ));
    }

    // Try to parse the URL
    Url::parse(git_url)
        .map_err(|_| MctlError::InvalidRepositoryUrl(format!("Invalid Git URL: {}", git_url)))?;

    // Validate path
    if path.is_empty() {
        return Err(MctlError::InvalidPath(PathBuf::from("<empty>")));
    }

    Ok(())
}
