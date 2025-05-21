//! Git operations module for MCTL
//!
//! This module handles git operations for repository management.

pub mod credentials;
pub mod operations;
pub mod repository;

pub use credentials::GitCredentials;
pub use operations::{clone, commit, pull, push, status};
pub use repository::GitRepository;
