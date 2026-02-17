//! Domain layer contracts and business logic primitives.
//!
//! This module provides the building blocks for Clean Architecture domain
//! layers, including resilience contracts, business rules, and domain models.
//!
//! The domain layer defines WHAT needs to be done without knowing HOW it's
//! implemented.

#![allow(missing_docs)]

pub mod architectural_tests;
pub mod resilience;

// Re-export domain types for convenience
pub use resilience::*;
