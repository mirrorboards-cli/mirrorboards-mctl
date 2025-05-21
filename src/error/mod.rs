//! Error handling module for MCTL
//!
//! This module provides centralized error handling and reporting for the MCTL application.
//! It defines error types, error codes, and error handling utilities.

pub mod handler;
pub mod types;

// Re-export common types for easier access
pub use handler::ErrorHandler;
pub use types::{ErrorCode, MctlError};
