use anyhow::{Result, Context};
use mirror_sdk::{ConfigManager, GitManager};
use super::{Command, print_error, print_info, print_success, print_verbose, print_warning};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use chrono::Utc;

pub struct SaveCommand {
    pub message: Option<String>,
}

impl Command for SaveCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        print_verbose("Starting save operation", verbose);
        
        if !config_manager.exists() {
            print_warning("Configuration file does not exist. Run 'mctl init' to create one.");
            return Ok(());
        }
        
        let repositories = config_manager.list_repositories()
            .context("Failed to load repositories from configuration")?;
        
        if repositories.is_empty() {
            print_info("No repositories configured.");
            if verbose {
                println!("Add repositories with: mctl add <git-url>");
            }
            return Ok(());
        }
        
        // Filter out repositories with skip_push = true
        let active_repositories: Vec<_> = repositories.iter()
            .filter(|repo| !repo.skip_push)
            .collect();
        
        if active_repositories.is_empty() {
            print_info("No active repositories found (all repositories have skip_push = true).");
            if verbose {
                println!("Total repositories configured: {}", repositories.len());
                println!("Use 'mctl list' to see all repositories including those with skip_push = true");
            }
            return Ok(());
        }
        
        print_verbose(&format!("Found {} active repositories (filtered {} with skip_push = true)", 
            active_repositories.len(), repositories.len() - active_repositories.len()), verbose);
        
        // Generate commit message
        let commit_message = match &self.message {
            Some(msg) => msg.clone(),
            None => {
                let timestamp = Utc::now().format("%Y-%m-%d %H:%M:%S UTC").to_string();
                format!("save {}", timestamp)
            }
        };
        
        print_verbose(&format!("Commit message: \"{}\"", commit_message), verbose);
        
        // Initialize Git manager
        let git_manager = GitManager::new_with_verbose(verbose)
            .context("Failed to initialize Git manager")?;
        
        // Setup progress tracking if multiple repos
        let multi_progress = if active_repositories.len() > 1 {
            let mp = MultiProgress::new();
            let main_progress = mp.add(ProgressBar::new(active_repositories.len() as u64));
            main_progress.set_style(
                ProgressStyle::default_bar()
                    .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {pos:>3}/{len:3} {msg}")
                    .unwrap()
                    .progress_chars("#>-")
            );
            main_progress.set_message("Saving repositories");
            Some((mp, main_progress))
        } else {
            None
        };
        
        // Process each repository
        let mut success_count = 0;
        let mut skipped_count = 0;
        let error_count = Arc::new(AtomicUsize::new(0));
        
        for (index, repo) in active_repositories.iter().enumerate() {
            let target_path = PathBuf::from(&repo.path);
            
            print_verbose(&format!("Processing repository {}/{}: {} -> {}",
                index + 1, active_repositories.len(), repo.git, target_path.display()), verbose);
            
            // Check if repository exists and is a git repository
            if !target_path.exists() {
                print_warning(&format!("Repository {} does not exist locally, skipping", repo.path));
                skipped_count += 1;
                continue;
            }
            
            // Check if the target path is a git repository
            if !target_path.join(".git").exists() {
                print_warning(&format!("Repository {} is not a git repository, skipping", repo.path));
                skipped_count += 1;
                continue;
            }
            
            match self.save_repository(&git_manager, &target_path, &commit_message, verbose) {
                Ok(()) => {
                    success_count += 1;
                    if verbose {
                        print_success(&format!("Successfully saved repository: {}", repo.path));
                    }
                }
                Err(e) => {
                    // Check if this is a "no changes" error
                    if e.to_string().contains("No changes to commit") {
                        skipped_count += 1;
                        if verbose {
                            print_info(&format!("Repository {} has no changes, skipped", repo.path));
                        }
                    } else {
                        error_count.fetch_add(1, Ordering::SeqCst);
                        print_error(&format!("Failed to save repository {}: {}", repo.path, e));
                        if verbose {
                            let mut source = e.source();
                            while let Some(err) = source {
                                eprintln!("  Caused by: {}", err);
                                source = err.source();
                            }
                        }
                    }
                }
            }
            
            // Update progress
            if let Some((_, ref main_progress)) = multi_progress {
                main_progress.inc(1);
                main_progress.set_message(format!("Saved {}/{}", index + 1, active_repositories.len()));
            }
        }
        
        // Finish progress
        if let Some((mp, main_progress)) = multi_progress {
            main_progress.finish_with_message("Save operation complete");
            mp.clear().unwrap_or(());
        }
        
        // Print summary
        let errors = error_count.load(Ordering::SeqCst);
        println!();
        print_info("Save Summary:");
        println!("  • Repositories processed: {}", active_repositories.len());
        println!("  • Successful saves: {}", success_count);
        if skipped_count > 0 {
            println!("  • Repositories with no changes: {}", skipped_count);
        }
        if errors > 0 {
            println!("  • Repositories with errors: {}", errors);
        }
        
        if verbose {
            println!();
            println!("Note: Save operation processes all configured repositories with changes");
            println!("Use 'mctl list' to see all configured repositories");
        }
        
        if errors > 0 {
            print_warning("Some repositories could not be saved. See error messages above.");
        } else if success_count > 0 {
            print_success("All active repositories saved successfully!");
        }
        
        Ok(())
    }
}

impl SaveCommand {
    /// Save a single repository by performing git add --all, commit, and push
    fn save_repository(&self, git_manager: &GitManager, repo_path: &PathBuf, commit_message: &str, verbose: bool) -> Result<()> {
        print_verbose(&format!("Starting save operation for: {}", repo_path.display()), verbose);
        
        // Step 1: Check if there are any changes to commit
        print_verbose("Checking for changes...", verbose);
        let detailed_status = git_manager.get_detailed_repository_status(repo_path)
            .context("Failed to get repository status")?;
        
        // Check if there are any files with changes (working directory or staged)
        let has_changes = detailed_status.files.iter().any(|file| {
            file.working_dir_status != mirror_sdk::git::FileChangeType::Unmodified ||
            file.index_status != mirror_sdk::git::FileChangeType::Unmodified
        });
        
        if !has_changes {
            print_verbose("No changes to commit, skipping", verbose);
            return Err(anyhow::anyhow!("No changes to commit"));
        }
        
        print_verbose(&format!("Found {} files with changes", detailed_status.files.len()), verbose);
        
        // Step 2: git add --all
        print_verbose("Staging all changes...", verbose);
        git_manager.add_all(repo_path)
            .context("Failed to stage all changes")?;
        
        // Step 3: git commit
        print_verbose(&format!("Creating commit with message: \"{}\"", commit_message), verbose);
        git_manager.commit(repo_path, commit_message)
            .context("Failed to create commit")?;
        
        // Step 4: git push to current branch
        let current_branch = git_manager.get_current_branch(repo_path)
            .context("Failed to get current branch name")?;
        
        print_verbose(&format!("Pushing to branch: {}", current_branch), verbose);
        git_manager.push_to_current_branch(repo_path)
            .context("Failed to push to remote")?;
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use mirror_sdk::Repository;

    fn create_test_repository(git: &str, path: &str, skip_push: bool) -> Repository {
        Repository::new(
            git.to_string(),
            path.to_string(),
            Some("main".to_string()),
            Some(skip_push),
        )
    }

    #[test]
    fn test_save_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let save_command = SaveCommand { message: None };
        
        // Should not error on empty config
        save_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_save_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let save_command = SaveCommand { message: None };
        
        // Should not error on missing config
        save_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_save_with_custom_message() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo", false);
        config_manager.add_repository(repo).unwrap();
        
        let save_command = SaveCommand { message: Some("Custom commit message".to_string()) };
        save_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_save_with_skip_push_repositories() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repositories - one with skip_push = true
        let repo1 = create_test_repository("git@github.com:org/repo1.git", "org/repo1", false);
        let repo2 = create_test_repository("git@github.com:org/repo2.git", "org/repo2", true);
        
        config_manager.add_repository(repo1).unwrap();
        config_manager.add_repository(repo2).unwrap();
        
        let save_command = SaveCommand { message: None };
        save_command.execute(&config_manager, false).unwrap();
    }
}