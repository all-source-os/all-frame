//! # AllFrame Core
//!
//! [![Crates.io](https://img.shields.io/crates/v/allframe-core.svg)](https://crates.io/crates/allframe-core)
//! [![Documentation](https://docs.rs/allframe-core/badge.svg)](https://docs.rs/allframe-core)
//! [![License](https://img.shields.io/crates/l/allframe-core.svg)](https://github.com/all-source-os/all-frame)
//!
//! **The Composable Rust API Framework** - Protocol-agnostic routing, CQRS/ES,
//! resilience patterns, and beautiful API documentation.
//!
//! AllFrame is the first Rust web framework designed, built, and evolved
//! exclusively through **Test-Driven Development (TDD)** with 500+ tests.
//!
//! ## Features at a Glance
//!
//! | Feature | Description |
//! |---------|-------------|
//! | 🔀 **Protocol-Agnostic** | Write once, expose via REST, GraphQL, and gRPC |
//! | 📖 **Auto Documentation** | Scalar UI, GraphiQL, gRPC Explorer built-in |
//! | 🔄 **CQRS/Event Sourcing** | 85% boilerplate reduction with CommandBus, Projections, Sagas |
//! | 🛡️ **Resilience Patterns** | Retry, Circuit Breaker, Rate Limiting |
//! | 🔒 **Security Utilities** | Safe logging, credential obfuscation |
//! | 💉 **Compile-time DI** | Dependency injection resolved at compile time |
//! | 📊 **OpenTelemetry** | Automatic tracing and metrics |
//! | 📱 **Offline-First** | SQLite event store, sync engine, zero network deps |
//!
//! ## Quick Start
//!
//! Add to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! allframe = "0.1"
//! tokio = { version = "1", features = ["full"] }
//! ```
//!
//! ### Basic Router Example
//!
//! ```rust
//! use allframe_core::router::{Router, RestAdapter, ProtocolAdapter};
//!
//! #[tokio::main]
//! async fn main() {
//!     // Create a router
//!     let mut router = Router::new();
//!
//!     // Register handlers - works with any protocol!
//!     router.register("get_users", || async {
//!         r#"[{"id": 1, "name": "Alice"}]"#.to_string()
//!     });
//!
//!     router.register("create_user", || async {
//!         r#"{"id": 2, "name": "Bob"}"#.to_string()
//!     });
//!
//!     // Expose via REST
//!     let mut rest = RestAdapter::new();
//!     rest.route("GET", "/users", "get_users");
//!     rest.route("POST", "/users", "create_user");
//!
//!     println!("Router configured with {} handlers", 2);
//! }
//! ```
//!
//! ### Protocol-Agnostic Handler
//!
//! ```rust
//! use allframe_core::router::{Router, RestAdapter, GraphQLAdapter, GrpcAdapter};
//!
//! // Same handler, multiple protocols!
//! let mut router = Router::new();
//! router.register("get_user", || async {
//!     r#"{"id": 42, "name": "John"}"#.to_string()
//! });
//!
//! // REST: GET /users/42
//! let mut rest = RestAdapter::new();
//! rest.route("GET", "/users/:id", "get_user");
//!
//! // GraphQL: query { user(id: 42) { name } }
//! let mut graphql = GraphQLAdapter::new();
//! graphql.query("user", "get_user");
//!
//! // gRPC: UserService.GetUser
//! let mut grpc = GrpcAdapter::new();
//! grpc.unary("UserService", "GetUser", "get_user");
//! ```
//!
//! ## Feature Flags
//!
//! AllFrame uses feature flags to minimize binary size. Only enable what you
//! need:
//!
//! | Feature | Description | Default |
//! |---------|-------------|---------|
//! | `router` | Protocol-agnostic routing | ✅ |
//! | `router-graphql` | GraphQL adapter with async-graphql | ❌ |
//! | `router-grpc` | gRPC adapter with tonic | ❌ |
//! | `di` | Compile-time dependency injection | ✅ |
//! | `cqrs` | CQRS + Event Sourcing infrastructure | ✅ |
//! | `otel` | OpenTelemetry tracing | ✅ |
//! | `health` | Health check endpoints | ✅ |
//! | `resilience` | Retry, Circuit Breaker, Rate Limiting | ❌ |
//! | `security` | Safe logging, credential obfuscation | ❌ |
//! | `cqrs-sqlite` | SQLite event store (WAL mode) | ❌ |
//! | `offline` | Full offline bundle (cqrs + sqlite + di + security) | ❌ |
//!
//! ### Feature Examples
//!
//! ```toml
//! # Minimal REST API
//! allframe = { version = "0.1", default-features = false, features = ["router"] }
//!
//! # Full-stack with resilience
//! allframe = { version = "0.1", features = ["resilience", "security"] }
//!
//! # Multi-protocol gateway
//! allframe = { version = "0.1", features = ["router-graphql", "router-grpc"] }
//!
//! # Offline desktop app (zero network deps)
//! allframe = { version = "0.1", features = ["offline"] }
//! ```
//!
//! ## Module Overview
//!
//! - [`router`] - Protocol-agnostic request routing (REST, GraphQL, gRPC)
//! - [`shutdown`] - Graceful shutdown utilities
//! - [`cache`] - Caching infrastructure
//! - `cqrs` - CQRS + Event Sourcing (requires `cqrs` feature)
//! - `resilience` - Retry, Circuit Breaker, Rate Limiting (requires
//!   `resilience` feature)
//! - `security` - Safe logging and credential obfuscation (requires `security`
//!   feature)
//! - `di` - Compile-time dependency injection (requires `di` feature)
//! - `otel` - OpenTelemetry instrumentation (requires `otel` feature)
//! - `health` - Health check infrastructure (requires `health` feature)
//!
//! ## Examples
//!
//! See the [examples directory](https://github.com/all-source-os/all-frame/tree/main/crates/allframe-core/examples)
//! for complete working examples:
//!
//! - `scalar_docs.rs` - REST API with Scalar documentation
//! - `graphql_docs.rs` - GraphQL API with GraphiQL playground
//! - `resilience.rs` - Retry, Circuit Breaker, Rate Limiting
//! - `graceful_shutdown.rs` - Production shutdown handling
//!
//! ## Learn More
//!
//! - [GitHub Repository](https://github.com/all-source-os/all-frame)
//! - [Feature Flags Guide](https://github.com/all-source-os/all-frame/blob/main/docs/guides/FEATURE_FLAGS.md)
//! - [CQRS Documentation](https://github.com/all-source-os/all-frame/blob/main/docs/phases/PHASE5_COMPLETE.md)

#![deny(warnings)]
#![deny(missing_docs)]
#![deny(unsafe_code)]
// Enable doc_cfg for showing feature requirements on docs.rs
#![cfg_attr(docsrs, feature(doc_cfg))]

/// Domain layer contracts and business logic primitives.
/// This module provides the building blocks for Clean Architecture domain
/// layers, including resilience contracts, business rules, and domain models.
pub mod domain;

/// Application layer orchestration and use case implementations.
/// This module provides the orchestration layer that coordinates between
/// domain logic and infrastructure, including resilience orchestration,
/// transaction management, and business workflow coordination.
pub mod application;

/// Clean Architecture enforcement with compile-time dependency injection.
///
/// The `arch` module provides traits and utilities for enforcing Clean
/// Architecture patterns in your application. Use the `#[inject]` macro to wire
/// up dependencies.
///
/// # Example
///
/// ```rust,ignore
/// use allframe::arch::*;
///
/// #[inject]
/// struct MyService {
///     repo: Arc<dyn UserRepository>,
/// }
/// ```
#[cfg(feature = "di")]
#[cfg_attr(docsrs, doc(cfg(feature = "di")))]
pub mod arch;

/// CQRS + Event Sourcing infrastructure with 85% boilerplate reduction.
///
/// This module provides production-ready CQRS primitives:
/// - [`cqrs::CommandBus`] - Type-safe command dispatch (90% less code)
/// - [`cqrs::EventStore`] - Event storage with replay capability
/// - [`cqrs::ProjectionRegistry`] - Automatic projection updates (90% less
///   code)
/// - [`cqrs::SagaOrchestrator`] - Distributed transaction handling (75% less
///   code)
///
/// # Example
///
/// ```rust,ignore
/// use allframe::cqrs::{CommandBus, Event, EventStore};
///
/// #[derive(Clone)]
/// struct CreateUser { name: String }
///
/// let bus = CommandBus::new();
/// bus.dispatch(CreateUser { name: "Alice".into() }).await?;
/// ```
#[cfg(feature = "cqrs")]
#[cfg_attr(docsrs, doc(cfg(feature = "cqrs")))]
pub mod cqrs;

/// OpenTelemetry automatic instrumentation for distributed tracing.
///
/// Use the `#[traced]` macro to automatically instrument your functions:
///
/// ```rust,ignore
/// use allframe::otel::traced;
///
/// #[traced]
/// async fn fetch_user(id: &str) -> User {
///     // Automatically creates a span with function name
/// }
/// ```
#[cfg(feature = "otel")]
#[cfg_attr(docsrs, doc(cfg(feature = "otel")))]
pub mod otel;

/// Cache abstraction with in-memory and Redis backends.
///
/// Provides a unified caching interface with configurable TTL and eviction.
pub mod cache;

/// Compile-time dependency injection infrastructure.
///
/// Build dependency graphs that are resolved at compile time for zero runtime
/// overhead.
///
/// # Example
///
/// ```rust,ignore
/// use allframe::di::{ContainerBuilder, Provider};
///
/// let container = ContainerBuilder::new()
///     .register::<DatabasePool>()
///     .register::<UserRepository>()
///     .build();
/// ```
#[cfg(feature = "di")]
#[cfg_attr(docsrs, doc(cfg(feature = "di")))]
pub mod di;

/// Health check infrastructure for Kubernetes-ready services.
///
/// Provides liveness and readiness probes with dependency health aggregation.
///
/// # Example
///
/// ```rust,ignore
/// use allframe::health::{HealthServer, HealthCheck};
///
/// let server = HealthServer::new()
///     .add_check("database", db_check)
///     .add_check("cache", cache_check);
///
/// server.serve(8080).await;
/// ```
#[cfg(feature = "health")]
#[cfg_attr(docsrs, doc(cfg(feature = "health")))]
pub mod health;

/// Protocol-agnostic request routing for REST, GraphQL, and gRPC.
///
/// Write handlers once, expose them via any protocol:
///
/// # Example
///
/// ```rust
/// use allframe_core::router::{Router, RestAdapter, GraphQLAdapter, GrpcAdapter};
///
/// let mut router = Router::new();
/// router.register("get_user", || async { r#"{"id": 1}"#.to_string() });
///
/// // Same handler, three protocols!
/// let mut rest = RestAdapter::new();
/// rest.route("GET", "/users/:id", "get_user");
///
/// let mut graphql = GraphQLAdapter::new();
/// graphql.query("user", "get_user");
///
/// let mut grpc = GrpcAdapter::new();
/// grpc.unary("UserService", "GetUser", "get_user");
/// ```
///
/// Also includes documentation generators:
/// - `scalar_html` - Scalar UI for REST APIs
/// - `graphiql_html` - GraphiQL playground for GraphQL
/// - `grpc_explorer_html` - gRPC Explorer
pub mod router;

/// Graceful shutdown utilities for production services.
///
/// Handle SIGTERM/SIGINT signals and coordinate clean shutdown across tasks.
///
/// # Example
///
/// ```rust,ignore
/// use allframe::shutdown::{ShutdownSignal, GracefulShutdownExt};
///
/// let signal = ShutdownSignal::new();
///
/// // In your main loop
/// tokio::select! {
///     _ = server.run() => {},
///     _ = signal.recv() => {
///         server.perform_shutdown().await;
///     }
/// }
/// ```
pub mod shutdown;

/// Resilience patterns: Retry, Circuit Breaker, and Rate Limiting.
///
/// Production-ready patterns for fault-tolerant microservices:
///
/// # Example
///
/// ```rust,ignore
/// use allframe::resilience::{RetryExecutor, CircuitBreaker, RateLimiter};
///
/// // Retry with exponential backoff
/// let retry = RetryExecutor::new(RetryConfig::default());
/// let result = retry.execute("api_call", || async {
///     external_api.call().await
/// }).await;
///
/// // Circuit breaker for fail-fast
/// let cb = CircuitBreaker::new("payments", CircuitBreakerConfig::default());
/// let result = cb.call(|| payment_service.charge()).await;
///
/// // Rate limiting
/// let limiter = RateLimiter::new(100, 10); // 100 RPS, burst of 10
/// if limiter.check().is_ok() {
///     process_request().await;
/// }
/// ```
#[cfg(feature = "resilience")]
#[cfg_attr(docsrs, doc(cfg(feature = "resilience")))]
pub mod resilience;

/// Security utilities for safe logging and credential obfuscation.
///
/// Prevent accidental credential leaks in logs:
///
/// # Example
///
/// ```rust,ignore
/// use allframe::security::{obfuscate_url, Sensitive};
///
/// let url = "https://user:password@api.example.com/v1/users";
/// println!("Connecting to: {}", obfuscate_url(url));
/// // Output: "Connecting to: https://api.example.com"
///
/// let api_key = Sensitive::new("sk_live_abcd1234");
/// println!("Using key: {:?}", api_key);
/// // Output: "Using key: ***"
/// ```
#[cfg(feature = "security")]
#[cfg_attr(docsrs, doc(cfg(feature = "security")))]
pub mod security;

/// gRPC server infrastructure with TLS support.
///
/// Production-ready gRPC server with health checks and reflection.
#[cfg(feature = "router-grpc")]
#[cfg_attr(docsrs, doc(cfg(feature = "router-grpc")))]
pub mod grpc;

/// Authentication primitives with layered feature flags.
///
/// This module provides authentication infrastructure that can be used
/// independently or integrated with your web framework:
///
/// - **`auth`**: Core traits only (zero dependencies)
/// - **`auth-jwt`**: JWT validation with HS256/RS256 support
/// - **`auth-axum`**: Axum extractors and middleware
/// - **`auth-tonic`**: gRPC interceptors
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::auth::{JwtValidator, JwtConfig, Authenticator};
///
/// let validator = JwtValidator::<Claims>::new(
///     JwtConfig::hs256("secret").with_issuer("my-app")
/// );
///
/// let claims = validator.authenticate("eyJ...").await?;
/// ```
#[cfg(feature = "auth")]
#[cfg_attr(docsrs, doc(cfg(feature = "auth")))]
pub mod auth;

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
