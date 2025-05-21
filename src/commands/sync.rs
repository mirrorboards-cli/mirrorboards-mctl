use crate::config::Config;
use crate::error::{MctlError, MctlResult};
use crate::git::GitOperations;
use log::{debug, error, info, warn};
use std::path::Path;
use std::sync::{Arc, Mutex};
use std::thread;

/// Execute the sync command
pub fn execute(
    config: &Config,
    no_pull: bool,
    force: bool,
    parallel: Option<usize>,
) -> MctlResult<()> {
    // Get the current directory
    let current_dir = std::env::current_dir()?;

    // Check if we should run in parallel
    if let Some(num_threads) = parallel {
        if num_threads > 1 {
            return execute_parallel(config, no_pull, force, num_threads, &current_dir);
        }
    }

    // Sequential execution
    let mut cloned = 0;
    let mut updated = 0;
    let mut skipped = 0;

    for (index, repository) in config.repositories.iter().enumerate() {
        info!(
            "[{}/{}] Processing repository: {}",
            index + 1,
            config.repositories.len(),
            repository.git_url
        );

        // Check if the repository already exists
        if repository.exists_locally(&current_dir) {
            if no_pull {
                info!(
                    "Repository already exists at {}, skipping (--no-pull flag)",
                    repository.absolute_path(&current_dir).display()
                );
                skipped += 1;
                continue;
            }

            // Update the repository
            match crate::git::GitOperations::update(repository, &current_dir, force) {
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
                    skipped += 1;
                }
                Err(e) => {
                    warn!(
                        "Failed to update repository at {}: {}",
                        repository.absolute_path(&current_dir).display(),
                        e
                    );
                    skipped += 1;
                }
            }
        } else {
            // Clone the repository
            match GitOperations::clone(repository, &current_dir) {
                Ok(_) => {
                    info!(
                        "Cloned repository to {}",
                        repository.absolute_path(&current_dir).display()
                    );
                    cloned += 1;
                }
                Err(e) => {
                    error!(
                        "Failed to clone repository to {}: {}",
                        repository.absolute_path(&current_dir).display(),
                        e
                    );
                    return Err(e);
                }
            }
        }
    }

    // Print summary
    info!(
        "Synchronization complete: {} repository/ies cloned, {} updated, {} skipped",
        cloned, updated, skipped
    );

    Ok(())
}

/// Execute the sync command in parallel
fn execute_parallel(
    config: &Config,
    no_pull: bool,
    force: bool,
    num_threads: usize,
    current_dir: &Path,
) -> MctlResult<()> {
    info!("Running sync in parallel with {} threads", num_threads);

    // Create shared counters
    let cloned = Arc::new(Mutex::new(0));
    let updated = Arc::new(Mutex::new(0));
    let skipped = Arc::new(Mutex::new(0));

    // Create a thread pool
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(num_threads)
        .build()
        .map_err(|e| MctlError::GitError(format!("Failed to create thread pool: {}", e)))?;

    // Clone repositories in parallel
    pool.scope(|s| {
        for (index, repository) in config.repositories.iter().enumerate() {
            let cloned = Arc::clone(&cloned);
            let updated = Arc::clone(&updated);
            let skipped = Arc::clone(&skipped);
            let current_dir = current_dir.to_path_buf();

            s.spawn(move |_| {
                info!(
                    "[{}/{}] Processing repository: {}",
                    index + 1,
                    config.repositories.len(),
                    repository.git_url
                );

                // Check if the repository already exists
                if repository.exists_locally(&current_dir) {
                    if no_pull {
                        info!(
                            "Repository already exists at {}, skipping (--no-pull flag)",
                            repository.absolute_path(&current_dir).display()
                        );
                        *skipped.lock().unwrap() += 1;
                        return;
                    }

                    // Update the repository
                    match crate::git::GitOperations::update(repository, &current_dir, force) {
                        Ok(crate::git::UpdateResult::Updated) => {
                            info!(
                                "Updated repository at {}",
                                repository.absolute_path(&current_dir).display()
                            );
                            *updated.lock().unwrap() += 1;
                        }
                        Ok(crate::git::UpdateResult::AlreadyUpToDate) => {
                            info!(
                                "Repository at {} is already up to date",
                                repository.absolute_path(&current_dir).display()
                            );
                            *skipped.lock().unwrap() += 1;
                        }
                        Err(e) => {
                            warn!(
                                "Failed to update repository at {}: {}",
                                repository.absolute_path(&current_dir).display(),
                                e
                            );
                            *skipped.lock().unwrap() += 1;
                        }
                    }
                } else {
                    // Clone the repository
                    match GitOperations::clone(repository, &current_dir) {
                        Ok(_) => {
                            info!(
                                "Cloned repository to {}",
                                repository.absolute_path(&current_dir).display()
                            );
                            *cloned.lock().unwrap() += 1;
                        }
                        Err(e) => {
                            error!(
                                "Failed to clone repository to {}: {}",
                                repository.absolute_path(&current_dir).display(),
                                e
                            );
                        }
                    }
                }
            });
        }
    });

    // Print summary
    let cloned = *cloned.lock().unwrap();
    let updated = *updated.lock().unwrap();
    let skipped = *skipped.lock().unwrap();

    info!(
        "Synchronization complete: {} repository/ies cloned, {} updated, {} skipped",
        cloned, updated, skipped
    );

    Ok(())
}