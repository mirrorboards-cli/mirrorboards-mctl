//! Reset command definition
//!
//! This module defines the CLI interface for the reset command,
//! which performs git reset operations across all managed repositories.

use clap::Args;

/// Arguments for the reset command
#[derive(Debug, Args, Clone)]
pub struct ResetArgs {
    /// Filter repositories by tag
    #[arg(short, long)]
    pub tag: Option<String>,
    
    /// Reset to specific commit (default: HEAD)
    #[arg(long)]
    pub commit: Option<String>,
    
    /// Reset mode: soft, mixed, or hard (default: hard)
    #[arg(long, default_value = "hard")]
    pub mode: String,
    
    /// Skip confirmation prompt
    #[arg(short, long)]
    pub force: bool,
}