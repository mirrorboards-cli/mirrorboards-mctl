//! Include file resolution with cycle detection.

use crate::core::config::RawMirrorConfig;
use crate::core::error::{ConfigError, ConfigResult};
use crate::core::repository::Repository;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

/// Resolved configuration from multiple include files.
#[derive(Debug, Clone)]
pub struct ResolvedConfig {
    /// All repositories from all included files
    pub repositories: Vec<RepositoryWithSource>,
    /// All source files that were processed
    pub source_files: Vec<PathBuf>,
}

/// A repository with its source file for debugging.
#[derive(Debug, Clone)]
pub struct RepositoryWithSource {
    pub repository: Repository,
    pub source_file: PathBuf,
}

/// Include resolver with cycle detection.
pub struct IncludeResolver {
    /// Stack of currently processing files (for cycle detection)
    processing_stack: Vec<PathBuf>,
    /// Set of already processed files
    processed_files: HashSet<PathBuf>,
    /// All collected repositories
    repositories: Vec<RepositoryWithSource>,
}

impl IncludeResolver {
    pub fn new() -> Self {
        Self {
            processing_stack: Vec::new(),
            processed_files: HashSet::new(),
            repositories: Vec::new(),
        }
    }

    /// Resolve all includes starting from the given config file.
    pub fn resolve(config_path: &Path) -> ConfigResult<ResolvedConfig> {
        let mut resolver = Self::new();
        resolver.resolve_file(config_path)?;

        // Check for duplicate paths
        resolver.check_duplicates()?;

        Ok(ResolvedConfig {
            repositories: resolver.repositories,
            source_files: resolver.processed_files.into_iter().collect(),
        })
    }

    fn resolve_file(&mut self, config_path: &Path) -> ConfigResult<()> {
        let canonical_path = config_path.canonicalize().map_err(|_| ConfigError::NotFound {
            path: config_path.to_path_buf(),
        })?;

        // Check for cycles
        if self.processing_stack.contains(&canonical_path) {
            let cycle = self
                .processing_stack
                .iter()
                .map(|p| p.display().to_string())
                .collect::<Vec<_>>()
                .join(" -> ");
            return Err(ConfigError::IncludeCycle {
                cycle: format!("{} -> {}", cycle, canonical_path.display()),
            });
        }

        // Skip already processed files
        if self.processed_files.contains(&canonical_path) {
            return Ok(());
        }

        // Push to stack
        self.processing_stack.push(canonical_path.clone());

        // Load and parse the file
        let content = std::fs::read_to_string(&canonical_path)?;
        let raw_config: RawMirrorConfig =
            toml::from_str(&content).map_err(|e| ConfigError::ParseError {
                message: format!("Failed to parse {}: {}", canonical_path.display(), e),
            })?;

        // Process includes first (depth-first)
        let base_dir = canonical_path.parent().unwrap_or(Path::new("."));
        let includes = raw_config.get_includes();
        for include_path in &includes {
            let resolved_include = if Path::new(include_path).is_absolute() {
                PathBuf::from(include_path)
            } else {
                base_dir.join(include_path)
            };

            if !resolved_include.exists() {
                return Err(ConfigError::IncludeNotFound {
                    path: resolved_include,
                    referenced_from: canonical_path.display().to_string(),
                });
            }

            self.resolve_file(&resolved_include)?;
        }

        // Add repositories from this file
        for repo in raw_config.repositories {
            self.repositories.push(RepositoryWithSource {
                repository: repo,
                source_file: canonical_path.clone(),
            });
        }

        // Pop from stack and mark as processed
        self.processing_stack.pop();
        self.processed_files.insert(canonical_path);

        Ok(())
    }

    fn check_duplicates(&self) -> ConfigResult<()> {
        let mut paths = HashSet::new();
        for repo_with_source in &self.repositories {
            let path = &repo_with_source.repository.path;
            if !paths.insert(path.clone()) {
                return Err(ConfigError::DuplicatePath { path: path.clone() });
            }
        }
        Ok(())
    }
}

impl Default for IncludeResolver {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use tempfile::TempDir;

    fn create_config_file(dir: &Path, name: &str, content: &str) -> PathBuf {
        let path = dir.join(name);
        fs::write(&path, content).unwrap();
        path
    }

    #[test]
    fn test_simple_include() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        create_config_file(
            base_dir,
            "included.toml",
            r#"
[[repositories]]
git = "git@github.com:test/repo1.git"
path = "repo1"
"#,
        );

        let main_config = create_config_file(
            base_dir,
            "mirror.toml",
            r#"
include = ["./included.toml"]

[[repositories]]
git = "git@github.com:test/repo2.git"
path = "repo2"
"#,
        );

        let resolved = IncludeResolver::resolve(&main_config).unwrap();
        assert_eq!(resolved.repositories.len(), 2);
    }

    #[test]
    fn test_cycle_detection() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        create_config_file(
            base_dir,
            "a.toml",
            r#"
include = ["./b.toml"]

[[repositories]]
git = "git@github.com:test/a.git"
path = "a"
"#,
        );

        create_config_file(
            base_dir,
            "b.toml",
            r#"
include = ["./a.toml"]

[[repositories]]
git = "git@github.com:test/b.git"
path = "b"
"#,
        );

        let result = IncludeResolver::resolve(&base_dir.join("a.toml"));
        assert!(matches!(result, Err(ConfigError::IncludeCycle { .. })));
    }

    #[test]
    fn test_duplicate_path_detection() {
        let temp_dir = TempDir::new().unwrap();
        let base_dir = temp_dir.path();

        create_config_file(
            base_dir,
            "included.toml",
            r#"
[[repositories]]
git = "git@github.com:test/repo1.git"
path = "same-path"
"#,
        );

        let main_config = create_config_file(
            base_dir,
            "mirror.toml",
            r#"
include = ["./included.toml"]

[[repositories]]
git = "git@github.com:test/repo2.git"
path = "same-path"
"#,
        );

        let result = IncludeResolver::resolve(&main_config);
        assert!(matches!(result, Err(ConfigError::DuplicatePath { .. })));
    }
}
