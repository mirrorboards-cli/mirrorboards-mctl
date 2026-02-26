use crate::cli::commands::{print_error, print_success, print_warning};
use crate::core::config::MirrorConfig;
use crate::core::repository::Repository;
use crate::git::GitClient;
use anyhow::Result;
use colored::Colorize;
use rayon::prelude::*;
use std::path::Path;

pub fn execute(config_path: &str, workspace: Option<String>) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        print_error(&format!(
            "Configuration file not found: {}",
            config_path
        ));
        return Ok(());
    }

    let config = MirrorConfig::load(config_file)?;

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

    if let Some(ws) = &workspace {
        println!(
            "{} {} ({} repositories)",
            "Pulling workspace:".bold(),
            ws.cyan(),
            repos.len()
        );
    } else {
        println!(
            "{} ({} repositories)",
            "Pulling all repositories".bold(),
            repos.len()
        );
    }
    println!();

    let git = GitClient::new();

    if let Err(e) = git.check_git_available() {
        print_error(&format!("Git is not available: {}", e));
        return Ok(());
    }

    let results: Vec<(&Repository, PullResult)> = repos
        .par_iter()
        .map(|repo| {
            let local_path = Path::new(&repo.path);

            if !local_path.exists() || !git.is_git_repository(local_path) {
                return (*repo, PullResult::Skipped);
            }

            match git.pull(local_path) {
                Ok(_) => (*repo, PullResult::Success),
                Err(e) => (*repo, PullResult::Failed(e.to_string())),
            }
        })
        .collect();

    let mut success_count = 0;
    let mut skip_count = 0;
    let mut error_count = 0;

    for (repo, result) in &results {
        match result {
            PullResult::Success => {
                println!("{} {}", "✓".green(), repo.path);
                success_count += 1;
            }
            PullResult::Skipped => {
                println!("{} {} - not cloned", "→".blue(), repo.path);
                skip_count += 1;
            }
            PullResult::Failed(err) => {
                println!("{} {} - {}", "✗".red(), repo.path, err);
                error_count += 1;
            }
        }
    }

    println!();
    let mut summary_parts = Vec::new();
    if success_count > 0 {
        summary_parts.push(format!("{} pulled", success_count));
    }
    if skip_count > 0 {
        summary_parts.push(format!("{} skipped", skip_count));
    }
    if error_count > 0 {
        summary_parts.push(format!("{} failed", error_count));
    }

    if error_count > 0 {
        print_warning(&format!("Pull complete: {}", summary_parts.join(", ")));
    } else {
        print_success(&format!("Pull complete: {}", summary_parts.join(", ")));
    }

    Ok(())
}

enum PullResult {
    Success,
    Skipped,
    Failed(String),
}
