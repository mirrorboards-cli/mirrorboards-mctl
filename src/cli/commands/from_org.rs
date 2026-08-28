//! from-org command - generate a mirror.toml from a GitHub organization.
//!
//! Uses the GitHub CLI (`gh`) to enumerate the repositories of an organization
//! (or user) and emits a ready-to-use mirror.toml. Following the project's
//! philosophy of shelling out to external CLI tools, this reuses `gh`'s auth
//! and pagination instead of embedding an HTTP client.

use crate::cli::commands::{print_info, print_success};
use crate::core::config::RawMirrorConfig;
use crate::core::repository::Repository;
use anyhow::{anyhow, Context, Result};
use serde::Deserialize;
use std::process::Command;

/// A repository entry as returned by `gh repo list --json`.
#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GhRepo {
    name: String,
    ssh_url: String,
    /// HTML URL, e.g. https://github.com/org/repo (no .git suffix)
    url: String,
    #[serde(default)]
    is_archived: bool,
    #[serde(default)]
    is_fork: bool,
    default_branch_ref: Option<GhBranchRef>,
}

#[derive(Debug, Deserialize)]
struct GhBranchRef {
    name: String,
}

#[allow(clippy::too_many_arguments)]
pub fn execute(
    org: &str,
    output: Option<String>,
    workspace: Option<String>,
    limit: u32,
    https: bool,
    pin_branch: bool,
    include_archived: bool,
    include_forks: bool,
) -> Result<()> {
    // When emitting to stdout, keep diagnostics on stderr so the output stays
    // a clean, pipeable mirror.toml (e.g. `mctl from-org acme > mirror.toml`).
    let to_stdout = output.is_none();

    // Query GitHub via the gh CLI.
    let fields = "name,sshUrl,url,defaultBranchRef,isArchived,isFork";
    let gh_output = Command::new("gh")
        .args(["repo", "list", org, "--json", fields, "--limit"])
        .arg(limit.to_string())
        .output()
        .map_err(|e| {
            anyhow!(
                "Failed to run 'gh' (is the GitHub CLI installed and on PATH?): {}",
                e
            )
        })?;

    if !gh_output.status.success() {
        let stderr = String::from_utf8_lossy(&gh_output.stderr);
        return Err(anyhow!(
            "'gh repo list {}' failed: {}",
            org,
            stderr.trim()
        ));
    }

    let gh_repos: Vec<GhRepo> = serde_json::from_slice(&gh_output.stdout)
        .context("Failed to parse JSON output from 'gh repo list'")?;

    // Build repository entries, applying filters.
    let mut repositories: Vec<Repository> = Vec::new();
    let mut skipped = 0;

    for gh in &gh_repos {
        if gh.is_archived && !include_archived {
            skipped += 1;
            continue;
        }
        if gh.is_fork && !include_forks {
            skipped += 1;
            continue;
        }

        let git_url = if https {
            format!("{}.git", gh.url.trim_end_matches(".git"))
        } else {
            gh.ssh_url.clone()
        };

        let mut repo = Repository::new(git_url, gh.name.clone());

        if pin_branch {
            if let Some(branch) = &gh.default_branch_ref {
                repo = repo.with_branch(&branch.name);
            }
        }

        if let Some(ws) = &workspace {
            repo = repo.with_workspaces(vec![ws.clone()]);
        }

        repositories.push(repo);
    }

    repositories.sort_by(|a, b| a.path.cmp(&b.path));

    let count = repositories.len();

    let config = RawMirrorConfig {
        include: Vec::new(),
        includes: None,
        remote: None,
        repositories,
    };

    let toml_content = toml::to_string_pretty(&config)?;

    let mut header = format!("# Generated with: mctl from-org {}\n", org);
    if let Some(ws) = &workspace {
        header.push_str(&format!("# Workspace: {}\n", ws));
    }
    let final_content = format!("{}\n{}", header, toml_content);

    if let Some(path) = output {
        std::fs::write(&path, final_content)?;
        print_success(&format!(
            "Wrote {} repositories from '{}' to {}",
            count, org, path
        ));
        if skipped > 0 {
            print_info(&format!(
                "{} repositories skipped (archived/forks)",
                skipped
            ));
        }
    } else {
        // toml -> stdout, diagnostics -> stderr
        print!("{}", final_content);
        if to_stdout {
            eprintln!();
            eprintln!(
                "# {} repositories from '{}'{}",
                count,
                org,
                if skipped > 0 {
                    format!(", {} skipped (archived/forks)", skipped)
                } else {
                    String::new()
                }
            );
        }
    }

    Ok(())
}
