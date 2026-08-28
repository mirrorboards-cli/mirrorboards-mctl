//! Mirror configuration management.

use crate::core::error::{ConfigError, ConfigResult};
use crate::core::include::IncludeResolver;
use crate::core::repository::Repository;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Default configuration file name.
pub const DEFAULT_CONFIG_FILE: &str = "mirror.toml";

/// Default snapshot file name.
pub const DEFAULT_SNAPSHOT_FILE: &str = "mirror.snapshot.toml";

/// Remote configuration for syncing mirror.toml.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct RemoteConfig {
    /// Git URL of the remote config repository
    pub git: String,

    /// Branch to use (default: "main")
    #[serde(default = "default_branch")]
    pub branch: String,

    /// Path to the config file in the repository (default: "mirror.toml")
    #[serde(default = "default_remote_path")]
    pub path: String,
}

fn default_branch() -> String {
    "main".to_string()
}

fn default_remote_path() -> String {
    "mirror.toml".to_string()
}

/// Include configuration section [includes].
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct IncludesSection {
    /// Paths to include
    #[serde(default)]
    pub paths: Vec<String>,
}

/// Raw mirror configuration as stored in TOML.
///
/// This is used for parsing before include resolution.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RawMirrorConfig {
    /// Include paths to other config files (top-level array format)
    /// Must be at the top of file, before any [section]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub include: Vec<String>,

    /// Include paths as section [includes] with paths = [...]
    /// Can be placed anywhere in the file
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub includes: Option<IncludesSection>,

    /// Remote config for syncing
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub remote: Option<RemoteConfig>,

    /// List of repositories
    #[serde(default)]
    pub repositories: Vec<Repository>,
}

impl RawMirrorConfig {
    /// Get all include paths (from both formats).
    pub fn get_includes(&self) -> Vec<String> {
        let mut result = self.include.clone();
        if let Some(ref section) = self.includes {
            result.extend(section.paths.clone());
        }
        result
    }
}

/// Resolved mirror configuration with all includes processed.
#[derive(Debug, Clone)]
pub struct MirrorConfig {
    /// Remote config for syncing
    pub remote: Option<RemoteConfig>,

    /// All repositories (from main file and includes)
    pub repositories: Vec<Repository>,

    /// Path to the main config file
    pub config_path: PathBuf,

    /// All source files (main + includes)
    pub source_files: Vec<PathBuf>,
}

impl MirrorConfig {
    /// Load configuration from the default file (mirror.toml).
    pub fn load_default() -> ConfigResult<Self> {
        Self::load(Path::new(DEFAULT_CONFIG_FILE))
    }

    /// Load configuration from a specific path with include resolution.
    pub fn load(path: &Path) -> ConfigResult<Self> {
        let resolved = IncludeResolver::resolve(path)?;

        // Load the main config for remote settings
        let content = std::fs::read_to_string(path)?;
        let raw_config: RawMirrorConfig = toml::from_str(&content)?;

        // Extract just the repositories (without source info)
        let repositories = resolved
            .repositories
            .into_iter()
            .map(|r| r.repository)
            .collect();

        Ok(Self {
            remote: raw_config.remote,
            repositories,
            config_path: path.to_path_buf(),
            source_files: resolved.source_files,
        })
    }

    /// Load configuration without resolving includes.
    ///
    /// Useful for editing the main config file directly.
    pub fn load_raw(path: &Path) -> ConfigResult<RawMirrorConfig> {
        let content = std::fs::read_to_string(path)?;
        let config: RawMirrorConfig = toml::from_str(&content)?;
        Ok(config)
    }

    /// Save the raw configuration to a file.
    pub fn save_raw(config: &RawMirrorConfig, path: &Path) -> ConfigResult<()> {
        let content = toml::to_string_pretty(config)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Filter repositories by workspace.
    pub fn filter_by_workspace(&self, workspace: &str) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|r| r.is_in_workspace(workspace))
            .collect()
    }

    /// Get all unique workspaces.
    pub fn list_workspaces(&self) -> Vec<String> {
        let mut workspaces: HashSet<String> = HashSet::new();
        for repo in &self.repositories {
            for ws in &repo.workspaces {
                workspaces.insert(ws.clone());
            }
        }
        let mut result: Vec<_> = workspaces.into_iter().collect();
        result.sort();
        result
    }

    /// Find a repository by path.
    pub fn find_by_path(&self, path: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.path == path)
    }

    /// Find a repository by git URL.
    pub fn find_by_git(&self, git: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.git == git)
    }

    /// Validate all repositories in the configuration.
    pub fn validate(&self) -> ConfigResult<()> {
        for repo in &self.repositories {
            repo.validate()
                .map_err(|msg| ConfigError::ValidationError { message: msg })?;
        }
        Ok(())
    }
}

/// Configuration manager for editing configs.
pub struct ConfigManager {
    path: PathBuf,
    config: RawMirrorConfig,
}

impl ConfigManager {
    /// Open a configuration file for editing.
    pub fn open(path: &Path) -> ConfigResult<Self> {
        let config = if path.exists() {
            MirrorConfig::load_raw(path)?
        } else {
            RawMirrorConfig::default()
        };

        Ok(Self {
            path: path.to_path_buf(),
            config,
        })
    }

    /// Create a new configuration file.
    pub fn create(path: &Path) -> ConfigResult<Self> {
        if path.exists() {
            return Err(ConfigError::ValidationError {
                message: format!("Configuration file already exists: {}", path.display()),
            });
        }

        Ok(Self {
            path: path.to_path_buf(),
            config: RawMirrorConfig::default(),
        })
    }

    /// Add a repository to the configuration.
    pub fn add_repository(&mut self, repo: Repository) -> ConfigResult<()> {
        // Check for duplicate path
        if self.config.repositories.iter().any(|r| r.path == repo.path) {
            return Err(ConfigError::DuplicatePath {
                path: repo.path.clone(),
            });
        }

        repo.validate()
            .map_err(|msg| ConfigError::ValidationError { message: msg })?;

        self.config.repositories.push(repo);
        Ok(())
    }

    /// Remove a repository by path.
    pub fn remove_repository(&mut self, path: &str) -> ConfigResult<Repository> {
        let index = self
            .config
            .repositories
            .iter()
            .position(|r| r.path == path)
            .ok_or_else(|| ConfigError::ValidationError {
                message: format!("Repository not found: {}", path),
            })?;

        Ok(self.config.repositories.remove(index))
    }

    /// Set remote configuration.
    pub fn set_remote(&mut self, remote: RemoteConfig) {
        self.config.remote = Some(remote);
    }

    /// Remove remote configuration.
    pub fn remove_remote(&mut self) {
        self.config.remote = None;
    }

    /// Add an include path.
    pub fn add_include(&mut self, path: String) {
        if !self.config.include.contains(&path) {
            self.config.include.push(path);
        }
    }

    /// Remove an include path.
    pub fn remove_include(&mut self, path: &str) -> bool {
        if let Some(index) = self.config.include.iter().position(|p| p == path) {
            self.config.include.remove(index);
            true
        } else {
            false
        }
    }

    /// Save the configuration to disk.
    pub fn save(&self) -> ConfigResult<()> {
        MirrorConfig::save_raw(&self.config, &self.path)
    }

    /// Get a reference to the raw config.
    pub fn config(&self) -> &RawMirrorConfig {
        &self.config
    }

    /// Get a mutable reference to the raw config.
    pub fn config_mut(&mut self) -> &mut RawMirrorConfig {
        &mut self.config
    }
}

/// Create a snapshot configuration from a list of repositories.
///
/// Converts all repositories to use `rev` (commit hash) instead of branch/tag.
pub fn create_snapshot(
    repositories: &[Repository],
    revisions: &[(String, String)], // (path, rev)
    workspace_filter: Option<&str>,
) -> RawMirrorConfig {
    let rev_map: std::collections::HashMap<_, _> = revisions.iter().cloned().collect();

    let repos: Vec<Repository> = repositories
        .iter()
        .filter(|r| workspace_filter.map_or(true, |ws| r.is_in_workspace(ws)))
        .map(|r| {
            let mut repo = r.clone();
            // Convert to rev-based
            if let Some(rev) = rev_map.get(&repo.path) {
                repo.branch = None;
                repo.tag = None;
                repo.rev = Some(rev.clone());
            }
            // Clear workspaces in snapshot (optional, keeps it cleaner)
            repo.workspaces.clear();
            repo
        })
        .collect();

    RawMirrorConfig {
        include: Vec::new(),
        includes: None,
        remote: None,
        repositories: repos,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    fn create_test_config(dir: &Path) -> PathBuf {
        let path = dir.join("mirror.toml");
        let content = r#"
[[repositories]]
git = "git@github.com:test/repo1.git"
path = "repo1"
branch = "main"
workspaces = ["api"]

[[repositories]]
git = "git@github.com:test/repo2.git"
path = "repo2"
tag = "v1.0.0"
workspaces = ["api", "core"]
"#;
        std::fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_load_config() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());

        let config = MirrorConfig::load(&config_path).unwrap();
        assert_eq!(config.repositories.len(), 2);
    }

    #[test]
    fn test_filter_by_workspace() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());

        let config = MirrorConfig::load(&config_path).unwrap();

        let api_repos = config.filter_by_workspace("api");
        assert_eq!(api_repos.len(), 2);

        let core_repos = config.filter_by_workspace("core");
        assert_eq!(core_repos.len(), 1);
    }

    #[test]
    fn test_list_workspaces() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = create_test_config(temp_dir.path());

        let config = MirrorConfig::load(&config_path).unwrap();
        let workspaces = config.list_workspaces();

        assert!(workspaces.contains(&"api".to_string()));
        assert!(workspaces.contains(&"core".to_string()));
    }

    #[test]
    fn test_config_manager() {
        let temp_dir = TempDir::new().unwrap();
        let config_path = temp_dir.path().join("mirror.toml");

        let mut manager = ConfigManager::create(&config_path).unwrap();

        let repo = Repository::new("git@github.com:test/repo.git", "test/repo");
        manager.add_repository(repo).unwrap();
        manager.save().unwrap();

        // Reload and verify
        let config = MirrorConfig::load(&config_path).unwrap();
        assert_eq!(config.repositories.len(), 1);
    }

    #[test]
    fn test_create_snapshot() {
        let repos = vec![
            Repository::new("git@github.com:test/repo1.git", "repo1")
                .with_branch("main")
                .with_workspaces(vec!["api".to_string()]),
            Repository::new("git@github.com:test/repo2.git", "repo2")
                .with_tag("v1.0.0")
                .with_workspaces(vec!["core".to_string()]),
        ];

        let revisions = vec![
            ("repo1".to_string(), "abc123".to_string()),
            ("repo2".to_string(), "def456".to_string()),
        ];

        let snapshot = create_snapshot(&repos, &revisions, None);
        assert_eq!(snapshot.repositories.len(), 2);
        assert!(snapshot.repositories[0].rev.is_some());
        assert!(snapshot.repositories[0].branch.is_none());
    }
}
