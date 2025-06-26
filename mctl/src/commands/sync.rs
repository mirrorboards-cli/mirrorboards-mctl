use anyhow::{Result, Context};
use mirror_sdk::{ConfigManager, GitManager, RepositoryStatus};
use super::{Command, print_success, print_error, print_info, print_warning, print_verbose};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};

pub struct SyncCommand {
    pub dry_run: bool,
    pub pull: bool,
    pub force: bool
}

impl Command for SyncCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        // Load configuration
        if !config_manager.exists() {
            print_error(&format!("Configuration file not found: {}", config_manager.path().display()));
            print_info("Run 'mctl init' to create a new configuration file");
            return Ok(());
        }

        let config = config_manager.load()
            .context("Failed to load configuration")?;

        if config.is_empty() {
            print_warning("No repositories configured");
            print_info("Add repositories with: mctl add <git-url>");
            return Ok(());
        }

        print_verbose(&format!("Found {} repositories to process", config.len()), verbose);

        // Initialize Git manager
        let git_manager = GitManager::new()
            .context("Failed to initialize Git manager")?;

        // Setup progress tracking
        let multi_progress = MultiProgress::new();
        let main_progress = multi_progress.add(ProgressBar::new(config.len() as u64));
        main_progress.set_style(
            ProgressStyle::default_bar()
                .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}/{len:3} {msg}")
                .unwrap()
                .progress_chars("#>-")
        );
        main_progress.set_message("Synchronizing repositories");

        // Counters for summary
        let cloned_count = Arc::new(AtomicUsize::new(0));
        let updated_count = Arc::new(AtomicUsize::new(0));
        let skipped_count = Arc::new(AtomicUsize::new(0));
        let error_count = Arc::new(AtomicUsize::new(0));

        // Process repositories
        for (index, repo) in config.repositories().iter().enumerate() {
            let target_path = PathBuf::from(&repo.path);
            
            print_verbose(&format!("Processing repository {}/{}: {} -> {}", 
                index + 1, config.len(), repo.git, target_path.display()), verbose);

            if self.dry_run {
                print_info(&format!("[DRY RUN] Would process: {} -> {}", repo.git, target_path.display()));
                self.dry_run_repository_analysis(&git_manager, repo, &target_path, verbose)?;
                skipped_count.fetch_add(1, Ordering::SeqCst);
            } else {
                match self.process_repository(&git_manager, repo, &target_path, verbose) {
                    Ok(action) => {
                        match action {
                            SyncAction::Cloned => {
                                cloned_count.fetch_add(1, Ordering::SeqCst);
                                print_success(&format!("Cloned: {}", repo.git));
                            }
                            SyncAction::Updated => {
                                updated_count.fetch_add(1, Ordering::SeqCst);
                                print_success(&format!("Updated: {}", repo.git));
                            }
                            SyncAction::Skipped(reason) => {
                                skipped_count.fetch_add(1, Ordering::SeqCst);
                                print_verbose(&format!("Skipped {}: {}", repo.git, reason), verbose);
                            }
                        }
                    }
                    Err(e) => {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        print_error(&format!("Failed to process {}: {}", repo.git, e));
                    }
                }
            }

            main_progress.inc(1);
            main_progress.set_message(format!("Processed {}/{}", index + 1, config.len()));
        }

        main_progress.finish_with_message("Synchronization complete");
        multi_progress.clear().unwrap_or(());

        // Print summary
        println!();
        print_info("Synchronization Summary:");
        
        let cloned = cloned_count.load(Ordering::SeqCst);
        let updated = updated_count.load(Ordering::SeqCst);
        let skipped = skipped_count.load(Ordering::SeqCst);
        let errors = error_count.load(Ordering::SeqCst);

        if self.dry_run {
            println!("  • Repositories analyzed: {}", config.len());
            println!("  • This was a dry run - no changes were made");
        } else {
            println!("  • Repositories cloned: {}", cloned);
            println!("  • Repositories updated: {}", updated);
            println!("  • Repositories skipped: {}", skipped);
            if errors > 0 {
                println!("  • Repositories with errors: {}", errors);
            }
        }

        if errors > 0 && !self.dry_run {
            print_warning("Some repositories could not be processed. Check the error messages above.");
        }

        Ok(())
    }
}

#[derive(Debug)]
enum SyncAction {
    Cloned,
    Updated,
    Skipped(String),
}

impl SyncCommand {
    fn process_repository(&self, git_manager: &GitManager, repo: &mirror_sdk::Repository, target_path: &PathBuf, verbose: bool) -> Result<SyncAction> {
        let status = git_manager.get_repository_status(target_path)
            .context("Failed to get repository status")?;

        print_verbose(&format!("Repository status: {:?}", status), verbose);

        match status {
            RepositoryStatus::Missing => {
                // Clone repository
                git_manager.clone_repository(repo, target_path)
                    .context("Failed to clone repository")?;
                Ok(SyncAction::Cloned)
            }
            RepositoryStatus::NotGitRepository => {
                if self.force {
                    // Remove directory and clone
                    if target_path.exists() {
                        std::fs::remove_dir_all(target_path)
                            .context("Failed to remove non-git directory")?;
                    }
                    git_manager.clone_repository(repo, target_path)
                        .context("Failed to clone repository after removing directory")?;
                    Ok(SyncAction::Cloned)
                } else {
                    Ok(SyncAction::Skipped("Directory exists but is not a git repository (use --force to re-clone)".to_string()))
                }
            }
            RepositoryStatus::UpToDate => {
                if self.force {
                    // Force re-clone
                    std::fs::remove_dir_all(target_path)
                        .context("Failed to remove existing repository directory")?;
                    git_manager.clone_repository(repo, target_path)
                        .context("Failed to re-clone repository")?;
                    Ok(SyncAction::Cloned)
                } else if self.pull {
                    // Already up to date, skip
                    Ok(SyncAction::Skipped("already up to date".to_string()))
                } else {
                    // Skip existing
                    Ok(SyncAction::Skipped("repository exists (use --pull to update or --force to re-clone)".to_string()))
                }
            }
            RepositoryStatus::NeedsUpdate => {
                if self.force {
                    // Force re-clone
                    std::fs::remove_dir_all(target_path)
                        .context("Failed to remove existing repository directory")?;
                    git_manager.clone_repository(repo, target_path)
                        .context("Failed to re-clone repository")?;
                    Ok(SyncAction::Cloned)
                } else if self.pull {
                    // Pull updates
                    git_manager.update_repository(target_path)
                        .context("Failed to update repository")?;
                    Ok(SyncAction::Updated)
                } else {
                    // Skip existing
                    Ok(SyncAction::Skipped("repository needs updates (use --pull to update or --force to re-clone)".to_string()))
                }
            }
            RepositoryStatus::HasConflicts => {
                if self.force {
                    // Force re-clone
                    std::fs::remove_dir_all(target_path)
                        .context("Failed to remove existing repository directory")?;
                    git_manager.clone_repository(repo, target_path)
                        .context("Failed to re-clone repository")?;
                    Ok(SyncAction::Cloned)
                } else {
                    Ok(SyncAction::Skipped("repository has local changes that conflict (use --force to re-clone)".to_string()))
                }
            }
        }
    }

    fn dry_run_repository_analysis(&self, git_manager: &GitManager, repo: &mirror_sdk::Repository, target_path: &PathBuf, verbose: bool) -> Result<()> {
        let status = git_manager.get_repository_status(target_path)
            .context("Failed to get repository status")?;

        print_verbose(&format!("Repository status: {:?}", status), verbose);

        let action_description = match status {
            RepositoryStatus::Missing => "Would clone repository".to_string(),
            RepositoryStatus::NotGitRepository => {
                if self.force {
                    "Would remove directory and clone repository".to_string()
                } else {
                    "Would skip (directory exists but is not a git repository)".to_string()
                }
            }
            RepositoryStatus::UpToDate => {
                if self.force {
                    "Would force re-clone repository".to_string()
                } else if self.pull {
                    "Would skip (already up to date)".to_string()
                } else {
                    "Would skip (repository exists)".to_string()
                }
            }
            RepositoryStatus::NeedsUpdate => {
                if self.force {
                    "Would force re-clone repository".to_string()
                } else if self.pull {
                    "Would update repository".to_string()
                } else {
                    "Would skip (repository needs updates)".to_string()
                }
            }
            RepositoryStatus::HasConflicts => {
                if self.force {
                    "Would force re-clone repository".to_string()
                } else {
                    "Would skip (repository has local changes)".to_string()
                }
            }
        };

        println!("  → {}: {}", repo.git, action_description);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use mirror_sdk::MirrorConfig;

    #[test]
    fn test_sync_command_creation() {
        let sync_command = SyncCommand {
            dry_run: true,
            pull: false,
            force: false,
        };
        
        assert!(sync_command.dry_run);
        assert!(!sync_command.pull);
        assert!(!sync_command.force);
    }

    #[test]
    fn test_sync_command_with_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        let empty_config = MirrorConfig::new();
        config_manager.save(&empty_config).unwrap();
        
        let sync_command = SyncCommand {
            dry_run: true,
            pull: false,
            force: false,
        };
        
        // Should complete without error for empty config
        let result = sync_command.execute(&config_manager, false);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sync_command_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let sync_command = SyncCommand {
            dry_run: true,
            pull: false,
            force: false,
        };
        
        // Should complete without error for nonexistent config
        let result = sync_command.execute(&config_manager, false);
        assert!(result.is_ok());
    }
}