//! # Presentation Layer
//!
//! This layer handles the user interface aspects of the application,
//! including CLI argument parsing and output formatting.
//!
//! The presentation layer:
//! - Defines the command-line interface structure
//! - Handles user input parsing and validation
//! - Formats output for the terminal
//! - Manages progress indicators and user feedback

pub mod cli;
pub mod output;
pub mod cli_runner;