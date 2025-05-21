use crate::config::Config;
use crate::error::{MctlError, MctlResult};
use crate::git::GitOperations;
use log::{debug, error, info, warn};
use std::path::Path;

/// Execute the update command
pub fn execute(
    config: &Config,
    force: bool,
    dry_run: bool,
    repo_name: Option<String>,
) -> MctlResult<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;

    // Filter repositories if a specific one is requested
    let repositories = if let Some(name) = repo_name {
        config
            .repositories
            .iter()
            .filter(|r| r.path == name)
            .collect::<Vec<_>>()
    } else {
        config.repositories.iter().collect()
    };

    if repositories.is_empty() {
        if let Some(name) = repo_name {
            return Err(MctlError::GitError(format!(
                "Repository '{}' not found in configuration",
                name
            )));
        } else {
            warn!("No repositories found in configuration");
            return Ok(());
        }
    }

    info!(
        "Updating {} repositories{}",
        repositories.len(),
        if dry_run { " (dry run)" } else { "" }
    );

    let mut updated = 0;
    let mut already_up_to_date = 0;
    let mut failed = 0;

    for (index, repository) in repositories.iter().enumerate() {
        info!(
            "[{}/{}] Processing repository: {}",
            index + 1,
            repositories.len(),
            repository.git_url
        );

        // Check if the repository exists
        if !repository.exists_locally(&current_dir) {
            warn!(
                "Repository not found at {}, skipping",
                repository.absolute_path(&current_dir).display()
            );
            failed += 1;
            continue;
        }

        if dry_run {
            info!(
                "DRY RUN: Would update repository at {}",
                repository.absolute_path(&current_dir).display()
            );
            continue;
        }

        // Update the repository
        match GitOperations::update(repository, &current_dir, force) {
            Ok(crate::git::UpdateResult::Updated) => {
                info!(
                    "Updated repository at {}",
                    repository.absolute_path(&current_dir).display()
                );
                updated += 1;
            }
            Ok(crate::git::UpdateResult::AlreadyUpToDate) => {
                info!(
                    "Repository at {} is already up to date",
                    repository.absolute_path(&current_dir).display()
                );
                already_up_to_date += 1;
            }
            Err(e) => {
                error!(
                    "Failed to update repository at {}: {}",
                    repository.absolute_path(&current_dir).display(),
                    e
                );
                failed += 1;
            }
        }
    }

    // Print summary
    if dry_run {
        info!("DRY RUN complete: Would update {} repositories", repositories.len());
    } else {
        info!(
            "Update complete: {} updated, {} already up to date, {} failed",
            updated, already_up_to_date, failed
        );
    }

    Ok(())
}