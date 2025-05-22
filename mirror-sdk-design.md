# Mirror SDK Design Document

## 1. Introduction

The Mirror SDK is a Rust library designed to manage mirror.toml configuration files. This document outlines the architecture, data structures, and API design for the SDK.

## 2. Project Structure

```
mirror-sdk/
├── Cargo.toml
├── README.md
├── LICENSE
├── .gitignore
├── examples/
│   ├── basic_usage.rs
│   ├── repository_management.rs
│   └── config_validation.rs
├── src/
│   ├── lib.rs              # Main library entry point, exports public API
│   ├── config/             # Configuration handling
│   │   ├── mod.rs
│   │   ├── paths.rs        # Default paths and environment variables
│   │   └── settings.rs     # SDK settings and configuration
│   ├── models/             # Core data structures
│   │   ├── mod.rs
│   │   ├── repository.rs   # Repository configuration
│   │   └── mirror_config.rs # Overall mirror.toml structure
│   ├── operations/         # Repository management operations
│   │   ├── mod.rs
│   │   ├── init.rs         # Initialize new configuration
│   │   ├── add.rs          # Add repository
│   │   ├── remove.rs       # Remove repository
│   │   └── update.rs       # Update repository
│   ├── fs/                 # File system operations
│   │   ├── mod.rs
│   │   ├── io.rs           # File I/O operations
│   │   └── path.rs         # Path handling utilities
│   ├── error.rs            # Error handling
│   └── utils/              # Utility functions
│       ├── mod.rs
│       └── validation.rs   # Validation utilities
└── tests/
    ├── integration_tests.rs
    └── test_data/
        └── sample_mirror.toml
```

## 3. Core Data Structures and Types

### 3.1 Repository Configuration

```rust
/// Represents a single repository configuration in mirror.toml
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Repository {
    /// Optional unique identifier for the repository
    pub id: Option<String>,
    
    /// Git repository origin URL
    pub origin: String,
    
    /// Git branch to use
    pub branch: String,
    
    /// Whether the branch is locked (cannot be changed)
    #[serde(default)]
    pub branch_lock: bool,
    
    /// Local filesystem path where the repository should be cloned
    pub path: String,
    
    /// Optional tags for categorizing repositories
    #[serde(default)]
    pub tags: Vec<String>,
}
```

### 3.2 Mirror Configuration

```rust
/// Represents the entire mirror.toml configuration
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct MirrorConfig {
    /// List of repository configurations
    pub repositories: Vec<Repository>,
}
```

### 3.3 Configuration Settings

```rust
/// SDK configuration settings
#[derive(Debug, Clone)]
pub struct ConfigSettings {
    /// Default path for mirror.toml
    pub default_config_path: PathBuf,
    
    /// Whether to validate repository paths
    pub validate_paths: bool,
    
    /// Whether to validate repository origins
    pub validate_origins: bool,
}
```

## 4. Main Traits and Interfaces

### 4.1 ConfigLoader Trait

```rust
/// Trait for loading mirror.toml configuration
pub trait ConfigLoader {
    /// Load configuration from a file
    fn load_from_file(&self, path: &Path) -> Result<MirrorConfig, MirrorError>;
    
    /// Load configuration from a string
    fn load_from_str(&self, content: &str) -> Result<MirrorConfig, MirrorError>;
    
    /// Save configuration to a file
    fn save_to_file(&self, config: &MirrorConfig, path: &Path) -> Result<(), MirrorError>;
    
    /// Convert configuration to a string
    fn to_string(&self, config: &MirrorConfig) -> Result<String, MirrorError>;
}
```

### 4.2 RepositoryManager Trait

```rust
/// Trait for managing repositories in a mirror.toml configuration
pub trait RepositoryManager {
    /// Add a new repository to the configuration
    fn add_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError>;
    
    /// Remove a repository from the configuration by path
    fn remove_repository_by_path(&self, config: &mut MirrorConfig, path: &str) -> Result<(), MirrorError>;
    
    /// Remove a repository from the configuration by ID
    fn remove_repository_by_id(&self, config: &mut MirrorConfig, id: &str) -> Result<(), MirrorError>;
    
    /// Update an existing repository in the configuration
    fn update_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError>;
    
    /// Find a repository by path
    fn find_by_path(&self, config: &MirrorConfig, path: &str) -> Option<&Repository>;
    
    /// Find a repository by ID
    fn find_by_id(&self, config: &MirrorConfig, id: &str) -> Option<&Repository>;
    
    /// Find repositories by tag
    fn find_by_tag(&self, config: &MirrorConfig, tag: &str) -> Vec<&Repository>;
}
```

### 4.3 ConfigValidator Trait

```rust
/// Trait for validating mirror.toml configuration
pub trait ConfigValidator {
    /// Validate the entire configuration
    fn validate_config(&self, config: &MirrorConfig) -> Result<(), ValidationError>;
    
    /// Validate a single repository configuration
    fn validate_repository(&self, repo: &Repository) -> Result<(), ValidationError>;
    
    /// Check for path conflicts between repositories
    fn check_path_conflicts(&self, config: &MirrorConfig) -> Result<(), ValidationError>;
}
```

## 5. Key Functionality Modules

### 5.1 Config Parsing/Serialization

The `config` module will handle loading and saving mirror.toml files:

- Parse TOML into strongly-typed Rust structures
- Serialize Rust structures back to TOML
- Handle default configuration paths
- Support environment variable overrides

Implementation will use the `toml` crate for parsing and serialization, with custom error handling for format issues.

### 5.2 Repository Management Operations

The `operations` module will provide functionality for managing repositories:

- Initialize a new mirror.toml configuration
- Add repositories to the configuration
- Remove repositories from the configuration
- Update existing repository configurations
- Query repositories by various criteria (path, ID, tags)

These operations will maintain the integrity of the configuration and prevent conflicts.

### 5.3 File System Operations

The `fs` module will handle file system interactions:

- Read and write mirror.toml files
- Validate paths for repositories
- Handle path normalization and resolution
- Support relative and absolute paths

This module will use Rust's standard library for file operations with additional error handling.

### 5.4 Configuration Handling

The `config` module will manage SDK configuration:

- Default paths for mirror.toml files
- Environment variable handling for configuration overrides
- SDK settings for validation and behavior customization

This provides flexibility for users to integrate the SDK into different environments.

## 6. Error Handling Strategy

The SDK will use a comprehensive error handling approach:

```rust
/// Main error type for the Mirror SDK
#[derive(Debug, thiserror::Error)]
pub enum MirrorError {
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    
    #[error("TOML parsing error: {0}")]
    TomlParse(#[from] toml::de::Error),
    
    #[error("TOML serialization error: {0}")]
    TomlSerialize(#[from] toml::ser::Error),
    
    #[error("Validation error: {0}")]
    Validation(#[from] ValidationError),
    
    #[error("Repository not found: {0}")]
    RepositoryNotFound(String),
    
    #[error("Repository already exists: {0}")]
    RepositoryAlreadyExists(String),
    
    #[error("Path conflict: {0}")]
    PathConflict(String),
    
    #[error("Invalid configuration: {0}")]
    InvalidConfiguration(String),
    
    #[error("Environment error: {0}")]
    Environment(String),
}

/// Validation-specific errors
#[derive(Debug, thiserror::Error)]
pub enum ValidationError {
    #[error("Invalid repository path: {0}")]
    InvalidPath(String),
    
    #[error("Invalid repository origin: {0}")]
    InvalidOrigin(String),
    
    #[error("Missing required field: {0}")]
    MissingField(String),
    
    #[error("Duplicate ID: {0}")]
    DuplicateId(String),
    
    #[error("Path conflict: {0} and {1}")]
    PathConflict(String, String),
}
```

Key aspects of the error handling strategy:

1. Use of the `thiserror` crate for ergonomic error definitions
2. Hierarchical error types for specific error categories
3. Detailed error messages for debugging
4. Error conversion using the `From` trait
5. Context-specific errors for different operations

## 7. Public API Design

The public API will be exposed through the `lib.rs` file:

```rust
// Public types
pub use crate::models::{Repository, MirrorConfig};
pub use crate::error::{MirrorError, ValidationError};
pub use crate::config::ConfigSettings;

// Main SDK struct
pub struct MirrorSdk {
    settings: ConfigSettings,
}

impl MirrorSdk {
    /// Create a new SDK instance with default settings
    pub fn new() -> Self {
        Self {
            settings: ConfigSettings::default(),
        }
    }
    
    /// Create a new SDK instance with custom settings
    pub fn with_settings(settings: ConfigSettings) -> Self {
        Self { settings }
    }
    
    /// Load a mirror.toml configuration from a file
    pub fn load_config(&self, path: impl AsRef<Path>) -> Result<MirrorConfig, MirrorError> {
        // Implementation
    }
    
    /// Save a mirror.toml configuration to a file
    pub fn save_config(&self, config: &MirrorConfig, path: impl AsRef<Path>) -> Result<(), MirrorError> {
        // Implementation
    }
    
    /// Create a new empty mirror.toml configuration
    pub fn new_config(&self) -> MirrorConfig {
        MirrorConfig { repositories: Vec::new() }
    }
    
    /// Add a repository to a configuration
    pub fn add_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
        // Implementation
    }
    
    /// Remove a repository from a configuration by path
    pub fn remove_repository_by_path(&self, config: &mut MirrorConfig, path: &str) -> Result<(), MirrorError> {
        // Implementation
    }
    
    /// Remove a repository from a configuration by ID
    pub fn remove_repository_by_id(&self, config: &mut MirrorConfig, id: &str) -> Result<(), MirrorError> {
        // Implementation
    }
    
    /// Update a repository in a configuration
    pub fn update_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError> {
        // Implementation
    }
    
    /// Find a repository by path
    pub fn find_repository_by_path<'a>(&self, config: &'a MirrorConfig, path: &str) -> Option<&'a Repository> {
        // Implementation
    }
    
    /// Find a repository by ID
    pub fn find_repository_by_id<'a>(&self, config: &'a MirrorConfig, id: &str) -> Option<&'a Repository> {
        // Implementation
    }
    
    /// Find repositories by tag
    pub fn find_repositories_by_tag<'a>(&self, config: &'a MirrorConfig, tag: &str) -> Vec<&'a Repository> {
        // Implementation
    }
    
    /// Validate a configuration
    pub fn validate_config(&self, config: &MirrorConfig) -> Result<(), ValidationError> {
        // Implementation
    }
}

// Builder for Repository
pub struct RepositoryBuilder {
    origin: Option<String>,
    branch: Option<String>,
    path: Option<String>,
    id: Option<String>,
    branch_lock: bool,
    tags: Vec<String>,
}

impl RepositoryBuilder {
    /// Create a new repository builder
    pub fn new() -> Self {
        Self {
            origin: None,
            branch: None,
            path: None,
            id: None,
            branch_lock: false,
            tags: Vec::new(),
        }
    }
    
    /// Set the repository origin
    pub fn origin(mut self, origin: impl Into<String>) -> Self {
        self.origin = Some(origin.into());
        self
    }
    
    /// Set the repository branch
    pub fn branch(mut self, branch: impl Into<String>) -> Self {
        self.branch = Some(branch.into());
        self
    }
    
    /// Set the repository path
    pub fn path(mut self, path: impl Into<String>) -> Self {
        self.path = Some(path.into());
        self
    }
    
    /// Set the repository ID
    pub fn id(mut self, id: impl Into<String>) -> Self {
        self.id = Some(id.into());
        self
    }
    
    /// Set branch lock
    pub fn branch_lock(mut self, lock: bool) -> Self {
        self.branch_lock = lock;
        self
    }
    
    /// Add a tag
    pub fn tag(mut self, tag: impl Into<String>) -> Self {
        self.tags.push(tag.into());
        self
    }
    
    /// Add multiple tags
    pub fn tags(mut self, tags: impl IntoIterator<Item = impl Into<String>>) -> Self {
        for tag in tags {
            self.tags.push(tag.into());
        }
        self
    }
    
    /// Build the repository
    pub fn build(self) -> Result<Repository, ValidationError> {
        let origin = self.origin.ok_or_else(|| ValidationError::MissingField("origin".to_string()))?;
        let branch = self.branch.ok_or_else(|| ValidationError::MissingField("branch".to_string()))?;
        let path = self.path.ok_or_else(|| ValidationError::MissingField("path".to_string()))?;
        
        Ok(Repository {
            id: self.id,
            origin,
            branch,
            branch_lock: self.branch_lock,
            path,
            tags: self.tags,
        })
    }
}
```

## 8. Usage Examples

### 8.1 Basic Usage

```rust
use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};
use std::path::Path;

fn main() -> Result<(), MirrorError> {
    // Create a new SDK instance
    let sdk = MirrorSdk::new();
    
    // Load an existing configuration
    let mut config = sdk.load_config("mirror.toml")?;
    
    // Create a new repository
    let repo = RepositoryBuilder::new()
        .origin("git@github.com:example/repo.git")
        .branch("main")
        .path("example/repo")
        .tag("example")
        .build()?;
    
    // Add the repository to the configuration
    sdk.add_repository(&mut config, repo)?;
    
    // Save the updated configuration
    sdk.save_config(&config, "mirror.toml")?;
    
    Ok(())
}
```

### 8.2 Finding Repositories

```rust
use mirror_sdk::{MirrorSdk, MirrorError};

fn main() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    let config = sdk.load_config("mirror.toml")?;
    
    // Find repositories by tag
    let repos = sdk.find_repositories_by_tag(&config, "example");
    println!("Found {} repositories with tag 'example'", repos.len());
    
    // Find a repository by path
    if let Some(repo) = sdk.find_repository_by_path(&config, "example/repo") {
        println!("Found repository: {}", repo.origin);
    }
    
    Ok(())
}
```

### 8.3 Creating a New Configuration

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};

fn main() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Create a new empty configuration
    let mut config = sdk.new_config();
    
    // Add repositories
    let repo1 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo1.git")
        .branch("main")
        .path("example/repo1")
        .build()?;
    
    let repo2 = RepositoryBuilder::new()
        .origin("git@github.com:example/repo2.git")
        .branch("main")
        .path("example/repo2")
        .build()?;
    
    sdk.add_repository(&mut config, repo1)?;
    sdk.add_repository(&mut config, repo2)?;
    
    // Save the configuration
    sdk.save_config(&config, "mirror.toml")?;
    
    Ok(())
}
```

## 9. Conclusion

The Mirror SDK provides a comprehensive solution for managing mirror.toml configuration files in Rust. The architecture is designed to be modular, type-safe, and user-friendly, with a focus on error handling and validation.

Key features of the design include:

1. Strong typing for configuration elements
2. Comprehensive error handling
3. Builder pattern for repository creation
4. Clear separation of concerns through modules
5. Flexible configuration options
6. Validation at multiple levels

This design document serves as a blueprint for implementing the Mirror SDK, providing a clear path forward for development.