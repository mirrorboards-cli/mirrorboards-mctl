# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- Initial workspace setup with mirror-sdk and mirror-cli crates
- Comprehensive documentation in the docs/ directory
- Integration tests demonstrating SDK and CLI working together
- GitHub Actions workflow for CI
- Examples showing SDK and CLI usage

## [0.1.0] - 2025-05-22

### Added
- Initial release of the Mirror Workspace
- mirror-sdk: Core library for managing mirror.toml configuration files
  - Parse and serialize mirror.toml configuration files
  - Manage repository configurations (add, remove, update)
  - Support file system operations for working with repositories
  - Handle configuration through default paths and environment variables
  - Provide a comprehensive error handling strategy
  - Expose a clean, well-documented public API
- mirror-cli: Command-line interface for the mirror-sdk
  - Create new mirror.toml configuration files
  - Add, remove, and update repositories
  - List repositories with optional tag filtering
  - Validate mirror.toml configurations
  - Colorful terminal output for better user experience
  - Specify mirror.toml file path via command-line argument or environment variable
- Documentation
  - SDK API Reference
  - CLI Command Reference
  - Configuration File Format
  - Common Workflows
- Examples
  - Basic SDK usage
  - SDK and CLI integration
- Integration tests
- CI/CD setup with GitHub Actions