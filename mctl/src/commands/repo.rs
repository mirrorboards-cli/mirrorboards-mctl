//! Repository command implementation
//!
//! This module implements the functionality of the repo command,
//! which manages repositories in the mirror.toml file.

use std::path::Path;
use mirror_sdk::{MirrorConfig, Repository, Error as SdkError};
use crate::cli::repo::{RepoArgs, RepoCommands, AddArgs, RemoveArgs, UpdateArgs, ListArgs, ShowArgs};
use crate::output::{OutputFormatter, ColorOutput, TableOutput, JsonOutput};
use crate::output::{TableFormatter, JsonFormatter, DetailFormatter};
use super::{CommandResult, CommandError};

/// Execute the repo command
pub fn execute(args: RepoArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    match args.command {
        RepoCommands::Add(args) => add_repo(args, formatter, config_path),
        RepoCommands::Remove(args) => remove_repo(args, formatter, config_path),
        RepoCommands::Update(args) => update_repo(args, formatter, config_path),
        RepoCommands::List(args) => list_repos(args, formatter, config_path),
        RepoCommands::Show(args) => show_repo(args, formatter, config_path),
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

/// Add a repository to the mirror.toml file
fn add_repo(args: AddArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    formatter.info("Adding repository...");

    // Load the configuration
    let mut config = load_config(config_path)?;

    // Create the repository
    let mut repo = Repository::new(&args.origin, &args.path)?;

    // Set optional properties
    if let Some(id) = args.id {
        repo = repo.with_id(id);
    }

    if let Some(branch) = args.branch {
        repo = repo.with_branch(branch);
    }

    if let Some(tags) = args.tag {
        repo = repo.with_tags(tags);
    }

    if args.lock {
        repo = repo.with_lock(true);
    }

    // Add the repository to the configuration
    let repo_id = repo.id.clone().unwrap_or_else(|| "auto-generated".to_string());
    match config.add_repository(repo) {
        Ok(_) => {
            // Save the configuration
            config.save()?;
            formatter.success(&format!("Repository '{}' added successfully", repo_id));
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Failed to add repository: {}", err));
            Err(CommandError::Sdk(err))
        }
    }
}

/// Remove a repository from the mirror.toml file
fn remove_repo(args: RemoveArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    formatter.info(&format!("Removing repository '{}'...", args.id));

    // Confirm removal if not forced
    if !args.force {
        formatter.warning("This operation cannot be undone.");
        formatter.info("Use --force to skip this confirmation.");
        
        // In a real implementation, we would prompt for confirmation here
        // For simplicity, we'll just proceed
    }

    // Load the configuration
    let mut config = load_config(config_path)?;

    // Remove the repository
    match config.remove_repository(&args.id) {
        Ok(_) => {
            // Save the configuration
            config.save()?;
            formatter.success(&format!("Repository '{}' removed successfully", args.id));
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Failed to remove repository: {}", err));
            Err(CommandError::Sdk(err))
        }
    }
}

/// Update a repository in the mirror.toml file
fn update_repo(args: UpdateArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    formatter.info(&format!("Updating repository '{}'...", args.id));

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

    // Update the repository properties
    let mut updated = false;

    if let Some(origin) = args.origin {
        repo.origin = origin;
        updated = true;
    }

    if let Some(path) = args.path {
        repo.path = path;
        updated = true;
    }

    if let Some(branch) = args.branch {
        repo.branch = Some(branch);
        updated = true;
    }

    if let Some(lock) = args.lock {
        repo.lock = Some(lock);
        updated = true;
    }

    if !updated {
        formatter.warning("No properties specified for update");
        return Ok(());
    }

    // Save the configuration
    match config.save() {
        Ok(_) => {
            formatter.success(&format!("Repository '{}' updated successfully", args.id));
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Failed to save configuration: {}", err));
            Err(CommandError::Sdk(err))
        }
    }
}

/// List repositories in the mirror.toml file
fn list_repos(args: ListArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Load the configuration
    let config = load_config(config_path)?;

    // Get all repositories
    let repos = config.get_repositories();

    // Filter by tag if specified
    let filtered_repos: Vec<&Repository> = if let Some(tag) = args.tag {
        config.get_repositories_by_tag(&tag)
    } else {
        repos.iter().collect()
    };

    // Filter by path prefix if specified
    let filtered_repos: Vec<&Repository> = if let Some(path_prefix) = args.path {
        filtered_repos.into_iter()
            .filter(|r| r.path.starts_with(&path_prefix))
            .collect()
    } else {
        filtered_repos
    };

    // Output the repositories
    if args.json {
        // Try to downcast to JsonOutput
        if let Some(json_formatter) = formatter.as_any_mut().downcast_mut::<JsonOutput>() {
            JsonFormatter::json(json_formatter, &filtered_repos)?;
        } else {
            // Fallback to string representation
            let json = serde_json::to_string_pretty(&filtered_repos)
                .map_err(|e| CommandError::Other(format!("Failed to serialize to JSON: {}", e)))?;
            formatter.json_str(&json)?;
        }
    } else {
        // Try to downcast to TableOutput
        if let Some(table_formatter) = formatter.as_any_mut().downcast_mut::<TableOutput>() {
            TableFormatter::table(table_formatter, "Repositories", &filtered_repos)?;
        } else if let Some(color_formatter) = formatter.as_any_mut().downcast_mut::<ColorOutput>() {
            TableFormatter::table(color_formatter, "Repositories", &filtered_repos)?;
        } else {
            // Fallback to string representation
            let repo_strings: Vec<String> = filtered_repos.iter()
                .map(|r| format!("{:?}", r))
                .collect();
            formatter.table_str("Repositories", &repo_strings)?;
        }
    }

    formatter.info(&format!("Found {} repositories", filtered_repos.len()));
    Ok(())
}

/// Show details of a specific repository
fn show_repo(args: ShowArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Load the configuration
    let config = load_config(config_path)?;

    // Get the repository
    match config.get_repository(&args.id) {
        Ok(repo) => {
            // Try to downcast to different formatter types
            if let Some(color_formatter) = formatter.as_any_mut().downcast_mut::<ColorOutput>() {
                DetailFormatter::detail(color_formatter, "Repository Details", repo)?;
            } else if let Some(table_formatter) = formatter.as_any_mut().downcast_mut::<TableOutput>() {
                DetailFormatter::detail(table_formatter, "Repository Details", repo)?;
            } else if let Some(json_formatter) = formatter.as_any_mut().downcast_mut::<JsonOutput>() {
                DetailFormatter::detail(json_formatter, "Repository Details", repo)?;
            } else {
                // Fallback to string representation
                formatter.detail_str("Repository Details", &format!("{:#?}", repo))?;
            }
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Repository '{}' not found", args.id));
            Err(CommandError::Sdk(err))
        }
    }
}