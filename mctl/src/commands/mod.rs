//! Command implementations for mctl CLI

pub mod add;
pub mod diff;
pub mod init;
pub mod list;
pub mod remove;
pub mod show;
pub mod status;
pub mod sync;
pub mod validate;

pub use add::AddCommand;
pub use diff::DiffCommand;
pub use init::InitCommand;
pub use list::ListCommand;
pub use remove::RemoveCommand;
pub use show::ShowCommand;
pub use status::StatusCommand;
pub use sync::SyncCommand;
pub use validate::ValidateCommand;

use anyhow::Result;
use mirror_sdk::ConfigManager;
use colored::Colorize;

/// Common functionality for all commands
pub trait Command {
    fn execute(&self, config_manager: &ConfigManager, verbose: bool) -> Result<()>;
}

/// Print success message with green color
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green().bold(), message);
}

/// Print error message with red color
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red().bold(), message);
}

/// Print warning message with yellow color
pub fn print_warning(message: &str) {
    println!("{} {}", "⚠".yellow().bold(), message);
}

/// Print info message with blue color
pub fn print_info(message: &str) {
    println!("{} {}", "ℹ".blue().bold(), message);
}

/// Print verbose message if verbose mode is enabled
pub fn print_verbose(message: &str, verbose: bool) {
    if verbose {
        println!("{} {}", "→".cyan(), message.dimmed());
    }
}