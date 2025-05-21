//! Configuration module for MCTL
//!
//! This module handles configuration file parsing and management.

pub mod mirror_config;
pub mod paths;

pub use mirror_config::{MirrorConfig, Repository};
pub use paths::ConfigPaths;
