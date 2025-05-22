//! Table output formatter
//!
//! This module provides a table-formatted output formatter for the CLI.

use tabled::{Table, Tabled};
use std::fmt::Debug;
use std::any::Any;
use serde::Serialize;
use super::{OutputFormatter, TableFormatter, ListFormatter, JsonFormatter, DetailFormatter};
use crate::commands::CommandResult;
use crate::commands::CommandError;

/// Table output formatter
pub struct TableOutput {
    /// Whether to use verbose output
    verbose: bool,
    
    /// Whether to use quiet output
    quiet: bool,
}

impl TableOutput {
    /// Create a new table output formatter
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self {
            verbose,
            quiet,
        }
    }
}

impl OutputFormatter for TableOutput {
    fn info(&mut self, message: &str) {
        if !self.quiet {
            println!("{}", message);
        }
    }
    
    fn success(&mut self, message: &str) {
        if !self.quiet {
            println!("Success: {}", message);
        }
    }
    
    fn warning(&mut self, message: &str) {
        if !self.quiet {
            println!("Warning: {}", message);
        }
    }
    
    fn error(&mut self, message: &str) {
        // Always show errors, even in quiet mode
        eprintln!("Error: {}", message);
    }
    
    fn table_str(&mut self, title: &str, data: &[String]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
        
        for (i, item) in data.iter().enumerate() {
            println!("{}. {}", i + 1, item);
        }
        
        println!();
        Ok(())
    }
    
    fn list_str(&mut self, title: &str, data: &[String]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
        
        for item in data {
            println!("- {}", item);
        }
        
        println!();
        Ok(())
    }
    
    fn json_str(&mut self, json_string: &str) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", json_string);
        Ok(())
    }
    
    fn detail_str(&mut self, title: &str, data: &str) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
        println!("{}", data);
        println!();
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl TableFormatter for TableOutput {
    fn table<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
        
        // For a proper table, we would need to implement the Tabled trait for T
        // For now, we'll just print the debug representation
        for (i, item) in data.iter().enumerate() {
            println!("{}. {:?}", i + 1, item);
        }
        
        println!();
        Ok(())
    }
}

impl ListFormatter for TableOutput {
    fn list<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
        
        for item in data {
            println!("- {:?}", item);
        }
        
        println!();
        Ok(())
    }
}

impl JsonFormatter for TableOutput {
    fn json<T: Serialize>(&mut self, data: &T) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        let json = serde_json::to_string_pretty(data)
            .map_err(|e| CommandError::Other(format!("Failed to serialize to JSON: {}", e)))?;
        
        println!("{}", json);
        Ok(())
    }
}

impl DetailFormatter for TableOutput {
    fn detail<T: Debug>(&mut self, title: &str, data: &T) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", title);
        println!("{}", "=".repeat(title.len()));
        println!("{:#?}", data);
        println!();
        Ok(())
    }
}

/// Helper function to create a table from a list of items that implement Tabled
pub fn create_table<T: Tabled>(items: &[T]) -> Table {
    Table::new(items)
}