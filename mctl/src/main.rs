use anyhow::Result;
use clap::Parser;
use colored::*;
use mirror_sdk::ConfigManager;

mod cli;
mod commands;

use cli::{Cli, Commands};
use commands::*;

fn main() {
    // Enable colored output on Windows
    #[cfg(windows)]
    let _ = colored::control::set_virtual_terminal(true);
    
    let args = Cli::parse();
    
    if let Err(e) = run(args) {
        print_error(&format!("Error: {}", e));
        
        // Print error chain if available
        let mut source = e.source();
        while let Some(err) = source {
            eprintln!("  Caused by: {}", err);
            source = err.source();
        }
        
        std::process::exit(1);
    }
}

fn run(args: Cli) -> Result<()> {
    let config_path = args.config_path();
    let verbose = args.verbose;
    
    if verbose {
        print_verbose(&format!("Using configuration file: {}", config_path), true);
    }
    
    let config_manager = ConfigManager::new(&config_path);
    
    match args.command {
        Commands::Init { force } => {
            let command = InitCommand { force };
            command.execute(&config_manager, verbose)
        },
        
        Commands::Add { git_url, path, branch, skip_push } => {
            let command = AddCommand {
                git_url,
                path,
                branch,
                skip_push,
            };
            command.execute(&config_manager, verbose)
        },
        
        Commands::List { json } => {
            let command = ListCommand { json };
            command.execute(&config_manager, verbose)
        },
        
        Commands::Remove { hash, force } => {
            let command = RemoveCommand { hash, force };
            command.execute(&config_manager, verbose)
        },
        
        Commands::Show { hash } => {
            let command = ShowCommand { hash };
            command.execute(&config_manager, verbose)
        },
        
        Commands::Validate { detailed } => {
            let command = ValidateCommand { detailed };
            command.execute(&config_manager, verbose)
        },
        
        Commands::Sync { dry_run, pull, force } => {
            let command = SyncCommand { dry_run, pull, force };
            command.execute(&config_manager, verbose)
        },
    }
}

/// Print error message with red color
fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print verbose message if verbose mode is enabled
fn print_verbose(message: &str, verbose: bool) {
    if verbose {
        println!("{} {}", "→".cyan(), message.dimmed());
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::NamedTempFile;
    use std::fs;

    #[test]
    fn test_init_command_integration() {
        let temp_file = NamedTempFile::new().unwrap();
        let temp_path = temp_file.path().to_path_buf();
        
        // Delete the temp file so we can test creation
        drop(temp_file);
        
        let args = Cli {
            command: Commands::Init { force: false },
            config: Some(temp_path.to_string_lossy().to_string()),
            verbose: false,
        };
        
        run(args).unwrap();
        
        // Verify file was created
        assert!(temp_path.exists());
        let content = fs::read_to_string(&temp_path).unwrap();
        assert!(content.contains("# Mirror Configuration File"));
    }
    
    #[test]
    fn test_add_and_list_integration() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_string_lossy().to_string();
        
        // Initialize
        let init_args = Cli {
            command: Commands::Init { force: false },
            config: Some(config_path.clone()),
            verbose: false,
        };
        run(init_args).unwrap();
        
        // Add repository
        let add_args = Cli {
            command: Commands::Add {
                git_url: "git@github.com:org/repo.git".to_string(),
                path: None,
                branch: None,
                skip_push: false,
            },
            config: Some(config_path.clone()),
            verbose: false,
        };
        run(add_args).unwrap();
        
        // List repositories
        let list_args = Cli {
            command: Commands::List { json: false },
            config: Some(config_path),
            verbose: false,
        };
        run(list_args).unwrap();
    }
    
    #[test]
    fn test_verbose_mode() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_string_lossy().to_string();
        
        let args = Cli {
            command: Commands::Init { force: false },
            config: Some(config_path),
            verbose: true,
        };
        
        run(args).unwrap();
    }
    
    #[test]
    fn test_validate_command_integration() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_string_lossy().to_string();
        
        // Initialize and add repository
        let init_args = Cli {
            command: Commands::Init { force: false },
            config: Some(config_path.clone()),
            verbose: false,
        };
        run(init_args).unwrap();
        
        let add_args = Cli {
            command: Commands::Add {
                git_url: "git@github.com:org/repo.git".to_string(),
                path: None,
                branch: None,
                skip_push: false,
            },
            config: Some(config_path.clone()),
            verbose: false,
        };
        run(add_args).unwrap();
        
        // Validate
        let validate_args = Cli {
            command: Commands::Validate { detailed: true },
            config: Some(config_path),
            verbose: false,
        };
        run(validate_args).unwrap();
    }
    
    #[test]
    fn test_sync_command_integration() {
        let temp_file = NamedTempFile::new().unwrap();
        let config_path = temp_file.path().to_string_lossy().to_string();
        
        // Initialize and add repository
        let init_args = Cli {
            command: Commands::Init { force: false },
            config: Some(config_path.clone()),
            verbose: false,
        };
        run(init_args).unwrap();
        
        let add_args = Cli {
            command: Commands::Add {
                git_url: "git@github.com:org/repo.git".to_string(),
                path: None,
                branch: None,
                skip_push: false,
            },
            config: Some(config_path.clone()),
            verbose: false,
        };
        run(add_args).unwrap();
        
        // Test sync with dry run
        let sync_args = Cli {
            command: Commands::Sync {
                dry_run: true,
                pull: false,
                force: false
            },
            config: Some(config_path),
            verbose: false,
        };
        run(sync_args).unwrap();
    }
}