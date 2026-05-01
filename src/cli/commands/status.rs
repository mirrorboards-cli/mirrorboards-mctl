//! Status command - show status of repositories.

use crate::cli::commands::print_error;
use crate::cli::table::{render_table, CellStyle, TableConfig, TableRow};
use crate::core::config::MirrorConfig;
use crate::core::repository::Repository;
use crate::git::status::RepositoryStatus;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use ratatui::layout::Constraint;
use rayon::prelude::*;
use std::path::Path;

pub fn execute(
    config_path: &str,
    workspace: Option<String>,
    detailed: bool,
    all: bool,
) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!("Configuration file not found: {}", config_path));
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
                    status
                        .as_ref()
                        .map(|s| !s.is_fully_synced())
                        .unwrap_or(true)
                })
                .collect()
        };

        if dirty_repos.is_empty() {
            println!("{}", "All repositories are synced".green());
            return Ok(());
        }

        let header = if let Some(ws) = &workspace {
            if all {
                format!(
                    "Status for workspace: {} ({} repositories)",
                    ws.cyan(),
                    dirty_repos.len()
                )
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

        for (repo, status) in dirty_repos {
            print_detailed_status_cached(config_file, repo, status.as_ref())?;
        }
    } else {
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

        let filtered: Vec<_> = if all {
            statuses
        } else {
            statuses
                .into_iter()
                .filter(|(_, status, error)| {
                    error.is_some()
                        || status
                            .as_ref()
                            .map(|s| !s.is_fully_synced())
                            .unwrap_or(true)
                })
                .collect()
        };

        if filtered.is_empty() {
            println!("{}", "All repositories are synced".green());
            return Ok(());
        }

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

        let table_config = TableConfig::new(vec!["Path", "Branch", "Sync", "Files"])
            .with_title(title)
            .with_widths(vec![
                Constraint::Percentage(25),
                Constraint::Percentage(12),
                Constraint::Percentage(13),
                Constraint::Percentage(50),
            ]);

        let rows: Vec<TableRow> = filtered
            .iter()
            .map(|(repo, status, error)| {
                let (path_cell, path_link) = path_cell(repo, config_file);

                if let Some(err) = error {
                    return TableRow::new(vec![
                        path_cell,
                        CellStyle::dimmed("-"),
                        CellStyle::warning(*err),
                        CellStyle::dimmed("-"),
                    ])
                    .with_hyperlinks(vec![path_link, None, None, None]);
                }

                let status = status.as_ref().unwrap();

                TableRow::new(vec![
                    path_cell,
                    CellStyle::normal(&status.branch.name),
                    sync_cell(status),
                    files_cell(status),
                ])
                .with_hyperlinks(vec![path_link, None, None, None])
            })
            .collect();

        if let Err(e) = render_table(&table_config, &rows) {
            eprintln!("Error rendering table: {}", e);
        }
    }

    Ok(())
}

fn path_cell(repo: &Repository, config_file: &Path) -> (CellStyle, Option<String>) {
    let local_path = repo.resolve_local_path(config_file);
    let hyperlink = vscode_uri_for_path(&local_path);
    (CellStyle::highlight(&repo.path), hyperlink)
}

fn vscode_uri_for_path(path: &Path) -> Option<String> {
    if !path.exists() {
        return None;
    }

    let absolute = path.canonicalize().unwrap_or_else(|_| path.to_path_buf());
    let mut encoded = absolute
        .to_string_lossy()
        .bytes()
        .flat_map(|byte| match byte {
            b'A'..=b'Z' | b'a'..=b'z' | b'0'..=b'9' | b'/' | b'-' | b'.' | b'_' | b'~' => {
                vec![byte as char].into_iter().collect::<Vec<_>>()
            }
            _ => format!("%{:02X}", byte).chars().collect::<Vec<_>>(),
        })
        .collect::<String>();

    if path.is_dir() && !encoded.ends_with('/') {
        encoded.push('/');
    }

    Some(format!("vscode://file/{}", encoded))
}

fn sync_cell(status: &RepositoryStatus) -> CellStyle {
    if status.branch.is_detached() {
        return CellStyle::warning("detached");
    }

    match (
        &status.branch.upstream,
        status.branch.ahead,
        status.branch.behind,
    ) {
        (None, _, _) => CellStyle::dimmed("local"),
        (Some(_), 0, 0) => CellStyle::success("Synced"),
        (Some(_), ahead, 0) => CellStyle::warning(format!("↑{}", ahead)),
        (Some(_), 0, behind) => CellStyle::warning(format!("↓{}", behind)),
        (Some(_), ahead, behind) => CellStyle::warning(format!("↑{} ↓{}", ahead, behind)),
    }
}

fn files_cell(status: &RepositoryStatus) -> CellStyle {
    const MAX_FILES: usize = 10;

    if status.files.is_empty() {
        return CellStyle::dimmed("-");
    }

    let file_names: Vec<_> = status
        .files
        .iter()
        .take(MAX_FILES)
        .map(|file| {
            let prefix = file_status_prefix(file);
            format!("{}{}", prefix, file.path)
        })
        .collect();

    let mut files = file_names.join("\n");
    if status.files.len() > MAX_FILES {
        files.push_str(&format!("\n(+{} more)", status.files.len() - MAX_FILES));
    }

    if status.has_conflicts() {
        CellStyle::error(files)
    } else {
        CellStyle::dimmed(files)
    }
}

fn file_status_prefix(file: &crate::git::status::FileStatus) -> &'static str {
    use crate::git::status::FileStatusCode;

    match (&file.index_status, &file.worktree_status) {
        (Some(FileStatusCode::Unmerged), _) | (_, Some(FileStatusCode::Unmerged)) => "!",
        (Some(_), Some(_)) => "*",
        (Some(_), None) => "+",
        (None, Some(FileStatusCode::Untracked)) => "?",
        (_, Some(FileStatusCode::Deleted)) => "-",
        _ => "~",
    }
}

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

                for file in &status.files {
                    let icon = match (&file.index_status, &file.worktree_status) {
                        (Some(_), None) => "S".green(),
                        (None, Some(crate::git::status::FileStatusCode::Untracked)) => "?".yellow(),
                        (_, Some(_)) => "M".yellow(),
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

#[cfg(test)]
mod tests {
    use super::{files_cell, sync_cell, vscode_uri_for_path};
    use crate::cli::table::CellStyle;
    use crate::git::status::{BranchInfo, FileStatus, FileStatusCode, RepositoryStatus};
    use std::fs;
    use tempfile::TempDir;

    #[test]
    fn files_cell_lists_changed_files_with_status_prefixes() {
        let status = RepositoryStatus {
            branch: BranchInfo {
                name: "main".into(),
                upstream: Some("origin/main".into()),
                ahead: 0,
                behind: 0,
            },
            files: vec![
                FileStatus {
                    path: "src/lib.rs".into(),
                    index_status: Some(FileStatusCode::Modified),
                    worktree_status: None,
                    original_path: None,
                },
                FileStatus {
                    path: "src/main.rs".into(),
                    index_status: None,
                    worktree_status: Some(FileStatusCode::Modified),
                    original_path: None,
                },
                FileStatus {
                    path: "README.md".into(),
                    index_status: None,
                    worktree_status: Some(FileStatusCode::Untracked),
                    original_path: None,
                },
            ],
            head_short: "abc1234".into(),
            head_full: "abc1234abc1234abc1234abc1234abc1234".into(),
        };

        match files_cell(&status) {
            CellStyle::Dimmed(text) => assert_eq!(text, "+src/lib.rs\n~src/main.rs\n?README.md"),
            other => panic!(
                "expected dimmed file list, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn files_cell_truncates_long_file_lists() {
        let files = (0..11)
            .map(|index| FileStatus {
                path: format!("file{index}.txt"),
                index_status: None,
                worktree_status: Some(FileStatusCode::Modified),
                original_path: None,
            })
            .collect();
        let status = RepositoryStatus {
            branch: BranchInfo {
                name: "main".into(),
                upstream: Some("origin/main".into()),
                ahead: 0,
                behind: 0,
            },
            files,
            head_short: String::new(),
            head_full: String::new(),
        };

        match files_cell(&status) {
            CellStyle::Dimmed(text) => {
                assert!(text.contains("~file9.txt"));
                assert!(!text.contains("~file10.txt"));
                assert!(text.ends_with("(+1 more)"));
            }
            other => panic!(
                "expected dimmed file list, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn sync_cell_reports_branch_drift() {
        let status = RepositoryStatus {
            branch: BranchInfo {
                name: "main".into(),
                upstream: Some("origin/main".into()),
                ahead: 2,
                behind: 3,
            },
            files: vec![],
            head_short: String::new(),
            head_full: String::new(),
        };

        match sync_cell(&status) {
            CellStyle::Warning(text) => assert_eq!(text, "↑2 ↓3"),
            other => panic!(
                "expected warning cell, got {:?}",
                std::mem::discriminant(&other)
            ),
        }
    }

    #[test]
    fn vscode_uri_encodes_spaces_for_existing_paths() {
        let temp_dir = TempDir::new().unwrap();
        let path = temp_dir.path().join("repo with spaces");
        fs::create_dir_all(&path).unwrap();

        let uri = vscode_uri_for_path(&path).unwrap();
        assert!(uri.starts_with("vscode://file//"));
        assert!(uri.contains("repo%20with%20spaces"));
        assert!(uri.ends_with('/'));
    }
}
