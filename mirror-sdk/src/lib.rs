//! # Mirror SDK
//! 
//! A Rust SDK for managing mirror.toml configuration files that define collections
//! of git repositories for large-scale IT projects.
//! 
//! ## Features
//! 
//! - Parse and generate mirror.toml files
//! - Generate unique hash IDs for repositories
//! - Support both SSH and HTTPS git URL formats
//! - Comprehensive error handling
//! - Configuration validation
//! 
//! ## Quick Start
//! 
//! ```rust
//! use mirror_sdk::{MirrorConfig, Repository, ConfigManager};
//! 
//! // Create a new repository configuration
//! let repo = Repository::from_url("git@github.com:org/repo.git".to_string())?;
//! 
//! // Create and manage configuration
//! let mut config = MirrorConfig::new();
//! config.add_repository(repo)?;
//! 
//! // Save to file
//! let manager = ConfigManager::new("mirror.toml");
//! manager.save(&config)?;
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
//! 
//! ## Configuration Format
//! 
//! The mirror.toml file uses the following format:
//! 
//! ```toml
//! [[repositories]]
//! git = "git@github.com:org/repo.git"
//! path = "org/repo"
//! branch = "main"        # optional, defaults to "main"
//! skip-push = false      # optional, defaults to false
//! ```

pub mod config;
pub mod error;
pub mod hash;
pub mod models;
pub mod url_parser;

// Re-export main types for easier access
pub use config::ConfigManager;
pub use error::{ConfigError, ConfigResult, HashError, MirrorSdkError, RepositoryError, Result};
pub use hash::{generate_hash, generate_extended_hash, verify_hash, validate_hash_format};
pub use models::{MirrorConfig, Repository};
pub use url_parser::{extract_path_from_url, validate_git_url, extract_hostname};

/// Crate version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default configuration file name
pub const DEFAULT_CONFIG_FILE: &str = "mirror.toml";

/// Default branch name for repositories
pub const DEFAULT_BRANCH: &str = "main";

/// Minimum hash length for unique identification
pub const MIN_HASH_LENGTH: usize = 4;

/// Maximum hash length (full SHA256 hex)
pub const MAX_HASH_LENGTH: usize = 64;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_workflow() {
        // Test the basic workflow described in the documentation
        let repo = Repository::from_url("git@github.com:org/repo.git".to_string()).unwrap();
        
        let mut config = MirrorConfig::new();
        config.add_repository(repo.clone()).unwrap();
        
        assert_eq!(config.len(), 1);
        assert_eq!(config.repositories()[0], repo);
        
        let hash = repo.compute_hash();
        assert_eq!(hash.len(), 8);
        
        let found = config.find_by_hash(&hash[..4]);
        assert!(found.is_some());
        assert_eq!(found.unwrap(), &repo);
    }
    
    #[test]
    fn test_url_parsing() {
        // Test SSH format
        let ssh_repo = Repository::from_url("git@github.com:org/repo.git".to_string()).unwrap();
        assert_eq!(ssh_repo.path, "org/repo");
        assert_eq!(ssh_repo.branch, DEFAULT_BRANCH);
        assert_eq!(ssh_repo.skip_push, false);
        
        // Test HTTPS format
        let https_repo = Repository::from_url("https://github.com/org/repo.git".to_string()).unwrap();
        assert_eq!(https_repo.path, "org/repo");
        assert_eq!(https_repo.branch, DEFAULT_BRANCH);
        assert_eq!(https_repo.skip_push, false);
    }
    
    #[test]
    fn test_hash_uniqueness() {
        let repo1 = Repository::from_url("git@github.com:org/repo1.git".to_string()).unwrap();
        let repo2 = Repository::from_url("git@github.com:org/repo2.git".to_string()).unwrap();
        
        let hash1 = repo1.compute_hash();
        let hash2 = repo2.compute_hash();
        
        assert_ne!(hash1, hash2);
        assert!(verify_hash(&repo1, &hash1));
        assert!(!verify_hash(&repo1, &hash2));
    }
    
    #[test]
    fn test_validation() {
        let valid_repo = Repository::new(
            "git@github.com:org/repo.git".to_string(),
            "org/repo".to_string(),
            Some("main".to_string()),
            Some(false),
        );
        assert!(valid_repo.validate().is_ok());
        
        let invalid_repo = Repository::new(
            "".to_string(), // Invalid empty git URL
            "org/repo".to_string(),
            Some("main".to_string()),
            Some(false),
        );
        assert!(invalid_repo.validate().is_err());
    }
}