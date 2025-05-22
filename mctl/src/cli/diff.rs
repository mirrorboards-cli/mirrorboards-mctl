//! Diff command definition
//!
//! This module defines the CLI interface for the diff command,
//! which shows git diffs across repositories defined in a mirror.toml file.

use clap::Args;

/// Arguments for the diff command
#[derive(Debug, Args, Clone)]
pub struct DiffArgs {
    /// Filter repositories by tag
    #[arg(short, long)]
    pub tag: Option<String>,

    /// Filter repositories by ID (supports multiple IDs)
    #[arg(short, long, action = clap::ArgAction::Append)]
    pub id: Vec<String>,

    /// Show only repositories with changes (default: show all)
    #[arg(short = 'C', long)]
    pub changes_only: bool,

    /// Diff against specific commit/branch/tag (default: HEAD vs working tree)
    #[arg(long)]
    pub base: Option<String>,

    /// Diff target commit/branch/tag (default: working tree)
    #[arg(long)]
    pub target: Option<String>,

    /// Show staged changes only (equivalent to git diff --cached)
    #[arg(long)]
    pub staged: bool,

    /// Show working tree changes only (default when no --staged)
    #[arg(long)]
    pub working_tree: bool,

    /// Number of context lines (default: 3)
    #[arg(short = 'U', long, default_value = "3")]
    pub context: u32,

    /// Show statistics only (files changed, insertions, deletions)
    #[arg(long)]
    pub stat: bool,

    /// Show only file names that have changes
    #[arg(long)]
    pub name_only: bool,

    /// Disable colored output
    #[arg(long)]
    pub no_color: bool,

    /// Include file patterns (glob patterns)
    #[arg(long, action = clap::ArgAction::Append)]
    pub include: Vec<String>,

    /// Exclude file patterns (glob patterns)
    #[arg(long, action = clap::ArgAction::Append)]
    pub exclude: Vec<String>,

    /// Pass-through arguments to git diff (use with caution)
    #[arg(last = true)]
    pub git_args: Vec<String>,
}