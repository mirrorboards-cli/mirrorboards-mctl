//! Core business logic for mctl.
//!
//! This module contains all the reusable business logic that is independent
//! of the CLI interface. It can be used as a library for other tools.

pub mod config;
pub mod error;
pub mod graph;
pub mod hash;
pub mod include;
pub mod repository;
pub mod url;

// Re-exports for convenience
pub use config::{ConfigManager, MirrorConfig, RawMirrorConfig, RemoteConfig};
pub use error::{ConfigError, ConfigResult, GitError, GitResult, UrlError, UrlResult};
pub use graph::{closure, Closure, Language};
pub use hash::{generate_repo_hash, generate_short_hash};
pub use include::{IncludeResolver, ResolvedConfig, RepositoryWithSource};
pub use repository::{Repository, VersionSpec};
pub use url::{parse_full_path, parse_repo_name, suggest_path, GitUrl, Protocol};
