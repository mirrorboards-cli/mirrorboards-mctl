//! Pin command - pin repositories to the commits their checkouts in a workspace have.

use crate::cli::commands::{print_info, print_success};
use crate::core::config::ConfigManager;
use crate::core::include::{IncludeResolver, RepositoryWithSource};
use crate::git::GitClient;
use anyhow::{bail, Context, Result};
use std::path::Path;

/// A repository whose pin changes or stays.
struct Pin<'a> {
    entry: &'a RepositoryWithSource,
    head: String,
}

/// Set `rev` of every repository the configuration pins to a commit (or only
/// of `paths`) to the HEAD of the checkout at the same path under `workspace`.
///
/// Used before a release whose `.github-images/mirror.toml` pins its sources,
/// so the release builds what was tested in the workspace. Every checkout is
/// fetched first and must be clean and level with its upstream: a pin never
/// names a commit that exists only locally or one the branch has moved past.
/// Nothing is written unless every checkout passes.
pub fn execute(config_path: &str, workspace: &str, paths: &[String]) -> Result<()> {
    let config_file = Path::new(config_path);
    let workspace = Path::new(workspace);

    if !config_file.exists() {
        bail!("Configuration file not found: {}", config_path);
    }
    if !workspace.is_dir() {
        bail!("Workspace not found: {}", workspace.display());
    }

    let resolved = IncludeResolver::resolve(config_file)?;
    let wanted: Vec<&str> = paths
        .iter()
        .map(|path| path.trim_end_matches('/'))
        .collect();

    for path in &wanted {
        let known = resolved
            .repositories
            .iter()
            .any(|entry| entry.repository.path == *path);
        if !known {
            bail!("Repository not found: {}", path);
        }
    }

    let pinned: Vec<&RepositoryWithSource> = resolved
        .repositories
        .iter()
        .filter(|entry| entry.repository.rev.is_some())
        .filter(|entry| wanted.is_empty() || wanted.contains(&entry.repository.path.as_str()))
        .collect();

    if pinned.is_empty() {
        print_info("No repository is pinned to a rev; nothing to do");
        return Ok(());
    }

    let git = GitClient::new();
    let mut pins = Vec::new();
    let mut problems = Vec::new();

    for entry in pinned {
        let checkout = workspace.join(&entry.repository.path);

        match checked_head(&git, &checkout) {
            Ok(head) => pins.push(Pin { entry, head }),
            Err(problem) => problems.push(format!("{}: {problem}", entry.repository.path)),
        }
    }

    if !problems.is_empty() {
        bail!(
            "Nothing pinned; these checkouts cannot give a commit:\n  {}",
            problems.join("\n  ")
        );
    }

    let mut changed = 0;
    for pin in &pins {
        let path = &pin.entry.repository.path;
        let previous = pin.entry.repository.rev.as_deref().unwrap_or("");
        if previous == pin.head {
            continue;
        }

        let mut manager = ConfigManager::open(&pin.entry.source_file)?;
        manager.set_repository_rev(path, &pin.head)?;
        manager.save()?;

        println!("  {}: {} -> {}", path, short(previous), short(&pin.head));
        changed += 1;
    }

    print_success(&format!(
        "Pinned {} of {} repositories to their checkouts in {} ({} unchanged)",
        changed,
        pins.len(),
        workspace.display(),
        pins.len() - changed
    ));

    Ok(())
}

/// HEAD of a checkout that is cloned, clean and level with its upstream
/// after a fetch; otherwise what is wrong with it.
fn checked_head(git: &GitClient, checkout: &Path) -> Result<String, String> {
    if !git.is_git_repository(checkout) {
        return Err(format!("no checkout at {}", checkout.display()));
    }

    git.fetch(checkout)
        .with_context(|| format!("fetching {}", checkout.display()))
        .map_err(|error| format!("{error:#}"))?;

    let status = git
        .status(checkout)
        .map_err(|error| format!("status: {error}"))?;

    if !status.is_clean() {
        return Err(format!("{} uncommitted changes", status.files.len()));
    }
    let Some(upstream) = &status.branch.upstream else {
        return Err(format!(
            "branch {} has no upstream, so its commit may exist only here",
            status.branch.name
        ));
    };
    if status.branch.ahead > 0 {
        return Err(format!(
            "{} commits not pushed to {upstream}",
            status.branch.ahead
        ));
    }
    if status.branch.behind > 0 {
        return Err(format!(
            "{} commits behind {upstream}; pull first",
            status.branch.behind
        ));
    }

    Ok(status.head_full)
}

fn short(rev: &str) -> &str {
    &rev[..rev.len().min(7)]
}
