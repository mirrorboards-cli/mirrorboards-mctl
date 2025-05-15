//! # Output Formatting Module
//!
//! This module handles formatting and displaying output to users.
//! It provides structured output capabilities for different command results.

use std::fmt;
use std::sync::Arc;
use std::sync::Mutex;
use colored::Colorize;
use std::io::{self, Write};
use indicatif::{ProgressBar, ProgressStyle, MultiProgress};
use serde::Serialize;
use clap::ValueEnum;
use crate::domain::repository::RepositoryStatus;
use crate::domain::error::{RepositoryError, GitError, ConfigError, CommandError};

/// Output format options
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum OutputFormat {
    /// Human-readable text with colors
    Text,
    /// JSON format for machine processing
    Json,
    /// Compact single-line format
    Compact,
}

impl fmt::Display for OutputFormat {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            OutputFormat::Text => write!(f, "text"),
            OutputFormat::Json => write!(f, "json"),
            OutputFormat::Compact => write!(f, "compact"),
        }
    }
}

/// Output colorization settings
#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum ColorMode {
    /// Always use colors
    Always,
    /// Never use colors
    Never,
    /// Use colors only when output is to a terminal
    Auto,
}

impl fmt::Display for ColorMode {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            ColorMode::Always => write!(f, "always"),
            ColorMode::Never => write!(f, "never"),
            ColorMode::Auto => write!(f, "auto"),
        }
    }
}

/// Progress indicator type
#[derive(Debug, Clone, Copy)]
pub enum ProgressType {
    /// Bar indicator (graphical)
    Bar,
    /// Spinner indicator (animated)
    Spinner,
    /// Simple text indicator (no animation)
    Simple,
}

/// Progress tracker for multiple operations
pub struct ProgressTracker {
    /// MultiProgress for tracking multiple progress bars
    multi_progress: MultiProgress,
    /// List of active progress bars
    progress_bars: Mutex<Vec<ProgressBar>>,
    /// Progress type for this tracker
    progress_type: ProgressType,
}

impl ProgressTracker {
    /// Create a new progress tracker
    pub fn new(progress_type: ProgressType) -> Self {
        Self {
            multi_progress: MultiProgress::new(),
            progress_bars: Mutex::new(Vec::new()),
            progress_type,
        }
    }
    
    /// Add a new progress bar
    pub fn add_progress_bar(&self, total: usize, message: &str) -> ProgressBar {
        let style = match self.progress_type {
            ProgressType::Bar => ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg} ({eta})")
                .unwrap()
                .progress_chars("=> "),
            ProgressType::Spinner => ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg} {pos}/{len} ({eta})")
                .unwrap(),
            ProgressType::Simple => ProgressStyle::default_bar()
                .template("{msg} {pos}/{len}")
                .unwrap(),
        };
        
        let progress_bar = if total > 0 {
            self.multi_progress.add(ProgressBar::new(total as u64))
        } else {
            self.multi_progress.add(ProgressBar::new_spinner())
        };
        
        progress_bar.set_style(style);
        progress_bar.set_message(message.to_string());
        
        // Store the progress bar in our list
        let mut bars = self.progress_bars.lock().unwrap();
        bars.push(progress_bar.clone());
        
        progress_bar
    }
    
    /// Complete all progress bars
    pub fn complete_all(&self, message: &str) {
        let bars = self.progress_bars.lock().unwrap();
        for bar in bars.iter() {
            bar.finish_with_message(message.to_string());
        }
    }
    
    /// Clear all progress bars
    pub fn clear_all(&self) {
        let bars = self.progress_bars.lock().unwrap();
        for bar in bars.iter() {
            bar.finish_and_clear();
        }
    }
}

/// Output formatter interface
pub trait OutputFormatter {
    /// Format repository status for display
    fn format_status(&self, status: &RepositoryStatus) -> String;
    
    /// Format error message for display
    fn format_error(&self, error: &str) -> String;
    
    /// Format success message for display
    fn format_success(&self, message: &str) -> String;
    
    /// Format information message for display
    fn format_info(&self, message: &str) -> String;
    
    /// Format warning message for display
    fn format_warning(&self, message: &str) -> String;
    
    /// Display a progress indicator
    fn show_progress(&self, message: &str, current: usize, total: usize);
    
    /// Complete a progress indicator
    fn complete_progress(&self, message: &str);
    
    /// Format a repository error
    fn format_repository_error(&self, error: &RepositoryError) -> String {
        self.format_error(&error.to_string())
    }
    
    /// Format a git error
    fn format_git_error(&self, error: &GitError) -> String {
        self.format_error(&error.to_string())
    }
    
    /// Format a config error
    fn format_config_error(&self, error: &ConfigError) -> String {
        self.format_error(&error.to_string())
    }
    
    /// Format a command error
    fn format_command_error(&self, error: &CommandError) -> String {
        self.format_error(&error.to_string())
    }
    
    /// Create a new progress tracker
    fn create_progress_tracker(&self) -> ProgressTracker;
}

/// Enhanced text output formatter implementation
pub struct EnhancedTextFormatter {
    /// Color mode setting
    color_mode: ColorMode,
    /// Progress type for this formatter
    progress_type: ProgressType,
}

impl EnhancedTextFormatter {
    /// Create a new enhanced text formatter
    pub fn new(color_mode: ColorMode, progress_type: ProgressType) -> Self {
        Self { 
            color_mode,
            progress_type,
        }
    }
    
    /// Create a new enhanced text formatter with default settings
    pub fn default() -> Self {
        Self { 
            color_mode: ColorMode::Auto,
            progress_type: ProgressType::Bar,
        }
    }
    
    /// Check if colors should be used
    fn use_colors(&self) -> bool {
        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

impl OutputFormatter for EnhancedTextFormatter {
    fn format_status(&self, status: &RepositoryStatus) -> String {
        let use_colors = self.use_colors();
        
        let branch = if use_colors {
            status.current_branch.blue().to_string()
        } else {
            status.current_branch.to_string()
        };
        
        let status_indicator = if status.has_changes {
            if use_colors {
                "● ".red().to_string()
            } else {
                "* ".to_string()
            }
        } else if status.has_unpushed_commits {
            if use_colors {
                "● ".yellow().to_string()
            } else {
                "^ ".to_string()
            }
        } else {
            if use_colors {
                "● ".green().to_string()
            } else {
                "  ".to_string()
            }
        };
        
        let mut result = format!("{}{}", status_indicator, branch);
        
        if status.has_changes {
            result.push_str(" (uncommitted changes)");
        }
        
        if status.has_unpushed_commits {
            result.push_str(" (unpushed commits)");
        }
        
        if !status.changed_files.is_empty() {
            result.push_str("\n  Changed files:");
            for file in &status.changed_files {
                result.push_str(&format!("\n    - {}", file));
            }
        }
        
        if let Some(message) = &status.message {
            result.push_str(&format!("\n  {}", message));
        }
        
        result
    }
    
    fn format_error(&self, error: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "ERROR".red().bold(), error)
        } else {
            format!("ERROR: {}", error)
        }
    }
    
    fn format_success(&self, message: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "SUCCESS".green().bold(), message)
        } else {
            format!("SUCCESS: {}", message)
        }
    }
    
    fn format_info(&self, message: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "INFO".blue().bold(), message)
        } else {
            format!("INFO: {}", message)
        }
    }
    
    fn format_warning(&self, message: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "WARNING".yellow().bold(), message)
        } else {
            format!("WARNING: {}", message)
        }
    }
    
    fn show_progress(&self, message: &str, current: usize, total: usize) {
        // Create one-time progress bar for simpler use cases
        let bar = ProgressBar::new(total as u64);
        
        let style = match self.progress_type {
            ProgressType::Bar => ProgressStyle::default_bar()
                .template("{spinner:.green} [{bar:40.cyan/blue}] {pos}/{len} {msg} ({eta})")
                .unwrap()
                .progress_chars("=> "),
            ProgressType::Spinner => ProgressStyle::default_spinner()
                .template("{spinner:.green} {msg} {pos}/{len} ({eta})")
                .unwrap(),
            ProgressType::Simple => ProgressStyle::default_bar()
                .template("{msg} {pos}/{len}")
                .unwrap(),
        };
        
        bar.set_style(style);
        bar.set_message(message.to_string());
        bar.set_position(current as u64);
        
        // For simple progress, we don't keep the ProgressBar, we just display it once
    }
    
    fn complete_progress(&self, message: &str) {
        println!("\r✓ {}", message);
    }
    
    fn create_progress_tracker(&self) -> ProgressTracker {
        ProgressTracker::new(self.progress_type)
    }
}

/// Text output formatter implementation (legacy)
pub struct TextFormatter {
    /// Color mode setting
    color_mode: ColorMode,
}

impl TextFormatter {
    /// Create a new text formatter with specified color mode
    pub fn new(color_mode: ColorMode) -> Self {
        Self { color_mode }
    }
    
    /// Check if colors should be used
    fn use_colors(&self) -> bool {
        match self.color_mode {
            ColorMode::Always => true,
            ColorMode::Never => false,
            ColorMode::Auto => atty::is(atty::Stream::Stdout),
        }
    }
}

impl OutputFormatter for TextFormatter {
    fn format_status(&self, status: &RepositoryStatus) -> String {
        let use_colors = self.use_colors();
        
        let branch = if use_colors {
            status.current_branch.blue().to_string()
        } else {
            status.current_branch.to_string()
        };
        
        let status_indicator = if status.has_changes {
            if use_colors {
                "● ".red().to_string()
            } else {
                "* ".to_string()
            }
        } else if status.has_unpushed_commits {
            if use_colors {
                "● ".yellow().to_string()
            } else {
                "^ ".to_string()
            }
        } else {
            if use_colors {
                "● ".green().to_string()
            } else {
                "  ".to_string()
            }
        };
        
        let mut result = format!("{}{}", status_indicator, branch);
        
        if status.has_changes {
            result.push_str(" (uncommitted changes)");
        }
        
        if status.has_unpushed_commits {
            result.push_str(" (unpushed commits)");
        }
        
        if !status.changed_files.is_empty() {
            result.push_str("\n  Changed files:");
            for file in &status.changed_files {
                result.push_str(&format!("\n    - {}", file));
            }
        }
        
        if let Some(message) = &status.message {
            result.push_str(&format!("\n  {}", message));
        }
        
        result
    }
    
    fn format_error(&self, error: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "ERROR".red().bold(), error)
        } else {
            format!("ERROR: {}", error)
        }
    }
    
    fn format_success(&self, message: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "SUCCESS".green().bold(), message)
        } else {
            format!("SUCCESS: {}", message)
        }
    }
    
    fn format_info(&self, message: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "INFO".blue().bold(), message)
        } else {
            format!("INFO: {}", message)
        }
    }
    
    fn format_warning(&self, message: &str) -> String {
        if self.use_colors() {
            format!("{}: {}", "WARNING".yellow().bold(), message)
        } else {
            format!("WARNING: {}", message)
        }
    }
    
    fn show_progress(&self, message: &str, current: usize, total: usize) {
        let percentage = (current as f32 / total as f32 * 100.0) as usize;
        let progress_message = format!("{} [{}/{}] {}%", message, current, total, percentage);
        
        // Print progress with carriage return to overwrite the line
        print!("\r{}", progress_message);
        io::stdout().flush().unwrap();
    }
    
    fn complete_progress(&self, message: &str) {
        // Print final message with newline
        println!("\r{}", message);
    }
    
    fn create_progress_tracker(&self) -> ProgressTracker {
        ProgressTracker::new(ProgressType::Simple)
    }
}

/// JSON output formatter implementation
pub struct JsonFormatter {}

impl JsonFormatter {
    /// Create a new JSON formatter
    pub fn new() -> Self {
        Self {}
    }
}

impl OutputFormatter for JsonFormatter {
    fn format_status(&self, status: &RepositoryStatus) -> String {
        serde_json::to_string_pretty(status).unwrap_or_else(|_| "{}".to_string())
    }
    
    fn format_error(&self, error: &str) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "type": "error",
            "message": error
        })).unwrap_or_else(|_| "{}".to_string())
    }
    
    fn format_success(&self, message: &str) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "type": "success",
            "message": message
        })).unwrap_or_else(|_| "{}".to_string())
    }
    
    fn format_info(&self, message: &str) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "type": "info",
            "message": message
        })).unwrap_or_else(|_| "{}".to_string())
    }
    
    fn format_warning(&self, message: &str) -> String {
        serde_json::to_string_pretty(&serde_json::json!({
            "type": "warning",
            "message": message
        })).unwrap_or_else(|_| "{}".to_string())
    }
    
    fn show_progress(&self, message: &str, current: usize, total: usize) {
        let progress = serde_json::json!({
            "type": "progress",
            "message": message,
            "current": current,
            "total": total,
            "percentage": (current as f32 / total as f32 * 100.0)
        });
        
        println!("{}", serde_json::to_string(&progress).unwrap());
    }
    
    fn complete_progress(&self, message: &str) {
        let complete = serde_json::json!({
            "type": "progress_complete",
            "message": message
        });
        
        println!("{}", serde_json::to_string(&complete).unwrap());
    }
    
    fn create_progress_tracker(&self) -> ProgressTracker {
        // Even for JSON, we need a tracker, but we'll use Simple style which has minimal UI
        ProgressTracker::new(ProgressType::Simple)
    }
}

/// Compact output formatter implementation (for scripts)
pub struct CompactFormatter {
    /// Include timestamps in output
    include_timestamp: bool,
}

impl CompactFormatter {
    /// Create a new compact formatter
    pub fn new(include_timestamp: bool) -> Self {
        Self { include_timestamp }
    }
    
    /// Format with optional timestamp
    fn format_with_timestamp(&self, message_type: &str, message: &str) -> String {
        if self.include_timestamp {
            let now = chrono::Local::now();
            format!("[{}] {}: {}", now.format("%H:%M:%S"), message_type, message)
        } else {
            format!("{}: {}", message_type, message)
        }
    }
}

impl OutputFormatter for CompactFormatter {
    fn format_status(&self, status: &RepositoryStatus) -> String {
        let status_type = if status.has_changes {
            "CHANGED"
        } else if status.has_unpushed_commits {
            "UNPUSHED"
        } else {
            "CLEAN"
        };
        
        self.format_with_timestamp(status_type, &status.current_branch)
    }
    
    fn format_error(&self, error: &str) -> String {
        self.format_with_timestamp("ERROR", error)
    }
    
    fn format_success(&self, message: &str) -> String {
        self.format_with_timestamp("SUCCESS", message)
    }
    
    fn format_info(&self, message: &str) -> String {
        self.format_with_timestamp("INFO", message)
    }
    
    fn format_warning(&self, message: &str) -> String {
        self.format_with_timestamp("WARNING", message)
    }
    
    fn show_progress(&self, message: &str, current: usize, total: usize) {
        let progress = format!("{}: {}/{}", message, current, total);
        println!("{}", self.format_with_timestamp("PROGRESS", &progress));
    }
    
    fn complete_progress(&self, message: &str) {
        println!("{}", self.format_with_timestamp("DONE", message));
    }
    
    fn create_progress_tracker(&self) -> ProgressTracker {
        // For compact output, we use simple progress
        ProgressTracker::new(ProgressType::Simple)
    }
}

/// Factory for creating output formatters
pub struct OutputFormatterFactory;

impl OutputFormatterFactory {
    /// Create a new output formatter based on format and color settings
    pub fn create_formatter(format: OutputFormat, color: ColorMode) -> Box<dyn OutputFormatter> {
        match format {
            OutputFormat::Text => Box::new(EnhancedTextFormatter::new(color, ProgressType::Bar)),
            OutputFormat::Json => Box::new(JsonFormatter::new()),
            OutputFormat::Compact => Box::new(CompactFormatter::new(true)),
        }
    }
}