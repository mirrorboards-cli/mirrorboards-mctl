//! Configuration module for MCTL
//!
//! This module handles configuration file parsing and management.

mod mirror_config;
mod paths;

pub use mirror_config::{MirrorConfig, Repository};
pub use paths::ConfigPaths;
