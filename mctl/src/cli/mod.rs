//! CLI definition and parsing module
//!
//! This module defines the command-line interface for the mctl tool,
//! including all commands, subcommands, arguments, and options.

pub mod init;
pub mod repo;
pub mod tag;
pub mod config;
pub mod sync;
pub mod status;
pub mod diff;
pub mod save;

pub use init::*;
pub use repo::*;
pub use tag::*;
pub use config::*;
pub use sync::*;
pub use status::*;
pub use diff::*;
pub use save::*;

use clap::{Parser, Subcommand};

/// Mirror Control (mctl) - CLI tool for managing mirror.toml files
#[derive(Debug, Parser, Clone)]
#[command(name = "mctl")]
#[command(about = "Mirror Control CLI tool for managing mirror.toml files", long_about = None)]
#[command(version)]
#[command(propagate_version = true)]
pub struct Cli {
    /// Path to the mirror.toml file
    #[arg(short, long, global = true)]
    pub config: Option<String>,

    /// Enable verbose output
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Enable quiet mode (minimal output)
    #[arg(short, long, global = true)]
    pub quiet: bool,

    /// Control when to use colored output (always, auto, never)
    #[arg(long, global = true, default_value = "auto")]
    pub color: String,

    /// Subcommand to execute
    #[command(subcommand)]
    pub command: Commands,
}

/// Subcommands for mctl
#[derive(Debug, Subcommand, Clone)]
pub enum Commands {
    /// Initialize a new mirror.toml file
    Init(init::InitArgs),

    /// Manage repositories
    Repo(repo::RepoArgs),

    /// Manage repository tags
    Tag(tag::TagArgs),

    /// Manage configuration settings
    Config(config::ConfigArgs),

    /// Sync repositories defined in mirror.toml
    Sync(sync::SyncArgs),

    /// Show git status of repositories defined in mirror.toml
    Status(status::StatusArgs),

    /// Show git diffs across repositories defined in mirror.toml
    Diff(diff::DiffArgs),

    /// Commit and push changes in repositories defined in mirror.toml
    Save(save::SaveArgs),
}