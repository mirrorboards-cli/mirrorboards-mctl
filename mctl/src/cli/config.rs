//! Config command definition
//!
//! This module defines the CLI interface for the config command,
//! which manages configuration settings.

use clap::{Args, Subcommand};

/// Arguments for the config command
#[derive(Debug, Args, Clone)]
pub struct ConfigArgs {
    /// Subcommand to execute
    #[command(subcommand)]
    pub command: ConfigCommands,
}

/// Subcommands for the config command
#[derive(Debug, Subcommand, Clone)]
pub enum ConfigCommands {
    /// Set a configuration option
    Set(SetArgs),

    /// Get a configuration option value
    Get(GetArgs),

    /// List all configuration options
    List(ListArgs),
}

/// Arguments for the config set command
#[derive(Debug, Args, Clone)]
pub struct SetArgs {
    /// Name of the configuration option
    pub name: String,

    /// Value of the configuration option
    pub value: String,
}

/// Arguments for the config get command
#[derive(Debug, Args, Clone)]
pub struct GetArgs {
    /// Name of the configuration option
    pub name: String,
}

/// Arguments for the config list command
#[derive(Debug, Args, Clone)]
pub struct ListArgs {
    /// Output in JSON format
    #[arg(short, long)]
    pub json: bool,
}