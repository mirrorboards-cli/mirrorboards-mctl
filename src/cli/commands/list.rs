//! List command - list repositories in the configuration.

use crate::cli::commands::print_error;
use crate::cli::table::{render_table, CellStyle, TableConfig, TableRow};
use crate::core::config::MirrorConfig;
use crate::core::repository::{Repository, VersionSpec};
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use ratatui::layout::Constraint;
use serde_json;
use std::path::Path;

fn repo_to_row(repo: &Repository, git: &GitClient) -> TableRow {
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
    let flags_str = if flags.is_empty() {
        "-".to_string()
    } else {
        flags.join(", ")
    };

    TableRow::new(vec![
        CellStyle::highlight(&repo.path),
        CellStyle::normal(truncate_git_url(&repo.git, 40)),
        CellStyle::normal(&version),
        CellStyle::dimmed(&workspaces),
        CellStyle::dimmed(&flags_str),
    ])
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
            // Build title
            let title = if let Some(ws) = &workspace {
                format!(" Workspace: {} ({} repositories) ", ws, repos.len())
            } else {
                format!(" Repositories ({}) ", repos.len())
            };

            // Build table config
            let table_config = TableConfig::new(vec!["Path", "Git", "Version", "Workspaces", "Flags"])
                .with_title(title)
                .with_widths(vec![
                    Constraint::Percentage(25),
                    Constraint::Percentage(35),
                    Constraint::Percentage(15),
                    Constraint::Percentage(15),
                    Constraint::Percentage(10),
                ]);

            // Build rows
            let rows: Vec<TableRow> = repos.iter().map(|r| repo_to_row(r, &git)).collect();

            // Render table
            if let Err(e) = render_table(&table_config, &rows) {
                eprintln!("Error rendering table: {}", e);
            }

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
