use crate::config::Config;
use crate::error::MctlResult;
use crate::git::GitOperations;
use log::{debug, error, info, warn};
use std::path::Path;

/// Execute the status command
pub fn execute(config: &Config) -> MctlResult<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;

    info!("Checking status of {} repositories", config.repositories.len());

    let mut clean_count = 0;
    let mut modified_count = 0;
    let mut missing_count = 0;

    for (index, repository) in config.repositories.iter().enumerate() {
        info!(
            "[{}/{}] Checking repository: {}",
            index + 1,
            config.repositories.len(),
            repository.git_url
        );

        // Check if the repository exists
        if !repository.exists_locally(&current_dir) {
            warn!(
                "Repository not found at {}",
                repository.absolute_path(&current_dir).display()
            );
            missing_count += 1;
            continue;
        }

        // Get the status of the repository
        match GitOperations::status(repository, &current_dir) {
            Ok(status) => {
                // Print repository information
                println!("\nRepository: {} ({})", repository.path, status.path.display());
                println!("Branch: {} ({})", status.branch, status.branch_status);

                // Print modified files
                if !status.modified_files.is_empty() {
                    println!("Modified files:");
                    for (path, status_code) in &status.modified_files {
                        println!("  {} {}", status_code, path.display());
                    }
                    modified_count += 1;
                }

                // Print untracked files
                if !status.untracked_files.is_empty() {
                    println!("Untracked files:");
                    for (path, status_code) in &status.untracked_files {
                        println!("  {} {}", status_code, path.display());
                    }
                    modified_count += 1;
                }

                // Print clean status
                if status.is_clean {
                    println!("Status: clean");
                    clean_count += 1;
                }
            }
            Err(e) => {
                error!(
                    "Failed to get status for repository at {}: {}",
                    repository.absolute_path(&current_dir).display(),
                    e
                );
            }
        }
    }

    // Print summary
    info!(
        "Status check complete: {} clean, {} modified, {} missing",
        clean_count, modified_count, missing_count
    );

    Ok(())
}