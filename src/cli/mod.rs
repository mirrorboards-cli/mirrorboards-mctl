//! CLI interface for mctl.

pub mod app;
pub mod commands;

pub use app::Cli;
pub use commands::execute;
