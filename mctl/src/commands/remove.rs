use anyhow::{Result, Context, bail};
use mirror_sdk::ConfigManager;
use dialoguer::Confirm;
use super::{Command, print_success, print_warning, print_verbose, print_info};

pub struct RemoveCommand {
    pub hash: String,
    pub force: bool,
}

impl Command for RemoveCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        print_verbose(&format!("Looking for repository with hash: {}", self.hash), verbose);
        
        if !config_manager.exists() {
            bail!("Configuration file does not exist. Run 'mctl init' to create one.");
        }
        
        // Find the repository by hash
        let repository = config_manager.find_repository(&self.hash)
            .context("Failed to search for repository")?;
        
        let repo = match repository {
            Some(repo) => repo,
            None => {
                print_warning(&format!("No repository found with hash starting with '{}'", self.hash));
                
                if verbose {
                    println!("Use 'mctl list' to see all available repositories and their hashes");
                }
                return Ok(());
            }
        };
        
        let full_hash = repo.compute_hash();
        print_verbose(&format!("Found repository: {} ({})", repo.git, full_hash), verbose);
        
        // Show repository details
        println!("Repository to remove:");
        print_info(&format!("  Hash: {}", full_hash));
        print_info(&format!("  Git URL: {}", repo.git));
        print_info(&format!("  Path: {}", repo.path));
        print_info(&format!("  Branch: {}", repo.branch));
        print_info(&format!("  Skip Push: {}", repo.skip_push));
        
        // Confirm removal unless force is specified
        if !self.force {
            let confirm = Confirm::new()
                .with_prompt("Are you sure you want to remove this repository?")
                .default(false)
                .interact()
                .context("Failed to get user confirmation")?;
            
            if !confirm {
                println!("Removal cancelled.");
                return Ok(());
            }
        } else {
            print_verbose("Skipping confirmation due to --force flag", verbose);
        }
        
        // Remove the repository
        let removed_repo = config_manager.remove_repository(&self.hash)
            .context("Failed to remove repository from configuration")?;
        
        print_success(&format!("Removed repository: {}", removed_repo.git));
        
        if verbose {
            print_info(&format!("Repository with hash {} has been removed from the configuration", full_hash));
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use mirror_sdk::Repository;

    fn create_test_repository(git: &str, path: &str) -> Repository {
        Repository::new(
            git.to_string(),
            path.to_string(),
            Some("main".to_string()),
            Some(false),
        )
    }

    #[test]
    fn test_remove_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let remove_command = RemoveCommand {
            hash: "abcd1234".to_string(),
            force: true,
        };
        
        let result = remove_command.execute(&config_manager, false);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_remove_nonexistent_repository() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let remove_command = RemoveCommand {
            hash: "nonexistent".to_string(),
            force: true,
        };
        
        // Should not error, but should report not found
        remove_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_remove_existing_repository() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo");
        let hash = repo.compute_hash();
        
        config_manager.add_repository(repo.clone()).unwrap();
        
        // Verify repository exists
        let repos_before = config_manager.list_repositories().unwrap();
        assert_eq!(repos_before.len(), 1);
        
        // Remove repository
        let remove_command = RemoveCommand {
            hash: hash[..4].to_string(), // Use partial hash
            force: true,
        };
        
        remove_command.execute(&config_manager, false).unwrap();
        
        // Verify repository was removed
        let repos_after = config_manager.list_repositories().unwrap();
        assert_eq!(repos_after.len(), 0);
    }
    
    #[test]
    fn test_remove_with_partial_hash() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo");
        let full_hash = repo.compute_hash();
        
        config_manager.add_repository(repo).unwrap();
        
        // Remove with partial hash (first 4 characters)
        let remove_command = RemoveCommand {
            hash: full_hash[..4].to_string(),
            force: true,
        };
        
        remove_command.execute(&config_manager, false).unwrap();
        
        // Verify repository was removed
        let repos = config_manager.list_repositories().unwrap();
        assert_eq!(repos.len(), 0);
    }
    
    #[test]
    fn test_remove_with_full_hash() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository("git@github.com:org/repo.git", "org/repo");
        let full_hash = repo.compute_hash();
        
        config_manager.add_repository(repo).unwrap();
        
        // Remove with full hash
        let remove_command = RemoveCommand {
            hash: full_hash,
            force: true,
        };
        
        remove_command.execute(&config_manager, false).unwrap();
        
        // Verify repository was removed
        let repos = config_manager.list_repositories().unwrap();
        assert_eq!(repos.len(), 0);
    }
}