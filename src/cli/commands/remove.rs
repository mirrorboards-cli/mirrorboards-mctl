//! Remove command - remove a repository from the configuration.

use crate::cli::commands::{print_error, print_info, print_success, print_warning};
use crate::core::config::ConfigManager;
use anyhow::Result;
use std::path::Path;

pub fn execute(config_path: &str, repo_path: &str, delete_local: bool) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let mut manager = ConfigManager::open(config_file)?;

    match manager.remove_repository(repo_path) {
        Ok(repo) => {
            manager.save()?;
            print_success(&format!("Removed repository: {}", repo_path));
            println!("  Git: {}", repo.git);

            // Delete local directory if requested
            if delete_local {
                let local_path = Path::new(&repo.path);
                if local_path.exists() {
                    print_warning(&format!("Deleting local directory: {}", repo.path));
                    std::fs::remove_dir_all(local_path)?;
                    print_success("Local directory deleted");
                } else {
                    print_info("Local directory does not exist");
                }
            }
        }
        Err(e) => {
            print_error(&format!("Failed to remove repository: {}", e));
        }
    }

    Ok(())
}
