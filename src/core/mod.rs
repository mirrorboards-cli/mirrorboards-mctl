//! Core business logic for mctl.
//!
//! This module contains all the reusable business logic that is independent
//! of the CLI interface. It can be used as a library for other tools.

pub mod config;
pub mod error;
pub mod forge;
pub mod graph;
pub mod hash;
pub mod image;
pub mod include;
pub mod repository;
pub mod url;

// Re-exports for convenience
pub use config::{ConfigManager, MirrorConfig, RawMirrorConfig, RemoteConfig};
pub use error::{ConfigError, ConfigResult, GitError, GitResult, UrlError, UrlResult};
pub use forge::{assemble_context, build_image, ForgeReceipt};
pub use graph::{image_graph, ImageGraph};
pub use hash::{generate_repo_hash, generate_short_hash};
pub use image::{ImageKind, ImageSpec};
pub use include::{IncludeResolver, ResolvedConfig, RepositoryWithSource};
pub use repository::{Repository, VersionSpec};
pub use url::{parse_full_path, parse_repo_name, suggest_path, GitUrl, Protocol};
