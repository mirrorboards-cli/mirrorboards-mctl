//! Repository command definition
//!
//! This module defines the CLI interface for the repo command,
//! which manages repositories in the mirror.toml file.

use clap::{Args, Subcommand};

/// Arguments for the repo command
#[derive(Debug, Args, Clone)]
pub struct RepoArgs {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: RepoCommands,
}

/// Subcommands for the repo command
#[derive(Debug, Subcommand, Clone)]
pub enum RepoCommands {
    /// Add a new repository to the mirror.toml file
    Add(AddArgs),

    /// Remove a repository from the mirror.toml file
    Remove(RemoveArgs),

    /// Update an existing repository's properties
    Update(UpdateArgs),

    /// List all repositories in the mirror.toml file
    List(ListArgs),

    /// Show details of a specific repository
    Show(ShowArgs),
}

/// Arguments for the repo add command
#[derive(Debug, Args, Clone)]
pub struct AddArgs {
    /// Git repository URL
    pub origin: String,

    /// Local path where the repository should be cloned
    pub path: String,

    /// Specify a custom ID for the repository
    #[arg(short, long)]
    pub id: Option<String>,

    /// Specify the branch to use
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Add tags to the repository (can be specified multiple times)
    #[arg(short, long, num_args = 1..)]
    pub tag: Option<Vec<String>>,

    /// Lock the repository
    #[arg(short, long)]
    pub lock: bool,
}

/// Arguments for the repo remove command
#[derive(Debug, Args, Clone)]
pub struct RemoveArgs {
    /// ID of the repository to remove
    pub id: String,

    /// Force removal without confirmation
    #[arg(long)]
    pub force: bool,
}

/// Arguments for the repo update command
#[derive(Debug, Args, Clone)]
pub struct UpdateArgs {
    /// ID of the repository to update
    pub id: String,

    /// Update the Git repository URL
    #[arg(short, long)]
    pub origin: Option<String>,

    /// Update the local path
    #[arg(short, long)]
    pub path: Option<String>,

    /// Update the branch
    #[arg(short, long)]
    pub branch: Option<String>,

    /// Update the lock status
    #[arg(short, long)]
    pub lock: Option<bool>,
}

/// Arguments for the repo list command
#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Filter repositories by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,

    /// Filter repositories by path prefix
    #[arg(long)]
    pub path: Option<String>,
}

/// Arguments for the repo show command
#[derive(Debug, Args, Clone)]
pub struct ShowArgs {
    /// ID of the repository to show
    pub id: String,
}