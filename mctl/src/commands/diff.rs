use anyhow::{Result, Context};
use mirror_sdk::{ConfigManager, GitManager, RepositoryDiff};
use super::{Command, print_error, print_info, print_warning, print_verbose};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;

pub struct DiffCommand {
    pub staged: bool,
    pub all: bool,
    pub detailed: bool,
}

impl Command for DiffCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        print_verbose("Loading repository configuration", verbose);
        
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
        
        // Initialize Git manager
        let git_manager = GitManager::new()
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
            main_progress.set_message("Checking repository diffs");
            Some((mp, main_progress))
        } else {
            None
        };
        
        // Collect diff for each repository
        let mut repo_diffs: BTreeMap<String, RepositoryDiff> = BTreeMap::new();
        let error_count = Arc::new(AtomicUsize::new(0));
        let mut repositories_with_changes = 0;
        
        for (index, repo) in active_repositories.iter().enumerate() {
            let target_path = PathBuf::from(&repo.path);
            
            print_verbose(&format!("Checking repository {}/{}: {} -> {}",
                index + 1, active_repositories.len(), repo.git, target_path.display()), verbose);
            
            match git_manager.get_repository_diff(&target_path) {
                Ok(diff) => {
                    // Only store repositories that have actual changes
                    let has_changes = match (&self.staged, &self.all) {
                        (true, false) => diff.staged_diff.is_some(),
                        (false, true) => diff.working_diff.is_some() || diff.staged_diff.is_some(),
                        _ => diff.working_diff.is_some(), // Default: working directory diff
                    };
                    
                    if has_changes {
                        repo_diffs.insert(repo.path.clone(), diff);
                        repositories_with_changes += 1;
                    }
                    
                    if verbose {
                        let status = if has_changes { "Has changes" } else { "No changes" };
                        println!("  ✓ {}: {}", repo.path, status);
                    }
                }
                Err(e) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    print_error(&format!("Failed to get diff for {}: {}", repo.path, e));
                }
            }
            
            // Update progress
            if let Some((_, ref main_progress)) = multi_progress {
                main_progress.inc(1);
                main_progress.set_message(format!("Checked {}/{}", index + 1, active_repositories.len()));
            }
        }
        
        // Finish progress
        if let Some((mp, main_progress)) = multi_progress {
            main_progress.finish_with_message("Diff check complete");
            mp.clear().unwrap_or(());
        }
        
        // Display results
        if repo_diffs.is_empty() {
            println!();
            if error_count.load(Ordering::SeqCst) == 0 {
                print_info("No repositories have changes to show.");
            } else {
                print_warning("No repositories could be processed successfully.");
            }
        } else {
            println!();
            let diff_type = match (self.staged, self.all) {
                (true, false) => "Staged Changes",
                (false, true) => "All Changes",
                _ => "Working Directory Changes",
            };
            print_info(&format!("{} ({} repositories):", diff_type, repo_diffs.len()));
            
            for (repo_path, diff) in &repo_diffs {
                // Find the corresponding repository config for additional details
                let repo_config = active_repositories.iter()
                    .find(|r| r.path == *repo_path);
                
                println!();
                println!("Repository: {}", repo_path);
                
                if self.detailed {
                    if let Some(config) = repo_config {
                        println!("Git URL: {}", config.git);
                        println!("Branch: {}", config.branch);
                        println!("Hash: {}", config.compute_hash());
                    }
                }
                
                // Show working directory changes
                if !self.staged && (self.all || !self.staged) {
                    if let Some(ref working_diff) = diff.working_diff {
                        if self.all && diff.staged_diff.is_some() {
                            println!("Working Directory Changes:");
                        }
                        println!("{}", working_diff);
                    }
                }
                
                // Show staged changes
                if self.staged || self.all {
                    if let Some(ref staged_diff) = diff.staged_diff {
                        if self.all && diff.working_diff.is_some() {
                            println!("Staged Changes:");
                        }
                        println!("{}", staged_diff);
                    }
                }
            }
        }
        
        // Print summary
        let errors = error_count.load(Ordering::SeqCst);
        println!();
        print_info("Diff Summary:");
        println!("  • Active repositories checked: {}", active_repositories.len());
        if repositories.len() > active_repositories.len() {
            println!("  • Repositories skipped (skip_push = true): {}", repositories.len() - active_repositories.len());
        }
        println!("  • Repositories with changes: {}", repositories_with_changes);
        if errors > 0 {
            println!("  • Repositories with errors: {}", errors);
        }
        
        if verbose {
            println!();
            let diff_type = match (self.staged, self.all) {
                (true, false) => "staged",
                (false, true) => "all",
                _ => "working directory",
            };
            println!("Use 'mctl diff --{}' for different change types", 
                if self.staged { "all" } else if self.all { "staged" } else { "staged" });
            println!("Use 'mctl status' to see repository status information");
        }
        
        if errors > 0 {
            print_warning("Some repositories could not be processed. See error messages above.");
        }
        
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
    fn test_diff_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let diff_command = DiffCommand { 
            staged: false, 
            all: false, 
            detailed: false 
        };
        
        // Should not error on empty config
        diff_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_diff_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let diff_command = DiffCommand { 
            staged: false, 
            all: false, 
            detailed: false 
        };
        
        // Should not error on missing config
        diff_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_diff_with_skip_push_repositories() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repositories - one with skip_push = true
        let repo1 = create_test_repository("git@github.com:org/repo1.git", "org/repo1", false);
        let repo2 = create_test_repository("git@github.com:org/repo2.git", "org/repo2", true);
        
        config_manager.add_repository(repo1).unwrap();
        config_manager.add_repository(repo2).unwrap();
        
        let diff_command = DiffCommand { 
            staged: false, 
            all: false, 
            detailed: false 
        };
        diff_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_diff_staged_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo", false);
        config_manager.add_repository(repo).unwrap();
        
        let diff_command = DiffCommand { 
            staged: true, 
            all: false, 
            detailed: false 
        };
        diff_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_diff_all_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo", false);
        config_manager.add_repository(repo).unwrap();
        
        let diff_command = DiffCommand { 
            staged: false, 
            all: true, 
            detailed: false 
        };
        diff_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_diff_detailed_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo", false);
        config_manager.add_repository(repo).unwrap();
        
        let diff_command = DiffCommand { 
            staged: false, 
            all: false, 
            detailed: true 
        };
        diff_command.execute(&config_manager, false).unwrap();
    }
}