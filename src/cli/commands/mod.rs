//! CLI command implementations.

pub mod add;
pub mod config;
pub mod diff;
pub mod init;
pub mod list;
pub mod remove;
pub mod save;
pub mod show;
pub mod snapshot;
pub mod status;
pub mod sync;
pub mod validate;

use crate::cli::app::{Cli, Commands, ConfigCommands};
use anyhow::Result;
use colored::Colorize;

/// Execute the CLI command.
pub fn execute(cli: Cli) -> Result<()> {
    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }

    match cli.command {
        Commands::Init { force } => init::execute(&cli.config, force),
        Commands::Add {
            git,
            path,
            branch,
            rev,
            tag,
            workspace,
            skip_push,
        } => add::execute(&cli.config, git, path, branch, rev, tag, workspace, skip_push),
        Commands::List { workspace, format } => list::execute(&cli.config, workspace, &format),
        Commands::Remove { path, delete } => remove::execute(&cli.config, &path, delete),
        Commands::Show { path } => show::execute(&cli.config, &path),
        Commands::Validate => validate::execute(&cli.config),
        Commands::Sync {
            workspace,
            dry_run,
            force,
        } => sync::execute(&cli.config, workspace, dry_run, force, cli.verbose),
        Commands::Status {
            workspace,
            detailed,
        } => status::execute(&cli.config, workspace, detailed),
        Commands::Diff { workspace, staged } => diff::execute(&cli.config, workspace, staged),
        Commands::Save {
            workspace,
            message,
            dry_run,
        } => save::execute(&cli.config, workspace, &message, dry_run, cli.verbose),
        Commands::Snapshot { workspace, output } => {
            snapshot::execute(&cli.config, workspace, output)
        }
        Commands::Config(config_cmd) => match config_cmd {
            ConfigCommands::Init { git, branch, path } => {
                config::init_remote(&cli.config, &git, &branch, &path)
            }
            ConfigCommands::Pull => config::pull(&cli.config, cli.verbose),
            ConfigCommands::Push { message } => config::push(&cli.config, &message, cli.verbose),
            ConfigCommands::Diff => config::diff(&cli.config),
        },
    }
}

/// Print success message.
pub fn print_success(message: &str) {
    println!("{} {}", "✓".green(), message);
}

/// Print error message.
pub fn print_error(message: &str) {
    eprintln!("{} {}", "✗".red(), message);
}

/// Print warning message.
pub fn print_warning(message: &str) {
    println!("{} {}", "!".yellow(), message);
}

/// Print info message.
pub fn print_info(message: &str) {
    println!("{} {}", "→".blue(), message);
}
