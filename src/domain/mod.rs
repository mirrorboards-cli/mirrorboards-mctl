//! # Domain Layer
//!
//! This layer contains the core business entities, interfaces, and validation logic.
//! It is independent of external frameworks and dependencies.
//!
//! The domain layer defines:
//! - Core entities representing business objects
//! - Interfaces (traits) that other layers implement
//! - Business rules and validation logic
//! - Error types and handling

pub mod repository;
pub mod configuration;
pub mod error;