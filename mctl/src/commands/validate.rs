use anyhow::{Result, Context, bail};
use mirror_sdk::{ConfigManager, validate_git_url};
use super::{Command, print_success, print_warning, print_error, print_verbose, print_info};

pub struct ValidateCommand {
    pub detailed: bool,
}

impl Command for ValidateCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        print_verbose("Starting configuration validation", verbose);
        
        if !config_manager.exists() {
            bail!("Configuration file does not exist. Run 'mctl init' to create one.");
        }
        
        let repositories = config_manager.list_repositories()
            .context("Failed to load repositories from configuration")?;
        
        if repositories.is_empty() {
            print_warning("Configuration file is empty (no repositories defined)");
            if verbose {
                println!("This is valid but may not be what you intended.");
                println!("Add repositories with: mctl add <git-url>");
            }
            return Ok(());
        }
        
        print_verbose(&format!("Validating {} repositories", repositories.len()), verbose);
        
        let mut errors = Vec::new();
        let mut warnings = Vec::new();
        let mut valid_count = 0;
        
        // Validate each repository
        for (index, repo) in repositories.iter().enumerate() {
            let repo_number = index + 1;
            let hash = repo.compute_hash();
            
            print_verbose(&format!("Validating repository {}: {}", repo_number, repo.git), verbose);
            
            // Validate repository configuration
            if let Err(e) = repo.validate() {
                errors.push(format!("Repository {} ({}): {}", repo_number, &hash[..8], e));
                continue;
            }
            
            // Validate git URL format
            if let Err(e) = validate_git_url(&repo.git) {
                errors.push(format!("Repository {} ({}): Invalid git URL - {}", repo_number, &hash[..8], e));
                continue;
            }
            
            // Check for potential issues (warnings)
            if repo.path.is_empty() {
                warnings.push(format!("Repository {} ({}): Empty path", repo_number, &hash[..8]));
            }
            
            if repo.branch.is_empty() {
                warnings.push(format!("Repository {} ({}): Empty branch name", repo_number, &hash[..8]));
            }
            
            if repo.git.contains(' ') {
                warnings.push(format!("Repository {} ({}): Git URL contains spaces", repo_number, &hash[..8]));
            }
            
            // Path validation warnings
            if repo.path.starts_with('/') {
                warnings.push(format!("Repository {} ({}): Absolute path detected - consider using relative paths", repo_number, &hash[..8]));
            }
            
            if repo.path.contains("..") {
                warnings.push(format!("Repository {} ({}): Path contains '..' - this may be unsafe", repo_number, &hash[..8]));
            }
            
            valid_count += 1;
            
            if self.detailed {
                print_info(&format!("✓ Repository {}: {} -> {}", repo_number, repo.git, repo.path));
            }
        }
        
        // Check for duplicate repositories
        let mut seen_urls = std::collections::HashSet::new();
        let mut seen_paths = std::collections::HashSet::new();
        
        for (index, repo) in repositories.iter().enumerate() {
            let repo_number = index + 1;
            let hash = repo.compute_hash();
            
            if !seen_urls.insert(&repo.git) {
                warnings.push(format!("Repository {} ({}): Duplicate git URL: {}", repo_number, &hash[..8], repo.git));
            }
            
            if !seen_paths.insert(&repo.path) {
                warnings.push(format!("Repository {} ({}): Duplicate path: {}", repo_number, &hash[..8], repo.path));
            }
        }
        
        // Report results
        println!("\nValidation Results:");
        println!("  Total repositories: {}", repositories.len());
        println!("  Valid repositories: {}", valid_count);
        println!("  Errors: {}", errors.len());
        println!("  Warnings: {}", warnings.len());
        
        // Display errors
        if !errors.is_empty() {
            println!("\nErrors:");
            for error in &errors {
                print_error(&format!("  {}", error));
            }
        }
        
        // Display warnings
        if !warnings.is_empty() {
            println!("\nWarnings:");
            for warning in &warnings {
                print_warning(&format!("  {}", warning));
            }
        }
        
        // Summary
        if errors.is_empty() && warnings.is_empty() {
            print_success("Configuration is valid with no issues detected");
        } else if errors.is_empty() {
            print_success("Configuration is valid but has some warnings");
        } else {
            print_error("Configuration has validation errors that should be fixed");
            bail!("Validation failed with {} errors", errors.len());
        }
        
        if verbose && (self.detailed || !errors.is_empty() || !warnings.is_empty()) {
            println!("\nRecommendations:");
            if !errors.is_empty() {
                println!("  • Fix the errors listed above before using the configuration");
                println!("  • Use 'mctl show <hash>' to see details for specific repositories");
                println!("  • Use 'mctl remove <hash>' to remove problematic repositories");
            }
            if !warnings.is_empty() {
                println!("  • Review the warnings to ensure they are acceptable");
                println!("  • Consider fixing path and URL format issues");
            }
            println!("  • Run 'mctl validate --detailed' for repository-by-repository validation");
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
    fn test_validate_nonexistent_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so config doesn't exist
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let validate_command = ValidateCommand { detailed: false };
        
        let result = validate_command.execute(&config_manager, false);
        assert!(result.is_err());
    }
    
    #[test]
    fn test_validate_empty_config() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create empty config
        config_manager.create_empty().unwrap();
        
        let validate_command = ValidateCommand { detailed: false };
        
        // Should not error on empty config
        validate_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_validate_valid_repositories() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add valid repositories
        let repo1 = create_test_repository(
            "git@github.com:org/repo1.git",
            "org/repo1",
            "main",
            false
        );
        let repo2 = create_test_repository(
            "https://github.com/org/repo2.git",
            "org/repo2",
            "develop",
            true
        );
        
        config_manager.add_repository(repo1).unwrap();
        config_manager.add_repository(repo2).unwrap();
        
        let validate_command = ValidateCommand { detailed: true };
        validate_command.execute(&config_manager, false).unwrap();
    }
    
    #[test]
    fn test_validate_with_warnings() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add repository with potential warning (absolute path)
        let repo = create_test_repository(
            "git@github.com:org/repo.git",
            "/absolute/path",  // This should generate a warning
            "main",
            false
        );
        
        config_manager.add_repository(repo).unwrap();
        
        let validate_command = ValidateCommand { detailed: false };
        validate_command.execute(&config_manager, true).unwrap(); // Should pass but with warnings
    }
    
    #[test]
    fn test_validate_detailed_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Add test repository
        let repo = create_test_repository(
            "git@github.com:org/repo.git",
            "org/repo",
            "main",
            false
        );
        
        config_manager.add_repository(repo).unwrap();
        
        let validate_command = ValidateCommand { detailed: true };
        validate_command.execute(&config_manager, true).unwrap();
    }
}