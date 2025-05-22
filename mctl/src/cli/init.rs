//! Init command definition
//!
//! This module defines the CLI interface for the init command,
//! which initializes a new mirror.toml file.

use clap::Args;

/// Arguments for the init command
#[derive(Debug, Args, Clone)]
pub struct InitArgs {
    /// Path where the mirror.toml file should be created
    #[arg(short, long)]
    pub path: Option<String>,

    /// Overwrite existing mirror.toml file if it exists
    #[arg(short, long)]
    pub force: bool,
}