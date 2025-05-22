# Mirror Workspace Examples

This directory contains examples demonstrating how to use the Mirror SDK and CLI together.

## Running Examples

You can run the examples using Cargo from the root of the workspace:

```bash
# Run the SDK and CLI integration example
cargo run --example sdk_and_cli
```

## Available Examples

### sdk_and_cli.rs

This example demonstrates how to use the mirror-sdk and mirror-cli together:

1. It creates a mirror.toml file using the SDK
2. Uses the CLI to add a repository
3. Uses the SDK again to verify the repository was added correctly
4. Uses the CLI to list repositories

This example shows how you can build tools that leverage both the programmatic API of the SDK and the user-friendly interface of the CLI.

## Creating Your Own Examples

To create your own examples:

1. Add a new Rust file in the examples directory
2. Update the workspace Cargo.toml to include your example:

```toml
[[example]]
name = "your_example_name"
path = "examples/your_example_file.rs"
```

3. Make sure to include the necessary dependencies in your example:

```rust
use mirror_sdk::{MirrorSdk, MirrorError};
// Other imports as needed
```

4. Run your example with:

```bash
cargo run --example your_example_name
```

## Dependencies

The examples use the following dependencies:

- `mirror-sdk`: The core library for managing mirror.toml files
- `mirror-cli`: The command-line interface for the mirror-sdk
- `tempfile`: For creating temporary directories and files

These dependencies are defined in the workspace Cargo.toml file.