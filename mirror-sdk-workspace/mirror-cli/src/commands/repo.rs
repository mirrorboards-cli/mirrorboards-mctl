//! Repository management commands for the Mirror CLI.

use clap::{Args, Subcommand};
use colored::Colorize;
use mirror_sdk::Repository;
use std::path::PathBuf;

use crate::error::{CliError, CliResult};
use crate::utils::{load_or_create_config, print_info, print_success};

/// Repository management commands
#[derive(Args)]
pub struct RepoCommand {
    /// Path to the mirror.toml file (optional)
    #[arg(short, long, value_name = "FILE")]
    config: Option<PathBuf>,

    /// The subcommand to execute
    #[command(subcommand)]
    command: RepoSubcommand,
}

/// Subcommands for repository management
#[derive(Subcommand)]
enum RepoSubcommand {
    /// Add a repository to the configuration
    #[command(about = "Add a repository to the configuration")]
    Add(AddArgs),

    /// Remove a repository from the configuration
    #[command(about = "Remove a repository from the configuration")]
    Remove(RemoveArgs),

    /// List repositories in the configuration
    #[command(about = "List repositories in the configuration")]
    List(ListArgs),

    /// Update a repository in the configuration
    #[command(about = "Update a repository in the configuration")]
    Update(UpdateArgs),
}

/// Arguments for the add command
#[derive(Args)]
struct AddArgs {
    /// The Git repository origin URL
    #[arg(required = true)]
    origin: String,

    /// The local path where the repository should be cloned
    #[arg(required = true)]
    path: String,

    /// The branch to use (defaults to "main" if not specified)
    #[arg(short, long)]
    branch: Option<String>,

    /// Whether the repository is locked
    #[arg(short, long)]
    lock: bool,

    /// Tags associated with the repository
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,

    /// Custom ID for the repository (optional)
    #[arg(short, long)]
    id: Option<String>,
}

/// Arguments for the remove command
#[derive(Args)]
struct RemoveArgs {
    /// The ID of the repository to remove
    #[arg(short, long, conflicts_with = "path")]
    id: Option<String>,

    /// The path of the repository to remove
    #[arg(short, long, conflicts_with = "id")]
    path: Option<String>,
}

/// Arguments for the list command
#[derive(Args)]
struct ListArgs {
    /// Filter repositories by tag
    #[arg(short, long)]
    tag: Option<String>,

    /// Show detailed information
    #[arg(short, long)]
    detailed: bool,
}

/// Arguments for the update command
#[derive(Args)]
struct UpdateArgs {
    /// The ID of the repository to update
    #[arg(required = true)]
    id: String,

    /// The new Git repository origin URL
    #[arg(short, long)]
    origin: Option<String>,

    /// The new local path where the repository should be cloned
    #[arg(short, long)]
    path: Option<String>,

    /// The new branch to use
    #[arg(short, long)]
    branch: Option<String>,

    /// Whether the repository is locked
    #[arg(short, long)]
    lock: Option<bool>,

    /// Tags associated with the repository
    #[arg(short, long, value_delimiter = ',')]
    tags: Option<Vec<String>>,
}

impl RepoCommand {
    /// Execute the repository command
    pub fn execute(&self) -> CliResult<()> {
        match &self.command {
            RepoSubcommand::Add(args) => self.add(args),
            RepoSubcommand::Remove(args) => self.remove(args),
            RepoSubcommand::List(args) => self.list(args),
            RepoSubcommand::Update(args) => self.update(args),
        }
    }

    /// Add a repository to the configuration
    fn add(&self, args: &AddArgs) -> CliResult<()> {
        // Load or create the configuration
        let mut config = load_or_create_config(self.config.as_deref(), true)?;

        // Build the repository
        let mut builder = Repository::new()
            .with_origin(&args.origin)
            .with_path(&args.path);

        // Add optional fields
        if let Some(branch) = &args.branch {
            builder = builder.with_branch(branch);
        }

        if args.lock {
            builder = builder.with_lock(true);
        }

        if let Some(tags) = &args.tags {
            builder = builder.with_tags(tags.iter().map(|s| s.as_str()));
        }

        if let Some(id) = &args.id {
            builder = builder.with_id(id);
        }

        // Build the repository
        let repo = builder.build()?;

        // Add the repository to the configuration
        config.add_repository(repo)?;

        // Save the configuration
        config.save()?;

        print_success(&format!("Repository added to {}", config.get_config_path().unwrap().display()));
        Ok(())
    }

    /// Remove a repository from the configuration
    fn remove(&self, args: &RemoveArgs) -> CliResult<()> {
        // Load the configuration
        let mut config = load_or_create_config(self.config.as_deref(), false)?;

        // Remove the repository
        if let Some(id) = &args.id {
            config.remove_repository_by_id(id)?;
            print_success(&format!("Repository with ID '{}' removed", id));
        } else if let Some(path) = &args.path {
            config.remove_repository_by_path(path)?;
            print_success(&format!("Repository with path '{}' removed", path));
        } else {
            return Err(CliError::MissingArgument("Either --id or --path must be specified".to_string()));
        }

        // Save the configuration
        config.save()?;

        Ok(())
    }

    /// List repositories in the configuration
    fn list(&self, args: &ListArgs) -> CliResult<()> {
        // Load the configuration
        let config = load_or_create_config(self.config.as_deref(), false)?;

        // Filter repositories by tag if specified
        let repositories = if let Some(tag) = &args.tag {
            config.get_repositories_by_tag(tag)
        } else {
            config.repositories.iter().collect()
        };

        // Print the repositories
        if repositories.is_empty() {
            print_info("No repositories found");
            return Ok(());
        }

        println!("{} repositories found:", repositories.len());
        for repo in repositories {
            if args.detailed {
                println!("ID: {}", repo.id.as_deref().unwrap_or("N/A").bright_green());
                println!("  Origin: {}", repo.origin);
                println!("  Path: {}", repo.path);
                println!("  Branch: {}", repo.get_branch());
                println!("  Locked: {}", repo.is_locked());
                if let Some(tags) = &repo.tags {
                    println!("  Tags: {}", tags.iter().map(|s| s.as_str()).collect::<Vec<_>>().join(", "));
                }
                println!();
            } else {
                println!("{} - {} ({})", 
                    repo.id.as_deref().unwrap_or("N/A").bright_green(),
                    repo.origin,
                    repo.path
                );
            }
        }

        Ok(())
    }

    /// Update a repository in the configuration
    fn update(&self, args: &UpdateArgs) -> CliResult<()> {
        // Load the configuration
        let mut config = load_or_create_config(self.config.as_deref(), false)?;

        // Get the existing repository
        let existing_repo = config.get_repository_by_id(&args.id)
            .ok_or_else(|| CliError::RepositoryNotFound(args.id.clone()))?;

        // Create a builder with the existing repository's values
        let mut builder = Repository::new()
            .with_id(&args.id)
            .with_origin(&existing_repo.origin)
            .with_path(&existing_repo.path);

        // Update the branch if specified
        if let Some(branch) = &args.branch {
            builder = builder.with_branch(branch);
        } else if let Some(branch) = &existing_repo.branch {
            builder = builder.with_branch(branch);
        }

        // Update the lock if specified
        if let Some(lock) = args.lock {
            builder = builder.with_lock(lock);
        } else if let Some(lock) = existing_repo.lock {
            builder = builder.with_lock(lock);
        }

        // Update the tags if specified
        if let Some(tags) = &args.tags {
            builder = builder.with_tags(tags.iter().map(|s| s.as_str()));
        } else if let Some(tags) = &existing_repo.tags {
            builder = builder.with_tags(tags.iter().map(|s| s.as_str()));
        }

        // Update the origin if specified
        if let Some(origin) = &args.origin {
            builder = builder.with_origin(origin);
        }

        // Update the path if specified
        if let Some(path) = &args.path {
            builder = builder.with_path(path);
        }

        // Build the updated repository
        let updated_repo = builder.build()?;

        // Update the repository in the configuration
        config.update_repository(&args.id, updated_repo)?;

        // Save the configuration
        config.save()?;

        print_success(&format!("Repository with ID '{}' updated", args.id));
        Ok(())
    }
}