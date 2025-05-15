//! # Logging Setup Module
//!
//! This module provides the logging infrastructure for the application.
//! It configures the env_logger crate according to application settings.

use log::{LevelFilter, info};
use env_logger::{Builder, Env};
use anyhow::Result;

/// Logging setup handler
pub struct LoggingSetup {
    /// Default log level
    default_level: LevelFilter,
}

impl LoggingSetup {
    /// Create a new logging setup with default settings
    pub fn new() -> Self {
        Self {
            default_level: LevelFilter::Info,
        }
    }
    
    /// Create a new logging setup with a specific default level
    pub fn with_level(default_level: LevelFilter) -> Self {
        Self {
            default_level,
        }
    }
    
    /// Initialize logging based on verbosity level
    pub fn init_logging(&self, verbosity: &u8) -> Result<()> {
        // Set log level based on verbosity flag
        let log_level = match verbosity {
            0 => self.default_level,
            1 => LevelFilter::Debug,
            _ => LevelFilter::Trace,
        };
        
        // Create and configure the logger
        let mut builder = Builder::from_env(Env::default());
        
        builder
            .filter(None, log_level)
            .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds))
            .format_module_path(true)
            .format_target(false)
            .init();
        
        info!("Logging initialized at level: {}", log_level);
        
        Ok(())
    }
    
    /// Initialize logging from configuration
    pub fn init_from_config(&self, level: &str, format: &str, file: Option<&str>) -> Result<()> {
        // Parse log level string
        let log_level = match level.to_lowercase().as_str() {
            "trace" => LevelFilter::Trace,
            "debug" => LevelFilter::Debug,
            "info" => LevelFilter::Info,
            "warn" => LevelFilter::Warn,
            "error" => LevelFilter::Error,
            _ => self.default_level,
        };
        
        // Create and configure the logger
        let mut builder = Builder::from_env(Env::default());
        
        builder
            .filter(None, log_level)
            .format_timestamp(Some(env_logger::fmt::TimestampPrecision::Seconds));
        
        // Configure JSON format if specified
        if format.to_lowercase() == "json" {
            // Note: For JSON formatting, a more complete solution would use a
            // JSON-structured logger like slog or log4rs
            builder.format(|buf, record| {
                use std::io::Write;
                writeln!(
                    buf,
                    "{{\"time\":\"{}\",\"level\":\"{}\",\"target\":\"{}\",\"message\":\"{}\"}}",
                    chrono::Local::now().format("%Y-%m-%dT%H:%M:%S"),
                    record.level(),
                    record.target(),
                    record.args()
                )
            });
        }
        
        // Write logs to file if specified
        if let Some(log_file) = file {
            use std::fs::File;
            match File::create(log_file) {
                Ok(file) => {
                    builder.target(env_logger::Target::Pipe(Box::new(file)));
                },
                Err(err) => {
                    eprintln!("Failed to open log file {}: {}", log_file, err);
                    // Continue with stdout logging
                }
            }
        }
        
        builder.init();
        
        info!("Logging initialized from config at level: {}", log_level);
        
        Ok(())
    }
}

impl Default for LoggingSetup {
    fn default() -> Self {
        Self::new()
    }
}