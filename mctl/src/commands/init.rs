//! Init command implementation
//!
//! This module implements the functionality of the init command,
//! which initializes a new mirror.toml file.

use std::path::Path;
use mirror_sdk::MirrorConfig;
use crate::cli::init::InitArgs;
use crate::output::OutputFormatter;
use super::{CommandResult, CommandError};

/// Execute the init command
pub fn execute(args: InitArgs, formatter: &mut dyn OutputFormatter, config_path: Option<String>) -> CommandResult<()> {
    // Determine the path to create the mirror.toml file
    let path = if let Some(path) = args.path {
        Path::new(&path).to_path_buf()
    } else if let Some(path) = config_path {
        Path::new(&path).to_path_buf()
    } else {
        Path::new(mirror_sdk::DEFAULT_FILENAME).to_path_buf()
    };

    formatter.info(&format!("Initializing mirror.toml at {}", path.display()));

    // Check if the file already exists and handle force flag
    if path.exists() && !args.force {
        formatter.error(&format!("File already exists at {}", path.display()));
        formatter.info("Use --force to overwrite the existing file");
        return Err(CommandError::Input(format!("File already exists at {}", path.display())));
    }

    // Initialize the mirror.toml file
    let result = if args.force {
        // If force is specified, create a new config and save it
        let config = MirrorConfig::new();
        config.save_to(&path).map(|_| config)
    } else {
        // Otherwise, use the SDK's init_at function
        MirrorConfig::init_at(&path)
    };

    match result {
        Ok(_) => {
            formatter.success(&format!("Successfully initialized mirror.toml at {}", path.display()));
            Ok(())
        }
        Err(err) => {
            formatter.error(&format!("Failed to initialize mirror.toml: {}", err));
            Err(CommandError::Sdk(err))
        }
    }
}