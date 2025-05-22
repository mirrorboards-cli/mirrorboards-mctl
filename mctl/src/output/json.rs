//! JSON output formatter
//!
//! This module provides a JSON output formatter for the CLI.

use std::fmt::Debug;
use std::any::Any;
use serde::Serialize;
use serde_json::json;
use super::{OutputFormatter, TableFormatter, ListFormatter, JsonFormatter, DetailFormatter};
use crate::commands::CommandResult;
use crate::commands::CommandError;

/// JSON output formatter
pub struct JsonOutput {
    /// Whether to use verbose output
    verbose: bool,
    
    /// Whether to use quiet output
    quiet: bool,
}

impl JsonOutput {
    /// Create a new JSON output formatter
    pub fn new(verbose: bool, quiet: bool) -> Self {
        Self {
            verbose,
            quiet,
        }
    }
}

impl OutputFormatter for JsonOutput {
    fn info(&mut self, message: &str) {
        if !self.quiet {
            let json = json!({
                "type": "info",
                "message": message,
            });
            println!("{}", serde_json::to_string(&json).unwrap());
        }
    }
    
    fn success(&mut self, message: &str) {
        if !self.quiet {
            let json = json!({
                "type": "success",
                "message": message,
            });
            println!("{}", serde_json::to_string(&json).unwrap());
        }
    }
    
    fn warning(&mut self, message: &str) {
        if !self.quiet {
            let json = json!({
                "type": "warning",
                "message": message,
            });
            println!("{}", serde_json::to_string(&json).unwrap());
        }
    }
    
    fn error(&mut self, message: &str) {
        // Always show errors, even in quiet mode
        let json = json!({
            "type": "error",
            "message": message,
        });
        eprintln!("{}", serde_json::to_string(&json).unwrap());
    }
    
    fn table_str(&mut self, title: &str, data: &[String]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        let json = json!({
            "type": "table",
            "title": title,
            "data": data,
        });
        
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        Ok(())
    }
    
    fn list_str(&mut self, title: &str, data: &[String]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        let json = json!({
            "type": "list",
            "title": title,
            "data": data,
        });
        
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
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
        
        let json = json!({
            "type": "detail",
            "title": title,
            "data": data,
        });
        
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        Ok(())
    }
    
    fn as_any(&self) -> &dyn Any {
        self
    }
    
    fn as_any_mut(&mut self) -> &mut dyn Any {
        self
    }
}

impl TableFormatter for JsonOutput {
    fn table<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        // For a proper JSON representation, we would need to implement Serialize for T
        // For now, we'll just convert to strings
        let items: Vec<String> = data.iter()
            .map(|item| format!("{:?}", item))
            .collect();
        
        let json = json!({
            "type": "table",
            "title": title,
            "data": items,
        });
        
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        Ok(())
    }
}

impl ListFormatter for JsonOutput {
    fn list<T: Debug>(&mut self, title: &str, data: &[T]) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        // For a proper JSON representation, we would need to implement Serialize for T
        // For now, we'll just convert to strings
        let items: Vec<String> = data.iter()
            .map(|item| format!("{:?}", item))
            .collect();
        
        let json = json!({
            "type": "list",
            "title": title,
            "data": items,
        });
        
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        Ok(())
    }
}

impl JsonFormatter for JsonOutput {
    fn json<T: Serialize>(&mut self, data: &T) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        let json_str = serde_json::to_string_pretty(data)
            .map_err(|e| CommandError::Other(format!("Failed to serialize to JSON: {}", e)))?;
        
        println!("{}", json_str);
        Ok(())
    }
}

impl DetailFormatter for JsonOutput {
    fn detail<T: Debug>(&mut self, title: &str, data: &T) -> CommandResult<()> {
        if self.quiet {
            return Ok(());
        }
        
        // For a proper JSON representation, we would need to implement Serialize for T
        // For now, we'll just convert to a string
        let json = json!({
            "type": "detail",
            "title": title,
            "data": format!("{:#?}", data),
        });
        
        println!("{}", serde_json::to_string_pretty(&json).unwrap());
        Ok(())
    }
}