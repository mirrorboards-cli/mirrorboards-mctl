# Mirror SDK

A Rust library for managing mirror.toml configuration files.

## Features

- Initialize new mirror.toml files
- Add repositories to mirror.toml
- Remove repositories from mirror.toml
- List repositories in mirror.toml
- Update repository configurations
- Support for custom paths and environment variables
- Automatic repository ID generation

## Usage

```rust
use mirror_sdk::{MirrorConfig, Repository};
use std::path::Path;

// Load a mirror.toml file
let mut config = MirrorConfig::load_from_file(Path::new("./mirror.toml"))?;

// Add a new repository
let repo = Repository::new()
    .with_origin("git@github.com:example/repo.git")
    .with_path("./example/repo")
    .build()?;

let mut config = config.add_repository(repo)?;

// Save the updated configuration
config.save()?;
```

## Configuration

By default, the SDK looks for a `mirror.toml` file in the current directory. You can specify a custom path:

```rust
// Using a custom path
let config = MirrorConfig::load_from_file(Path::new("/custom/path/mirror.toml"))?;
```

Or use an environment variable:

```rust
// Set MIRROR_CONFIG_PATH environment variable
std::env::set_var("MIRROR_CONFIG_PATH", "/custom/path/mirror.toml");

// Load using the environment variable
let config = MirrorConfig::load_from_env()?;
```

## License

This project is licensed under the MIT License - see the LICENSE file for details.