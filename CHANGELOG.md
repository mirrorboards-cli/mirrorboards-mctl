# Changelog

All notable changes to MCTL will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.0] - 2025-05-15

### Added

- Initial release of MCTL with complete rewrite and improved architecture
- Secure SSH authentication with comprehensive error handling
- Repository-specific SSH key configuration
- Parallel processing of repository operations with configurable thread limits
- Multiple repository synchronization (clone/pull)
- Status checking across repositories
- Saving changes (commit/push) across repositories
- Repository filtering by tags
- TOML-based configuration with rich options
- Environment variable substitution in configuration
- Command-specific settings with repository-specific overrides
- Comprehensive error handling and diagnostics
- Detailed logging capabilities
- Clean, layered architecture:
  - Presentation Layer: CLI interface, command parsing
  - Application Layer: Command orchestration, business logic
  - Domain Layer: Core entities, repository operations
  - Infrastructure Layer: Git integration, filesystem operations

## Changelog Guidelines

When updating this changelog, please follow these guidelines:

- Add new entries under the "Unreleased" section
- Use the following categories as needed:
  - **Added**: New features
  - **Changed**: Changes to existing functionality
  - **Deprecated**: Features that will be removed in upcoming releases
  - **Removed**: Features that have been removed
  - **Fixed**: Bug fixes
  - **Security**: Vulnerabilities or security-related changes
- When releasing a new version:
  1. Change "Unreleased" to the version number and add the release date
  2. Add a new "Unreleased" section at the top
  3. Update the links at the bottom of the file

[Unreleased]: https://github.com/example/mctl/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/example/mctl/releases/tag/v0.1.0