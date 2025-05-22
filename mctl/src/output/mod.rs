//! Output formatting module
//!
//! This module provides functionality for formatting output to the user,
//! including color coding, tables, and JSON output.

mod color;
mod table;
mod json;

pub use color::ColorOutput;
pub use table::TableOutput;
pub use json::JsonOutput;

use std::fmt::Debug;
use std::any::Any;
use serde::Serialize;
use super::commands::CommandResult;

/// Output formatter trait for basic message output
pub trait OutputFormatter {
    /// Output an informational message
    fn info(&mut self, message: &str);

    /// Output a success message
    fn success(&mut self, message: &str);

    /// Output a warning message
    fn warning(&mut self, message: &str);

    /// Output an error message
    fn error(&mut self, message: &str);

    /// Output data as a table for string data
    fn table_str(&mut self, title: &str, data: &[String]) -> CommandResult<()>;

    /// Output data as a list for string data
    fn list_str(&mut self, title: &str, data: &[String]) -> CommandResult<()>;

    /// Output data as JSON string
    fn json_str(&mut self, json_string: &str) -> CommandResult<()>;

    /// Output detailed information as a string
    fn detail_str(&mut self, title: &str, data: &str) -> CommandResult<()>;
    
    /// Convert to Any for downcasting
    fn as_any(&self) -> &dyn Any;
    
    /// Convert to mutable Any for downcasting
    fn as_any_mut(&mut self) -> &mut dyn Any;
}

/// Extension trait for table output
pub trait TableFormatter {
    /// Output data as a table
    fn table<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()>;
}

/// Extension trait for list output
pub trait ListFormatter {
    /// Output data as a list
    fn list<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()>;
}

/// Extension trait for JSON output
pub trait JsonFormatter {
    /// Output data as JSON
    fn json<T: Serialize>(&mut self, data: &T) -> CommandResult<()>;
}

/// Extension trait for detailed output
pub trait DetailFormatter {
    /// Output detailed information about an item
    fn detail<T: Debug>(&mut self, title: &str, data: &T) -> CommandResult<()>;
}

/// Create a new output formatter based on the CLI options
pub fn create_formatter(verbose: bool, quiet: bool, color: &str) -> Box<dyn OutputFormatter> {
    // Determine whether to use color
    let use_color = match color {
        "always" => true,
        "never" => false,
        _ => atty::is(atty::Stream::Stdout), // "auto" - use color if stdout is a terminal
    };

    // Create the formatter
    Box::new(ColorOutput::new(verbose, quiet, use_color))
}