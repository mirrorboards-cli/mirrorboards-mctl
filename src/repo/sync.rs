//! Repository synchronization module for MCTL
//!
//! This module provides functionality to synchronize repositories.

use crate::error::types::MctlError;
use crate::git::credentials::GitCredentials;
use crate::git::operations;
use crate::repo::manager::RepositoryManager;
use log::{debug, info, warn};

/// Synchronization result for a repository
#[derive(Debug, Clone)]
pub struct SyncResult {
    /// Repository name
    pub name: String,

    /// Whether the synchronization was successful
    pub success: bool,

    /// Error message if synchronization failed
    pub error_message: Option<String>,
}

/// Synchronize a specific repository
///
/// This function pulls changes from the remote repository.
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `repo_name` - Name of the repository to synchronize
/// * `credentials` - Optional credentials for authentication
///
/// # Returns
///
/// * `Ok(SyncResult)` with the synchronization result
/// * `Err(MctlError)` if an error occurred
pub fn sync_repository(
    repo_manager: &RepositoryManager,
    repo_name: &str,
    credentials: Option<GitCredentials>,
) -> Result<SyncResult, MctlError> {
    debug!("Synchronizing repository: {}", repo_name);

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

    // Pull changes
    match operations::pull(&repo_config.path, credentials) {
        Ok(_) => {
            info!("Successfully synchronized repository: {}", repo_name);
            Ok(SyncResult {
                name: repo_name.to_string(),
                success: true,
                error_message: None,
            })
        }
        Err(e) => {
            warn!("Failed to synchronize repository {}: {}", repo_name, e);
            Ok(SyncResult {
                name: repo_name.to_string(),
                success: false,
                error_message: Some(e.to_string()),
            })
        }
    }
}

/// Synchronize all repositories
///
/// This function pulls changes from all remote repositories.
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `credentials` - Optional credentials for authentication
///
/// # Returns
///
/// * `Ok(Vec<SyncResult>)` with the synchronization results for all repositories
/// * `Err(MctlError)` if an error occurred
pub fn sync_repositories(
    repo_manager: &RepositoryManager,
    credentials: Option<GitCredentials>,
) -> Result<Vec<SyncResult>, MctlError> {
    debug!("Synchronizing all repositories");

    let repo_names = repo_manager.get_repository_names();
    let mut results = Vec::new();

    for name in &repo_names {
        match sync_repository(repo_manager, name, credentials.clone()) {
            Ok(result) => {
                results.push(result);
            }
            Err(e) => {
                warn!("Failed to synchronize repository {}: {}", name, e);
                results.push(SyncResult {
                    name: name.clone(),
                    success: false,
                    error_message: Some(e.to_string()),
                });
            }
        }
    }

    let success_count = results.iter().filter(|r| r.success).count();

    info!(
        "Synchronized {}/{} repositories successfully",
        success_count,
        repo_names.len()
    );

    Ok(results)
}

/// Get a summary of synchronization results
///
/// # Arguments
///
/// * `results` - Synchronization results
///
/// # Returns
///
/// * A string with a summary of the synchronization results
pub fn get_sync_summary(results: &[SyncResult]) -> String {
    let mut summary = String::new();

    let total = results.len();
    let success = results.iter().filter(|r| r.success).count();
    let failed = total - success;

    summary.push_str(&format!(
        "Synchronization summary: {} total, {} successful, {} failed\n\n",
        total, success, failed
    ));

    if failed > 0 {
        summary.push_str("Failed repositories:\n");
        for result in results.iter().filter(|r| !r.success) {
            summary.push_str(&format!(
                "- {}: {}\n",
                result.name,
                result.error_message.as_deref().unwrap_or("Unknown error")
            ));
        }
    }

    summary
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_get_sync_summary() {
        let mut results = Vec::new();

        // Add successful result
        results.push(SyncResult {
            name: "repo1".to_string(),
            success: true,
            error_message: None,
        });

        // Add failed result
        results.push(SyncResult {
            name: "repo2".to_string(),
            success: false,
            error_message: Some("Failed to pull changes".to_string()),
        });

        let summary = get_sync_summary(&results);

        assert!(summary.contains("Synchronization summary: 2 total, 1 successful, 1 failed"));
        assert!(summary.contains("Failed repositories:"));
        assert!(summary.contains("- repo2: Failed to pull changes"));
    }
}
