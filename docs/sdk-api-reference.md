# Mirror SDK API Reference

This document provides a comprehensive reference for the Mirror SDK API.

## Core Types

### `MirrorSdk`

The main entry point for interacting with the SDK.

```rust
pub struct MirrorSdk {
    settings: ConfigSettings,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new() -> Self` | Create a new SDK instance with default settings. |
| `with_settings(settings: ConfigSettings) -> Self` | Create a new SDK instance with custom settings. |
| `load_config<P: AsRef<Path>>(&self, path: P) -> Result<MirrorConfig, MirrorError>` | Load a mirror.toml configuration from a file. |
| `save_config<P: AsRef<Path>>(&self, config: &MirrorConfig, path: P) -> Result<(), MirrorError>` | Save a mirror.toml configuration to a file. |
| `new_config(&self) -> MirrorConfig` | Create a new empty mirror.toml configuration. |
| `init_config<P: AsRef<Path>>(&self, path: P, force: bool) -> Result<MirrorConfig, MirrorError>` | Initialize a new mirror.toml configuration file. |
| `add_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError>` | Add a repository to a configuration. |
| `remove_repository_by_path(&self, config: &mut MirrorConfig, path: &str) -> Result<(), MirrorError>` | Remove a repository from a configuration by path. |
| `remove_repository_by_id(&self, config: &mut MirrorConfig, id: &str) -> Result<(), MirrorError>` | Remove a repository from a configuration by ID. |
| `update_repository(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError>` | Update a repository in a configuration. |
| `update_repository_by_id(&self, config: &mut MirrorConfig, repo: Repository) -> Result<(), MirrorError>` | Update a repository in a configuration by ID. |
| `find_repository_by_path<'a>(&self, config: &'a MirrorConfig, path: &str) -> Option<&'a Repository>` | Find a repository by path. |
| `find_repository_by_id<'a>(&self, config: &'a MirrorConfig, id: &str) -> Option<&'a Repository>` | Find a repository by ID. |
| `find_repositories_by_tag<'a>(&self, config: &'a MirrorConfig, tag: &str) -> Vec<&'a Repository>` | Find repositories by tag. |
| `validate_config(&self, config: &MirrorConfig) -> Result<(), ValidationError>` | Validate a configuration. |
| `get_config_path(&self) -> Result<PathBuf, MirrorError>` | Get the path to the mirror.toml file. |

### `MirrorConfig`

Represents a mirror.toml configuration file.

```rust
pub struct MirrorConfig {
    pub repositories: Vec<Repository>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new() -> Self` | Create a new empty configuration. |
| `find_by_path<'a>(&'a self, path: &str) -> Option<&'a Repository>` | Find a repository by path. |
| `find_by_id<'a>(&'a self, id: &str) -> Option<&'a Repository>` | Find a repository by ID. |
| `find_by_tag<'a>(&'a self, tag: &str) -> Vec<&'a Repository>` | Find repositories by tag. |

### `Repository`

Represents a Git repository in the configuration.

```rust
pub struct Repository {
    pub id: Option<String>,
    pub origin: String,
    pub branch: String,
    pub path: String,
    pub branch_lock: bool,
    pub tags: Vec<String>,
}
```

### `RepositoryBuilder`

A builder for creating `Repository` instances.

```rust
pub struct RepositoryBuilder {
    id: Option<String>,
    origin: Option<String>,
    branch: Option<String>,
    path: Option<String>,
    branch_lock: bool,
    tags: Vec<String>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `new() -> Self` | Create a new builder. |
| `id<S: Into<String>>(mut self, id: S) -> Self` | Set the repository ID. |
| `origin<S: Into<String>>(mut self, origin: S) -> Self` | Set the repository origin. |
| `branch<S: Into<String>>(mut self, branch: S) -> Self` | Set the repository branch. |
| `path<S: Into<String>>(mut self, path: S) -> Self` | Set the repository path. |
| `branch_lock(mut self, branch_lock: bool) -> Self` | Set the branch lock flag. |
| `tag<S: Into<String>>(mut self, tag: S) -> Self` | Add a tag to the repository. |
| `build(self) -> Result<Repository, MirrorError>` | Build the repository. |

### `ConfigSettings`

Settings for the SDK.

```rust
pub struct ConfigSettings {
    pub validate_paths: bool,
    pub validate_origins: bool,
    pub default_config_path: Option<String>,
}
```

#### Methods

| Method | Description |
|--------|-------------|
| `default() -> Self` | Create default settings. |
| `with_validate_paths(mut self, validate: bool) -> Self` | Set whether to validate paths. |
| `with_validate_origins(mut self, validate: bool) -> Self` | Set whether to validate origins. |
| `with_default_config_path<S: Into<String>>(mut self, path: S) -> Self` | Set the default config path. |

### `MirrorError`

Error type for the SDK.

```rust
pub enum MirrorError {
    Io(std::io::Error),
    Parse(toml::de::Error),
    Serialize(toml::ser::Error),
    InvalidConfiguration(String),
    RepositoryNotFound(String),
    DuplicateRepository(String),
    Validation(ValidationError),
}
```

### `ValidationError`

Error type for validation errors.

```rust
pub enum ValidationError {
    InvalidPath(String),
    InvalidOrigin(String),
    DuplicatePath(String),
    DuplicateId(String),
}
```

## Modules

### `config`

Handles configuration settings and paths.

### `error`

Defines error types for the SDK.

### `fs`

Handles file system operations.

### `models`

Defines data models for the SDK.

### `operations`

Implements operations on configurations.

### `utils`

Provides utility functions.

## Examples

### Loading and Saving a Configuration

```rust
use mirror_sdk::{MirrorSdk, MirrorError};
use std::path::Path;

fn load_and_save() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load a configuration
    let config = sdk.load_config("mirror.toml")?;
    
    // Save the configuration to a different file
    sdk.save_config(&config, "mirror_backup.toml")?;
    
    Ok(())
}
```

### Adding a Repository

```rust
use mirror_sdk::{MirrorSdk, RepositoryBuilder, MirrorError};

fn add_repository() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load or create a configuration
    let mut config = match sdk.load_config("mirror.toml") {
        Ok(config) => config,
        Err(_) => sdk.new_config(),
    };
    
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

### Finding Repositories by Tag

```rust
use mirror_sdk::{MirrorSdk, MirrorError};

fn find_by_tag() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load a configuration
    let config = sdk.load_config("mirror.toml")?;
    
    // Find repositories with a specific tag
    let repos = sdk.find_repositories_by_tag(&config, "example");
    
    // Print the paths of the found repositories
    for repo in repos {
        println!("Found repository at path: {}", repo.path);
    }
    
    Ok(())
}
```

### Validating a Configuration

```rust
use mirror_sdk::{MirrorSdk, MirrorError};

fn validate_config() -> Result<(), MirrorError> {
    let sdk = MirrorSdk::new();
    
    // Load a configuration
    let config = sdk.load_config("mirror.toml")?;
    
    // Validate the configuration
    match sdk.validate_config(&config) {
        Ok(_) => println!("Configuration is valid"),
        Err(err) => println!("Configuration is invalid: {}", err),
    }
    
    Ok(())
}