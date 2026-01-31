//! Add command - add a repository to the configuration.

use crate::cli::commands::{print_error, print_info, print_success};
use crate::core::config::ConfigManager;
use crate::core::repository::Repository;
use crate::core::url::suggest_path;
use anyhow::Result;
use std::path::Path;

pub fn execute(
    config_path: &str,
    git: String,
    path: Option<String>,
    branch: Option<String>,
    rev: Option<String>,
    tag: Option<String>,
    workspaces: Vec<String>,
    skip_push: bool,
) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        print_info("Run 'mctl init' to create a new configuration");
        return Ok(());
    }

    // Determine local path
    let local_path = match path {
        Some(p) => p,
        None => {
            match suggest_path(&git) {
                Some(p) => {
                    print_info(&format!("Using suggested path: {}", p));
                    p
                }
                None => {
                    print_error("Could not determine local path from git URL");
                    print_info("Please specify a path with --path");
                    return Ok(());
                }
            }
        }
    };

    // Validate version spec (only one allowed)
    let version_count = [branch.is_some(), rev.is_some(), tag.is_some()]
        .iter()
        .filter(|&&x| x)
        .count();

    if version_count > 1 {
        print_error("Only one of --branch, --rev, or --tag can be specified");
        return Ok(());
    }

    // Create repository
    let mut repo = Repository::new(&git, &local_path);

    if let Some(ref b) = branch {
        repo = repo.with_branch(b);
    }
    if let Some(ref r) = rev {
        repo = repo.with_rev(r);
    }
    if let Some(ref t) = tag {
        repo = repo.with_tag(t);
    }
    if skip_push {
        repo = repo.with_skip_push(true);
    }
    if !workspaces.is_empty() {
        repo = repo.with_workspaces(workspaces.clone());
    }

    // Load and update config
    let mut manager = ConfigManager::open(config_file)?;

    match manager.add_repository(repo) {
        Ok(_) => {}
        Err(e) => {
            print_error(&format!("Failed to add repository: {}", e));
            return Ok(());
        }
    }

    manager.save()?;

    print_success(&format!("Added repository: {}", local_path));

    // Show details
    println!("  Git: {}", git);
    println!("  Path: {}", local_path);

    if let Some(ref b) = branch {
        println!("  Branch: {}", b);
    }
    if let Some(ref r) = rev {
        println!("  Rev: {}", r);
    }
    if let Some(ref t) = tag {
        println!("  Tag: {}", t);
    }
    if skip_push {
        println!("  Skip-push: true");
    }
    if !workspaces.is_empty() {
        println!("  Workspaces: {}", workspaces.join(", "));
    }

    println!();
    print_info("Run 'mctl sync' to clone the repository");

    Ok(())
}
