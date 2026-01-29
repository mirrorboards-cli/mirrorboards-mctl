use std::collections::HashSet;

use crate::error::{MirrorError, Result};
use crate::models::MirrorConfig;

/// Validator for mirror configuration
pub struct ConfigValidator;

impl ConfigValidator {
    /// Validate the configuration
    pub fn validate(config: &MirrorConfig) -> Result<()> {
        Self::check_duplicate_paths(config)?;
        Ok(())
    }

    /// Check for duplicate repository paths
    fn check_duplicate_paths(config: &MirrorConfig) -> Result<()> {
        let mut seen_paths = HashSet::new();

        for repo in &config.repositories {
            if !seen_paths.insert(&repo.path) {
                return Err(MirrorError::DuplicatePath(repo.path.clone()));
            }
        }

        Ok(())
    }

    /// Validate that all workspaces referenced exist
    #[allow(dead_code)]
    pub fn validate_workspace_exists(config: &MirrorConfig, workspace: &str) -> Result<()> {
        if config.workspaces.contains(&workspace.to_string()) {
            Ok(())
        } else {
            Err(MirrorError::WorkspaceNotFound(workspace.to_string()))
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::models::{RefSpec, Repository};

    fn create_test_repo(path: &str) -> Repository {
        Repository {
            git: format!("git@github.com:test/{path}.git"),
            path: path.to_string(),
            ref_spec: RefSpec::default(),
            workspaces: vec![],
        }
    }

    #[test]
    fn test_validate_unique_paths() {
        let mut config = MirrorConfig::new();
        config.repositories = vec![
            create_test_repo("repo1"),
            create_test_repo("repo2"),
        ];

        assert!(ConfigValidator::validate(&config).is_ok());
    }

    #[test]
    fn test_detect_duplicate_paths() {
        let mut config = MirrorConfig::new();
        config.repositories = vec![
            create_test_repo("repo1"),
            create_test_repo("repo1"),
        ];

        let result = ConfigValidator::validate(&config);
        assert!(matches!(result, Err(MirrorError::DuplicatePath(_))));
    }

    #[test]
    fn test_validate_workspace_exists() {
        let mut config = MirrorConfig::new();
        config.workspaces = vec!["ws1".to_string(), "ws2".to_string()];

        assert!(ConfigValidator::validate_workspace_exists(&config, "ws1").is_ok());
        assert!(ConfigValidator::validate_workspace_exists(&config, "ws3").is_err());
    }
}
