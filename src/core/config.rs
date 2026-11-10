//! Mirror configuration management.

use crate::core::error::{ConfigError, ConfigResult};
use crate::core::include::IncludeResolver;
use crate::core::repository::Repository;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::path::{Component, Path, PathBuf};
use toml_edit::{Array, ArrayOfTables, DocumentMut, Item, Table, Value};

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

/// Edits one configuration file in place.
///
/// Edits go through `toml_edit`, so comments, blank lines and the order of
/// sections survive; only the entries an operation touches change.
pub struct ConfigManager {
    path: PathBuf,
    document: DocumentMut,
}

impl ConfigManager {
    /// Open a configuration file for editing. A missing file starts empty.
    pub fn open(path: &Path) -> ConfigResult<Self> {
        let content = if path.exists() {
            std::fs::read_to_string(path)?
        } else {
            String::new()
        };

        // Refuse to edit a file that does not follow the configuration schema.
        toml::from_str::<RawMirrorConfig>(&content).map_err(|error| ConfigError::ParseError {
            message: format!("{}: {}", path.display(), error),
        })?;

        let document = content.parse::<DocumentMut>()?;

        Ok(Self {
            path: path.to_path_buf(),
            document,
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
            document: DocumentMut::new(),
        })
    }

    /// Add a repository to the configuration.
    pub fn add_repository(&mut self, repo: Repository) -> ConfigResult<()> {
        repo.validate()
            .map_err(|msg| ConfigError::ValidationError { message: msg })?;

        let path_taken = self
            .raw_config()?
            .repositories
            .iter()
            .any(|r| r.path == repo.path);
        if path_taken {
            return Err(ConfigError::DuplicatePath {
                path: repo.path.clone(),
            });
        }

        let entry = toml_edit::ser::to_document(&repo)?;
        self.repositories_mut()?.push(entry.as_table().clone());

        Ok(())
    }

    /// Remove a repository by path.
    ///
    /// Comment lines directly above the entry go with it. Text separated from
    /// the entry by a blank line, such as the description at the top of an
    /// include file, describes the file and stays in it.
    pub fn remove_repository(&mut self, path: &str) -> ConfigResult<Repository> {
        let repository = self
            .raw_config()?
            .repositories
            .into_iter()
            .find(|r| r.path == path)
            .ok_or_else(|| ConfigError::ValidationError {
                message: format!("Repository not found: {}", path),
            })?;

        let repositories = self.repositories_mut()?;
        let index = repositories
            .iter()
            .position(|table| table.get("path").and_then(|value| value.as_str()) == Some(path))
            .ok_or_else(|| ConfigError::ValidationError {
                message: format!("Repository not found: {}", path),
            })?;

        let removed_prefix = leading_text(repositories.get(index));
        let file_text = text_describing_the_file(&removed_prefix).to_string();
        repositories.remove(index);

        if file_text.is_empty() {
            if index == 0 {
                self.drop_blank_lines_at_the_top()?;
            }
            return Ok(repository);
        }

        // The entry that followed the removed one now sits at the same index.
        let next_entry = repositories.get_mut(index);
        match next_entry {
            Some(next) => {
                let next_prefix = leading_text(Some(next));
                next.decor_mut().set_prefix(file_text + &next_prefix);
            }
            None => {
                let trailing = self.document.trailing().as_str().unwrap_or("");
                let new_trailing = file_text + trailing;
                self.document.set_trailing(new_trailing);
            }
        }

        Ok(repository)
    }

    /// Whether the file holds nothing at all: no entries, no includes, no text.
    pub fn is_empty(&self) -> bool {
        self.document.to_string().trim().is_empty()
    }

    /// Drop `include_file` from `[includes].paths`.
    ///
    /// Returns `false` when this configuration does not list that file there,
    /// under any spelling of its path.
    pub fn remove_include(&mut self, include_file: &Path) -> ConfigResult<bool> {
        let target = include_file.canonicalize()?;
        let base_dir = self.path.parent().unwrap_or(Path::new(".")).to_path_buf();

        let paths = self
            .document
            .get_mut("includes")
            .and_then(Item::as_table_mut)
            .and_then(|includes| includes.get_mut("paths"))
            .and_then(Item::as_array_mut);
        let Some(paths) = paths else {
            return Ok(false);
        };

        let position = paths.iter().position(|listed| {
            let Some(listed_path) = listed.as_str() else {
                return false;
            };

            let listed_file = base_dir.join(without_current_dir(listed_path));
            listed_file.canonicalize().ok().as_ref() == Some(&target)
        });
        let Some(index) = position else {
            return Ok(false);
        };

        paths.remove(index);
        Ok(true)
    }

    /// Set remote configuration.
    pub fn set_remote(&mut self, remote: RemoteConfig) -> ConfigResult<()> {
        let table = toml_edit::ser::to_document(&remote)?;
        self.document["remote"] = Item::Table(table.as_table().clone());

        Ok(())
    }

    /// Register an include file in `[includes].paths`.
    ///
    /// Returns `false` when the configuration already includes that file,
    /// under any spelling of the same path.
    pub fn add_include(&mut self, include_path: &str) -> ConfigResult<bool> {
        let new_file = without_current_dir(include_path);

        let already_included = self
            .raw_config()?
            .get_includes()
            .iter()
            .any(|existing| without_current_dir(existing) == new_file);
        if already_included {
            return Ok(false);
        }

        let file_name = self.path.display().to_string();
        let includes = self
            .document
            .entry("includes")
            .or_insert(Item::Table(Table::new()))
            .as_table_mut()
            .ok_or_else(|| ConfigError::ValidationError {
                message: format!("{}: includes must be a [includes] table", file_name),
            })?;
        let paths = includes
            .entry("paths")
            .or_insert(toml_edit::value(Array::new()))
            .as_array_mut()
            .ok_or_else(|| ConfigError::ValidationError {
                message: format!("{}: includes.paths must be an array", file_name),
            })?;

        // A new entry copies the layout of the first one, so a list written
        // one path per line stays that way.
        let mut entry = Value::from(include_path);
        if let Some(first) = paths.iter().next() {
            *entry.decor_mut() = first.decor().clone();
        }
        paths.push_formatted(entry);

        Ok(true)
    }

    /// Save the configuration to disk.
    pub fn save(&self) -> ConfigResult<()> {
        // A file opened as missing may also be the first one in its directory.
        if let Some(directory) = self.path.parent() {
            std::fs::create_dir_all(directory)?;
        }

        std::fs::write(&self.path, self.document.to_string())?;
        Ok(())
    }

    /// Removing the first entry leaves the blank line that separated it from the
    /// next one at the top of the file; the file starts with its text instead.
    fn drop_blank_lines_at_the_top(&mut self) -> ConfigResult<()> {
        let starts_with_blank_line = self.document.to_string().starts_with('\n');
        if !starts_with_blank_line {
            return Ok(());
        }

        if let Some(first) = self.repositories_mut()?.get_mut(0) {
            let prefix = leading_text(Some(first));
            let without_blank_lines = prefix.trim_start_matches('\n').to_string();
            first.decor_mut().set_prefix(without_blank_lines);
        }

        Ok(())
    }

    /// The configuration as it stands after the edits so far.
    fn raw_config(&self) -> ConfigResult<RawMirrorConfig> {
        let config = toml::from_str(&self.document.to_string())?;
        Ok(config)
    }

    fn repositories_mut(&mut self) -> ConfigResult<&mut ArrayOfTables> {
        let file_name = self.path.display().to_string();

        self.document
            .entry("repositories")
            .or_insert(Item::ArrayOfTables(ArrayOfTables::new()))
            .as_array_of_tables_mut()
            .ok_or_else(|| ConfigError::ValidationError {
                message: format!(
                    "{}: repositories must be [[repositories]] tables",
                    file_name
                ),
            })
    }
}

/// The text of a file between the previous entry and this table's header.
fn leading_text(table: Option<&Table>) -> String {
    let prefix = table.and_then(|table| table.decor().prefix());
    let text = prefix.and_then(|raw| raw.as_str()).unwrap_or("");

    text.to_string()
}

/// The part of the text above an entry that is separated from it by a blank
/// line; what remains below the last blank line describes the entry itself.
fn text_describing_the_file(leading_text: &str) -> &str {
    match leading_text.rfind("\n\n") {
        Some(blank_line) => &leading_text[..blank_line + 1],
        None => "",
    }
}

/// An include path without `.` components, so `./a.toml` and `a.toml` match.
fn without_current_dir(include_path: &str) -> PathBuf {
    Path::new(include_path)
        .components()
        .filter(|component| *component != Component::CurDir)
        .collect()
}

/// Find a configuration in a parent directory that includes `config_path`.
///
/// Repository paths in an included file are relative to the configuration
/// that includes it, so commands given only the included file would read and
/// write repositories in the wrong place. A parent configuration that does
/// not resolve is skipped: it cannot say anything about this file.
pub fn find_including_config(config_path: &Path) -> Option<PathBuf> {
    let config_file = config_path.canonicalize().ok()?;
    let config_dir = config_file.parent()?;

    for directory in config_dir.ancestors() {
        let candidate = directory.join(DEFAULT_CONFIG_FILE);

        let is_other_config = candidate.exists() && candidate != config_file;
        if !is_other_config {
            continue;
        }

        let Ok(resolved) = IncludeResolver::resolve(&candidate) else {
            continue;
        };

        if resolved.source_files.contains(&config_file) {
            return Some(candidate);
        }
    }

    None
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
