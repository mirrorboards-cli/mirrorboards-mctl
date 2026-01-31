//! Diff command - show diff of changes in repositories.

use crate::cli::commands::print_error;
use crate::core::config::MirrorConfig;
use crate::core::repository::Repository;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;

pub fn execute(config_path: &str, workspace: Option<String>, staged_only: bool) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    // Filter repositories
    let repos: Vec<&Repository> = if let Some(ws) = &workspace {
        config.filter_by_workspace(ws)
    } else {
        config.repositories.iter().collect()
    };

    if repos.is_empty() {
        if let Some(ws) = &workspace {
            println!("No repositories in workspace '{}'", ws);
        } else {
            println!("No repositories configured");
        }
        return Ok(());
    }

    let git = GitClient::new();
    let mut has_output = false;

    for repo in repos {
        let local_path = Path::new(&repo.path);

        if !local_path.exists() || !git.is_git_repository(local_path) {
            continue;
        }

        let diff_output = if staged_only {
            git.diff_staged(local_path)
        } else {
            git.diff(local_path)
        };

        match diff_output {
            Ok(diff) => {
                if !diff.is_empty() {
                    if has_output {
                        println!();
                    }

                    println!(
                        "{}",
                        format!("━━━ {} ━━━", repo.path).cyan().bold()
                    );
                    println!();

                    // Print diff with syntax highlighting
                    for line in diff.lines() {
                        if line.starts_with('+') && !line.starts_with("+++") {
                            println!("{}", line.green());
                        } else if line.starts_with('-') && !line.starts_with("---") {
                            println!("{}", line.red());
                        } else if line.starts_with("@@") {
                            println!("{}", line.cyan());
                        } else if line.starts_with("diff ") || line.starts_with("index ") {
                            println!("{}", line.dimmed());
                        } else {
                            println!("{}", line);
                        }
                    }

                    has_output = true;
                }
            }
            Err(e) => {
                print_error(&format!("{}: Failed to get diff: {}", repo.path, e));
            }
        }
    }

    if !has_output {
        println!("No changes to show");
    }

    Ok(())
}
