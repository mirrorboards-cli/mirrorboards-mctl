use std::collections::HashSet;
use std::path::{Path, PathBuf};

use crate::error::{MirrorError, Result};
use crate::models::{MirrorConfig, RawMirrorConfig};

/// Loader for mirror configuration files with include resolution
pub struct ConfigLoader {
    /// Track visited files to detect cycles
    visited: HashSet<PathBuf>,
}

impl ConfigLoader {
    /// Create a new config loader
    pub fn new() -> Self {
        Self {
            visited: HashSet::new(),
        }
    }

    /// Load configuration from a file path
    pub fn load(&mut self, path: impl AsRef<Path>) -> Result<MirrorConfig> {
        let path = path.as_ref().canonicalize().map_err(|e| {
            MirrorError::Config(format!("Failed to resolve path {}: {}", path.as_ref().display(), e))
        })?;

        self.load_recursive(&path)
    }

    /// Load configuration from string content
    pub fn load_from_str(&mut self, content: &str, base_path: impl AsRef<Path>) -> Result<MirrorConfig> {
        let raw: RawMirrorConfig = toml::from_str(content)?;
        self.process_raw_config(raw, base_path.as_ref())
    }

    fn load_recursive(&mut self, path: &Path) -> Result<MirrorConfig> {
        // Check for cycles
        if self.visited.contains(path) {
            return Err(MirrorError::IncludeCycle(path.display().to_string()));
        }
        self.visited.insert(path.to_path_buf());

        // Read and parse the file
        let content = std::fs::read_to_string(path).map_err(|e| {
            MirrorError::Config(format!("Failed to read {}: {}", path.display(), e))
        })?;

        let raw: RawMirrorConfig = toml::from_str(&content)?;

        let base_dir = path.parent().ok_or_else(|| {
            MirrorError::Config(format!("Invalid path: {}", path.display()))
        })?;

        self.process_raw_config(raw, base_dir)
    }

    fn process_raw_config(&mut self, raw: RawMirrorConfig, base_dir: &Path) -> Result<MirrorConfig> {
        let mut config = MirrorConfig::new();

        // Set config repo from root config only
        config.config_repo = raw.config_repo;

        // Process repositories from this file
        for raw_repo in raw.repositories {
            let repo = raw_repo.into_repository()?;
            config.repositories.push(repo);
        }

        // Process includes
        for include_path in raw.include {
            let resolved_path = base_dir.join(&include_path);
            let canonical_path = resolved_path.canonicalize().map_err(|e| {
                MirrorError::Config(format!(
                    "Failed to resolve include path {}: {}",
                    include_path, e
                ))
            })?;

            let included_config = self.load_recursive(&canonical_path)?;

            // Merge repositories from included config
            for repo in included_config.repositories {
                config.repositories.push(repo);
            }
        }

        // Collect unique workspaces
        config.collect_workspaces();

        Ok(config)
    }
}

impl Default for ConfigLoader {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_test_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_load_simple_config() {
        let dir = TempDir::new().unwrap();
        let config_content = r#"
[[repositories]]
git = "git@github.com:test/repo1.git"
path = "repo1"
workspaces = ["ws1"]
"#;

        let path = create_test_file(dir.path(), "mirror.toml", config_content);
        let config = ConfigLoader::new().load(&path).unwrap();

        assert_eq!(config.repositories.len(), 1);
        assert_eq!(config.repositories[0].path, "repo1");
    }

    #[test]
    fn test_load_with_include() {
        let dir = TempDir::new().unwrap();

        let included_content = r#"
[[repositories]]
git = "git@github.com:test/repo2.git"
path = "repo2"
"#;
        create_test_file(dir.path(), "included.toml", included_content);

        let main_content = r#"
include = ["./included.toml"]

[[repositories]]
git = "git@github.com:test/repo1.git"
path = "repo1"
"#;
        let path = create_test_file(dir.path(), "mirror.toml", main_content);

        let config = ConfigLoader::new().load(&path).unwrap();
        assert_eq!(config.repositories.len(), 2);
    }

    #[test]
    fn test_detect_cycle() {
        let dir = TempDir::new().unwrap();

        let a_content = r#"include = ["./b.toml"]"#;
        let b_content = r#"include = ["./a.toml"]"#;

        create_test_file(dir.path(), "a.toml", a_content);
        create_test_file(dir.path(), "b.toml", b_content);

        let path = dir.path().join("a.toml");
        let result = ConfigLoader::new().load(&path);

        assert!(matches!(result, Err(MirrorError::IncludeCycle(_))));
    }

    #[test]
    fn test_config_repo_parsing() {
        let dir = TempDir::new().unwrap();
        let config_content = r#"
[config-repo]
git = "git@github.com:test/config.git"
branch = "main"
config_path = "mirror.toml"
snapshots_dir = "snapshots"
"#;

        let path = create_test_file(dir.path(), "mirror.toml", config_content);
        let config = ConfigLoader::new().load(&path).unwrap();

        assert!(config.config_repo.is_some());
        let cr = config.config_repo.unwrap();
        assert_eq!(cr.git, "git@github.com:test/config.git");
    }
}
