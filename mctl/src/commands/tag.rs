//! Tag command implementation
//!
//! This module implements the functionality of the tag command,
//! which manages repository tags in the mirror.toml file.

use std::path::Path;
use std::collections::HashSet;
use mirror_sdk::{MirrorConfig, Error as SdkError};
use crate::cli::tag::{TagArgs, TagCommands, AddArgs, RemoveArgs, ListArgs};
use crate::output::{OutputFormatter, ColorOutput, TableOutput, JsonOutput};
use crate::output::{JsonFormatter, ListFormatter};
use super::{CommandResult, CommandError};

/// Execute the tag command
pub fn execute(args: TagArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    match args.command {
        TagCommands::Add(args) => add_tags(args, formatter, config_path),
        TagCommands::Remove(args) => remove_tags(args, formatter, config_path),
        TagCommands::List(args) => list_tags(args, formatter, config_path),
    }
}

/// Load the mirror configuration from the specified path or default
fn load_config(config_path: Option<String>) -> Result<MirrorConfig, SdkError> {
    if let Some(path) = config_path {
        MirrorConfig::load_from(Path::new(&path))
    } else {
        MirrorConfig::load()
    }
}

/// Add tags to a repository
fn add_tags(args: AddArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    formatter.info(&format!("Adding tags to repository '{}'...", args.id));

    // Load the configuration
    let mut config = load_config(config_path)?;

    // Get the repository
    let repo = match config.get_repository_mut(&args.id) {
        Ok(repo) => repo,
        Err(err) => {
            formatter.error(&format!("Repository '{}' not found", args.id));
            return Err(CommandError::Sdk(err));
        }
    };

    // Add the tags
    let mut current_tags = repo.tags.clone().unwrap_or_else(|| Vec::new());
    let mut added_count = 0;

    for tag in args.tags {
        if !current_tags.contains(&tag) {
            current_tags.push(tag.clone());
            added_count += 1;
            formatter.info(&format!("Added tag '{}'", tag));
        } else {
            formatter.warning(&format!("Tag '{}' already exists", tag));
        }
    }

    // Update the repository
    repo.tags = Some(current_tags);

    // Save the configuration
    match config.save() {
        Ok(_) => {
            formatter.success(&format!("Added {} tags to repository '{}'", added_count, args.id));
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Failed to save configuration: {}", err));
            Err(CommandError::Sdk(err))
        }
    }
}

/// Remove tags from a repository
fn remove_tags(args: RemoveArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    formatter.info(&format!("Removing tags from repository '{}'...", args.id));

    // Load the configuration
    let mut config = load_config(config_path)?;

    // Get the repository
    let repo = match config.get_repository_mut(&args.id) {
        Ok(repo) => repo,
        Err(err) => {
            formatter.error(&format!("Repository '{}' not found", args.id));
            return Err(CommandError::Sdk(err));
        }
    };

    // Remove the tags
    let mut current_tags = repo.tags.clone().unwrap_or_else(|| Vec::new());
    let mut removed_count = 0;

    for tag in args.tags {
        if let Some(pos) = current_tags.iter().position(|t| t == &tag) {
            current_tags.remove(pos);
            removed_count += 1;
            formatter.info(&format!("Removed tag '{}'", tag));
        } else {
            formatter.warning(&format!("Tag '{}' not found", tag));
        }
    }

    // Update the repository
    if current_tags.is_empty() {
        repo.tags = None;
    } else {
        repo.tags = Some(current_tags);
    }

    // Save the configuration
    match config.save() {
        Ok(_) => {
            formatter.success(&format!("Removed {} tags from repository '{}'", removed_count, args.id));
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Failed to save configuration: {}", err));
            Err(CommandError::Sdk(err))
        }
    }
}

/// List all tags used in the mirror.toml file
fn list_tags(args: ListArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    formatter.info("Listing all tags...");

    // Load the configuration
    let config = load_config(config_path)?;

    // Collect all unique tags
    let mut tags = HashSet::new();
    for repo in config.get_repositories() {
        if let Some(repo_tags) = &repo.tags {
            for tag in repo_tags {
                tags.insert(tag.clone());
            }
        }
    }

    // Convert to a sorted vector
    let mut tags_vec: Vec<String> = tags.into_iter().collect();
    tags_vec.sort();

    // Output the tags
    if args.json {
        // Try to downcast to JsonOutput
        if let Some(json_formatter) = formatter.as_any_mut().downcast_mut::<JsonOutput>() {
            JsonFormatter::json(json_formatter, &tags_vec)?;
        } else {
            // Fallback to string representation
            let json = serde_json::to_string_pretty(&tags_vec)
                .map_err(|e| CommandError::Other(format!("Failed to serialize to JSON: {}", e)))?;
            formatter.json_str(&json)?;
        }
    } else {
        // Try to downcast to different formatter types
        if let Some(color_formatter) = formatter.as_any_mut().downcast_mut::<ColorOutput>() {
            ListFormatter::list(color_formatter, "Tags", &tags_vec)?;
        } else if let Some(table_formatter) = formatter.as_any_mut().downcast_mut::<TableOutput>() {
            ListFormatter::list(table_formatter, "Tags", &tags_vec)?;
        } else {
            // Fallback to string representation
            formatter.list_str("Tags", &tags_vec)?;
        }
    }

    formatter.info(&format!("Found {} tags", tags_vec.len()));
    Ok(())
}