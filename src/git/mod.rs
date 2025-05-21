//! Git operations module for MCTL
//!
//! This module handles git operations for repository management.

mod credentials;
mod operations;
mod repository;

pub use credentials::GitCredentials;
pub use operations::{clone, commit, pull, push, status};
pub use repository::GitRepository;
