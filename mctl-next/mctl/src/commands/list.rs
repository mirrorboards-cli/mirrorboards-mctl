use anyhow::{Context, Result};
use colored::Colorize;
use tabled::{Table, Tabled};

use mirror_sdk::{ConfigLoader, ConfigValidator, GitManager, RefSpec, WorkspaceManager};

#[derive(Tabled)]
struct RepoRow {
    #[tabled(rename = "Path")]
    path: String,
    #[tabled(rename = "Git")]
    git: String,
    #[tabled(rename = "Ref")]
    ref_spec: String,
    #[tabled(rename = "Workspaces")]
    workspaces: String,
    #[tabled(rename = "Status")]
    status: String,
}

pub fn run(config_path: &str, workspace: Option<&str>, by_workspace: bool) -> Result<()> {
    // Load and validate config
    let config = ConfigLoader::new()
        .load(config_path)
        .context("Failed to load configuration")?;

    ConfigValidator::validate(&config).context("Configuration validation failed")?;

    // Get base path
    let base_path = std::path::Path::new(config_path)
        .parent()
        .unwrap_or(std::path::Path::new("."));

    let git_manager = GitManager::new(base_path);

    if by_workspace {
        print_by_workspace(&config, &git_manager)?;
    } else {
        print_flat(&config, &git_manager, workspace)?;
    }

    Ok(())
}

fn print_flat(
    config: &mirror_sdk::MirrorConfig,
    git_manager: &GitManager,
    workspace: Option<&str>,
) -> Result<()> {
    let repositories = config.get_repositories(workspace);

    if repositories.is_empty() {
        if let Some(ws) = workspace {
            println!(
                "{}",
                format!("No repositories found in workspace '{}'", ws).yellow()
            );
        } else {
            println!("{}", "No repositories configured".yellow());
        }
        return Ok(());
    }

    let rows: Vec<RepoRow> = repositories
        .iter()
        .map(|repo| {
            let ref_spec = match &repo.ref_spec {
                RefSpec::Branch(b) => format!("branch:{}", b),
                RefSpec::Tag(t) => format!("tag:{}", t),
                RefSpec::Rev(r) => format!("rev:{}", &r[..7.min(r.len())]),
            };

            let status = if git_manager.exists(repo) {
                match git_manager.has_changes(repo) {
                    Ok(true) => "modified".yellow().to_string(),
                    Ok(false) => "clean".green().to_string(),
                    Err(_) => "error".red().to_string(),
                }
            } else {
                "not cloned".dimmed().to_string()
            };

            RepoRow {
                path: repo.path.clone(),
                git: shorten_git_url(&repo.git),
                ref_spec,
                workspaces: repo.workspaces.join(", "),
                status,
            }
        })
        .collect();

    let table = Table::new(rows).to_string();
    println!("{}", table);

    Ok(())
}

fn print_by_workspace(
    config: &mirror_sdk::MirrorConfig,
    git_manager: &GitManager,
) -> Result<()> {
    let ws_manager = WorkspaceManager::new(config);
    let groups = ws_manager.group_by_workspace();

    let mut workspace_names: Vec<_> = groups.keys().collect();
    workspace_names.sort();

    for ws_name in workspace_names {
        let repos = groups.get(ws_name).unwrap();

        println!();
        println!("{}", format!("Workspace: {}", ws_name).cyan().bold());
        println!("{}", "─".repeat(50));

        for repo in repos {
            let ref_spec = match &repo.ref_spec {
                RefSpec::Branch(b) => format!("({})", b).dimmed().to_string(),
                RefSpec::Tag(t) => format!("(tag:{})", t).yellow().to_string(),
                RefSpec::Rev(r) => format!("({})", &r[..7.min(r.len())]).dimmed().to_string(),
            };

            let status = if git_manager.exists(repo) {
                match git_manager.has_changes(repo) {
                    Ok(true) => " [modified]".yellow().to_string(),
                    Ok(false) => "".to_string(),
                    Err(_) => " [error]".red().to_string(),
                }
            } else {
                " [not cloned]".dimmed().to_string()
            };

            println!("  {} {} {}", repo.path, ref_spec, status);
        }
    }

    println!();

    Ok(())
}

fn shorten_git_url(url: &str) -> String {
    // Convert git@github.com:org/repo.git to org/repo
    if let Some(rest) = url.strip_prefix("git@github.com:") {
        rest.strip_suffix(".git").unwrap_or(rest).to_string()
    } else if let Some(rest) = url.strip_prefix("https://github.com/") {
        rest.strip_suffix(".git").unwrap_or(rest).to_string()
    } else {
        url.to_string()
    }
}
