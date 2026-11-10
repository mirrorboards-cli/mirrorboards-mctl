//! Remove command - remove a repository from the configuration.

use crate::cli::commands::{print_info, print_success, print_warning};
use crate::core::config::ConfigManager;
use crate::core::include::IncludeResolver;
use anyhow::{bail, Result};
use std::path::Path;

pub fn execute(config_path: &str, repo_path: &str, delete_local: bool) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        bail!("Configuration file not found: {}", config_path);
    }

    // The entry may live in an included file; edit the file that defines it.
    let resolved = IncludeResolver::resolve(config_file)?;
    let Some(defined) = resolved
        .repositories
        .iter()
        .find(|entry| entry.repository.path == repo_path)
    else {
        bail!("Repository not found: {}", repo_path);
    };

    let mut manager = ConfigManager::open(&defined.source_file)?;
    let repo = manager.remove_repository(repo_path)?;
    manager.save()?;

    print_success(&format!("Removed repository: {}", repo_path));
    println!("  Git: {}", repo.git);
    println!("  File: {}", defined.source_file.display());

    if delete_local {
        let local_path = repo.resolve_local_path(config_file);

        if local_path.exists() {
            print_warning(&format!(
                "Deleting local directory: {}",
                local_path.display()
            ));
            std::fs::remove_dir_all(&local_path)?;
            print_success("Local directory deleted");
        } else {
            print_info("Local directory does not exist");
        }
    }

    Ok(())
}
