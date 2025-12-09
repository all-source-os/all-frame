//! Template modules for project scaffolding
//!
//! This module contains all the template strings used when generating
//! new AllFrame projects with the `allframe ignite` command.
//!
//! Templates are organized by archetype:
//! - `basic`: Simple Clean Architecture project (default)
//! - `gateway`: API Gateway service with gRPC, resilience, and caching

pub mod basic;
pub mod gateway;

pub use basic::*;
