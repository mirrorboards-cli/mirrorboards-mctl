//! # Infrastructure Layer
//!
//! This layer provides implementations of interfaces defined in the domain layer,
//! and handles interactions with external systems and resources.
//!
//! ## Modules
//!
//! - `git`: Git repository operations with SSH authentication
//! - `filesystem`: File and directory operations
//! - `config`: Configuration file loading and parsing
//! - `logging`: Structured logging implementation

pub mod git;
pub mod filesystem;
pub mod config;
pub mod logging;