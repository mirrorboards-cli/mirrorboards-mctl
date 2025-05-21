//! Repository update module for MCTL
//!
//! This module provides functionality to update repository configurations.

use crate::config::mirror_config::Repository;
use crate::error::types::MctlError;
use crate::repo::manager::RepositoryManager;
use log::{debug, info, warn};
use std::collections::HashMap;
use std::path::PathBuf;

/// Update result for a repository
#[derive(Debug, Clone)]
pub struct UpdateResult {
    /// Repository name
    pub name: String,

    /// Whether the update was successful
    pub success: bool,

    /// Error message if update failed
    pub error_message: Option<String>,

    /// Updated fields
    pub updated_fields: Vec<String>,
}

/// Update a repository configuration
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `repo_name` - Name of the repository to update
/// * `url` - Optional new URL
/// * `path` - Optional new path
/// * `branch` - Optional new branch
/// * `auth_method` - Optional new authentication method
///
/// # Returns
///
/// * `Ok(UpdateResult)` with the update result
/// * `Err(MctlError)` if an error occurred
pub fn update_repository(
    repo_manager: &mut RepositoryManager,
    repo_name: &str,
    url: Option<String>,
    path: Option<PathBuf>,
    branch: Option<String>,
    auth_method: Option<String>,
) -> Result<UpdateResult, MctlError> {
    debug!("Updating repository configuration: {}", repo_name);

    // Get the repository configuration
    let config = repo_manager.get_config_mut();
    let repo = config.repositories.get_mut(repo_name).ok_or_else(|| {
        crate::error::types::ConfigError::new(
            crate::error::types::ErrorCode::RepositoryNotFound,
            format!("Repository '{}' not found", repo_name),
            "".to_string(),
        )
        .into()
    })?;

    let mut updated_fields = Vec::new();

    // Update URL if provided
    if let Some(new_url) = url {
        if new_url != repo.url {
            debug!(
                "Updating URL for repository {}: {} -> {}",
                repo_name, repo.url, new_url
            );
            repo.url = new_url;
            updated_fields.push("url".to_string());
        }
    }

    // Update path if provided
    if let Some(new_path) = path {
        if new_path != repo.path {
            debug!(
                "Updating path for repository {}: {} -> {}",
                repo_name,
                repo.path.display(),
                new_path.display()
            );
            repo.path = new_path;
            updated_fields.push("path".to_string());
        }
    }

    // Update branch if provided
    if let Some(new_branch) = branch {
        if repo.branch.as_deref() != Some(new_branch.as_str()) {
            debug!(
                "Updating branch for repository {}: {:?} -> {}",
                repo_name, repo.branch, new_branch
            );
            repo.branch = Some(new_branch);
            updated_fields.push("branch".to_string());
        }
    }

    // Update authentication method if provided
    if let Some(new_auth_method) = auth_method {
        if repo.auth_method.as_deref() != Some(new_auth_method.as_str()) {
            debug!(
                "Updating authentication method for repository {}: {:?} -> {}",
                repo_name, repo.auth_method, new_auth_method
            );
            repo.auth_method = Some(new_auth_method);
            updated_fields.push("auth_method".to_string());
        }
    }

    // Validate the updated repository
    if let Err(e) = repo.validate() {
        return Ok(UpdateResult {
            name: repo_name.to_string(),
            success: false,
            error_message: Some(e.to_string()),
            updated_fields: Vec::new(),
        });
    }

    // Save the configuration
    if !updated_fields.is_empty() {
        match repo_manager.save_config() {
            Ok(_) => {
                info!(
                    "Successfully updated repository configuration: {}",
                    repo_name
                );
                Ok(UpdateResult {
                    name: repo_name.to_string(),
                    success: true,
                    error_message: None,
                    updated_fields,
                })
            }
            Err(e) => {
                warn!(
                    "Failed to save configuration after updating repository {}: {}",
                    repo_name, e
                );
                Ok(UpdateResult {
                    name: repo_name.to_string(),
                    success: false,
                    error_message: Some(e.to_string()),
                    updated_fields: Vec::new(),
                })
            }
        }
    } else {
        debug!("No changes to repository configuration: {}", repo_name);
        Ok(UpdateResult {
            name: repo_name.to_string(),
            success: true,
            error_message: None,
            updated_fields: Vec::new(),
        })
    }
}

/// Update multiple repositories
///
/// # Arguments
///
/// * `repo_manager` - Repository manager instance
/// * `updates` - Map of repository names to update parameters
///
/// # Returns
///
/// * `Ok(HashMap<String, UpdateResult>)` with the update results
/// * `Err(MctlError)` if an error occurred
pub fn update_repositories(
    repo_manager: &mut RepositoryManager,
    updates: HashMap<
        String,
        (
            Option<String>,
            Option<PathBuf>,
            Option<String>,
            Option<String>,
        ),
    >,
) -> Result<HashMap<String, UpdateResult>, MctlError> {
    debug!("Updating multiple repository configurations");

    let mut results = HashMap::new();

    for (name, (url, path, branch, auth_method)) in updates {
        match update_repository(repo_manager, &name, url, path, branch, auth_method) {
            Ok(result) => {
                results.insert(name, result);
            }
            Err(e) => {
                warn!("Failed to update repository {}: {}", name, e);
                results.insert(
                    name.clone(),
                    UpdateResult {
                        name,
                        success: false,
                        error_message: Some(e.to_string()),
                        updated_fields: Vec::new(),
                    },
                );
            }
        }
    }

    let success_count = results.values().filter(|r| r.success).count();

    info!(
        "Updated {}/{} repository configurations successfully",
        success_count,
        results.len()
    );

    Ok(results)
}

/// Get a summary of update results
///
/// # Arguments
///
/// * `results` - Update results
///
/// # Returns
///
/// * A string with a summary of the update results
pub fn get_update_summary(results: &HashMap<String, UpdateResult>) -> String {
    let mut summary = String::new();

    let total = results.len();
    let success = results.values().filter(|r| r.success).count();
    let failed = total - success;

    summary.push_str(&format!(
        "Update summary: {} total, {} successful, {} failed\n\n",
        total, success, failed
    ));

    // Show successful updates
    if success > 0 {
        summary.push_str("Successfully updated repositories:\n");
        for result in results
            .values()
            .filter(|r| r.success && !r.updated_fields.is_empty())
        {
            summary.push_str(&format!(
                "- {}: Updated fields: {}\n",
                result.name,
                result.updated_fields.join(", ")
            ));
        }
        summary.push_str("\n");
    }

    // Show failed updates
    if failed > 0 {
        summary.push_str("Failed updates:\n");
        for result in results.values().filter(|r| !r.success) {
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
    fn test_get_update_summary() {
        let mut results = HashMap::new();

        // Add successful result with updates
        results.insert(
            "repo1".to_string(),
            UpdateResult {
                name: "repo1".to_string(),
                success: true,
                error_message: None,
                updated_fields: vec!["url".to_string(), "branch".to_string()],
            },
        );

        // Add successful result without updates
        results.insert(
            "repo2".to_string(),
            UpdateResult {
                name: "repo2".to_string(),
                success: true,
                error_message: None,
                updated_fields: Vec::new(),
            },
        );

        // Add failed result
        results.insert(
            "repo3".to_string(),
            UpdateResult {
                name: "repo3".to_string(),
                success: false,
                error_message: Some("Invalid URL".to_string()),
                updated_fields: Vec::new(),
            },
        );

        let summary = get_update_summary(&results);

        assert!(summary.contains("Update summary: 3 total, 2 successful, 1 failed"));
        assert!(summary.contains("Successfully updated repositories:"));
        assert!(summary.contains("- repo1: Updated fields: url, branch"));
        assert!(!summary.contains("- repo2:"));
        assert!(summary.contains("Failed updates:"));
        assert!(summary.contains("- repo3: Invalid URL"));
    }
}
