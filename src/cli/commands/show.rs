//! Show command - show details of a repository.

use crate::cli::commands::print_error;
use crate::core::config::MirrorConfig;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub fn execute(config_path: &str, repo_path: &str) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    let repo = match config.find_by_path(repo_path) {
        Some(r) => r,
        None => {
            print_error(&format!("Repository not found: {}", repo_path));
            return Ok(());
        }
    };

    // Print basic info
    println!("{}", "Repository Details".bold());
    println!();
    println!("  {}: {}", "Path".cyan(), repo.path);
    println!("  {}: {}", "Git".cyan(), repo.git);

    // Version
    let version_str = if let Some(rev) = &repo.rev {
        format!("rev: {}", rev)
    } else if let Some(tag) = &repo.tag {
        format!("tag: {}", tag)
    } else {
        format!("branch: {}", repo.branch.as_deref().unwrap_or("main"))
    };
    println!("  {}: {}", "Version".cyan(), version_str);

    // Workspaces
    if !repo.workspaces.is_empty() {
        println!("  {}: {}", "Workspaces".cyan(), repo.workspaces.join(", "));
    }

    // Flags
    if repo.skip_push {
        println!("  {}: true", "Skip-push".cyan());
    }

    // Check local state if exists
    let local_path = Path::new(&repo.path);
    println!();

    if local_path.exists() {
        println!("{}", "Local State".bold());
        println!();

        let git = GitClient::new();

        if git.is_git_repository(local_path) {
            match git.status_fast(local_path) {
                Ok(status) => {
                    println!("  {}: {}", "Branch".cyan(), status.branch.name);
                    println!("  {}: {}", "HEAD".cyan(), status.head_short);

                    if let Some(upstream) = &status.branch.upstream {
                        println!("  {}: {}", "Upstream".cyan(), upstream);
                        println!(
                            "  {}: +{} -{}",
                            "Ahead/Behind".cyan(),
                            status.branch.ahead,
                            status.branch.behind
                        );
                    }

                    if status.is_clean() {
                        println!("  {}: {}", "Status".cyan(), "Clean".green());
                    } else {
                        println!("  {}: {}", "Status".cyan(), status.summary().yellow());
                    }
                }
                Err(e) => {
                    println!("  {}: {}", "Error".red(), e);
                }
            }
        } else {
            println!("  {}: Not a git repository", "Status".cyan());
        }
    } else {
        println!("{}", "Local State".bold());
        println!();
        println!("  {}: Not cloned", "Status".cyan());
        println!("  Run 'mctl sync' to clone the repository");
    }

    Ok(())
}
