use anyhow::{Result, Context};
use mirror_sdk::{ConfigManager, Repository, validate_git_url, MirrorSdkError, ConfigError};
use super::{Command, print_success, print_verbose, print_info};

pub struct AddCommand {
    pub git_url: String,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub skip_push: bool,
}

impl Command for AddCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        print_verbose(&format!("Adding repository: {}", self.git_url), verbose);
        
        // Validate the git URL first
        validate_git_url(&self.git_url)
            .context("Invalid git URL format")?;
        
        // Create repository from URL with defaults
        let mut repo = Repository::from_url(self.git_url.clone())
            .context("Failed to parse git URL")?;
        
        // Apply command-line overrides
        if let Some(custom_path) = &self.path {
            repo.path = custom_path.clone();
            print_verbose(&format!("Using custom path: {}", custom_path), verbose);
        } else {
            print_verbose(&format!("Auto-detected path: {}", repo.path), verbose);
        }
        
        if let Some(custom_branch) = &self.branch {
            repo.branch = custom_branch.clone();
            print_verbose(&format!("Using custom branch: {}", custom_branch), verbose);
        } else {
            print_verbose(&format!("Using default branch: {}", repo.branch), verbose);
        }
        
        if self.skip_push {
            repo.skip_push = true;
            print_verbose("Repository marked as skip-push", verbose);
        }
        
        // Validate the repository configuration
        repo.validate()
            .context("Repository configuration validation failed")?;
        
        // Compute hash for reference
        let hash = repo.compute_hash();
        print_verbose(&format!("Repository hash: {}", hash), verbose);
        
        // Add to configuration
        config_manager.add_repository(repo.clone())
            .map_err(|e| match e {
                MirrorSdkError::Config(ConfigError::PathConflict { path, existing_git, new_git }) => {
                    anyhow::anyhow!(
                        "Path conflict detected: '{}' is already used by repository '{}', cannot be used by '{}'",
                        path, existing_git, new_git
                    )
                },
                _ => anyhow::anyhow!("Failed to add repository to configuration: {}", e)
            })?;
        
        print_success(&format!("Added repository: {}", self.git_url));
        
        if verbose {
            print_info(&format!("  Hash: {}", hash));
            print_info(&format!("  Path: {}", repo.path));
            print_info(&format!("  Branch: {}", repo.branch));
            print_info(&format!("  Skip Push: {}", repo.skip_push));
        } else {
            println!("  Hash: {}", hash);
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;

    #[test]
    fn test_add_ssh_repository() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        let add_command = AddCommand {
            git_url: "git@github.com:org/repo.git".to_string(),
            path: None,
            branch: None,
            skip_push: false,
        };
        
        add_command.execute(&config_manager, false).unwrap();
        
        let repos = config_manager.list_repositories().unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].git, "git@github.com:org/repo.git");
        assert_eq!(repos[0].path, "org/repo");
        assert_eq!(repos[0].branch, "main");
        assert_eq!(repos[0].skip_push, false);
    }
    
    #[test]
    fn test_add_https_repository() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        let add_command = AddCommand {
            git_url: "https://github.com/org/repo.git".to_string(),
            path: None,
            branch: None,
            skip_push: false,
        };
        
        add_command.execute(&config_manager, false).unwrap();
        
        let repos = config_manager.list_repositories().unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].git, "https://github.com/org/repo.git");
        assert_eq!(repos[0].path, "org/repo");
    }
    
    #[test]
    fn test_add_with_custom_options() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        let add_command = AddCommand {
            git_url: "git@github.com:org/repo.git".to_string(),
            path: Some("custom/path".to_string()),
            branch: Some("develop".to_string()),
            skip_push: true,
        };
        
        add_command.execute(&config_manager, false).unwrap();
        
        let repos = config_manager.list_repositories().unwrap();
        assert_eq!(repos.len(), 1);
        assert_eq!(repos[0].path, "custom/path");
        assert_eq!(repos[0].branch, "develop");
        assert_eq!(repos[0].skip_push, true);
    }
    
    #[test]
    fn test_add_invalid_url() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        let add_command = AddCommand {
            git_url: "invalid-url".to_string(),
            path: None,
            branch: None,
            skip_push: false,
        };
        
        let result = add_command.execute(&config_manager, false);
        assert!(result.is_err());
    }
}