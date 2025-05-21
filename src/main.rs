//! MCTL (Mirror Control) - A tool for efficient git repository synchronization and mirroring
//!
//! This is the main entry point for the MCTL application.
//! It initializes logging, parses command-line arguments, and executes the appropriate command.

mod cli;
mod config;
mod error;
mod git;
mod repo;
mod security;
mod status;

use cli::Cli;
use error::handler::ErrorHandler;
use log::{error, info};

/// Main entry point for the MCTL application
fn main() {
    // Initialize logging
    env_logger::init();
    info!("Starting MCTL");

    // Create CLI instance
    let mut cli = Cli::new();

    // Parse arguments and execute command
    match cli.parse_args() {
        Ok(command) => {
            if let Err(err) = cli.execute(command) {
                let error_handler = ErrorHandler::new();
                error!("Error executing command: {}", err);
                eprintln!("{}", error_handler.handle_error(&err));
                std::process::exit(1);
            }
        }
        Err(err) => {
            let error_handler = ErrorHandler::new();
            error!("Error parsing arguments: {}", err);
            eprintln!("{}", error_handler.handle_error(&err));
            std::process::exit(1);
        }
    }

    info!("MCTL completed successfully");
}
