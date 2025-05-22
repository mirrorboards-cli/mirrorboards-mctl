//! Status command definition
//!
//! This module defines the CLI interface for the status command,
//! which shows the git status of all repositories defined in a mirror.toml file.

use clap::Args;

/// Arguments for the status command
#[derive(Debug, Args, Clone)]
pub struct StatusArgs {
    /// Filter repositories by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Show clean repositories (by default, clean repositories are hidden)
    #[arg(short, long)]
    pub show_clean: bool,
}