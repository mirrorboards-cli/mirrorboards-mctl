//! Set-url command - point a repository at a new git URL.

use crate::cli::commands::{print_info, print_success};
use crate::core::config::ConfigManager;
use crate::core::include::IncludeResolver;
use crate::git::GitClient;
use anyhow::{bail, Context, Result};
use std::path::Path;

/// Change the git URL of the repository at `repo_path` in the file that
/// defines it and, when it is cloned, in the `origin` remote of its checkout.
/// Used after a repository or its organization is renamed on the server, so
/// the checkout and its local files stay where they are.
pub fn execute(config_path: &str, repo_path: &str, git_url: &str) -> Result<()> {
    let config_file = Path::new(config_path);
    let repo_path = repo_path.trim_end_matches('/');

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
    let previous_git = manager.set_repository_git(repo_path, git_url)?;
    manager.save()?;

    print_success(&format!("Updated repository: {}", repo_path));
    println!("  Git: {} -> {}", previous_git, git_url);
    println!("  File: {}", defined.source_file.display());

    let local_path = defined.repository.resolve_local_path(config_file);
    let is_cloned = local_path.join(".git").exists();
    if !is_cloned {
        print_info("Not cloned; only the configuration changed");
        return Ok(());
    }

    let git = GitClient::new();
    git.set_remote_url(&local_path, "origin", git_url)
        .with_context(|| format!("Setting origin of {}", local_path.display()))?;
    print_success(&format!("Set origin of {}", local_path.display()));

    Ok(())
}
