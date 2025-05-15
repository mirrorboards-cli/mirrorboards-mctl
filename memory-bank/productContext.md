# Product Context

This file provides a high-level overview of the project and the expected product that will be created. This file is intended to be updated as the project evolves and should be used to inform all other modes of the project's goals and context.
2025-05-15 10:35:54 - Initial creation based on code analysis.

## Project Goal

The goal is to rewrite the MCTL (Mirror Control) tool with an improved architecture that addresses the limitations of the current implementation, focusing on military-grade quality, reliability, and maintainability.

## Key Features

* Repository synchronization with Git SSH authentication
* Status checking across multiple repositories
* Saving (commit and push) changes across repositories
* TOML-based configuration for repository definitions
* Support for parallel processing of repositories
* Comprehensive logging and error handling
* Extensible command structure
* High test coverage

## Overall Architecture

The new architecture will follow a layered, modular approach with clear separation of concerns:

1. **Presentation Layer**: CLI interface, command parsing, user output
2. **Application Layer**: Command implementations, business logic
3. **Domain Layer**: Core entities, repository operations, interfaces
4. **Infrastructure Layer**: External system integrations (Git, filesystem)