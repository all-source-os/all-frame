# AllFrame Feature Flags

AllFrame uses Cargo feature flags to provide modular compilation and flexible deployment options. This allows you to include only the features you need, reducing binary size and compilation time.

## Quick Reference

```toml
# Minimal build (MVP adapters only)
cargo build --no-default-features

# Default build (recommended for most projects)
cargo build

# Production build with all features
cargo build --all-features

# Custom feature selection
cargo build --no-default-features --features "di,router,cqrs"
```

---

## Default Features

The following features are enabled by default:

```toml
default = ["di", "openapi", "router", "otel", "health"]
```

- `di` - Dependency Injection and Clean Architecture
- `openapi` - OpenAPI schema generation
- `router` - Protocol-agnostic routing (REST, GraphQL, gRPC)
- `otel` - OpenTelemetry observability
- `health` - Health check server (requires hyper)

**Note**: `cqrs` is **NOT** in the default features. You must explicitly enable it if needed.

---

## Core Features

### `di` - Dependency Injection

**Enables**: Dependency Injection container and Clean Architecture macros

**Macros**:
- `#[domain]` - Domain layer (pure business logic)
- `#[repository]` - Repository layer (data access)
- `#[use_case]` - Use case layer (application logic)
- `#[handler]` - Handler layer (interface)

**Example**:
```rust
use allframe_core::arch::*;

#[domain]
struct User {
    id: String,
    email: String,
}

#[repository]
#[async_trait::async_trait]
trait UserRepository: Send + Sync {
    async fn find(&self, id: &str) -> Option<User>;
}
```

**Binary Impact**: +50KB (macros + runtime)

---

### `openapi` - OpenAPI Schema Generation

**Enables**: Automatic OpenAPI 3.0 schema generation from handlers

**Macros**:
- `#[api_handler]` - Generates OpenAPI schema from function signature

**Example**:
```rust
use allframe_core::router::*;

#[api_handler(
    path = "/users",
    method = "GET",
    summary = "List all users"
)]
async fn list_users() -> Vec<User> {
    vec![]
}
```

**Binary Impact**: +30KB (schema generation)

---

### `router` - Protocol-Agnostic Routing

**Enables**: Configuration-based routing with REST, GraphQL, and gRPC support

**APIs**:
- `Router` - Central routing coordinator
- `RestAdapter`, `GraphQLAdapter`, `GrpcAdapter` - Protocol adapters
- `RouterConfig` - TOML-based configuration

**Example**:
```toml
# router.toml
[server]
host = "0.0.0.0"
port = 3000

[rest]
enabled = true
prefix = "/api"

[graphql]
enabled = true
path = "/graphql"
```

```rust
use allframe_core::router::*;

let config = RouterConfig::from_file("router.toml").await?;
let router = Router::from_config(config)?;
```

**Binary Impact**: +100KB (router + config parsing)

---

### `health` - Health Check Server

**Enables**: HTTP health check server infrastructure

**Dependencies**: `hyper`, `hyper-util`

**APIs**:
- `HealthServer` - HTTP server for health endpoints
- `HealthCheck` trait - Custom health check implementation
- `HealthReport` - Health status reporting
- `Dependency` / `DependencyStatus` - Dependency health tracking

**Example**:
```rust
use allframe_core::health::{HealthServer, HealthReport, OverallStatus};

let server = HealthServer::new("0.0.0.0:8080");
server.start().await?;
```

**Binary Impact**: +400KB (hyper + HTTP server)

---

### `otel` - OpenTelemetry Observability

**Enables**: Automatic distributed tracing, metrics, and context propagation

**Macros**:
- `#[traced]` - Automatic span creation and tracing

**APIs**:
- `SpanRecorder` - Test helper for span recording
- `MetricsRecorder` - Test helper for metrics
- `inject_context` / `extract_context` - W3C Trace Context
- `ExporterType` - Configure exporters (Jaeger, OTLP, Stdout)

**Example**:
```rust
use allframe_core::otel::*;

#[traced]
async fn get_user(user_id: String) -> Result<User, String> {
    // Automatically traced with span creation
}
```

**Binary Impact**: +80KB (tracing infrastructure)

---

### `otel-otlp` - Full OpenTelemetry Stack

**Enables**: Complete OpenTelemetry SDK with OTLP exporter for production observability

**Dependencies**: `otel`, `opentelemetry`, `opentelemetry_sdk`, `opentelemetry-otlp`, `tracing-opentelemetry`, `tracing-subscriber`

**Re-exports**:
- `opentelemetry` - Core OpenTelemetry API
- `opentelemetry_sdk` - SDK configuration
- `opentelemetry_otlp` - OTLP exporter
- `tracing_opentelemetry` - Tracing integration
- `tracing_subscriber` - Log/trace configuration

**Example**:
```rust
use allframe_core::{opentelemetry, opentelemetry_sdk, tracing_subscriber};
use opentelemetry_sdk::trace::TracerProvider;

let tracer = opentelemetry_otlp::new_pipeline()
    .tracing()
    .with_exporter(opentelemetry_otlp::new_exporter().tonic())
    .install_batch(opentelemetry_sdk::runtime::Tokio)?;
```

**Binary Impact**: +300KB (full OTEL stack)

---

## Optional Features

### `cqrs` - CQRS + Event Sourcing

**Status**: ⚠️ **NOT in default features** - Must be explicitly enabled

**Enables**: Command Query Responsibility Segregation and Event Sourcing patterns

**Macros**:
- `#[command]` - Mark command structs
- `#[query]` - Mark query structs
- `#[event]` - Mark event enums
- `#[command_handler]` - Command handler functions
- `#[query_handler]` - Query handler functions

**APIs**:
- `EventStore<E>` - Append-only event log (in-memory for MVP)
- `Projection` trait - Read model interface
- `Aggregate` trait - Event-sourced aggregate pattern
- `Saga` trait - Multi-aggregate transaction coordination
- `CommandBus` - Command dispatch (placeholder for MVP)
- `QueryBus` - Query dispatch

**Example**:
```rust
use allframe_core::cqrs::*;

#[derive(Clone)]
enum UserEvent {
    Created { user_id: String, email: String },
    EmailUpdated { new_email: String },
}

impl Event for UserEvent {}

let store = EventStore::new();
store.append("user-123", vec![
    UserEvent::Created {
        user_id: "user-123".to_string(),
        email: "test@example.com".to_string(),
    }
]).await?;
```

**Binary Impact**: +150KB (event store + CQRS runtime)

**Future enhancements** (planned):
- `cqrs-chronos` - Integration with Chronos event store
- `cqrs-postgres` - PostgreSQL persistence
- `cqrs-sqlite` - SQLite persistence

---

### `mcp` - Model Context Protocol

**Status**: 🚧 **Placeholder** - Not yet implemented

**Future**: Integration with Claude Desktop and AI tools

**Binary Impact**: TBD

---

## HTTP Client & Networking Features

### `http-client` - HTTP Client

**Enables**: Re-exports `reqwest` for making HTTP requests

**Dependencies**: `reqwest` (with `json` and `rustls-tls` features)

**Example**:
```rust
use allframe_core::reqwest;

async fn fetch_data() -> Result<String, reqwest::Error> {
    let response = reqwest::get("https://api.example.com/data").await?;
    response.text().await
}
```

**Binary Impact**: +500KB (reqwest + TLS)

---

## Router Production Features

### `router-graphql` - Production GraphQL

**Enables**: Full GraphQL implementation using `async-graphql`

**Dependencies**: `async-graphql`, `async-graphql-parser`

**APIs**:
- `GraphQLProductionAdapter` - Full GraphQL server
- Schema introspection
- GraphiQL interface

**Binary Impact**: +800KB (async-graphql + dependencies)

---

### `router-grpc` - Production gRPC

**Enables**: Full gRPC implementation using `tonic`

**Dependencies**: `tonic`, `prost`, `prost-types`, `tonic-reflection`, `tonic-health`, `tonic-build`, `tokio-stream`, `futures`

**APIs**:
- `GrpcProductionAdapter` - Full gRPC server
- Protobuf code generation
- Service reflection
- Health checking

**Binary Impact**: +1.2MB (tonic + protobuf)

---

### `router-grpc-tls` - gRPC with TLS Support

**Enables**: TLS/mTLS support for gRPC servers and clients

**Dependencies**: `router-grpc`, `tonic/tls-ring`, `tonic/tls-native-roots`, `rustls-pemfile`, `tokio-rustls`

**Example**:
```rust
use allframe_core::grpc::{GrpcServerBuilder, TlsConfig};

let tls_config = TlsConfig::new(
    include_bytes!("server.crt"),
    include_bytes!("server.key"),
);

let server = GrpcServerBuilder::new()
    .with_tls(tls_config)
    .build()?;
```

**Binary Impact**: +200KB (TLS libraries)

---

### `router-full` - All Production Features

**Enables**: Both `router-graphql` and `router-grpc`

**Binary Impact**: +2.0MB (combined)

---

## Observability & Metrics Features

### `metrics` - Prometheus Metrics

**Enables**: Prometheus metrics collection and exposition

**Dependencies**: `prometheus`

**Re-exports**: `prometheus` crate

**Example**:
```rust
use allframe_core::prometheus::{Counter, Opts, Registry};

let counter = Counter::with_opts(Opts::new("requests_total", "Total requests"))?;
let registry = Registry::new();
registry.register(Box::new(counter.clone()))?;

counter.inc();
```

**Binary Impact**: +100KB (prometheus)

---

## Caching Features

### `cache-memory` - In-Memory Caching

**Enables**: High-performance in-memory caching with `moka` and concurrent maps with `dashmap`

**Dependencies**: `moka` (with `future` feature), `dashmap`

**Re-exports**:
- `moka` - High-performance bounded cache
- `dashmap` - Concurrent HashMap

**Example**:
```rust
use allframe_core::moka::future::Cache;
use allframe_core::dashmap::DashMap;

// Moka cache with TTL
let cache: Cache<String, String> = Cache::builder()
    .max_capacity(10_000)
    .time_to_live(std::time::Duration::from_secs(300))
    .build();

// DashMap for concurrent access
let map: DashMap<String, i32> = DashMap::new();
map.insert("key".to_string(), 42);
```

**Binary Impact**: +150KB (moka + dashmap)

---

### `cache-redis` - Redis Caching

**Enables**: Redis client for distributed caching

**Dependencies**: `redis` (with `tokio-comp` and `connection-manager` features)

**Re-exports**: `redis` crate

**Example**:
```rust
use allframe_core::redis::{Client, AsyncCommands};

let client = Client::open("redis://127.0.0.1/")?;
let mut con = client.get_multiplexed_async_connection().await?;

con.set("my_key", "my_value").await?;
let value: String = con.get("my_key").await?;
```

**Binary Impact**: +200KB (redis client)

---

## Rate Limiting & Resilience Features

### `rate-limit` - Rate Limiting

**Enables**: Token bucket rate limiting with `governor`

**Dependencies**: `governor`

**Re-exports**: `governor` crate

**Example**:
```rust
use allframe_core::governor::{Quota, RateLimiter};
use std::num::NonZeroU32;

let limiter = RateLimiter::direct(Quota::per_second(NonZeroU32::new(10).unwrap()));

if limiter.check().is_ok() {
    // Request allowed
} else {
    // Rate limited
}
```

**Binary Impact**: +80KB (governor)

---

### `resilience` - Comprehensive Resilience Module

**Enables**: Production-ready retry, circuit breaker, and rate limiting patterns

**Dependencies**: `allframe-macros`, `backoff`, `governor`, `dashmap`, `parking_lot`, `rand`

**Macros**:
- `#[retry(max_retries = 3)]` - Wrap async functions with exponential backoff
- `#[circuit_breaker(failure_threshold = 5)]` - Fail-fast pattern for functions
- `#[rate_limited(rps = 100, burst = 10)]` - Token bucket rate limiting

**APIs**:
- `RetryConfig` - Configurable retry behavior (max_retries, intervals, randomization)
- `RetryExecutor` - Async retry execution with exponential backoff
- `RetryPolicy` trait - Custom retry decision logic
- `RetryBudget` - System-wide retry token management (prevents retry storms)
- `AdaptiveRetry` - Adjusts retry behavior based on success/failure rates
- `RateLimiter` - Token bucket rate limiting with burst support
- `AdaptiveRateLimiter` - Backs off when receiving external 429 responses
- `KeyedRateLimiter<K>` - Per-key rate limiting (per-endpoint, per-user)
- `CircuitBreaker` - Fail-fast pattern with Closed/Open/HalfOpen states
- `CircuitBreakerConfig` - Configurable thresholds and timeouts
- `CircuitBreakerManager` - Manages multiple circuit breakers by name

**Example (Retry)**:
```rust
use allframe_core::resilience::{RetryConfig, RetryExecutor};
use std::time::Duration;

let config = RetryConfig::new(3)
    .with_initial_interval(Duration::from_millis(100))
    .with_max_interval(Duration::from_secs(5))
    .with_multiplier(2.0);

let executor = RetryExecutor::new(config);
let result = executor.execute("fetch_data", || async {
    // Fallible operation
    Ok::<_, std::io::Error>("success")
}).await;
```

**Example (Circuit Breaker)**:
```rust
use allframe_core::resilience::{CircuitBreaker, CircuitBreakerConfig};
use std::time::Duration;

let config = CircuitBreakerConfig::new(5)
    .with_success_threshold(2)
    .with_timeout(Duration::from_secs(30));

let cb = CircuitBreaker::new("payment_service", config);
let result = cb.call(|| async { fetch_from_service().await }).await;
```

**Example (Rate Limiting)**:
```rust
use allframe_core::resilience::{RateLimiter, KeyedRateLimiter};

// Simple rate limiter: 100 requests/second, burst of 10
let limiter = RateLimiter::new(100, 10);
if limiter.check().is_ok() {
    // Request allowed
}

// Per-endpoint rate limiting
let keyed: KeyedRateLimiter<&str> = KeyedRateLimiter::new(10, 5);
keyed.set_limit("premium_api", 100, 20);  // Higher limit for premium
```

**Example (Attribute Macros)**:
```rust
use allframe_core::{retry, circuit_breaker, rate_limited};

#[retry(max_retries = 3, initial_interval_ms = 100)]
async fn fetch_user(id: &str) -> Result<User, Error> {
    // Automatically retried with exponential backoff
    api.get_user(id).await
}

#[circuit_breaker(failure_threshold = 5, timeout_secs = 30)]
async fn call_payment_service() -> Result<Payment, Error> {
    // Fails fast when circuit is open
    payment_api.process().await
}

#[rate_limited(rps = 100, burst = 10)]
async fn handle_request() -> Response {
    // Rate limited to 100 requests/second
    process_request().await
}
```

**Binary Impact**: +120KB (resilience patterns + dependencies)

---

### `security` - Security Utilities

**Enables**: Safe logging utilities for sensitive data obfuscation

**Dependencies**: `allframe-macros`, `url`

**Macros**:
- `#[derive(Obfuscate)]` - Auto-generate safe Debug/Display with `#[sensitive]` field attribute

**APIs**:
- `obfuscate_url(url)` - Strips credentials, path, and query from URLs
- `obfuscate_redis_url(url)` - Preserves host/port, hides auth
- `obfuscate_api_key(key)` - Shows prefix/suffix only (e.g., "sk_l***mnop")
- `obfuscate_header(name, value)` - Smart header obfuscation (Authorization, Cookie, etc.)
- `Obfuscate` trait - Custom obfuscation for user types
- `Sensitive<T>` wrapper - Debug/Display always shows "***"

**Example (Obfuscation Functions)**:
```rust
use allframe_core::security::{obfuscate_url, obfuscate_api_key, obfuscate_header};

let url = "https://user:pass@api.example.com/v1/users?token=secret";
println!("Safe: {}", obfuscate_url(url));
// Output: "https://api.example.com/***"

let key = "sk_live_1234567890abcdef";
println!("Key: {}", obfuscate_api_key(key));
// Output: "sk_l***cdef"

let auth_header = "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.secret";
println!("Auth: {}", obfuscate_header("Authorization", auth_header));
// Output: "Bearer ***"
```

**Example (Sensitive Wrapper)**:
```rust
use allframe_core::security::Sensitive;

let password = Sensitive::new("super_secret");
println!("{:?}", password);  // Output: Sensitive(***)
println!("{}", password);    // Output: ***

// Access the inner value when needed
let actual = password.as_inner();
```

**Example (Derive Macro)**:
```rust
use allframe_core::Obfuscate;
use allframe_core::security::Obfuscate as ObfuscateTrait;

#[derive(Obfuscate)]
struct DatabaseConfig {
    host: String,
    port: u16,
    #[sensitive]
    username: String,
    #[sensitive]
    password: String,
}

let config = DatabaseConfig {
    host: "db.example.com".to_string(),
    port: 5432,
    username: "admin".to_string(),
    password: "secret123".to_string(),
};

// Safe for logging!
println!("Connecting to: {}", config.obfuscate());
// Output: DatabaseConfig { host: "db.example.com", port: 5432, username: ***, password: *** }
```

**Binary Impact**: +30KB (url parsing + obfuscation)

---

## Utility Features

### `utils` - Common Utilities Bundle

**Enables**: Common utility crates for date/time, URLs, synchronization, and random numbers

**Dependencies**: `chrono`, `url`, `parking_lot`, `rand`

**Re-exports**:
- `chrono` - Date and time handling (with serde support)
- `url` - URL parsing and manipulation
- `parking_lot` - Efficient synchronization primitives (Mutex, RwLock)
- `rand` - Random number generation

**Example**:
```rust
use allframe_core::{chrono, url, parking_lot, rand};
use chrono::{DateTime, Utc};
use url::Url;
use parking_lot::Mutex;
use rand::Rng;

let now: DateTime<Utc> = Utc::now();
let api_url = Url::parse("https://api.example.com")?;
let counter = Mutex::new(0);
let random: u32 = rand::thread_rng().gen();
```

**Binary Impact**: +200KB (combined utilities)

---

## Feature Combinations

### Minimal Build (Prototype/MVP)
```bash
cargo build --no-default-features
```
**Size**: ~800KB
**Features**: Basic router with MVP adapters only
**Use case**: Prototypes, learning, embedded systems

---

### Default Build (Recommended)
```bash
cargo build
```
**Size**: ~1.1MB
**Features**: DI, OpenAPI, Router, OTEL
**Use case**: Most applications

---

### CQRS Application
```bash
cargo build --features cqrs
```
**Size**: ~1.3MB
**Features**: Default + CQRS/Event Sourcing
**Use case**: Event-sourced systems, audit logging

---

### Production API (REST only)
```bash
cargo build --features router
```
**Size**: ~1.1MB
**Features**: Default configuration
**Use case**: REST APIs

---

### Production API (GraphQL)
```bash
cargo build --features router-graphql
```
**Size**: ~1.9MB
**Features**: Default + Production GraphQL
**Use case**: GraphQL APIs

---

### Production API (gRPC)
```bash
cargo build --features router-grpc
```
**Size**: ~2.3MB
**Features**: Default + Production gRPC
**Use case**: Microservices, high-performance APIs

---

### Full Production Build
```bash
cargo build --all-features
```
**Size**: ~3.3MB
**Features**: Everything enabled
**Use case**: Enterprise applications, maximum flexibility

---

### Production API with Caching
```bash
cargo build --features "cache-memory,cache-redis"
```
**Features**: Default + Memory + Redis caching
**Use case**: High-traffic APIs requiring caching layer

---

### Resilient Microservice
```bash
cargo build --features "router-grpc,router-grpc-tls,rate-limit,resilience,metrics"
```
**Features**: gRPC + TLS + Rate limiting + Retry patterns + Prometheus metrics
**Use case**: Production microservices with full resilience

---

### Observable Service
```bash
cargo build --features "otel-otlp,metrics"
```
**Features**: Full OpenTelemetry + Prometheus
**Use case**: Services requiring comprehensive observability

---

## Feature Dependencies

```
di
  └─ allframe-macros

openapi
  └─ (no dependencies)

health
  ├─ hyper
  └─ hyper-util

router
  └─ toml (for config parsing)

router-graphql
  ├─ router
  ├─ async-graphql
  └─ async-graphql-parser

router-grpc
  ├─ router
  ├─ tonic
  ├─ prost
  ├─ prost-types
  ├─ tonic-build
  ├─ tonic-reflection
  ├─ tonic-health
  ├─ tokio-stream
  └─ futures

router-grpc-tls
  ├─ router-grpc
  ├─ tonic/tls-ring
  ├─ tonic/tls-native-roots
  ├─ rustls-pemfile
  └─ tokio-rustls

router-full
  ├─ router-graphql
  └─ router-grpc

otel
  ├─ allframe-macros
  └─ tracing

otel-otlp
  ├─ otel
  ├─ opentelemetry
  ├─ opentelemetry_sdk
  ├─ opentelemetry-otlp
  ├─ tracing-opentelemetry
  └─ tracing-subscriber

http-client
  └─ reqwest (json, rustls-tls)

metrics
  └─ prometheus

cache-memory
  ├─ moka (future)
  └─ dashmap

cache-redis
  └─ redis (tokio-comp, connection-manager)

rate-limit
  └─ governor

resilience
  ├─ allframe-macros
  ├─ backoff (tokio)
  ├─ governor
  ├─ dashmap
  ├─ parking_lot
  └─ rand

security
  ├─ allframe-macros
  └─ url

utils
  ├─ chrono (serde)
  ├─ url
  ├─ parking_lot
  └─ rand

cqrs
  └─ allframe-macros

cqrs-allsource
  ├─ cqrs
  └─ allsource-core

cqrs-postgres
  ├─ cqrs-allsource
  └─ allsource-core/postgres

cqrs-rocksdb
  ├─ cqrs-allsource
  └─ allsource-core/rocksdb-storage
```

---

## Testing with Feature Flags

### Test with default features
```bash
cargo test
```

### Test with specific features
```bash
cargo test --no-default-features --features "di,router"
```

### Test with CQRS
```bash
cargo test --features cqrs
```

### Test with all features
```bash
cargo test --all-features
```

### Test CQRS-specific tests
```bash
cargo test --features cqrs --test 06_cqrs_commands
```

---

## Configuration in Cargo.toml

### As a library user

```toml
[dependencies]
allframe = "0.1"  # Uses default features

# Or customize:
allframe = { version = "0.1", default-features = false, features = ["di", "router"] }

# Enable CQRS:
allframe = { version = "0.1", features = ["cqrs"] }

# Production GraphQL:
allframe = { version = "0.1", features = ["router-graphql"] }

# Everything:
allframe = { version = "0.1", features = ["cqrs", "router-full"] }
```

---

## Binary Size Comparison

| Configuration | Unoptimized | Optimized (--release) |
|---------------|-------------|----------------------|
| Minimal (no features) | ~800 KB | ~400 KB |
| Default | ~1.1 MB | ~550 KB |
| + CQRS | ~1.3 MB | ~650 KB |
| + router-graphql | ~1.9 MB | ~950 KB |
| + router-grpc | ~2.3 MB | ~1.1 MB |
| All features | ~3.3 MB | ~1.6 MB |

*Note: Sizes are approximate and may vary based on platform and Rust version*

---

## Recommendations

### For Prototypes
```toml
allframe = { version = "0.1", default-features = false, features = ["router"] }
```

### For REST APIs
```toml
allframe = { version = "0.1", features = [] }  # Use defaults
```

### For GraphQL APIs
```toml
allframe = { version = "0.1", features = ["router-graphql"] }
```

### For Microservices
```toml
allframe = { version = "0.1", features = ["router-grpc"] }
```

### For Event-Sourced Systems
```toml
allframe = { version = "0.1", features = ["cqrs"] }
```

### For Enterprise Applications
```toml
allframe = { version = "0.1", features = ["cqrs", "router-full"] }
```

---

## FAQ

### Q: Why isn't CQRS in the default features?

A: CQRS adds 150KB to binary size and is only needed for event-sourced systems. Most applications don't need it, so we keep it optional to reduce default binary size.

### Q: Can I use CQRS without the router?

A: Yes! Use:
```bash
cargo build --no-default-features --features cqrs
```

### Q: What's the difference between `router` and `router-graphql`?

A: `router` provides configuration-based routing with MVP adapters. `router-graphql` adds the full `async-graphql` library for production GraphQL support.

### Q: Do I need `otel` if I don't want tracing?

A: No, you can disable it:
```bash
cargo build --no-default-features --features "di,openapi,router"
```

### Q: Can I combine CQRS with GraphQL?

A: Absolutely! Use:
```bash
cargo build --features "cqrs,router-graphql"
```

---

## Future Feature Flags (Planned)

- `cqrs-sqlite` - SQLite persistence for CQRS
- `mcp` - Model Context Protocol integration
- `auth` - Authentication/authorization helpers
- `websockets` - WebSocket support

---

## See Also

- [CQRS + Chronos Assessment](./CQRS_CHRONOS_ASSESSMENT.md)
- [Milestone 0.4 Complete](./MILESTONE_0.4_COMPLETE.md)
- Main README.md
