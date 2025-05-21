# MCTL Command Specifications

This directory contains detailed specifications for all MCTL (Mirror Control) commands. MCTL is a Rust-based command-line interface (CLI) tool designed for efficient git repository synchronization and mirroring.

## Available Commands

- [`mctl add`](add.md) - Add a git repository to mirror.toml with specific configuration
- [`mctl sync`](sync.md) - Clone all repositories defined in mirror.toml
- [`mctl status`](status.md) - Check status of all repositories defined in mirror.toml
- [`mctl save`](save.md) - Commit and push changes in all repositories defined in mirror.toml

## Command Structure

Each command specification includes:

- Command name and syntax
- Description
- Examples
- Parameters/options
- Usage notes

## Configuration

MCTL uses a TOML configuration file (`mirror.toml`) to define repository relationships. See the [example configuration](../example/mirror.toml) for reference.