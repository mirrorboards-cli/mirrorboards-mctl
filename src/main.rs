//! # MCTL - Multiple Repository Management Tool
//!
//! This application provides tools for managing multiple Git repositories
//! with a focus on synchronization, status checking, and SSH authentication.
//!
//! The application follows a layered architecture:
//! - Presentation: CLI interface and output formatting
//! - Application: Command orchestration and business logic
//! - Domain: Core entities and interfaces
//! - Infrastructure: External system integrations

use anyhow::Result;
use clap::Parser;

mod domain;
mod application;
mod presentation;
mod infrastructure;

use crate::presentation::cli::Cli;
use crate::presentation::cli_runner::CliRunner;

/// Main entry point for the MCTL application
fn main() -> Result<()> {
    // Parse command line arguments
    let cli = Cli::parse();
    
    // Create CLI runner and execute command
    let runner = CliRunner::new(cli);
    runner.run()
}