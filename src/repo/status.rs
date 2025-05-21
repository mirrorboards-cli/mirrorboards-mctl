//! Repository status module for MCTL
//!
//! This module provides functionality to check the status of repositories.

use crate::error::types::MctlError;
use crate::git::operations;
use crate::repo::manager::RepositoryManager;
use log::{debug, info};
use std::collections::HashMap;

/// Repository status information
#[derive(Debug, Clone)]
pub struct RepositoryStatus {
    /// Repository name
    pub name: String,

    /// Status of files in the repository
    pub file_statuses: Vec<(String, git2::Status)>,

    /// Formatted status string for display
    pub formatted_status: String,

    /// Whether the repository has changes
    pub has_changes: bool,
}

/// Check the status of a specific repository
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `repo_name` - Name of the repository to check status for
///
/// # Returns
///
/// * `Ok(RepositoryStatus)` with the status information
/// * `Err(MctlError)` if an error occurred
pub fn check_status(
    repo_manager: &RepositoryManager,
    repo_name: &str,
) -> Result<RepositoryStatus, MctlError> {
    debug!("Checking status for repository: {}", repo_name);

    // Get the repository configuration
    let repo_config = repo_manager
        .get_config()
        .repositories
        .get(repo_name)
        .ok_or_else(|| {
            crate::error::types::ConfigError::new(
                crate::error::types::ErrorCode::RepositoryNotFound,
                format!("Repository '{}' not found", repo_name),
                "".to_string(),
            )
            .into()
        })?;

    // Get the status
    let file_statuses = operations::status(&repo_config.path)?;

    // Format the status
    let formatted_status = operations::format_status(&file_statuses);

    // Check if there are changes
    let has_changes = !file_statuses.is_empty();

    let status = RepositoryStatus {
        name: repo_name.to_string(),
        file_statuses,
        formatted_status,
        has_changes,
    };

    debug!("Repository {} has changes: {}", repo_name, has_changes);

    Ok(status)
}

/// Check the status of all repositories
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
///
/// # Returns
///
/// * `Ok(HashMap<String, RepositoryStatus>)` with the status information for all repositories
/// * `Err(MctlError)` if an error occurred
pub fn check_all_status(
    repo_manager: &RepositoryManager,
) -> Result<HashMap<String, RepositoryStatus>, MctlError> {
    debug!("Checking status for all repositories");

    let repo_names = repo_manager.get_repository_names();
    let mut statuses = HashMap::new();
    let mut error_count = 0;

    for name in &repo_names {
        match check_status(repo_manager, name) {
            Ok(status) => {
                statuses.insert(name.clone(), status);
            }
            Err(e) => {
                debug!("Failed to check status for repository {}: {}", name, e);
                error_count += 1;
            }
        }
    }

    info!(
        "Checked status for {}/{} repositories",
        statuses.len(),
        repo_names.len()
    );

    if error_count > 0 {
        debug!("Failed to check status for {} repositories", error_count);
    }

    Ok(statuses)
}

/// Get a summary of repository statuses
///
/// # Arguments
///
/// * `statuses` - Map of repository statuses
///
/// # Returns
///
/// * A string with a summary of the statuses
pub fn get_status_summary(statuses: &HashMap<String, RepositoryStatus>) -> String {
    let mut summary = String::new();

    let total = statuses.len();
    let changed = statuses.values().filter(|s| s.has_changes).count();

    summary.push_str(&format!(
        "Repositories: {} total, {} with changes\n\n",
        total, changed
    ));

    for (name, status) in statuses {
        if status.has_changes {
            summary.push_str(&format!("Repository: {}\n", name));
            summary.push_str(&status.formatted_status);
            summary.push_str("\n");
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::mirror_config::{MirrorConfig, Repository};
    use std::path::PathBuf;
    use tempfile::tempdir;

    #[test]
    fn test_get_status_summary() {
        let mut statuses = HashMap::new();

        // Add a repository with changes
        let mut file_statuses = Vec::new();
        file_statuses.push(("file1.txt".to_string(), git2::Status::WT_NEW));
        file_statuses.push(("file2.txt".to_string(), git2::Status::WT_MODIFIED));

        let status1 = RepositoryStatus {
            name: "repo1".to_string(),
            file_statuses: file_statuses.clone(),
            formatted_status: "?? file1.txt\n M file2.txt\n".to_string(),
            has_changes: true,
        };

        statuses.insert("repo1".to_string(), status1);

        // Add a repository without changes
        let status2 = RepositoryStatus {
            name: "repo2".to_string(),
            file_statuses: Vec::new(),
            formatted_status: "No changes\n".to_string(),
            has_changes: false,
        };

        statuses.insert("repo2".to_string(), status2);

        let summary = get_status_summary(&statuses);

        assert!(summary.contains("Repositories: 2 total, 1 with changes"));
        assert!(summary.contains("Repository: repo1"));
        assert!(summary.contains("?? file1.txt"));
        assert!(summary.contains(" M file2.txt"));
        assert!(!summary.contains("Repository: repo2"));
    }
}
