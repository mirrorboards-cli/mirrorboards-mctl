use serde::{Deserialize, Serialize};

use super::{ConfigRepo, RawRepository};

/// Raw configuration as parsed from TOML
#[derive(Debug, Clone, Deserialize, Serialize, Default)]
pub struct RawMirrorConfig {
    /// Paths to include (relative to config file)
    #[serde(default)]
    pub include: Vec<String>,

    /// Central config repository configuration
    #[serde(rename = "config-repo", default, skip_serializing_if = "Option::is_none")]
    pub config_repo: Option<ConfigRepo>,

    /// Repository definitions
    #[serde(default)]
    pub repositories: Vec<RawRepository>,
}

use super::Repository;

/// Resolved mirror configuration
#[derive(Debug, Clone, Default)]
pub struct MirrorConfig {
    /// Central config repository configuration
    pub config_repo: Option<ConfigRepo>,

    /// All resolved repositories
    pub repositories: Vec<Repository>,

    /// All unique workspaces found in repositories
    pub workspaces: Vec<String>,
}

impl MirrorConfig {
    /// Create a new empty configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Get repositories filtered by workspace
    pub fn repositories_in_workspace(&self, workspace: &str) -> Vec<&Repository> {
        self.repositories
            .iter()
            .filter(|r| r.in_workspace(workspace))
            .collect()
    }

    /// Get all repositories (optionally filtered by workspace)
    pub fn get_repositories(&self, workspace: Option<&str>) -> Vec<&Repository> {
        match workspace {
            Some(ws) => self.repositories_in_workspace(ws),
            None => self.repositories.iter().collect(),
        }
    }

    /// Group repositories by workspace
    pub fn group_by_workspace(&self) -> std::collections::HashMap<String, Vec<&Repository>> {
        let mut groups: std::collections::HashMap<String, Vec<&Repository>> =
            std::collections::HashMap::new();

        for repo in &self.repositories {
            if repo.workspaces.is_empty() {
                groups
                    .entry("(no workspace)".to_string())
                    .or_default()
                    .push(repo);
            } else {
                for ws in &repo.workspaces {
                    groups.entry(ws.clone()).or_default().push(repo);
                }
            }
        }

        groups
    }

    /// Find repository by path
    pub fn find_by_path(&self, path: &str) -> Option<&Repository> {
        self.repositories.iter().find(|r| r.path == path)
    }

    /// Collect unique workspaces from all repositories
    pub fn collect_workspaces(&mut self) {
        let mut workspaces: std::collections::HashSet<String> = std::collections::HashSet::new();

        for repo in &self.repositories {
            for ws in &repo.workspaces {
                workspaces.insert(ws.clone());
            }
        }

        self.workspaces = workspaces.into_iter().collect();
        self.workspaces.sort();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RefSpec;

    fn create_test_repo(path: &str, workspaces: Vec<&str>) -> Repository {
        Repository {
            git: format!("git@github.com:test/{path}.git"),
            path: path.to_string(),
            ref_spec: RefSpec::default(),
            workspaces: workspaces.into_iter().map(String::from).collect(),
        }
    }

    #[test]
    fn test_repositories_in_workspace() {
        let mut config = MirrorConfig::new();
        config.repositories = vec![
            create_test_repo("repo1", vec!["ws1", "ws2"]),
            create_test_repo("repo2", vec!["ws1"]),
            create_test_repo("repo3", vec!["ws2"]),
        ];

        let ws1_repos = config.repositories_in_workspace("ws1");
        assert_eq!(ws1_repos.len(), 2);

        let ws2_repos = config.repositories_in_workspace("ws2");
        assert_eq!(ws2_repos.len(), 2);
    }

    #[test]
    fn test_group_by_workspace() {
        let mut config = MirrorConfig::new();
        config.repositories = vec![
            create_test_repo("repo1", vec!["ws1"]),
            create_test_repo("repo2", vec!["ws1", "ws2"]),
            create_test_repo("repo3", vec![]),
        ];

        let groups = config.group_by_workspace();
        assert_eq!(groups.get("ws1").map(|v| v.len()), Some(2));
        assert_eq!(groups.get("ws2").map(|v| v.len()), Some(1));
        assert_eq!(groups.get("(no workspace)").map(|v| v.len()), Some(1));
    }

    #[test]
    fn test_collect_workspaces() {
        let mut config = MirrorConfig::new();
        config.repositories = vec![
            create_test_repo("repo1", vec!["ws1", "ws2"]),
            create_test_repo("repo2", vec!["ws1", "ws3"]),
        ];

        config.collect_workspaces();
        assert_eq!(config.workspaces, vec!["ws1", "ws2", "ws3"]);
    }
}
