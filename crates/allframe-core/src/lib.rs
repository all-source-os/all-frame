//! # AllFrame Core
//!
//! The composable Rust API framework.
//!
//! AllFrame is the first Rust web framework designed, built, and evolved
//! exclusively through Test-Driven Development (TDD).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use allframe_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     println!("AllFrame - One frame. Infinite transformations.");
//! }
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// Clean Architecture enforcement
#[cfg(feature = "di")]
pub mod arch;

/// CQRS + Event Sourcing
#[cfg(feature = "di")]
pub mod cqrs;

/// Router module for protocol-agnostic request handling
pub mod router;

/// Prelude module for convenient imports
///
/// Commonly used imports for AllFrame applications
pub mod prelude {
    pub use crate::router::{
        GraphQLAdapter, GrpcAdapter, GrpcRequest, GrpcStatus, ProtocolAdapter, RestAdapter,
        RestRequest, RestResponse, Router,
    };

    #[cfg(feature = "router")]
    pub use crate::router::{GraphQLConfig, GrpcConfig, RestConfig, RouterConfig, ServerConfig};
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_allframe_core_exists() {
        // This test verifies the crate compiles
        assert!(true);
    }
}
