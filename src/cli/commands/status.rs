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

pub fn execute(config_path: &str, workspace: Option<String>, detailed: bool, all: bool) -> Result<()> {
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
        // Collect statuses concurrently (use status_fast to handle repos without commits)
        let statuses: Vec<_> = repos
            .par_iter()
            .map(|repo| {
                let local_path = repo.resolve_local_path(config_file);
                if !local_path.exists() || !local_path.join(".git").exists() {
                    return (repo, None);
                }
                let git = GitClient::new();
                (repo, git.status_fast(&local_path).ok())
            })
            .collect();

        // Filter to dirty repos only (unless --all)
        let dirty_repos: Vec<_> = if all {
            statuses
        } else {
            statuses
                .into_iter()
                .filter(|(_, status)| {
                    status.as_ref().map(|s| !s.is_fully_synced()).unwrap_or(true)
                })
                .collect()
        };

        if dirty_repos.is_empty() {
            println!("{}", "All repositories are synced".green());
            return Ok(());
        }

        // Print header for detailed view
        let header = if let Some(ws) = &workspace {
            if all {
                format!("Status for workspace: {} ({} repositories)", ws.cyan(), dirty_repos.len())
            } else {
                format!("Dirty repositories in {}: {}", ws.cyan(), dirty_repos.len())
            }
        } else if all {
            format!("All repositories ({})", dirty_repos.len())
        } else {
            format!("Dirty repositories ({})", dirty_repos.len())
        };
        println!("{}", header.bold());
        println!();

        // Detailed view - show each repo
        for (repo, status) in dirty_repos {
            print_detailed_status_cached(config_file, repo, status.as_ref())?;
        }
    } else {
        // Collect statuses concurrently
        let statuses: Vec<_> = repos
            .par_iter()
            .map(|repo| {
                let local_path = repo.resolve_local_path(config_file);
                if !local_path.exists() {
                    return (repo, None, Some("Not cloned"));
                }
                if !local_path.join(".git").exists() {
                    return (repo, None, Some("Not a git repo"));
                }
                let git = GitClient::new();
                match git.status_fast(&local_path) {
                    Ok(status) => (repo, Some(status), None),
                    Err(_) => (repo, None, Some("Error")),
                }
            })
            .collect();

        // Filter to dirty repos only (unless --all)
        let filtered: Vec<_> = if all {
            statuses
        } else {
            statuses
                .into_iter()
                .filter(|(_, status, error)| {
                    error.is_some() || status.as_ref().map(|s| !s.is_fully_synced()).unwrap_or(true)
                })
                .collect()
        };

        if filtered.is_empty() {
            println!("{}", "All repositories are synced".green());
            return Ok(());
        }

        // Build title
        let title = if let Some(ws) = &workspace {
            if all {
                format!(" Status: {} ({} repositories) ", ws, filtered.len())
            } else {
                format!(" Dirty: {} ({}) ", ws, filtered.len())
            }
        } else if all {
            format!(" Repository Status ({}) ", filtered.len())
        } else {
            format!(" Dirty Repositories ({}) ", filtered.len())
        };

        // Build table config
        let table_config = TableConfig::new(vec!["Path", "Branch", "Status", "Files"])
            .with_title(title)
            .with_widths(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(12),
                Constraint::Percentage(13),
                Constraint::Percentage(50),
            ]);

        // Build rows from filtered statuses
        let rows: Vec<TableRow> = filtered
            .iter()
            .map(|(repo, status, error)| {
                if let Some(err) = error {
                    return TableRow::new(vec![
                        CellStyle::highlight(&repo.path),
                        CellStyle::dimmed("-"),
                        CellStyle::warning(*err),
                        CellStyle::dimmed("-"),
                    ]);
                }

                let status = status.as_ref().unwrap();
                let status_cell = if status.is_fully_synced() {
                    CellStyle::success("Clean")
                } else if status.is_clean() && status.has_unpushed_commits() {
                    CellStyle::warning(format!("↑{} unpushed", status.branch.ahead))
                } else {
                    CellStyle::warning(status.summary())
                };

                // Build files list
                const MAX_FILES: usize = 10;
                let files_cell = if status.files.is_empty() {
                    CellStyle::dimmed("-")
                } else {
                    let file_names: Vec<_> = status.files
                        .iter()
                        .take(MAX_FILES)
                        .map(|f| {
                            let prefix = match (&f.index_status, &f.worktree_status) {
                                (Some(_), None) => "+",
                                (None, Some(crate::git::status::FileStatusCode::Untracked)) => "?",
                                (Some(_), Some(_)) => "*",
                                (_, Some(crate::git::status::FileStatusCode::Deleted)) => "-",
                                _ => "~",
                            };
                            format!("{}{}", prefix, f.path)
                        })
                        .collect();

                    let mut files_str = file_names.join("\n");
                    if status.files.len() > MAX_FILES {
                        files_str.push_str(&format!("\n(+{} more)", status.files.len() - MAX_FILES));
                    }
                    CellStyle::dimmed(files_str)
                };

                TableRow::new(vec![
                    CellStyle::highlight(&repo.path),
                    CellStyle::normal(&status.branch.name),
                    status_cell,
                    files_cell,
                ])
            })
            .collect();

        // Render table
        if let Err(e) = render_table(&table_config, &rows) {
            eprintln!("Error rendering table: {}", e);
        }
    }

    Ok(())
}

use crate::git::status::RepositoryStatus;

fn print_detailed_status_cached(
    config_file: &Path,
    repo: &Repository,
    status: Option<&RepositoryStatus>,
) -> Result<()> {
    let local_path = repo.resolve_local_path(config_file);

    println!("{}", repo.path.bold());

    if !local_path.exists() {
        println!("  {}: Not cloned", "Status".cyan());
        println!();
        return Ok(());
    }

    if !local_path.join(".git").exists() {
        println!("  {}: Not a git repository", "Status".cyan());
        println!();
        return Ok(());
    }

    match status {
        Some(status) => {
            println!("  {}: {}", "Branch".cyan(), status.branch.name);
            if !status.head_short.is_empty() {
                println!("  {}: {}", "HEAD".cyan(), status.head_short);
            }

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
        None => {
            println!("  {}: {}", "Error".red(), "Failed to get status");
        }
    }

    println!();
    Ok(())
}
