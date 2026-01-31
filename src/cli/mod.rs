//! CLI interface for mctl.

pub mod app;
pub mod commands;
pub mod table;

pub use app::Cli;
pub use commands::execute;
