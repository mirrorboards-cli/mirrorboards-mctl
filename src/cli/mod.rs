//! CLI module for MCTL
//!
//! This module handles command-line argument parsing and command execution.

mod args;
mod commands;

pub use args::Cli;
pub use commands::Command;
