//! Add command - add a repository to the configuration.

use crate::cli::commands::{print_info, print_success};
use crate::core::config::{ConfigManager, MirrorConfig};
use crate::core::repository::Repository;
use crate::core::url::suggest_path;
use anyhow::{bail, Result};
use std::path::Path;

/// What `mctl add` records and where.
pub struct AddOptions {
    pub git: String,
    pub path: Option<String>,
    pub branch: Option<String>,
    pub rev: Option<String>,
    pub tag: Option<String>,
    pub workspaces: Vec<String>,
    pub skip_push: bool,
    /// Include file, relative to the configuration, that receives the entry.
    pub include: Option<String>,
}

pub fn execute(config_path: &str, options: AddOptions) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        bail!(
            "Configuration file not found: {} (run 'mctl init' to create one)",
            config_path
        );
    }

    let local_path = match options.path {
        Some(path) => path,
        None => {
            let Some(suggested) = suggest_path(&options.git) else {
                bail!(
                    "Could not determine a local path from {}; pass --path",
                    options.git
                );
            };
            print_info(&format!("Using suggested path: {}", suggested));
            suggested
        }
    };

    let version_count = [
        options.branch.is_some(),
        options.rev.is_some(),
        options.tag.is_some(),
    ]
    .iter()
    .filter(|&&given| given)
    .count();
    if version_count > 1 {
        bail!("Only one of --branch, --rev, or --tag can be specified");
    }

    let mut repo = Repository::new(&options.git, &local_path);
    if let Some(ref branch) = options.branch {
        repo = repo.with_branch(branch);
    }
    if let Some(ref rev) = options.rev {
        repo = repo.with_rev(rev);
    }
    if let Some(ref tag) = options.tag {
        repo = repo.with_tag(tag);
    }
    if options.skip_push {
        repo = repo.with_skip_push(true);
    }
    if !options.workspaces.is_empty() {
        repo = repo.with_workspaces(options.workspaces.clone());
    }

    // The path must be free in the whole mirror, not only in the edited file.
    let mirror = MirrorConfig::load(config_file)?;
    if let Some(existing) = mirror.find_by_path(&local_path) {
        bail!(
            "Path {} is already mirrored from {}",
            local_path,
            existing.git
        );
    }

    // Repository paths are relative to the configuration, wherever the entry lives.
    let config_dir = config_file.parent().unwrap_or(Path::new("."));
    let entry_file = match options.include {
        Some(ref include) => config_dir.join(include),
        None => config_file.to_path_buf(),
    };

    let mut entry_manager = ConfigManager::open(&entry_file)?;
    entry_manager.add_repository(repo)?;
    entry_manager.save()?;

    print_success(&format!("Added repository: {}", local_path));
    println!("  Git: {}", options.git);
    println!("  Path: {}", local_path);
    println!("  File: {}", entry_file.display());

    if let Some(ref branch) = options.branch {
        println!("  Branch: {}", branch);
    }
    if let Some(ref rev) = options.rev {
        println!("  Rev: {}", rev);
    }
    if let Some(ref tag) = options.tag {
        println!("  Tag: {}", tag);
    }
    if options.skip_push {
        println!("  Skip-push: true");
    }
    if !options.workspaces.is_empty() {
        println!("  Workspaces: {}", options.workspaces.join(", "));
    }

    if let Some(ref include) = options.include {
        let mut config_manager = ConfigManager::open(config_file)?;
        let registered = config_manager.add_include(include)?;

        if registered {
            config_manager.save()?;
            print_success(&format!("Included {} in {}", include, config_path));
        }
    }

    println!();
    print_info(&format!(
        "Run 'mctl sync --path {}' to clone the repository",
        local_path
    ));

    Ok(())
}
