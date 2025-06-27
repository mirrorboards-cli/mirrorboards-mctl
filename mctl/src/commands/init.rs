use anyhow::{Result, Context};
use mirror_sdk::ConfigManager;
use dialoguer::Confirm;
use super::{Command, print_success, print_warning, print_verbose};

pub struct InitCommand {
    pub force: bool,
}

impl Command for InitCommand {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()> {
        let config_path = config_manager.path();
        
        print_verbose(&format!("Initializing configuration at: {}", config_path.display()), verbose);
        
        // Check if file already exists
        if config_manager.exists() && !self.force {
            print_warning(&format!("Configuration file already exists: {}", config_path.display()));
            
            let overwrite = Confirm::new()
                .with_prompt("Do you want to overwrite the existing file?")
                .default(false)
                .interact()
                .context("Failed to get user confirmation")?;
            
            if !overwrite {
                println!("Initialization cancelled.");
                return Ok(());
            }
        }
        
        // Create empty configuration
        config_manager.create_empty()
            .context("Failed to create configuration file")?;
        
        print_success(&format!("Initialized empty mirror configuration at {}", config_path.display()));
        
        if verbose {
            println!("\nNext steps:");
            println!("  • Add repositories with: mctl add <git-url>");
            println!("  • List repositories with: mctl list");
            println!("  • View help with: mctl --help");
        }
        
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_init_new_file() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so we can test creation
        drop(temp_file);
        
        let config_manager = ConfigManager::new(&temp_path);
        let init_command = InitCommand { force: false };
        
        assert!(!config_manager.exists());
        init_command.execute(&config_manager, false).unwrap();
        assert!(config_manager.exists());
        
        // Verify file contents
        let content = fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("# Mirror Configuration File"));
    }
    
    #[test]
    fn test_init_force_overwrite() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_manager = ConfigManager::new(temp_file.path());
        
        // Create initial content
        fs::write(temp_file.path(), "existing content").unwrap();
        
        let init_command = InitCommand { force: true };
        init_command.execute(&config_manager, false).unwrap();
        
        // Verify file was overwritten
        let content = fs::read_to_string(temp_file.path()).unwrap();
        assert!(content.contains("# Mirror Configuration File"));
        assert!(!content.contains("existing content"));
    }
}