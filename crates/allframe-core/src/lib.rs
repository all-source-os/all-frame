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
#[cfg(feature = "cqrs")]
pub mod cqrs;

/// OpenTelemetry automatic instrumentation
#[cfg(feature = "otel")]
pub mod otel;

/// Cache abstraction
pub mod cache;

/// Dependency injection infrastructure
#[cfg(feature = "di")]
pub mod di;

/// Health check infrastructure
#[cfg(feature = "health")]
pub mod health;

/// Router module for protocol-agnostic request handling
pub mod router;

/// Graceful shutdown utilities
pub mod shutdown;

/// Resilience patterns (retry, circuit breaker, rate limiting)
#[cfg(feature = "resilience")]
pub mod resilience;

/// Security utilities (obfuscation, safe logging)
#[cfg(feature = "security")]
pub mod security;

/// gRPC server infrastructure
#[cfg(feature = "router-grpc")]
pub mod grpc;

// ============================================================================
// Re-exported dependencies
// ============================================================================
// These re-exports allow consumers to use common dependencies without adding
// them explicitly to their Cargo.toml. This ensures version consistency and
// reduces boilerplate in downstream crates.

// ============================================================================
// Re-exported macros
// ============================================================================
/// Re-export circuit_breaker attribute macro
#[cfg(feature = "resilience")]
pub use allframe_macros::circuit_breaker;
/// Re-export rate_limited attribute macro
#[cfg(feature = "resilience")]
pub use allframe_macros::rate_limited;
/// Re-export retry attribute macro
#[cfg(feature = "resilience")]
pub use allframe_macros::retry;
/// Re-export GrpcError derive macro for automatic tonic::Status conversion
#[cfg(feature = "router-grpc")]
pub use allframe_macros::GrpcError;
/// Re-export HealthCheck derive macro for automatic health check implementation
#[cfg(feature = "di")]
pub use allframe_macros::HealthCheck;
/// Re-export Obfuscate derive macro for safe logging
#[cfg(feature = "security")]
pub use allframe_macros::Obfuscate;
/// Re-export async_graphql for GraphQL support
#[cfg(feature = "router-graphql")]
pub use async_graphql;
/// Re-export async_graphql_parser for GraphQL parsing
#[cfg(feature = "router-graphql")]
pub use async_graphql_parser;
/// Re-export async_trait for async trait definitions
pub use async_trait;
/// Re-export backoff for retry/resilience patterns
#[cfg(feature = "resilience")]
pub use backoff;
/// Re-export chrono for date/time handling
#[cfg(feature = "utils")]
pub use chrono;
/// Re-export dashmap for concurrent hash maps
#[cfg(feature = "cache-memory")]
pub use dashmap;
/// Re-export futures for async utilities
#[cfg(feature = "router-grpc")]
pub use futures;
/// Re-export governor for rate limiting
#[cfg(feature = "rate-limit")]
pub use governor;
/// Re-export hyper for HTTP primitives
#[cfg(feature = "health")]
pub use hyper;
/// Re-export moka for high-performance caching
#[cfg(feature = "cache-memory")]
pub use moka;
/// Re-export opentelemetry for full observability
#[cfg(feature = "otel-otlp")]
pub use opentelemetry;
/// Re-export opentelemetry_otlp for OTLP exporter
#[cfg(feature = "otel-otlp")]
pub use opentelemetry_otlp;
/// Re-export opentelemetry_sdk for SDK configuration
#[cfg(feature = "otel-otlp")]
pub use opentelemetry_sdk;
/// Re-export parking_lot for efficient synchronization primitives
#[cfg(feature = "utils")]
pub use parking_lot;
/// Re-export prometheus for metrics
#[cfg(feature = "metrics")]
pub use prometheus;
/// Re-export prost for protobuf support
#[cfg(feature = "router-grpc")]
pub use prost;
/// Re-export prost_types for well-known protobuf types
#[cfg(feature = "router-grpc")]
pub use prost_types;
/// Re-export rand for random number generation
#[cfg(feature = "utils")]
pub use rand;
/// Re-export redis for Redis client
#[cfg(feature = "cache-redis")]
pub use redis;
/// Re-export reqwest for HTTP client functionality
#[cfg(feature = "http-client")]
pub use reqwest;
/// Re-export serde for serialization
pub use serde;
/// Re-export serde_json for JSON handling
pub use serde_json;
/// Re-export tokio for async runtime
pub use tokio;
/// Re-export tokio_stream for async streams
#[cfg(feature = "router-grpc")]
pub use tokio_stream;
/// Re-export tonic for gRPC support
#[cfg(feature = "router-grpc")]
pub use tonic;
/// Re-export tonic_reflection for gRPC reflection
#[cfg(feature = "router-grpc")]
pub use tonic_reflection;
/// Re-export tracing for observability
#[cfg(feature = "otel")]
pub use tracing;
/// Re-export tracing_opentelemetry for tracing integration
#[cfg(feature = "otel-otlp")]
pub use tracing_opentelemetry;
/// Re-export tracing_subscriber for log configuration
#[cfg(feature = "otel-otlp")]
pub use tracing_subscriber;
/// Re-export url for URL parsing
#[cfg(feature = "utils")]
pub use url;

/// Prelude module for convenient imports
///
/// Commonly used imports for AllFrame applications
pub mod prelude {
    /// Re-export cache utilities
    pub use crate::cache::{Cache, CacheConfig, CacheKey, MemoryCache};
    /// Re-export DI utilities
    #[cfg(feature = "di")]
    pub use crate::di::{
        AsyncInit, AsyncInitWith, ContainerBuilder, DependencyError, DependencyRegistry, FromEnv,
        Provider, Scope,
    };
    /// Re-export gRPC server utilities
    #[cfg(feature = "router-grpc")]
    pub use crate::grpc::{GrpcServer, GrpcServerBuilder, GrpcServerError, TlsConfig};
    /// Re-export health check utilities
    #[cfg(feature = "health")]
    pub use crate::health::{
        Dependency, DependencyStatus, HealthCheck, HealthReport, HealthServer, OverallStatus,
        SimpleHealthCheck,
    };
    pub use crate::router::{
        GraphQLAdapter, GrpcAdapter, GrpcRequest, GrpcStatus, Method, ProtocolAdapter, RestAdapter,
        RestRequest, RestResponse, RouteMetadata, Router, ToJsonSchema,
    };
    #[cfg(feature = "router")]
    pub use crate::router::{GraphQLConfig, GrpcConfig, RestConfig, RouterConfig, ServerConfig};
    /// Re-export shutdown utilities
    pub use crate::shutdown::{
        GracefulShutdown, GracefulShutdownExt, ShutdownAwareTaskSpawner, ShutdownSignal,
        ShutdownToken,
    };
    /// Re-export GrpcError for convenient error handling
    #[cfg(feature = "router-grpc")]
    pub use crate::GrpcError;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_allframe_core_exists() {
        // This test verifies the crate compiles
        assert!(true);
    }
}
