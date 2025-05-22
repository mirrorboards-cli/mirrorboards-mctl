# Mirror SDK

A Rust library for managing mirror.toml configuration files.

## Features

- Parse and serialize mirror.toml configuration files
- Manage repository configurations (add, remove, update)
- Support file system operations for working with repositories
- Handle configuration through default paths and environment variables
- Provide a comprehensive error handling strategy
- Expose a clean, well-documented public API

## Usage

```rust
use mirror_sdk::{MirrorSdk, Repository, RepositoryBuilder, MirrorError};

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

## License

This project is licensed under the MIT License - see the LICENSE file for details.