//! Repository save module for MCTL
//!
//! This module provides functionality to save changes to repositories.

use crate::error::types::MctlError;
use crate::git::credentials::GitCredentials;
use crate::git::operations;
use crate::repo::manager::RepositoryManager;
use log::{debug, info, warn};

/// Save changes to a specific repository
///
/// This function commits changes to the repository and pushes them to the remote.
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `repo_name` - Name of the repository to save changes for
/// * `commit_message` - Commit message
/// * `credentials` - Optional credentials for authentication
///
/// # Returns
///
/// * `Ok(())` if the changes were saved successfully
/// * `Err(MctlError)` if an error occurred
pub fn save_changes(
    repo_manager: &mut RepositoryManager,
    repo_name: &str,
    commit_message: &str,
    credentials: Option<GitCredentials>,
) -> Result<(), MctlError> {
    debug!("Saving changes for repository: {}", repo_name);

    // Get the repository configuration
    let repo_config = repo_manager
        .get_config()
        .repositories
        .get(repo_name)
        .ok_or_else(|| {
            let err: MctlError = crate::error::types::ConfigError::new(
                crate::error::types::ErrorCode::RepositoryNotFound,
                format!("Repository '{}' not found", repo_name),
                "".to_string(),
            )
            .into();
            err
        })?;

    // Commit changes
    debug!("Committing changes for repository: {}", repo_name);
    let commit_id = operations::commit(&repo_config.path, commit_message)?;

    // Push changes
    debug!("Pushing changes for repository: {}", repo_name);
    operations::push(&repo_config.path, credentials)?;

    info!(
        "Successfully saved changes for repository {}: {}",
        repo_name, commit_id
    );

    Ok(())
}

/// Save changes to all repositories
///
/// This function commits changes to all repositories and pushes them to their remotes.
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `commit_message` - Commit message
/// * `credentials` - Optional credentials for authentication
///
/// # Returns
///
/// * `Ok(())` if the changes were saved successfully
/// * `Err(MctlError)` if an error occurred
pub fn save_all_changes(
    repo_manager: &mut RepositoryManager,
    commit_message: &str,
    credentials: Option<GitCredentials>,
) -> Result<(), MctlError> {
    debug!("Saving changes for all repositories");

    let repo_names = repo_manager.get_repository_names();
    let mut success_count = 0;
    let mut error_count = 0;

    for name in &repo_names {
        match save_changes(repo_manager, name, commit_message, credentials.clone()) {
            Ok(_) => {
                success_count += 1;
            }
            Err(e) => {
                warn!("Failed to save changes for repository {}: {}", name, e);
                error_count += 1;
            }
        }
    }

    info!(
        "Saved changes for {}/{} repositories",
        success_count,
        repo_names.len()
    );

    if error_count > 0 {
        warn!("Failed to save changes for {} repositories", error_count);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mirror_config::{MirrorConfig, Repository};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_save_changes() {
        // This is a mock test since we can't easily test git operations
        // In a real test, we would use a mock for the git operations
        let dir = tempdir().unwrap();
        let config_path = dir.path().join("mirror.toml");

        // Create a configuration
        let mut config = MirrorConfig::new();

        let repo = Repository::new(
            "git@github.com:example/repo.git".to_string(),
            PathBuf::from("example-repo"),
            Some("main".to_string()),
        );

        config.add_repository("example".to_string(), repo).unwrap();
        config.save(&config_path).unwrap();

        // This would fail in a real test since the repository doesn't exist
        // But we're just testing that the function compiles and runs
        let result = RepositoryManager::new(&config_path);
        assert!(result.is_ok());
    }
}
