use crate::config::Config;
use crate::error::MctlResult;
use crate::git::GitOperations;
use log::{debug, error, info, warn};
use std::path::Path;

/// Execute the save command
pub fn execute(config: &Config, message: Option<String>) -> MctlResult<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;

    info!("Saving changes in {} repositories", config.repositories.len());

    let mut saved = 0;
    let mut no_changes = 0;
    let mut failed = 0;

    for (index, repository) in config.repositories.iter().enumerate() {
        info!(
            "[{}/{}] Processing repository: {}",
            index + 1,
            config.repositories.len(),
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

        // Save changes in the repository
        match GitOperations::save(repository, &current_dir, message.as_deref()) {
            Ok(crate::git::SaveResult::Saved) => {
                info!(
                    "Saved changes in repository at {}",
                    repository.absolute_path(&current_dir).display()
                );
                saved += 1;
            }
            Ok(crate::git::SaveResult::NoChanges) => {
                info!(
                    "No changes to save in repository at {}",
                    repository.absolute_path(&current_dir).display()
                );
                no_changes += 1;
            }
            Err(e) => {
                error!(
                    "Failed to save changes in repository at {}: {}",
                    repository.absolute_path(&current_dir).display(),
                    e
                );
                failed += 1;
            }
        }
    }

    // Print summary
    info!(
        "Save complete: {} repositories with changes saved, {} with no changes, {} failed",
        saved, no_changes, failed
    );

    Ok(())
}