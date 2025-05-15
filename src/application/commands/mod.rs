//! # Command Module
//!
//! This module defines the Command trait and provides implementations for
//! the core commands: sync, status, and save.

use std::sync::Arc;
use anyhow::Result;
use crate::domain::configuration::Config;

/// Command trait for command implementations
pub trait Command: Send + Sync {
    /// Execute the command
    fn execute(&self) -> Result<()>;
    
    /// Get the command name
    fn name(&self) -> &'static str;
    
    /// Get the command description
    fn description(&self) -> &'static str;
}

/// Command factory trait for creating command instances
pub trait CommandFactory {
    /// Create a command instance from command arguments
    fn create_command(&self, config: &Config, args: &[String]) -> Result<Box<dyn Command>>;
}

// Re-export command modules
pub mod factory;
pub mod sync;
pub mod status;
pub mod save;
pub mod init;

#[cfg(test)]
pub mod tests;