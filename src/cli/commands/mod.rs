//! CLI command implementations.

pub mod add;
pub mod config;
pub mod diff;
pub mod forge;
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
        } => add::execute(
            &cli.config,
            git,
            path,
            branch,
            rev,
            tag,
            workspace,
            skip_push,
        ),
        Commands::List { workspace, format } => list::execute(&cli.config, workspace, &format),
        Commands::Hydrate { image } => hydrate::execute(std::path::Path::new(&cli.config), &image),
        Commands::Images => forge::list(std::path::Path::new(&cli.config)),
        Commands::Graph { image, format } => forge::graph(std::path::Path::new(&cli.config), &image, &format),
        Commands::Context { image, out } => forge::context(std::path::Path::new(&cli.config), &image, &out),
        Commands::Build {
            image,
            tag,
            push,
            load,
            no_cache_store,
            keep_context,
        } => forge::build(
            std::path::Path::new(&cli.config),
            &image,
            &tag,
            push,
            load,
            no_cache_store,
            keep_context,
        ),
        Commands::Remove { path, delete } => remove::execute(&cli.config, &path, delete),
        Commands::Show { path } => show::execute(&cli.config, &path),
        Commands::Validate => validate::execute(&cli.config),
        Commands::Pull { workspace } => pull::execute(&cli.config, workspace),
        Commands::Sync {
            workspace,
            dry_run,
            force,
            create_missing_branches,
        } => sync::execute(
            &cli.config,
            workspace,
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
