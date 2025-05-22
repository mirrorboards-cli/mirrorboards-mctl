//! Sync command definition
//!
//! This module defines the CLI interface for the sync command,
//! which clones all repositories defined in a mirror.toml file.

use clap::Args;

/// Arguments for the sync command
#[derive(Debug, Args, Clone)]
pub struct SyncArgs {
    /// Skip repositories that are already cloned
    #[arg(short, long)]
    pub skip_existing: bool,

    /// Filter repositories by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// SSH key path for git authentication
    #[arg(long)]
    pub ssh_key: Option<String>,

    /// Skip SSH authentication
    #[arg(long)]
    pub no_auth: bool,
}