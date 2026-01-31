//! Status command - show status of repositories.

use crate::cli::commands::print_error;
use crate::cli::table::{render_table, CellStyle, TableConfig, TableRow};
use crate::core::config::MirrorConfig;
use crate::core::repository::Repository;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use ratatui::layout::Constraint;
use rayon::prelude::*;
use std::path::Path;

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

    if detailed {
        let git = GitClient::new();

        // Print header for detailed view
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

        // Detailed view - show each repo separately
        for repo in repos {
            print_detailed_status(&git, repo)?;
        }
    } else {
        // Build title
        let title = if let Some(ws) = &workspace {
            format!(" Status: {} ({} repositories) ", ws, repos.len())
        } else {
            format!(" Repository Status ({}) ", repos.len())
        };

        // Build table config
        let table_config = TableConfig::new(vec!["Path", "Branch", "Status", "Sync"])
            .with_title(title)
            .with_widths(vec![
                Constraint::Percentage(35),
                Constraint::Percentage(20),
                Constraint::Percentage(25),
                Constraint::Percentage(20),
            ]);

        // Build rows concurrently
        let rows: Vec<TableRow> = repos
            .par_iter()
            .map(|repo| {
                let git = GitClient::new();
                let local_path = Path::new(&repo.path);

                if !local_path.exists() {
                    return TableRow::new(vec![
                        CellStyle::highlight(&repo.path),
                        CellStyle::dimmed("-"),
                        CellStyle::warning("Not cloned"),
                        CellStyle::dimmed("-"),
                    ]);
                }

                if !git.is_git_repository(local_path) {
                    return TableRow::new(vec![
                        CellStyle::highlight(&repo.path),
                        CellStyle::dimmed("-"),
                        CellStyle::error("Not a git repo"),
                        CellStyle::dimmed("-"),
                    ]);
                }

                match git.status(local_path) {
                    Ok(status) => {
                        let status_cell = if status.is_clean() {
                            CellStyle::success("Clean")
                        } else {
                            CellStyle::warning(status.summary())
                        };

                        let sync_cell = if status.branch.is_synced() {
                            CellStyle::success("Up to date")
                        } else if status.branch.upstream.is_some() {
                            CellStyle::warning(format!("+{} -{}", status.branch.ahead, status.branch.behind))
                        } else {
                            CellStyle::dimmed("No upstream")
                        };

                        TableRow::new(vec![
                            CellStyle::highlight(&repo.path),
                            CellStyle::normal(&status.branch.name),
                            status_cell,
                            sync_cell,
                        ])
                    }
                    Err(e) => TableRow::new(vec![
                        CellStyle::highlight(&repo.path),
                        CellStyle::dimmed("-"),
                        CellStyle::error(format!("Error: {}", e)),
                        CellStyle::dimmed("-"),
                    ]),
                }
            })
            .collect();

        // Render table
        if let Err(e) = render_table(&table_config, &rows) {
            eprintln!("Error rendering table: {}", e);
        }
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
