//! Status module for MCTL
//!
//! This module provides functionality for checking and displaying
//! the status of repositories.

use crate::error::types::MctlError;
use crate::git::repository::GitRepository;
use git2::Status;
use log::{debug, info};
use std::collections::HashMap;
use std::fmt;

/// Repository status information
#[derive(Debug)]
pub struct RepositoryStatus {
    /// Repository name
    pub name: String,

    /// Repository path
    pub path: String,

    /// Status of files in the repository
    pub file_statuses: Vec<FileStatus>,

    /// Whether the repository has uncommitted changes
    pub has_changes: bool,

    /// Whether the repository is ahead of the remote
    pub ahead: bool,

    /// Whether the repository is behind the remote
    pub behind: bool,
}

/// Status of a file in a repository
#[derive(Debug)]
pub struct FileStatus {
    /// File path
    pub path: String,

    /// Status of the file
    pub status: Status,
}

impl fmt::Display for RepositoryStatus {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "Repository: {}", self.name)?;
        writeln!(f, "Path: {}", self.path)?;

        if self.file_statuses.is_empty() {
            writeln!(f, "No changes")?;
        } else {
            writeln!(f, "Changes:")?;
            for file_status in &self.file_statuses {
                let status_str = format_status(file_status.status);
                writeln!(f, "  {} {}", status_str, file_status.path)?;
            }
        }

        if self.ahead {
            writeln!(f, "Ahead of remote")?;
        }

        if self.behind {
            writeln!(f, "Behind remote")?;
        }

        Ok(())
    }
}

/// Format git status for display
fn format_status(status: Status) -> &'static str {
    if status.is_index_new() || status.is_wt_new() {
        "NEW"
    } else if status.is_index_modified() || status.is_wt_modified() {
        "MODIFIED"
    } else if status.is_index_deleted() || status.is_wt_deleted() {
        "DELETED"
    } else if status.is_index_renamed() || status.is_wt_renamed() {
        "RENAMED"
    } else if status.is_index_typechange() || status.is_wt_typechange() {
        "TYPECHANGE"
    } else if status.is_ignored() {
        "IGNORED"
    } else if status.is_conflicted() {
        "CONFLICTED"
    } else {
        "UNKNOWN"
    }
}

/// Check the status of a repository
pub fn check_repository_status(repo: &mut GitRepository) -> Result<RepositoryStatus, MctlError> {
    debug!("Checking status of repository at {}", repo.path.display());

    // Get the repository status
    let statuses = repo.status()?;

    // Create file status objects
    let mut file_statuses = Vec::new();
    for (path, status) in statuses {
        file_statuses.push(FileStatus { path, status });
    }

    // Determine if there are changes
    let has_changes = !file_statuses.is_empty();

    // Create the repository status
    let status = RepositoryStatus {
        name: repo
            .path
            .file_name()
            .map(|name| name.to_string_lossy().to_string())
            .unwrap_or_else(|| "unknown".to_string()),
        path: repo.path.display().to_string(),
        file_statuses,
        has_changes,
        ahead: false, // TODO: Implement ahead/behind detection
        behind: false,
    };

    info!("Repository status checked: {}", status.path);

    Ok(status)
}

/// Check the status of multiple repositories
pub fn check_status(
    repos: &mut HashMap<String, GitRepository>,
) -> Result<Vec<RepositoryStatus>, MctlError> {
    debug!("Checking status of {} repositories", repos.len());

    let mut statuses = Vec::new();

    for (name, repo) in repos {
        match check_repository_status(repo) {
            Ok(status) => {
                statuses.push(status);
            }
            Err(e) => {
                // Log the error but continue with other repositories
                log::error!("Failed to check status of repository {}: {}", name, e);
            }
        }
    }

    info!("Checked status of {} repositories", statuses.len());

    Ok(statuses)
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::tempdir;

    #[test]
    fn test_format_status() {
        // This is a simple test to ensure the function doesn't panic
        let status = Status::empty();
        let result = format_status(status);
        assert_eq!(result, "UNKNOWN");
    }
}
