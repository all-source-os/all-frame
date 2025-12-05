//! # AllFrame
//!
//! **The Composable Rust API Framework**
//!
//! > *One frame to rule them all. Transform, compose, ignite.*
//!
//! AllFrame is the first Rust web framework designed, built, and evolved
//! exclusively through Test-Driven Development (TDD).
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! allframe = "0.1"
//! ```
//!
//! ## Features
//!
//! - **Compile-time DI** - Dependency injection resolved at compile time
//! - **Auto OpenAPI 3.1** - API documentation generated automatically
//! - **CQRS + Event Sourcing** - Production-ready infrastructure (85% less
//!   boilerplate)
//! - **Protocol-Agnostic** - Write once, expose via REST, GraphQL, and gRPC
//! - **Beautiful API Docs** - Scalar UI, GraphiQL, gRPC Explorer built-in
//! - **Zero Runtime Deps** - Only Tokio, Hyper, and std
//!
//! ## Example
//!
//! ```rust,ignore
//! use allframe::prelude::*;
//!
//! #[allframe::main]
//! async fn main() {
//!     let app = App::new()
//!         .route("/hello", get(hello_handler));
//!
//!     app.run().await;
//! }
//!
//! #[api_handler]
//! async fn hello_handler() -> &'static str {
//!     "Hello, AllFrame!"
//! }
//! ```
//!
//! See the [GitHub repository](https://github.com/all-source-os/all-frame) for more examples.

// Re-export everything from allframe-core
pub use allframe_core::*;
