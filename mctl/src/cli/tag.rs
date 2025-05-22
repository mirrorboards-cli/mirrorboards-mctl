//! Tag command definition
//!
//! This module defines the CLI interface for the tag command,
//! which manages repository tags in the mirror.toml file.

use clap::{Args, Subcommand};

/// Arguments for the tag command
#[derive(Debug, Args, Clone)]
pub struct TagArgs {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: TagCommands,
}

/// Subcommands for the tag command
#[derive(Debug, Subcommand, Clone)]
pub enum TagCommands {
    /// Add tags to a repository
    Add(AddArgs),

    /// Remove tags from a repository
    Remove(RemoveArgs),

    /// List all tags used in the mirror.toml file
    List(ListArgs),
}

/// Arguments for the tag add command
#[derive(Debug, Args, Clone)]
pub struct AddArgs {
    /// ID of the repository to add tags to
    pub id: String,

    /// Tags to add
    #[arg(required = true, num_args = 1..)]
    pub tags: Vec<String>,
}

/// Arguments for the tag remove command
#[derive(Debug, Args, Clone)]
pub struct RemoveArgs {
    /// ID of the repository to remove tags from
    pub id: String,

    /// Tags to remove
    #[arg(required = true, num_args = 1..)]
    pub tags: Vec<String>,
}

/// Arguments for the tag list command
#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,
}