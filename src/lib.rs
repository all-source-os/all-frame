//! # AllFrame
//!
//! **Complete Rust Web Framework with Built-in HTTP/2 Server**
//!
//! > *One frame to rule them all. Transform, compose, ignite.*
//!
//! AllFrame is a complete Rust web framework with a built-in HTTP/2 server,
//! designed and evolved exclusively through Test-Driven Development (TDD).
//!
//! ## Quick Start
//!
//! ```toml
//! [dependencies]
//! allframe = "0.1"
//! ```
//!
//! ## What's Included
//!
//! - **Built-in HTTP/2 Server** - Powered by Hyper, no external server needed
//! - **Multi-Protocol Support** - REST, GraphQL, and gRPC from one codebase
//! - **Compile-time DI** - Dependency injection resolved at compile time
//! - **Auto OpenAPI 3.1** - API documentation generated automatically
//! - **CQRS + Event Sourcing** - Production-ready infrastructure (85% less
//!   boilerplate)
//! - **Beautiful API Docs** - Scalar UI, GraphiQL, gRPC Explorer built-in
//! - **Zero External Dependencies** - Only Tokio, Hyper, and std
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
