//! # Mirror SDK
//!
//! A Rust library for managing mirror.toml configuration files in MirrorBoards projects.
//!
//! This SDK provides functionality to create, read, update, and delete mirror.toml files,
//! which are used to manage repository configurations in MirrorBoards projects.
//!
//! ## Features
//!
//! - Create, read, update, and delete mirror.toml files
//! - Manage repository configurations (add, remove, update)
//! - Auto-generate repository IDs
//! - Support for custom paths and environment variable configuration
//! - Clean, well-documented API with proper error handling
//!
//! ## Example
//!
//! ```rust,no_run
//! use mirror_sdk::{MirrorConfig, Repository};
//! use std::path::Path;
//!
//! fn main() -> Result<(), mirror_sdk::Error> {
//!     // Initialize a new mirror configuration
//!     let mut config = MirrorConfig::new();
//!     
//!     // Add a repository
//!     config.add_repository(Repository::new(
//!         "git@github.com:mirrorboards/example-repo.git",
//!         "example/path",
//!     )?)?;
//!     
//!     // Save the configuration to the default location (./mirror.toml)
//!     config.save()?;
//!     
//!     // Or specify a custom path
//!     config.save_to(Path::new("custom/path/mirror.toml"))?;
//!     
//!     Ok(())
//! }
//! ```

// Module declarations
pub mod config;
pub mod repository;
pub mod error;
mod utils;

// Re-exports for public API
pub use config::{MirrorConfig, DEFAULT_FILENAME, ENV_MIRROR_PATH};
pub use repository::Repository;
pub use error::{Error, Result};

// Re-export dependencies that are part of our public API
pub use uuid;

/// Creates a new mirror configuration at the default location
///
/// This is a convenience function that calls `MirrorConfig::init()`.
///
/// # Returns
///
/// A `Result` containing the new configuration or an error
///
/// # Example
///
/// ```rust,no_run
/// use mirror_sdk;
///
/// let config = mirror_sdk::init().unwrap();
/// ```
pub fn init() -> Result<MirrorConfig> {
    MirrorConfig::init()
}

/// Creates a new mirror configuration at the specified path
///
/// This is a convenience function that calls `MirrorConfig::init_at()`.
///
/// # Arguments
///
/// * `path` - Path to create the mirror.toml file
///
/// # Returns
///
/// A `Result` containing the new configuration or an error
///
/// # Example
///
/// ```rust,no_run
/// use mirror_sdk;
/// use std::path::Path;
///
/// let config = mirror_sdk::init_at(Path::new("custom/path/mirror.toml")).unwrap();
/// ```
pub fn init_at<P: AsRef<std::path::Path>>(path: P) -> Result<MirrorConfig> {
    MirrorConfig::init_at(path)
}

/// Loads a mirror configuration from the default location or environment variable
///
/// This is a convenience function that calls `MirrorConfig::load()`.
///
/// # Returns
///
/// A `Result` containing the loaded configuration or an error
///
/// # Example
///
/// ```rust,no_run
/// use mirror_sdk;
///
/// let config = mirror_sdk::load().unwrap();
/// ```
pub fn load() -> Result<MirrorConfig> {
    MirrorConfig::load()
}

/// Loads a mirror configuration from the specified path
///
/// This is a convenience function that calls `MirrorConfig::load_from()`.
///
/// # Arguments
///
/// * `path` - Path to the mirror.toml file
///
/// # Returns
///
/// A `Result` containing the loaded configuration or an error
///
/// # Example
///
/// ```rust,no_run
/// use mirror_sdk;
/// use std::path::Path;
///
/// let config = mirror_sdk::load_from(Path::new("custom/path/mirror.toml")).unwrap();
/// ```
pub fn load_from<P: AsRef<std::path::Path>>(path: P) -> Result<MirrorConfig> {
    MirrorConfig::load_from(path)
}