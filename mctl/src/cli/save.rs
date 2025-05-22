//! Save command definition
//!
//! This module defines the CLI interface for the save command,
//! which commits and pushes all changes in repositories defined in a mirror.toml file.

use clap::Args;

/// Arguments for the save command
#[derive(Debug, Args, Clone)]
pub struct SaveArgs {
    /// Custom commit message (default: "${repo.org}/${repo.name} - ${timestamp}")
    #[arg(short, long)]
    pub message: Option<String>,

    /// Filter repositories by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// SSH key path for git authentication
    #[arg(long)]
    pub ssh_key: Option<String>,

    /// Skip SSH authentication
    #[arg(long)]
    pub no_auth: bool,

    /// Commit only, skip pushing to remote
    #[arg(long)]
    pub no_push: bool,
}