//! mctl - Mirror configuration management tool
//!
//! CLI entry point.

use anyhow::Result;
use mctl::cli::{execute, Cli};

fn main() -> Result<()> {
    // Parse CLI arguments
    let cli = Cli::parse_args();

    // Execute the command!
    execute(cli)
}
