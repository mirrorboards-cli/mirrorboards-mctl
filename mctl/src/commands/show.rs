use anyhow::{Result, Context, bail};
use mirror_sdk::{ConfigManager, extract_hostname};
use super::{Command, print_warning, print_verbose, print_info};

pub struct ShowCommand {
    pub hash: String,
}

impl Command for ShowCommand {
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
        print_verbose(&format!("Found repository: {}", repo.git), verbose);
        
        // Display detailed repository information
        println!("Repository Details:");
        println!("  Hash: {}", full_hash);
        println!("  Git URL: {}", repo.git);
        println!("  Local Path: {}", repo.path);
        println!("  Branch: {}", repo.branch);
        println!("  Skip Push: {}", if repo.skip_push { "Yes" } else { "No" });
        
        // Extract and display hostname
        if let Ok(hostname) = extract_hostname(&repo.git) {
            println!("  Hostname: {}", hostname);
        }
        
        // Additional information in verbose mode
        if verbose {
            println!("\nAdditional Information:");
            
            // URL format detection
            if repo.git.starts_with("git@") {
                print_info("  URL Format: SSH");
            } else if repo.git.starts_with("https://") || repo.git.starts_with("http://") {
                print_info("  URL Format: HTTPS");
            } else {
                print_info("  URL Format: Unknown");
            }
            
            // Path information
            if repo.path.contains('/') {
                let parts: Vec<&str> = repo.path.split('/').collect();
                if parts.len() >= 2 {
                    print_info(&format!("  Organization: {}", parts[0]));
                    print_info(&format!("  Repository: {}", parts[1]));
                }
            }
            
            // Configuration status
            if repo.branch != "main" {
                print_info("  Uses non-default branch");
            }
            
            if repo.skip_push {
                print_info("  Configured as read-only (skip-push enabled)");
            }
            
            println!("\nManagement Commands:");
            println!("  Update: mctl remove {} && mctl add {}", &full_hash[..8], repo.git);
            println!("  Remove: mctl remove {}", &full_hash[..8]);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use mirror_sdk::Repository;

    fn create_test_repository(git: &str, path: &str, branch: &str, skip_push: bool) -> Repository {
        Repository::new(
            git.to_string(),
            path.to_string(),
            Some(branch.to_string()),
            Some(skip_push),
        )
    }

    #[test]
    fn test_show_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let show_command = ShowCommand {
            hash: "abcd1234".to_string(),
        };
        
        let result = show_command.execute(&config_manager, false);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_show_nonexistent_repository() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let show_command = ShowCommand {
            hash: "nonexistent".to_string(),
        };
        
        // Should not error, but should report not found
        show_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_show_existing_repository() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository(
            "git@github.com:org/repo.git",
            "org/repo",
            "main",
            false
        );
        let hash = repo.compute_hash();
        
        config_manager.add_repository(repo).unwrap();
        
        let show_command = ShowCommand {
            hash: hash[..4].to_string(), // Use partial hash
        };
        
        show_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_show_with_custom_settings() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository with custom settings
        let repo = create_test_repository(
            "https://github.com/external/readonly.git",
            "external/readonly",
            "develop",
            true
        );
        let hash = repo.compute_hash();
        
        config_manager.add_repository(repo).unwrap();
        
        let show_command = ShowCommand {
            hash: hash[..6].to_string(), // Use longer partial hash
        };
        
        show_command.execute(&config_manager, true).unwrap(); // Test verbose mode
    }
    
    #[test]
    fn test_show_with_full_hash() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository(
            "git@gitlab.com:group/project.git",
            "group/project",
            "main",
            false
        );
        let full_hash = repo.compute_hash();
        
        config_manager.add_repository(repo).unwrap();
        
        let show_command = ShowCommand {
            hash: full_hash,
        };
        
        show_command.execute(&config_manager, false).unwrap();
    }
}