use crate::models::{MirrorConfig, Repository};

/// Manager for workspace-related operations
pub struct WorkspaceManager<'a> {
    config: &'a MirrorConfig,
}

impl<'a> WorkspaceManager<'a> {
    /// Create a new workspace manager
    pub fn new(config: &'a MirrorConfig) -> Self {
        Self { config }
    }

    /// Get all workspace names
    pub fn list_workspaces(&self) -> &[String] {
        &self.config.workspaces
    }

    /// Get repositories for a specific workspace
    pub fn repositories_for(&self, workspace: &str) -> Vec<&Repository> {
        self.config.repositories_in_workspace(workspace)
    }

    /// Get all repositories (optionally filtered by workspace)
    pub fn get_repositories(&self, workspace: Option<&str>) -> Vec<&Repository> {
        self.config.get_repositories(workspace)
    }

    /// Check if a workspace exists
    pub fn workspace_exists(&self, workspace: &str) -> bool {
        self.config.workspaces.contains(&workspace.to_string())
    }

    /// Group repositories by workspace
    pub fn group_by_workspace(&self) -> std::collections::HashMap<String, Vec<&Repository>> {
        self.config.group_by_workspace()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::RefSpec;

    fn create_test_config() -> MirrorConfig {
        let mut config = MirrorConfig::new();
        config.repositories = vec![
            Repository {
                git: "git@github.com:test/repo1.git".to_string(),
                path: "repo1".to_string(),
                ref_spec: RefSpec::default(),
                workspaces: vec!["ws1".to_string(), "ws2".to_string()],
            },
            Repository {
                git: "git@github.com:test/repo2.git".to_string(),
                path: "repo2".to_string(),
                ref_spec: RefSpec::default(),
                workspaces: vec!["ws1".to_string()],
            },
            Repository {
                git: "git@github.com:test/repo3.git".to_string(),
                path: "repo3".to_string(),
                ref_spec: RefSpec::default(),
                workspaces: vec![],
            },
        ];
        config.collect_workspaces();
        config
    }

    #[test]
    fn test_list_workspaces() {
        let config = create_test_config();
        let manager = WorkspaceManager::new(&config);

        let workspaces = manager.list_workspaces();
        assert!(workspaces.contains(&"ws1".to_string()));
        assert!(workspaces.contains(&"ws2".to_string()));
    }

    #[test]
    fn test_repositories_for_workspace() {
        let config = create_test_config();
        let manager = WorkspaceManager::new(&config);

        let repos = manager.repositories_for("ws1");
        assert_eq!(repos.len(), 2);

        let repos = manager.repositories_for("ws2");
        assert_eq!(repos.len(), 1);
    }

    #[test]
    fn test_workspace_exists() {
        let config = create_test_config();
        let manager = WorkspaceManager::new(&config);

        assert!(manager.workspace_exists("ws1"));
        assert!(!manager.workspace_exists("ws3"));
    }

    #[test]
    fn test_group_by_workspace() {
        let config = create_test_config();
        let manager = WorkspaceManager::new(&config);

        let groups = manager.group_by_workspace();
        assert_eq!(groups.get("ws1").map(|v| v.len()), Some(2));
        assert_eq!(groups.get("(no workspace)").map(|v| v.len()), Some(1));
    }
}
