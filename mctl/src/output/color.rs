//! Color output formatter
//!
//! This module provides a color-coded output formatter for the CLI.

use colored::*;
use std::fmt::Debug;
use std::any::Any;
use serde::Serialize;
use super::{OutputFormatter, TableFormatter, ListFormatter, JsonFormatter, DetailFormatter};
use crate::commands::CommandResult;
use crate::commands::CommandError;

/// Color output formatter
pub struct ColorOutput {
    /// Whether to use verbose output
    verbose: bool,
    
    /// Whether to use quiet output
    quiet: bool,
    
    /// Whether to use color
    use_color: bool,
}

impl ColorOutput {
    /// Create a new color output formatter
    pub fn new(verbose: bool, quiet: bool, use_color: bool) -> Self {
        Self {
            verbose,
            quiet,
            use_color,
        }
    }
    
    /// Format a message with color
    fn format(&self, message: &str, color: Color) -> String {
        if self.use_color {
            message.color(color).to_string()
        } else {
            message.to_string()
        }
    }
}

impl OutputFormatter for ColorOutput {
    fn info(&mut self, message: &str) {
        if !self.quiet {
            println!("{}", self.format(message, Color::White));
        }
    }
    
    fn success(&mut self, message: &str) {
        if !self.quiet {
            println!("{}", self.format(message, Color::Green));
        }
    }
    
    fn warning(&mut self, message: &str) {
        if !self.quiet {
            println!("{}", self.format(message, Color::Yellow));
        }
    }
    
    fn error(&mut self, message: &str) {
        // Always show errors, even in quiet mode
        eprintln!("{}", self.format(message, Color::Red));
    }
    
    fn table_str(&mut self, title: &str, data: &[String]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", self.format(title, Color::Cyan));
        println!("{}", self.format(&"=".repeat(title.len()), Color::Cyan));
        
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
        
        println!("{}", self.format(title, Color::Cyan));
        println!("{}", self.format(&"=".repeat(title.len()), Color::Cyan));
        
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
        
        println!("{}", self.format(title, Color::Cyan));
        println!("{}", self.format(&"=".repeat(title.len()), Color::Cyan));
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

impl TableFormatter for ColorOutput {
    fn table<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", self.format(title, Color::Cyan));
        println!("{}", self.format(&"=".repeat(title.len()), Color::Cyan));
        
        for (i, item) in data.iter().enumerate() {
            println!("{}. {:?}", i + 1, item);
        }
        
        println!();
        Ok(())
    }
}

impl ListFormatter for ColorOutput {
    fn list<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", self.format(title, Color::Cyan));
        println!("{}", self.format(&"=".repeat(title.len()), Color::Cyan));
        
        for item in data {
            println!("- {:?}", item);
        }
        
        println!();
        Ok(())
    }
}

impl JsonFormatter for ColorOutput {
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

impl DetailFormatter for ColorOutput {
    fn detail<T: Debug>(&mut self, title: &str, data: &T) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        println!("{}", self.format(title, Color::Cyan));
        println!("{}", self.format(&"=".repeat(title.len()), Color::Cyan));
        println!("{:#?}", data);
        println!();
        Ok(())
    }
}