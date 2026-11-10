//! CLI command implementations.

pub mod add;
pub mod config;
pub mod diff;
pub mod graph;
pub mod from_org;
pub mod hydrate;
pub mod init;
pub mod list;
pub mod pull;
pub mod remove;
pub mod save;
pub mod show;
pub mod snapshot;
pub mod status;
pub mod sync;
pub mod validate;

use crate::cli::app::{Cli, Commands, ConfigCommands};
use crate::core::config::find_including_config;
use anyhow::{bail, Result};
use colored::Colorize;
use std::path::Path;

/// Execute the CLI command.
pub fn execute(cli: Cli) -> Result<()> {
    // Disable colors if requested
    if cli.no_color {
        colored::control::set_override(false);
    }

    let reads_configuration = !matches!(
        cli.command,
        Commands::Init { .. } | Commands::FromOrg { .. }
    );
    if reads_configuration {
        ensure_not_included(Path::new(&cli.config))?;
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
            include,
        } => add::execute(
            &cli.config,
            add::AddOptions {
                git,
                path,
                branch,
                rev,
                tag,
                workspaces: workspace,
                skip_push,
                include,
            },
        ),
        Commands::List { workspace, format } => list::execute(&cli.config, workspace, &format),
        Commands::Hydrate { dir } => hydrate::execute(std::path::Path::new(&cli.config), &dir),
        Commands::Graph { dir, format } => {
            graph::execute(std::path::Path::new(&cli.config), &dir, &format)
        }
        Commands::Remove { path, delete } => remove::execute(&cli.config, &path, delete),
        Commands::Show { path } => show::execute(&cli.config, &path),
        Commands::Validate => validate::execute(&cli.config),
        Commands::Pull { workspace } => pull::execute(&cli.config, workspace),
        Commands::Sync {
            workspace,
            path,
            dry_run,
            force,
            create_missing_branches,
        } => sync::execute(
            &cli.config,
            workspace,
            path,
            dry_run,
            force,
            create_missing_branches,
            cli.verbose,
        ),
        Commands::Status {
            workspace,
            detailed,
            all,
        } => status::execute(&cli.config, workspace, detailed, all),
        Commands::Diff { workspace, staged } => diff::execute(&cli.config, workspace, staged),
        Commands::Save {
            workspace,
            message,
            dry_run,
        } => save::execute(&cli.config, workspace, &message, dry_run, cli.verbose),
        Commands::Snapshot { workspace, output } => {
            snapshot::execute(&cli.config, workspace, output)
        }
        Commands::FromOrg {
            org,
            output,
            workspace,
            limit,
            https,
            pin_branch,
            include_archived,
            include_forks,
        } => from_org::execute(
            &org,
            output,
            workspace,
            limit,
            https,
            pin_branch,
            include_archived,
            include_forks,
        ),
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

/// Stop when the configuration given is an include of another one.
///
/// Repository paths in an included file are relative to the configuration
/// that includes it, so running against the include alone would clone and
/// look for repositories in the wrong directory.
fn ensure_not_included(config_path: &Path) -> Result<()> {
    let Some(including_config) = find_including_config(config_path) else {
        return Ok(());
    };

    bail!(
        "{} is included by {}, and the paths in it are relative to that file. \
         Run mctl with --config {} (to add a repository to the included file: mctl add <git> --include <file>)",
        config_path.display(),
        including_config.display(),
        including_config.display()
    );
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
