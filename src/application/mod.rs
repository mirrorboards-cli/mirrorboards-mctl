//! # Application Layer
//!
//! This layer contains the business logic and orchestration of commands.
//! It coordinates the interaction between presentation and domain layers.
//!
//! The application layer:
//! - Implements use cases by coordinating domain entities
//! - Orchestrates command execution flow
//! - Translates between domain and presentation models
//! - Handles application-level error management

pub mod commands;
pub mod orchestrator;
pub mod repository_orchestrator;
#[cfg(test)]
pub mod repository_orchestrator_tests;