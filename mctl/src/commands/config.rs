//! Config command implementation
//!
//! This module implements the functionality of the config command,
//! which manages configuration settings.

use std::collections::HashMap;
use std::fs::{File};
use std::io::{Read, Write};
use serde::{Deserialize, Serialize};
use crate::cli::config::{ConfigArgs, ConfigCommands, SetArgs, GetArgs, ListArgs};
use crate::output::{OutputFormatter, ColorOutput, TableOutput, JsonOutput};
use crate::output::{TableFormatter, JsonFormatter};
use super::{CommandResult, CommandError};

/// User configuration file
const USER_CONFIG_FILE: &str = ".mctl.json";

/// User configuration
#[derive(Debug, Serialize, Deserialize, Default)]
struct UserConfig {
    /// Configuration options
    options: HashMap<String, String>,
}

/// Execute the config command
pub fn execute(args: ConfigArgs, formatter: &mut dyn OutputFormatter, _config_path: Option<String>) -> CommandResult<()> {
    match args.command {
        ConfigCommands::Set(args) => set_config(args, formatter),
        ConfigCommands::Get(args) => get_config(args, formatter),
        ConfigCommands::List(args) => list_config(args, formatter),
    }
}

/// Get the path to the user configuration file
fn get_config_path() -> CommandResult<std::path::PathBuf> {
    let home_dir = dirs::home_dir()
        .ok_or_else(|| CommandError::Config("Could not determine home directory".to_string()))?;
    Ok(home_dir.join(USER_CONFIG_FILE))
}

/// Load the user configuration
fn load_user_config() -> CommandResult<UserConfig> {
    let config_path = get_config_path()?;
    
    // If the file doesn't exist, return a default configuration
    if !config_path.exists() {
        return Ok(UserConfig::default());
    }
    
    // Read the file
    let mut file = File::open(&config_path)
        .map_err(|e| CommandError::File(format!("Failed to open config file: {}", e)))?;
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .map_err(|e| CommandError::File(format!("Failed to read config file: {}", e)))?;
    
    // Parse the JSON
    serde_json::from_str(&contents)
        .map_err(|e| CommandError::Config(format!("Failed to parse config file: {}", e)))
}

/// Save the user configuration
fn save_user_config(config: &UserConfig) -> CommandResult<()> {
    let config_path = get_config_path()?;
    
    // Serialize to JSON
    let json = serde_json::to_string_pretty(config)
        .map_err(|e| CommandError::Config(format!("Failed to serialize config: {}", e)))?;
    
    // Write to file
    let mut file = File::create(&config_path)
        .map_err(|e| CommandError::File(format!("Failed to create config file: {}", e)))?;
    file.write_all(json.as_bytes())
        .map_err(|e| CommandError::File(format!("Failed to write config file: {}", e)))?;
    
    Ok(())
}

/// Set a configuration option
fn set_config(args: SetArgs, formatter: &mut dyn OutputFormatter) -> CommandResult<()> {
    formatter.info(&format!("Setting configuration option '{}'...", args.name));
    
    // Load the user configuration
    let mut config = load_user_config()?;
    
    // Set the option
    config.options.insert(args.name.clone(), args.value.clone());
    
    // Save the configuration
    save_user_config(&config)?;
    
    formatter.success(&format!("Configuration option '{}' set to '{}'", args.name, args.value));
    Ok(())
}

/// Get a configuration option
fn get_config(args: GetArgs, formatter: &mut dyn OutputFormatter) -> CommandResult<()> {
    formatter.info(&format!("Getting configuration option '{}'...", args.name));
    
    // Load the user configuration
    let config = load_user_config()?;
    
    // Get the option
    match config.options.get(&args.name) {
        Some(value) => {
            formatter.info(&format!("{} = {}", args.name, value));
            Ok(())
        }
        None => {
            formatter.warning(&format!("Configuration option '{}' not found", args.name));
            Err(CommandError::Config(format!("Option '{}' not found", args.name)))
        }
    }
}

/// List all configuration options
fn list_config(args: ListArgs, formatter: &mut dyn OutputFormatter) -> CommandResult<()> {
    formatter.info("Listing configuration options...");
    
    // Load the user configuration
    let config = load_user_config()?;
    
    // Convert to a sorted vector of key-value pairs
    let mut options: Vec<(String, String)> = config.options.iter()
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect();
    options.sort_by(|a, b| a.0.cmp(&b.0));
    
    // Output the options
    if args.json {
        // Try to downcast to JsonOutput
        if let Some(json_formatter) = formatter.as_any_mut().downcast_mut::<JsonOutput>() {
            JsonFormatter::json(json_formatter, &config.options)?;
        } else {
            // Fallback to string representation
            let json = serde_json::to_string_pretty(&config.options)
                .map_err(|e| CommandError::Other(format!("Failed to serialize to JSON: {}", e)))?;
            formatter.json_str(&json)?;
        }
    } else {
        // Try to downcast to different formatter types
        if let Some(color_formatter) = formatter.as_any_mut().downcast_mut::<ColorOutput>() {
            TableFormatter::table(color_formatter, "Configuration Options", &options)?;
        } else if let Some(table_formatter) = formatter.as_any_mut().downcast_mut::<TableOutput>() {
            TableFormatter::table(table_formatter, "Configuration Options", &options)?;
        } else {
            // Fallback to string representation
            let option_strings: Vec<String> = options.iter()
                .map(|(k, v)| format!("{} = {}", k, v))
                .collect();
            formatter.table_str("Configuration Options", &option_strings)?;
        }
    }
    
    formatter.info(&format!("Found {} configuration options", options.len()));
    Ok(())
}