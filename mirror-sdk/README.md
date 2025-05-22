# Mirror SDK

A Rust library for managing mirror.toml configuration files in MirrorBoards projects.

## Features

- Create, read, update, and delete mirror.toml files
- Manage repository configurations (add, remove, update)
- Auto-generate repository IDs
- Support for custom paths and environment variable configuration
- Clean, well-documented API with proper error handling

## Usage

Add this to your `Cargo.toml`:

```toml
[dependencies]
mirror-sdk = "0.1.0"
```

### Basic Example

```rust
use mirror_sdk::{MirrorConfig, Repository};
use std::path::Path;

fn main() -> Result<(), mirror_sdk::Error> {
    // Initialize a new mirror configuration
    let mut config = MirrorConfig::new();
    
    // Add a repository
    config.add_repository(Repository::new(
        "git@github.com:mirrorboards/example-repo.git",
        "example/path",
    )?);
    
    // Save the configuration to the default location (./mirror.toml)
    config.save()?;
    
    // Or specify a custom path
    config.save_to(Path::new("custom/path/mirror.toml"))?;
    
    Ok(())
}
```

## Documentation

For detailed API documentation, run:

```
cargo doc --open
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.