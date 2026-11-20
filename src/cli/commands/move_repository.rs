//! Move command - move a repository to a new path, in the configuration and on disk.

use crate::cli::commands::remove::{drop_include_file, remove_directories_left_empty};
use crate::cli::commands::{print_info, print_success};
use crate::core::config::ConfigManager;
use crate::core::include::IncludeResolver;
use anyhow::{bail, Context, Result};
use std::path::Path;

/// What `mctl move` changes.
pub struct MoveOptions {
    /// The repository's path now.
    pub path: String,
    /// The repository's new path.
    pub to: String,
    /// Include file, relative to the configuration, that receives the entry; the entry stays in
    /// its own file when none is given.
    pub include: Option<String>,
}

/// Move the repository at `options.path` to `options.to`: its checkout, with its history and its
/// uncommitted changes, and its entry, into `options.include` when given. The checkout moves first,
/// so a move that fails on disk leaves the configuration as it was.
pub fn execute(config_path: &str, options: MoveOptions) -> Result<()> {
    let config_file = Path::new(config_path);
    let from = options.path.trim_end_matches('/');
    let to = options.to.trim_end_matches('/');

    if !config_file.exists() {
        bail!("Configuration file not found: {}", config_path);
    }

    let resolved = IncludeResolver::resolve(config_file)?;
    let Some(defined) = resolved
        .repositories
        .iter()
        .find(|entry| entry.repository.path == from)
    else {
        bail!("Repository not found: {}", from);
    };
    let taken = resolved
        .repositories
        .iter()
        .find(|entry| entry.repository.path == to);
    if let Some(existing) = taken {
        bail!(
            "Path {} is already mirrored from {}",
            to,
            existing.repository.git
        );
    }

    let config_dir = config_file.parent().unwrap_or(Path::new("."));
    let target_file = match options.include {
        Some(ref include) => config_dir.join(include),
        None => defined.source_file.clone(),
    };
    let stays_in_its_file = target_file.exists()
        && target_file.canonicalize()? == defined.source_file.canonicalize()?;

    let mut moved = defined.repository.clone();
    moved.path = to.to_string();
    let old_checkout = defined.repository.resolve_local_path(config_file);
    let new_checkout = moved.resolve_local_path(config_file);

    if old_checkout.exists() {
        if new_checkout.exists() {
            bail!("{} already exists on disk", new_checkout.display());
        }
        if let Some(parent) = new_checkout.parent() {
            std::fs::create_dir_all(parent)?;
        }

        std::fs::rename(&old_checkout, &new_checkout).with_context(|| {
            format!(
                "Moving {} to {}",
                old_checkout.display(),
                new_checkout.display()
            )
        })?;
        print_success(&format!("Moved the checkout to {}", new_checkout.display()));

        remove_directories_left_empty(&old_checkout, config_file)?;
    } else {
        print_info("Not cloned; only the configuration changes");
    }

    if stays_in_its_file {
        let mut manager = ConfigManager::open(&defined.source_file)?;
        manager.set_repository_path(from, to)?;
        manager.save()?;
    } else {
        let mut source = ConfigManager::open(&defined.source_file)?;
        source.remove_repository(from)?;
        let is_include_file = defined.source_file != config_file.canonicalize()?;
        if is_include_file && source.is_empty() {
            drop_include_file(&resolved, &defined.source_file, &source)?;
        } else {
            source.save()?;
        }

        let mut target = ConfigManager::open(&target_file)?;
        target.add_repository(moved)?;
        target.save()?;

        if let Some(ref include) = options.include {
            let mut config_manager = ConfigManager::open(config_file)?;
            let registered = config_manager.add_include(include)?;
            if registered {
                config_manager.save()?;
                print_success(&format!("Included {} in {}", include, config_path));
            }
        }
    }

    print_success(&format!("Moved repository: {} -> {}", from, to));
    println!("  File: {}", target_file.display());

    Ok(())
}
