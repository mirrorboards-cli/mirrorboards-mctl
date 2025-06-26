use anyhow::{Result, Context};
use mirror_sdk::{ConfigManager, GitManager, RepositoryStatus, DetailedRepositoryStatus, FileChangeType};
use tabled::{Tabled, Table};
use super::{Command, print_error, print_info, print_warning, print_verbose};
use indicatif::{MultiProgress, ProgressBar, ProgressStyle};
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::{AtomicUsize, Ordering};
use std::collections::BTreeMap;

pub struct StatusCommand {
    pub detailed: bool,
}

#[derive(Tabled)]
struct RepositoryStatusRow {
    #[tabled(rename = "Hash")]
    hash: String,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Branch")]
    branch: String,
    #[tabled(rename = "Git URL")]
    git_url: String,
}

#[derive(Tabled)]
struct DetailedRepositoryStatusRow {
    #[tabled(rename = "Hash")]
    hash: String,
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Branch")]
    branch: String,
    #[tabled(rename = "Git URL")]
    git_url: String,
    #[tabled(rename = "Skip Push")]
    skip_push: String,
}

impl Command for StatusCommand {
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
            main_progress.set_message("Checking repository status");
            Some((mp, main_progress))
        } else {
            None
        };
        
        // Collect status for each repository
        let mut status_rows = Vec::new();
        let mut detailed_status_rows = Vec::new();
        let mut repo_file_changes: BTreeMap<String, DetailedRepositoryStatus> = BTreeMap::new();
        let error_count = Arc::new(AtomicUsize::new(0));
        
        for (index, repo) in active_repositories.iter().enumerate() {
            let target_path = PathBuf::from(&repo.path);
            
            print_verbose(&format!("Checking repository {}/{}: {} -> {}",
                index + 1, active_repositories.len(), repo.git, target_path.display()), verbose);
            
            match git_manager.get_detailed_repository_status(&target_path) {
                Ok(detailed_status) => {
                    let status_text = format_repository_status(&detailed_status.status);
                    let hash = repo.compute_hash();
                    
                    // Store file changes for repositories that have them
                    if !detailed_status.files.is_empty() {
                        repo_file_changes.insert(repo.path.clone(), detailed_status.clone());
                    }
                    
                    // Add to regular status table
                    status_rows.push(RepositoryStatusRow {
                        hash: hash.clone(),
                        path: repo.path.clone(),
                        status: status_text.clone(),
                        branch: repo.branch.clone(),
                        git_url: repo.git.clone(),
                    });
                    
                    // Add to detailed status table
                    detailed_status_rows.push(DetailedRepositoryStatusRow {
                        hash,
                        path: repo.path.clone(),
                        status: status_text,
                        branch: repo.branch.clone(),
                        git_url: repo.git.clone(),
                        skip_push: if repo.skip_push { "✓".to_string() } else { "✗".to_string() },
                    });
                    
                    if verbose {
                        println!("  ✓ {}: {}", repo.path, format_repository_status(&detailed_status.status));
                    }
                }
                Err(e) => {
                    error_count.fetch_add(1, Ordering::SeqCst);
                    let error_status = "Error".to_string();
                    let hash = repo.compute_hash();
                    
                    // Add error entries to tables
                    status_rows.push(RepositoryStatusRow {
                        hash: hash.clone(),
                        path: repo.path.clone(),
                        status: error_status.clone(),
                        branch: repo.branch.clone(),
                        git_url: repo.git.clone(),
                    });
                    
                    detailed_status_rows.push(DetailedRepositoryStatusRow {
                        hash,
                        path: repo.path.clone(),
                        status: error_status,
                        branch: repo.branch.clone(),
                        git_url: repo.git.clone(),
                        skip_push: if repo.skip_push { "✓".to_string() } else { "✗".to_string() },
                    });
                    
                    print_error(&format!("Failed to get status for {}: {}", repo.path, e));
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
            main_progress.finish_with_message("Status check complete");
            mp.clear().unwrap_or(());
        }
        
        // Display results
        println!();
        if self.detailed {
            let table = Table::new(detailed_status_rows).to_string();
            println!("{}", table);
        } else {
            let table = Table::new(status_rows).to_string();
            println!("{}", table);
        }
        
        // Display file-level changes for repositories that have them
        if !repo_file_changes.is_empty() {
            println!();
            print_info("File Changes:");
            
            for (repo_path, detailed_status) in &repo_file_changes {
                println!();
                println!("Repository: {}", repo_path);
                println!("Status: {}", format_repository_status(&detailed_status.status));
                println!("Files:");
                
                // Group files by their status
                let mut modified_files = Vec::new();
                let mut new_files = Vec::new();
                let mut deleted_files = Vec::new();
                let mut staged_files = Vec::new();
                let mut renamed_files = Vec::new();
                
                for file in &detailed_status.files {
                    // Check working directory changes first
                    match file.working_dir_status {
                        FileChangeType::New => new_files.push(&file.path),
                        FileChangeType::Modified => modified_files.push(&file.path),
                        FileChangeType::Deleted => deleted_files.push(&file.path),
                        FileChangeType::Renamed => renamed_files.push(&file.path),
                        _ => {}
                    }
                    
                    // Check index/staged changes
                    match file.index_status {
                        FileChangeType::New | FileChangeType::Modified | FileChangeType::Deleted | FileChangeType::Renamed => {
                            staged_files.push(&file.path);
                        }
                        _ => {}
                    }
                }
                
                // Display grouped files
                if !staged_files.is_empty() {
                    println!("  Staged:");
                    for file in staged_files {
                        println!("    - {}", file);
                    }
                }
                
                if !modified_files.is_empty() {
                    println!("  Modified:");
                    for file in modified_files {
                        println!("    - {}", file);
                    }
                }
                
                if !new_files.is_empty() {
                    println!("  Untracked:");
                    for file in new_files {
                        println!("    - {}", file);
                    }
                }
                
                if !deleted_files.is_empty() {
                    println!("  Deleted:");
                    for file in deleted_files {
                        println!("    - {}", file);
                    }
                }
                
                if !renamed_files.is_empty() {
                    println!("  Renamed:");
                    for file in renamed_files {
                        println!("    - {}", file);
                    }
                }
            }
        }
        
        // Print summary
        let errors = error_count.load(Ordering::SeqCst);
        println!();
        print_info("Status Summary:");
        println!("  • Active repositories checked: {}", active_repositories.len());
        if repositories.len() > active_repositories.len() {
            println!("  • Repositories skipped (skip_push = true): {}", repositories.len() - active_repositories.len());
        }
        if !repo_file_changes.is_empty() {
            println!("  • Repositories with file changes: {}", repo_file_changes.len());
        }
        if errors > 0 {
            println!("  • Repositories with errors: {}", errors);
        }
        
        if verbose {
            println!();
            println!("Use 'mctl show <hash>' for detailed information about a repository");
            println!("Use 'mctl sync --pull' to update repositories that need updates");
        }
        
        if errors > 0 {
            print_warning("Some repositories could not be checked. See error messages above.");
        }
        
        Ok(())
    }
}

/// Format RepositoryStatus enum to user-friendly string
fn format_repository_status(status: &RepositoryStatus) -> String {
    match status {
        RepositoryStatus::Missing => "Missing".to_string(),
        RepositoryStatus::NotGitRepository => "Not Git Repo".to_string(),
        RepositoryStatus::UpToDate => "Up to Date".to_string(),
        RepositoryStatus::NeedsUpdate => "Needs Update".to_string(),
        RepositoryStatus::HasConflicts => "Has Conflicts".to_string(),
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
    fn test_status_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let status_command = StatusCommand { detailed: false };
        
        // Should not error on empty config
        status_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_status_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let status_command = StatusCommand { detailed: false };
        
        // Should not error on missing config
        status_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_status_with_skip_push_repositories() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repositories - one with skip_push = true
        let repo1 = create_test_repository("git@github.com:org/repo1.git", "org/repo1", false);
        let repo2 = create_test_repository("git@github.com:org/repo2.git", "org/repo2", true);
        
        config_manager.add_repository(repo1).unwrap();
        config_manager.add_repository(repo2).unwrap();
        
        let status_command = StatusCommand { detailed: false };
        status_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_status_detailed_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo", false);
        config_manager.add_repository(repo).unwrap();
        
        let status_command = StatusCommand { detailed: true };
        status_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_format_repository_status() {
        assert_eq!(format_repository_status(&RepositoryStatus::Missing), "Missing");
        assert_eq!(format_repository_status(&RepositoryStatus::NotGitRepository), "Not Git Repo");
        assert_eq!(format_repository_status(&RepositoryStatus::UpToDate), "Up to Date");
        assert_eq!(format_repository_status(&RepositoryStatus::NeedsUpdate), "Needs Update");
        assert_eq!(format_repository_status(&RepositoryStatus::HasConflicts), "Has Conflicts");
    }
}