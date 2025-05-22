# Mirror Workspace Documentation

This directory contains comprehensive documentation for the Mirror SDK and CLI.

## Documentation Structure

- [SDK API Reference](./sdk-api-reference.md): Detailed reference for the Mirror SDK API
- [CLI Command Reference](./cli-command-reference.md): Comprehensive guide to the Mirror CLI commands
- [Configuration File Format](./configuration-format.md): Specification of the mirror.toml file format
- [Common Workflows](./common-workflows.md): Examples of common workflows using the Mirror SDK and CLI

## Additional Resources

- [API Documentation](../target/doc/mirror_sdk/index.html): Generated API documentation (available after running `cargo doc`)
- [Examples](../examples/): Example code demonstrating the usage of the Mirror SDK and CLI
- [Integration Tests](../tests/): Tests demonstrating the integration between the Mirror SDK and CLI

## Generating API Documentation

You can generate the API documentation using Cargo:

```bash
# Generate documentation for all workspace members
cargo doc --workspace --no-deps --open
```

This will generate HTML documentation in the `target/doc` directory and open it in your default web browser.

## Contributing to Documentation

When contributing to the documentation, please follow these guidelines:

1. Use Markdown for all documentation files
2. Keep the documentation up-to-date with the code
3. Include examples for complex concepts
4. Use clear and concise language
5. Follow the existing documentation structure

## Documentation TODOs

- [ ] Add troubleshooting guide
- [ ] Add migration guide for upgrading from older versions
- [ ] Add more examples for advanced use cases
- [ ] Add diagrams for complex workflows