//! # Mirror SDK
//!
//! A Rust library for managing mirror.toml configuration files.
//!
//! This SDK provides functionality to manage mirror.toml files, which are used to
//! configure repository mirroring. It supports operations such as initialization,
//! adding repositories, removing repositories, and updating repository configurations.
//!
//! ## Features
//!
//! - Initialize new mirror.toml files
//! - Add repositories to mirror.toml
//! - Remove repositories from mirror.toml
//! - List repositories in mirror.toml
//! - Update repository configurations
//! - Support for custom paths and environment variables
//! - Automatic repository ID generation
//!
//! ## Example
//!
//! ```rust,no_run
//! use mirror_sdk::{MirrorConfig, Repository};
//! use std::path::Path;
//!
//! // Load a mirror.toml file
//! let mut config = MirrorConfig::load_from_file(Path::new("./mirror.toml")).unwrap();
//!
//! // Add a new repository
//! let repo = Repository::new()
//!     .with_origin("git@github.com:example/repo.git")
//!     .with_path("./example/repo")
//!     .build()
//!     .unwrap();
//!
//! let mut config = config.add_repository(repo).unwrap();
//!
//! // Save the updated configuration
//! config.save().unwrap();
//! ```

mod config;
mod error;
mod repository;
mod utils;

pub use config::MirrorConfig;
pub use error::MirrorError;
pub use repository::{Repository, RepositoryBuilder};

// Re-export Result type for convenience
pub type Result<T> = std::result::Result<T, MirrorError>;

/// The default filename for mirror configuration
pub const DEFAULT_CONFIG_FILENAME: &str = "mirror.toml";

/// The environment variable name for the mirror configuration path
pub const CONFIG_PATH_ENV_VAR: &str = "MIRROR_CONFIG_PATH";