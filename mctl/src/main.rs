//! Mirror Control (mctl) CLI tool
//!
//! A command-line interface tool for managing mirror.toml files.

mod cli;
mod commands;
mod output;
mod utils;

use clap::Parser;
use cli::Cli;
use commands::CommandError;
use output::create_formatter;

/// Main entry point for the CLI
fn main() {
    // Parse command-line arguments
    let cli = Cli::parse();

    // Create the output formatter
    let mut formatter = create_formatter(cli.verbose, cli.quiet, &cli.color);

    // Execute the command
    if let Err(err) = commands::execute(cli.clone(), formatter.as_mut()) {
        // Format and display the error
        match err {
            CommandError::Sdk(sdk_err) => {
                let error_msg = utils::format_sdk_error(&sdk_err, true);
                eprintln!("{}", error_msg);
            }
            CommandError::Input(msg) => {
                eprintln!("Input error: {}", msg);
            }
            CommandError::File(msg) => {
                eprintln!("File error: {}", msg);
            }
            CommandError::Config(msg) => {
                eprintln!("Configuration error: {}", msg);
            }
            CommandError::Other(msg) => {
                eprintln!("Error: {}", msg);
            }
        }
        std::process::exit(1);
    }
}
