//! Status command - show status of repositories.

use crate::cli::commands::print_error;
use crate::core::config::MirrorConfig;
use crate::core::repository::Repository;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use std::path::Path;
use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct StatusRow {
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Branch")]
    branch: String,
    #[tabled(rename = "Status")]
    status: String,
    #[tabled(rename = "Sync")]
    sync: String,
}

pub fn execute(config_path: &str, workspace: Option<String>, detailed: bool) -> Result<()> {
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

    // Print header
    if let Some(ws) = &workspace {
        println!(
            "{} {} ({} repositories)",
            "Status for workspace:".bold(),
            ws.cyan(),
            repos.len()
        );
    } else {
        println!(
            "{} ({} repositories)",
            "Status for all repositories".bold(),
            repos.len()
        );
    }
    println!();

    let git = GitClient::new();

    if detailed {
        // Detailed view - show each repo separately
        for repo in repos {
            print_detailed_status(&git, repo)?;
        }
    } else {
        // Table view
        let mut rows = Vec::new();

        for repo in repos {
            let local_path = Path::new(&repo.path);

            if !local_path.exists() {
                rows.push(StatusRow {
                    path: repo.path.clone(),
                    branch: "-".to_string(),
                    status: "Not cloned".yellow().to_string(),
                    sync: "-".to_string(),
                });
                continue;
            }

            if !git.is_git_repository(local_path) {
                rows.push(StatusRow {
                    path: repo.path.clone(),
                    branch: "-".to_string(),
                    status: "Not a git repo".red().to_string(),
                    sync: "-".to_string(),
                });
                continue;
            }

            match git.status(local_path) {
                Ok(status) => {
                    let status_str = if status.is_clean() {
                        "Clean".green().to_string()
                    } else {
                        status.summary().yellow().to_string()
                    };

                    let sync_str = if status.branch.is_synced() {
                        "Up to date".green().to_string()
                    } else if status.branch.upstream.is_some() {
                        format!(
                            "+{} -{}",
                            status.branch.ahead, status.branch.behind
                        )
                        .yellow()
                        .to_string()
                    } else {
                        "No upstream".dimmed().to_string()
                    };

                    rows.push(StatusRow {
                        path: repo.path.clone(),
                        branch: status.branch.name,
                        status: status_str,
                        sync: sync_str,
                    });
                }
                Err(e) => {
                    rows.push(StatusRow {
                        path: repo.path.clone(),
                        branch: "-".to_string(),
                        status: format!("Error: {}", e).red().to_string(),
                        sync: "-".to_string(),
                    });
                }
            }
        }

        let table = Table::new(rows).with(Style::rounded()).to_string();
        println!("{}", table);
    }

    Ok(())
}

fn print_detailed_status(git: &GitClient, repo: &Repository) -> Result<()> {
    let local_path = Path::new(&repo.path);

    println!("{}", repo.path.bold());

    if !local_path.exists() {
        println!("  {}: Not cloned", "Status".cyan());
        println!();
        return Ok(());
    }

    if !git.is_git_repository(local_path) {
        println!("  {}: Not a git repository", "Status".cyan());
        println!();
        return Ok(());
    }

    match git.status(local_path) {
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

                // Show file changes
                for file in &status.files {
                    let icon = match (&file.index_status, &file.worktree_status) {
                        (Some(_), None) => "S".green(), // Staged
                        (None, Some(crate::git::status::FileStatusCode::Untracked)) => "?".yellow(),
                        (_, Some(_)) => "M".yellow(), // Modified
                        _ => " ".normal(),
                    };
                    println!("    {} {}", icon, file.path);
                }
            }
        }
        Err(e) => {
            println!("  {}: {}", "Error".red(), e);
        }
    }

    println!();
    Ok(())
}
