# Product Context

This file provides a high-level overview of the project and the expected product that will be created. Initially it is based upon projectBrief.md (if provided) and all other available project-related information in the working directory. This file is intended to be updated as the project evolves, and should be used to inform all other modes of the project's goals and context.
2025-05-22 20:00:18 - Initial creation of Memory Bank.

## Project Goal

* Design a CLI tool called "mctl" (Mirror Control) that leverages the mirror-sdk to manage repositories
* Create a comprehensive design document covering command structure, user experience, and implementation strategy
* Ensure the CLI covers all major functionality of the mirror-sdk

## Key Features

* Initialize mirror.toml files
* Add, remove, and update repositories
* List repositories with filtering by tags
* Show repository details
* Tag repositories
* Provide a user-friendly command-line interface

## Overall Architecture

* CLI tool built on top of the mirror-sdk
* Command-line interface with intuitive commands and subcommands
* Integration with the mirror-sdk's functionality
* Focus on user experience with helpful error messages and formatted output