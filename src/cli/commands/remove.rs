//! Remove command - remove a repository from the configuration.

use crate::cli::commands::{print_info, print_success, print_warning};
use crate::core::config::ConfigManager;
use crate::core::include::{IncludeResolver, ResolvedConfig};
use anyhow::{bail, Result};
use std::path::Path;

pub fn execute(config_path: &str, repo_path: &str, delete_local: bool) -> Result<()> {
    let config_file = Path::new(config_path);

    if !config_file.exists() {
        bail!("Configuration file not found: {}", config_path);
    }

    // The entry may live in an included file; edit the file that defines it.
    let resolved = IncludeResolver::resolve(config_file)?;
    let Some(defined) = resolved
        .repositories
        .iter()
        .find(|entry| entry.repository.path == repo_path)
    else {
        bail!("Repository not found: {}", repo_path);
    };

    let mut manager = ConfigManager::open(&defined.source_file)?;
    let repo = manager.remove_repository(repo_path)?;

    print_success(&format!("Removed repository: {}", repo_path));
    println!("  Git: {}", repo.git);
    println!("  File: {}", defined.source_file.display());

    // An include file left with nothing in it goes, so the mirror does not
    // collect empty files; the configuration itself always stays.
    let is_include_file = defined.source_file != config_file.canonicalize()?;
    let is_left_empty = is_include_file && manager.is_empty();
    if is_left_empty {
        drop_include_file(&resolved, &defined.source_file, &manager)?;
    } else {
        manager.save()?;
    }

    if delete_local {
        let local_path = repo.resolve_local_path(config_file);

        if local_path.exists() {
            print_warning(&format!(
                "Deleting local directory: {}",
                local_path.display()
            ));
            std::fs::remove_dir_all(&local_path)?;
            print_success("Local directory deleted");

            remove_directories_left_empty(&local_path, config_file)?;
        } else {
            print_info("Local directory does not exist");
        }
    }

    Ok(())
}

/// Delete an include file left without entries, after removing it from the
/// configuration that lists it under `[includes].paths`. A file referred to
/// some other way is kept, emptied, since deleting it would break that
/// configuration.
fn drop_include_file(
    resolved: &ResolvedConfig,
    include_file: &Path,
    emptied: &ConfigManager,
) -> Result<()> {
    for source_file in &resolved.source_files {
        let mut listing = ConfigManager::open(source_file)?;
        let was_listed = listing.remove_include(include_file)?;
        if !was_listed {
            continue;
        }

        listing.save()?;
        std::fs::remove_file(include_file)?;

        print_success(&format!(
            "Removed the empty include file: {}",
            include_file.display()
        ));
        println!("  Unlisted from: {}", source_file.display());
        return Ok(());
    }

    emptied.save()?;
    Ok(())
}

/// Remove the directories above a deleted checkout that it leaves empty, up to
/// the directory of the configuration: `applications/org/` goes with the last
/// repository under it.
fn remove_directories_left_empty(local_path: &Path, config_file: &Path) -> Result<()> {
    let config_file = config_file.canonicalize()?;
    let Some(mirror_root) = config_file.parent() else {
        return Ok(());
    };

    let mut directory = local_path.parent().map(Path::to_path_buf);
    while let Some(current) = directory {
        let current = current.canonicalize()?;

        let is_inside_mirror = current.starts_with(mirror_root) && current != mirror_root;
        if !is_inside_mirror {
            break;
        }

        let is_empty = std::fs::read_dir(&current)?.next().is_none();
        if !is_empty {
            break;
        }

        std::fs::remove_dir(&current)?;
        print_info(&format!("Removed empty directory: {}", current.display()));

        directory = current.parent().map(Path::to_path_buf);
    }

    Ok(())
}
