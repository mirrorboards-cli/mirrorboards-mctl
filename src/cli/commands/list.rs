//! List command - list repositories in the configuration.

use crate::cli::commands::print_error;
use crate::core::config::MirrorConfig;
use crate::core::repository::{Repository, VersionSpec};
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use serde_json;
use std::path::Path;
use tabled::{settings::Style, Table, Tabled};

#[derive(Tabled)]
struct RepoRow {
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Git")]
    git: String,
    #[tabled(rename = "Version")]
    version: String,
    #[tabled(rename = "Workspaces")]
    workspaces: String,
    #[tabled(rename = "Flags")]
    flags: String,
}

impl RepoRow {
    fn from_repo(repo: &Repository, git: &GitClient) -> Self {
        let version = match repo.version_spec() {
            VersionSpec::DefaultBranch => {
                // Try to get actual branch from cloned repo
                let local_path = Path::new(&repo.path);
                if local_path.exists() && git.is_git_repository(local_path) {
                    match git.get_current_branch(local_path) {
                        Ok(branch) => format!("branch:{}", branch),
                        Err(_) => "default".to_string(),
                    }
                } else {
                    "default".to_string()
                }
            }
            other => other.to_string(),
        };

        let workspaces = if repo.workspaces.is_empty() {
            "-".to_string()
        } else {
            repo.workspaces.join(", ")
        };

        let mut flags = Vec::new();
        if repo.skip_push {
            flags.push("skip-push");
        }
        let flags = if flags.is_empty() {
            "-".to_string()
        } else {
            flags.join(", ")
        };

        RepoRow {
            path: repo.path.clone(),
            git: truncate_git_url(&repo.git, 40),
            version,
            workspaces,
            flags,
        }
    }
}

fn truncate_git_url(url: &str, max_len: usize) -> String {
    if url.len() <= max_len {
        url.to_string()
    } else {
        format!("{}...", &url[..max_len - 3])
    }
}

pub fn execute(config_path: &str, workspace: Option<String>, format: &str) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

    // Filter by workspace if specified
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

    match format {
        "json" => {
            let json = serde_json::to_string_pretty(&repos)?;
            println!("{}", json);
        }
        "table" | _ => {
            // Print header
            if let Some(ws) = &workspace {
                println!(
                    "{} {} ({} repositories)",
                    "Workspace:".bold(),
                    ws.cyan(),
                    repos.len()
                );
            } else {
                println!(
                    "{} ({} repositories)",
                    "All repositories".bold(),
                    repos.len()
                );
            }
            println!();

            // Print table
            let rows: Vec<RepoRow> = repos.iter().map(|r| RepoRow::from_repo(r, &git)).collect();
            let table = Table::new(rows).with(Style::rounded()).to_string();
            println!("{}", table);

            // Print workspace summary
            let workspaces = config.list_workspaces();
            if !workspaces.is_empty() {
                println!();
                println!("{} {}", "Workspaces:".dimmed(), workspaces.join(", ").dimmed());
            }
        }
    }

    Ok(())
}
