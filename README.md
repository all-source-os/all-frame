# AllFrame

**The Composable Rust API Framework**

> *One frame to rule them all. Transform, compose, ignite.*

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange.svg)](https://www.rust-lang.org)
[![TDD](https://img.shields.io/badge/TDD-100%25-green.svg)](docs/current/PRD_01.md)
[![CQRS](https://img.shields.io/badge/CQRS-Complete-success.svg)](docs/announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)
[![Tests](https://img.shields.io/badge/tests-500%2B%20passing-brightgreen.svg)](docs/PROJECT_STATUS.md)
[![Routing](https://img.shields.io/badge/Protocol%20Agnostic-Complete-success.svg)](docs/phases/PROTOCOL_AGNOSTIC_ROUTING_COMPLETE.md)
[![MCP](https://img.shields.io/badge/MCP%20Server-Zero%20Bloat-success.svg)](docs/phases/MCP_ZERO_BLOAT_COMPLETE.md)
[![Offline](https://img.shields.io/badge/Offline--First-Complete-success.svg)](docs/current/UC_036_OFFLINE_FIRST.md)

---

## What is AllFrame?

AllFrame is a **complete Rust web framework** with a built-in HTTP/2 server, designed and evolved exclusively through Test-Driven Development (TDD). Every feature, macro, and public API has a failing test before it is written.

**AllFrame includes everything you need to build production APIs:**
- **Built-in HTTP/2 server** powered by Hyper - no external server required
- **Multi-protocol support** - REST, GraphQL, and gRPC from a single codebase
- **Zero external runtime dependencies** - only Tokio, Hyper, and std

We ship **composable crates** that give you exactly what you need:

- ✅ **Project Scaffolding** - `allframe ignite` creates Clean Architecture projects (v0.1)
- ✅ **Compile-time DI** - Dependency injection resolved at compile time (v0.2)
- ✅ **Auto OpenAPI 3.1** - API documentation generated automatically (v0.2)
- ✅ **CQRS + Event Sourcing** - Production-ready CQRS infrastructure (Phases 1-5) **[NEW!]**
  - CommandBus with 90% boilerplate reduction
  - ProjectionRegistry with automatic lifecycle
  - Event Versioning with auto-upcasting (95% reduction)
  - Saga Orchestration with automatic compensation (75% reduction)
  - Pluggable backends (in-memory, AllSource)
- ✅ **OpenTelemetry** - Tracing support built-in (v0.3)
- ✅ **Scalar API Documentation** - Beautiful OpenAPI docs (<50KB, 10x smaller than Swagger!) **[COMPLETE!]**
  - CDN version pinning for stability
  - SRI hashes for security
  - CORS proxy for "Try It" functionality
  - Custom theming and CSS
  - Production-ready with 42 tests
- ✅ **GraphQL Documentation** - Interactive GraphiQL playground (<100KB) **[COMPLETE!]**
  - GraphiQL 3.0 playground integration
  - Interactive schema explorer
  - WebSocket subscription support
  - Query history persistence
  - Dark/Light themes
  - Production-ready with 7 tests
- 🚧 **gRPC Documentation** - Interactive service explorer **[IN PROGRESS]**
  - Config builder and HTML generator complete (8 tests)
  - Browser-side grpc-web client pending
- ✅ **Contract Testing** - Built-in contract test generators **[COMPLETE!]**
  - Automatic test generation from router
  - Schema validation framework
  - Coverage reporting (shows test coverage percentage)
  - Breaking change detection
  - Production-ready with 9 tests
- ✅ **Protocol-Agnostic Routing** - Write once, expose via REST, GraphQL & gRPC **[COMPLETE!]**
  - ✅ Full REST adapter with path parameters and HTTP methods
  - ✅ Full GraphQL adapter with queries, mutations, and schema generation
  - ✅ Full gRPC adapter with all streaming modes and proto generation
  - ✅ Single handler exposed via multiple protocols
  - ✅ Automatic schema generation (OpenAPI, GraphQL SDL, .proto)
  - ✅ Protocol-specific error handling
  - Production-ready with 78 tests across 5 phases
- ✅ **Native MCP Server** - LLMs can call your API as tools **[Separate Crate - 100% Zero Bloat!]**
  - ✅ Auto-discovery: Handlers automatically become MCP tools
  - ✅ JSON Schema generation and validation
  - ✅ Type coercion (string → number, boolean)
  - ✅ Tool listing and invocation
  - ✅ Claude Desktop integration ready
  - Separate `allframe-mcp` crate with 33 tests
  - **Zero overhead** when not used (opt-in only)
- ✅ **Resilience Patterns** - Production-ready retry, circuit breaker, rate limiting **[NEW!]**
  - RetryExecutor with exponential backoff and jitter
  - CircuitBreaker with Closed/Open/HalfOpen states
  - **KeyedCircuitBreaker** for per-resource isolation (database, API, etc.)
  - RateLimiter with token bucket (per-endpoint, per-user)
  - **Redis-backed RateLimiter** for distributed rate limiting
  - AdaptiveRetry that adjusts based on success rates
  - RetryBudget to prevent retry storms
  - `#[retry]`, `#[circuit_breaker]`, `#[rate_limited]` macros
- ✅ **Layered Authentication** - Protocol-agnostic auth primitives **[NEW!]**
  - Core `Authenticator` trait with zero dependencies
  - JWT validation (HS256, RS256, EdDSA) with `auth-jwt`
  - Axum extractors and middleware with `auth-axum`
  - gRPC interceptors with `auth-tonic`
  - `AuthContext<C>` for type-safe claims access
- ✅ **Security Utilities** - Safe logging for sensitive data **[NEW!]**
  - URL/credential obfuscation for logs
  - `#[derive(Obfuscate)]` with `#[sensitive]` field attribute
  - `Sensitive<T>` wrapper (Debug/Display always shows "***")
  - Smart header obfuscation (Authorization, Cookie, API keys)
- ✅ **Graceful Shutdown** - Production-ready shutdown utilities
  - `ShutdownAwareTaskSpawner` for named tasks with automatic cancellation
  - `GracefulShutdownExt` for cleanup orchestration with error handling
  - `spawn_with_result()` for tasks that return values
  - `ShutdownExt` trait for making any future cancellable
- ✅ **Offline-First Architecture** - Run fully offline on desktop and embedded devices **[NEW!]**
  - SQLite event store backend (WAL mode, zero network deps)
  - Offline circuit breaker with operation queuing and replay
  - Store-and-forward pattern for intermittent connectivity
  - Bidirectional projection sync with pluggable conflict resolution
  - Lazy DI initialization with concurrent warm-up
  - Saga compensation with file snapshots and SQLite savepoints
  - Embedded MCP server for local-only LLM tool dispatch
  - `allframe-tauri` crate for Tauri 2.x desktop integration
  - **Zero network dependencies** in offline builds (verified by CI)
- 📋 **LLM-powered code generation** - `allframe forge` CLI (command registered, implementation pending)

**Target**: Binaries < 8 MB, > 500k req/s (TechEmpower parity with Actix), and **100% test coverage enforced by CI**.

**Current Status**: **v0.1.16 - Offline-First Complete!** 500+ tests passing. SQLite event store, offline resilience, projection sync, embedded MCP, Tauri desktop integration, and lazy DI -- all with zero network dependencies!
**Latest**: [Offline-First Spec](docs/current/UC_036_OFFLINE_FIRST.md) - Run AllFrame applications fully offline on desktop and embedded devices!

---

## Quick Start

```bash
# Install AllFrame CLI
cargo install allframe

# Create a new project
allframe ignite my-api

# Run your API
cd my-api
cargo run

# Visit http://localhost:8080/swagger-ui
```

### Try the Examples

AllFrame includes comprehensive examples demonstrating all features:

```bash
# REST API example - Build REST APIs with AllFrame
cargo run --example rest_api

# GraphQL API example - Build GraphQL APIs with AllFrame
cargo run --example graphql_api

# gRPC API example - Build gRPC services with AllFrame
cargo run --example grpc_api

# Multi-Protocol example - Same handler, multiple protocols!
cargo run --example multi_protocol

# Resilience patterns - Retry, circuit breaker, rate limiting
cargo run --example resilience --features resilience

# Security utilities - Safe logging and obfuscation
cargo run --example security --features security

# Graceful shutdown patterns - Task spawning, cleanup, cancellation
cargo run -p allframe-core --example graceful_shutdown
cargo run -p allframe-core --example shutdown_patterns
```

See [examples/README.md](examples/README.md) for detailed documentation.

---

## Core Features

### 🔧 Compile-Time Dependency Injection

```rust
use allframe::prelude::*;

#[di_container]
struct AppContainer {
    user_repo: Arc<dyn UserRepository>,
    user_service: Arc<UserService>,
}

// Dependencies resolved at compile time - zero runtime overhead
```

### 📝 Beautiful API Documentation with Scalar

```rust
use allframe::prelude::*;
use allframe::router::{OpenApiGenerator, ScalarConfig, ScalarTheme};

// Generate OpenAPI 3.1 spec with server configuration
let spec = OpenApiGenerator::new("My API", "1.0.0")
    .with_server("http://localhost:3000", Some("Development"))
    .with_server("https://api.example.com", Some("Production"))
    .generate(&router);

// Configure Scalar UI (10x smaller than Swagger!)
let config = ScalarConfig::new()
    .theme(ScalarTheme::Dark)
    .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
    .proxy_url("https://proxy.scalar.com"); // Enable "Try It" functionality

let html = scalar_html(&config, "My API", &spec);
// Beautiful, interactive docs at /docs
```

**Features**:
- 📦 <50KB bundle (vs 500KB for Swagger UI)
- 🎨 Modern UI with dark mode by default
- ⚡ Interactive "Try It" functionality
- 🔒 CDN version pinning + SRI hashes
- 🎯 CORS proxy support

### 🎮 Beautiful GraphQL Documentation with GraphiQL

```rust
use allframe::prelude::*;
use allframe::router::{GraphiQLConfig, GraphiQLTheme, graphiql_html};

// Configure GraphiQL playground with all features
let config = GraphiQLConfig::new()
    .endpoint_url("/graphql")
    .subscription_url("ws://localhost:3000/graphql")  // WebSocket for subscriptions
    .theme(GraphiQLTheme::Dark)
    .enable_explorer(true)   // Interactive schema explorer
    .enable_history(true)    // Query history persistence
    .add_header("Authorization", "Bearer your-token-here");

let html = graphiql_html(&config, "My GraphQL API");
// Beautiful, interactive GraphQL playground at /graphql/playground
```

**Features**:
- 🎮 Interactive GraphQL playground (GraphiQL 3.0)
- 📚 Built-in schema explorer sidebar
- 🔄 WebSocket subscription support
- 📝 Query history with localStorage persistence
- 🎨 Dark/Light themes
- 🎯 Variables editor with JSON validation
- 🔒 Custom header configuration

See **[GraphQL Documentation Guide](docs/guides/GRAPHQL_DOCUMENTATION.md)** for complete setup with Axum, Actix, and Rocket.

### 🌐 Interactive gRPC Service Explorer

```rust
use allframe::prelude::*;
use allframe::router::{GrpcExplorerConfig, GrpcExplorerTheme, grpc_explorer_html};

// Configure gRPC Explorer with all features
let config = GrpcExplorerConfig::new()
    .server_url("http://localhost:50051")
    .enable_reflection(true)      // Auto-discover services
    .enable_tls(false)             // TLS for production
    .theme(GrpcExplorerTheme::Dark)
    .timeout_seconds(30)
    .add_header("Authorization", "Bearer your-token-here");

let html = grpc_explorer_html(&config, "My gRPC API");
// Interactive gRPC service explorer at /grpc/explorer
```

**Features**:
- 🌐 Interactive gRPC service browser
- 📡 Automatic service discovery via gRPC reflection
- 🔄 Support for all call types (unary, server/client/bidirectional streaming)
- 🎨 Dark/Light themes
- 🔒 TLS/SSL support
- ⏱️ Configurable timeouts
- 📝 Custom metadata headers

See example at `examples/grpc_docs.rs` for complete Tonic integration.

### ✅ Contract Testing

```rust
use allframe::router::{Router, ContractTester, ContractTestConfig};

let router = Router::new();

// Simple usage - test all routes
let results = router.generate_contract_tests();
assert!(results.all_passed());
println!("Coverage: {:.1}%", results.coverage);

// Advanced usage with configuration
let tester = ContractTester::with_config(
    &router,
    ContractTestConfig::new()
        .validate_requests(true)
        .validate_responses(true)
        .detect_breaking_changes(true)
        .fail_fast(false)
);

let results = tester.test_all_routes();
println!("Passed: {}/{}", results.passed, results.total);

// Test specific route
let result = router.test_route_contract("/users", "GET");
assert!(result.passed);
```

**Features**:
- ✅ Automatic test generation from router
- 📋 Schema validation (requests/responses)
- 🔍 Breaking change detection
- 📊 Coverage reporting
- 🎯 Test specific routes or all at once

### 🔄 Protocol-Agnostic Handlers

```rust
#[handler(protocols = ["rest", "graphql", "grpc"])]
async fn create_user(input: CreateUserInput) -> Result<User, Error> {
    // Same handler works as:
    // - POST /users (REST)
    // - mutation { createUser } (GraphQL)
    // - CreateUser(request) (gRPC)
}
```

### 🏛️ CQRS + Event Sourcing (85% Less Boilerplate!)

```rust
use allframe::prelude::*;

// Commands - 90% reduction (3 lines vs 30-40)
#[command_handler]
async fn create_user(cmd: CreateUserCommand) -> CommandResult<UserEvent> {
    Ok(vec![UserEvent::Created { user_id: cmd.user_id, email: cmd.email }])
}

// Projections - 90% reduction (5 lines vs 50+)
let registry = ProjectionRegistry::new(event_store);
registry.register("users", UserProjection::new()).await;
registry.rebuild("users").await?;

// Event Versioning - 95% reduction (5 lines vs 30-40)
registry.register_upcaster(AutoUpcaster::<V1, V2>::new()).await;
// Events automatically upcasted during replay!

// Sagas - 75% reduction (20 lines vs 100+)
let saga = SagaDefinition::new("transfer")
    .add_step(DebitStep { account: from, amount })
    .add_step(CreditStep { account: to, amount });
orchestrator.execute(saga).await?;
// Automatic compensation on failure!
```

**[Read the full announcement →](docs/announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)**

### 🤖 MCP Server (LLM Tool Calling)

Expose your AllFrame APIs as LLM-callable tools using the [Model Context Protocol](https://modelcontextprotocol.io).

**Installation:**

```toml
# Opt-in to MCP server (zero overhead if not used!)
[dependencies]
allframe = "0.1.16"       # Core framework
allframe-mcp = "0.1.16"   # MCP server - separate crate for zero bloat
tokio = { version = "1.48", features = ["full"] }
```

**Quick Start:**

```rust
use allframe::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() {
    // Create router with handlers
    let mut router = Router::new();
    router.register("get_user", |user_id: String| async move {
        format!("User: {}", user_id)
    });
    router.register("create_order", |product: String| async move {
        format!("Order created for: {}", product)
    });

    // Handlers automatically become LLM-callable tools!
    let mcp = McpServer::with_router(router);

    // List available tools
    let tools = mcp.list_tools();
    println!("Available tools: {}", tools.len());

    // Call a tool
    let result = mcp.call_tool(
        "get_user",
        serde_json::json!({"user_id": "123"})
    ).await;
}
```

**Features:**
- ✅ Auto-discovery: Every handler becomes an MCP tool
- ✅ JSON Schema generation and validation
- ✅ Type coercion (string → number, boolean)
- ✅ **100% zero overhead** when not used (opt-in only)
- ✅ Flexible deployment (standalone, embedded, serverless)

**Usage Patterns:**
- 📱 Standalone MCP server binary for Claude Desktop
- 🌐 Embedded in web applications (Axum, Actix, Rocket)
- ☁️ Serverless deployment (AWS Lambda, etc.)

**Documentation:**
- [allframe-mcp README](crates/allframe-mcp/README.md) - Complete usage guide
- [MCP Distribution Model](docs/MCP_DISTRIBUTION_MODEL.md) - Library vs binary distribution
- [Example: STDIO Server](crates/allframe-mcp/examples/mcp_stdio_server.rs) - Full implementation

### 🔐 Layered Authentication

Protocol-agnostic authentication with zero-bloat feature flags:

```rust
use allframe::auth::{Authenticator, AuthError, JwtValidator, JwtConfig};

// Configure JWT validation
let config = JwtConfig::hs256("your-secret-key")
    .with_issuer("your-app")
    .with_audience("your-api");

let validator = JwtValidator::<MyClaims>::new(config);

// Or load from environment
let config = JwtConfig::from_env()?; // JWT_SECRET, JWT_ALGORITHM, etc.
```

**Axum Integration:**

```rust
use allframe::auth::{AuthLayer, AuthenticatedUser, JwtValidator};

// Add auth middleware
let app = Router::new()
    .route("/protected", get(protected_handler))
    .layer(AuthLayer::new(validator));

// Extract authenticated user in handler
async fn protected_handler(
    AuthenticatedUser(claims): AuthenticatedUser<MyClaims>,
) -> String {
    format!("Hello, {}!", claims.sub)
}
```

**gRPC Integration:**

```rust
use allframe::auth::{AuthInterceptor, GrpcAuthExt};

let interceptor = AuthInterceptor::new(validator);
let service = MyServiceServer::with_interceptor(impl, interceptor);

// In your gRPC handler
async fn my_method(&self, request: Request<Input>) -> Result<Response<Output>, Status> {
    let claims = request.require_auth::<MyClaims>()?;
    // ...
}
```

**Features:**
- 🔑 Core traits with zero dependencies (`auth`)
- 🎫 JWT validation: HS256, RS256, EdDSA (`auth-jwt`)
- 🌐 Axum extractors and middleware (`auth-axum`)
- 📡 gRPC interceptors (`auth-tonic`)
- 🔒 Type-safe claims with `AuthContext<C>`

### 🛡️ Resilience Patterns

Production-ready retry, circuit breaker, and rate limiting for robust microservices:

```rust
use allframe::resilience::{RetryConfig, RetryExecutor, CircuitBreaker, RateLimiter};
use std::time::Duration;

// Retry with exponential backoff
let config = RetryConfig::new(3)
    .with_initial_interval(Duration::from_millis(100))
    .with_max_interval(Duration::from_secs(5))
    .with_multiplier(2.0);

let executor = RetryExecutor::new(config);
let result = executor.execute("fetch_data", || async {
    external_api.fetch().await
}).await;

// Circuit breaker for fail-fast behavior
let cb = CircuitBreaker::new("payment_service", CircuitBreakerConfig::new(5));
let result = cb.call(|| async { payment_api.process().await }).await;

// Rate limiting (100 req/s with burst of 10)
let limiter = RateLimiter::new(100, 10);
if limiter.check().is_ok() {
    handle_request().await;
}
```

**Or use attribute macros:**

```rust
use allframe::{retry, circuit_breaker, rate_limited};

#[retry(max_retries = 3, initial_interval_ms = 100)]
async fn fetch_user(id: &str) -> Result<User, Error> {
    api.get_user(id).await  // Automatically retried!
}

#[circuit_breaker(failure_threshold = 5, timeout_secs = 30)]
async fn call_payment() -> Result<Payment, Error> {
    payment_api.process().await  // Fails fast when circuit is open!
}
```

**Features:**
- 🔄 Retry with exponential backoff and jitter
- ⚡ Circuit breaker (Closed/Open/HalfOpen states)
- 🔑 **KeyedCircuitBreaker** for per-resource isolation (database, API endpoints)
- 🎯 Rate limiting (token bucket with burst support)
- 🌐 **Redis-backed RateLimiter** for distributed rate limiting
- 📊 AdaptiveRetry (adjusts based on success rate)
- 🛡️ RetryBudget (prevents retry storms)
- 🔑 KeyedRateLimiter (per-endpoint, per-user limits)

### 🔒 Security Utilities

Safe logging utilities to prevent credential leaks:

```rust
use allframe::security::{obfuscate_url, obfuscate_api_key, Sensitive};
use allframe::Obfuscate;
use allframe::security::Obfuscate as ObfuscateTrait;

// URL obfuscation
let url = "https://user:pass@api.example.com/v1/users?token=secret";
println!("Connecting to: {}", obfuscate_url(url));
// Output: "https://api.example.com/***"

// API key obfuscation
let key = "sk_live_1234567890abcdef";
println!("Using key: {}", obfuscate_api_key(key));
// Output: "sk_l***cdef"

// Sensitive wrapper (Debug/Display always shows ***)
let password = Sensitive::new("super_secret");
println!("{:?}", password);  // Output: Sensitive(***)

// Derive macro for structs
#[derive(Obfuscate)]
struct DbConfig {
    host: String,
    #[sensitive]
    password: String,
}

let config = DbConfig { host: "db.example.com".into(), password: "secret".into() };
println!("{}", config.obfuscate());
// Output: DbConfig { host: "db.example.com", password: *** }
```

**Features:**
- 🔗 URL obfuscation (strips credentials, paths, queries)
- 🔑 API key obfuscation (shows prefix/suffix only)
- 📝 Header obfuscation (Authorization, Cookie, etc.)
- 🏷️ `Sensitive<T>` wrapper type
- ⚙️ `#[derive(Obfuscate)]` with `#[sensitive]` fields

---

## Why AllFrame?

| Feature | AllFrame | Actix | Axum | Rocket |
|---------|----------|-------|------|--------|
| TDD-First | ✅ 100% | ❌ | ❌ | ❌ |
| Compile-time DI | ✅ | ❌ | ❌ | ❌ |
| Auto OpenAPI 3.1 | ✅ | 🟡 Manual | 🟡 Manual | 🟡 Manual |
| **CQRS + Event Sourcing** | ✅ **Built-in** | ❌ | ❌ | ❌ |
| **CommandBus** | ✅ **90% less code** | ❌ | ❌ | ❌ |
| **Saga Orchestration** | ✅ **Auto compensation** | ❌ | ❌ | ❌ |
| **Resilience Patterns** | ✅ **Built-in** | 🟡 External | 🟡 External | ❌ |
| **Layered Auth** | ✅ **Protocol-agnostic** | 🟡 External | 🟡 External | 🟡 External |
| **Safe Logging** | ✅ **Built-in** | ❌ | ❌ | ❌ |
| Protocol-agnostic | ✅ | ❌ | ❌ | ❌ |
| MCP Server | ✅ **Zero Bloat** | ❌ | ❌ | ❌ |
| **Offline-First** | ✅ **Built-in** | ❌ | ❌ | ❌ |
| **Desktop/Tauri** | ✅ **Built-in** | ❌ | ❌ | ❌ |
| Zero runtime deps | ✅ | ❌ | ✅ | ❌ |
| Binary size | < 8 MB | ~12 MB | ~6 MB | ~10 MB |

---

## Why This Matters for LLMs and LLM Wrappers

AllFrame v0.1.16 is the first Rust web framework purpose-built for the way LLMs actually consume and produce APIs. Here's why this release matters if you're building LLM-powered applications:

### The Problem

LLM wrappers (ChatGPT plugins, Claude Desktop tools, Copilot extensions, custom agents) need to call APIs. Today, every wrapper reinvents the same plumbing: tool registration, schema generation, input validation, offline fallbacks. When the network drops, the LLM session breaks. When the user is on a plane, the desktop app is useless.

### What AllFrame Solves

**1. Native MCP Server with Local-Only Mode**

LLMs speak [Model Context Protocol](https://modelcontextprotocol.io). AllFrame handlers automatically become MCP tools -- no glue code. New in v0.1.16: the embedded MCP server runs without a network port, so a desktop app (Tauri, Electron) can dispatch tool calls in-process:

```rust
let mcp = McpServer::new(); // No network binding
mcp.register_tool("search_notes", |args| async move {
    // LLM calls this tool directly in-process
    Ok(search_local_db(args).await)
});
```

**2. Offline Event Sourcing for LLM Context**

LLM applications need durable context: conversation history, user preferences, tool results. AllFrame's SQLite event store persists everything locally with zero network dependencies:

```rust
let store = EventStore::new(SqliteEventStoreBackend::new("app.db")?);
store.append("conversation-1", vec![MessageReceived { content }]).await?;
// Works on a plane, in a tunnel, or air-gapped
```

**3. Store-and-Forward for Intermittent Connectivity**

When an LLM agent tries to call an external API but the network is down, AllFrame queues the operation and replays it when connectivity returns:

```rust
let saf = StoreAndForward::new(queue, probe);
let result = saf.execute("sync-to-cloud", || api.upload(data)).await;
// Returns CallResult::Queued when offline -- no crash, no lost data
```

**4. Conflict-Free Sync Between Devices**

Desktop LLM apps need to sync context across devices. AllFrame's projection sync engine handles bidirectional event replication with pluggable conflict resolution:

```rust
let engine = SyncEngine::with_resolver(local_store, cloud_store, LastWriteWins);
let report = engine.sync().await?; // Pushes local, pulls remote, resolves conflicts
```

**5. Zero Network Dependencies (Verified by CI)**

The `offline` feature compiles without `reqwest`, `redis`, `tonic`, `hyper`, or any network crate. This is enforced by CI -- if a network dependency leaks in, the build fails. LLM wrapper developers can ship a fully self-contained binary.

### Who Benefits

- **Claude Desktop / ChatGPT plugin builders** -- native MCP tool dispatch without network overhead
- **Local-first AI apps** (Obsidian plugins, IDE extensions, personal assistants) -- SQLite-backed event sourcing with offline resilience
- **Tauri / Electron desktop apps** -- `allframe-tauri` crate for IPC handler dispatch
- **Edge / embedded AI** -- air-gapped deployments with store-and-forward sync
- **Multi-device LLM agents** -- bidirectional sync with conflict resolution

---

## Installation

### As a Library

```toml
[dependencies]
allframe = "0.1.16"
```

### As a CLI Tool

```bash
cargo install allframe
```

---

## Example: Hello World

```rust
use allframe::prelude::*;

#[allframe::main]
async fn main() {
    let app = App::new()
        .route("/hello", get(hello_handler));

    app.run().await;
}

#[api_handler]
async fn hello_handler() -> &'static str {
    "Hello, AllFrame!"
}
```

**Run:**
```bash
cargo run
```

**OpenAPI docs automatically available at:**
- http://localhost:8080/swagger-ui
- http://localhost:8080/openapi.json

---

## Feature Flags

AllFrame uses Cargo feature flags to minimize bloat - you only pay for what you use:

```toml
[dependencies]
allframe = { version = "0.1.16", features = ["di", "openapi"] }
```

### Core Features

| Feature | Description | Binary Impact | Default |
|---------|-------------|---------------|---------|
| `di` | Compile-time dependency injection | +0KB | ✅ |
| `openapi` | Auto OpenAPI 3.1 + Swagger UI | +0KB | ✅ |
| `router` | Protocol-agnostic routing + TOML config | +50KB | ✅ |

### Router Features (Protocol Support)

| Feature | Description | Binary Impact | Default |
|---------|-------------|---------------|---------|
| `router-graphql` | Production GraphQL (async-graphql, GraphiQL) | +2MB | ❌ |
| `router-grpc` | Production gRPC (tonic, streaming, reflection) | +3MB | ❌ |
| `router-full` | Both GraphQL + gRPC production adapters | +5MB | ❌ |
| `resilience` | Retry, circuit breaker, rate limiting | +120KB | ❌ |
| `resilience-keyed` | KeyedCircuitBreaker for per-resource isolation | +10KB | ❌ |
| `resilience-redis` | Redis-backed distributed rate limiting | +50KB | ❌ |
| `auth` | Core authentication traits (zero deps) | +5KB | ❌ |
| `auth-jwt` | JWT validation (HS256, RS256, EdDSA) | +80KB | ❌ |
| `auth-axum` | Axum extractors and middleware | +20KB | ❌ |
| `auth-tonic` | gRPC interceptors | +15KB | ❌ |
| `security` | Safe logging, obfuscation utilities | +30KB | ❌ |

### CQRS Features (✅ Complete - Phases 1-5)

| Feature | Description | Reduction | Default |
|---------|-------------|-----------|---------|
| `cqrs` | CQRS + Event Sourcing infrastructure | 85% avg | ✅ |
| `cqrs-sqlite` | SQLite event store (WAL mode, zero network deps) | - | ❌ |
| `cqrs-allsource` | AllSource backend (embedded DB) | - | ❌ |
| `cqrs-postgres` | ⚠️ **DEPRECATED** - Use `cqrs-allsource` | - | ❌ |
| `cqrs-rocksdb` | RocksDB backend (planned) | - | ❌ |
| `vector-search` | Vector similarity search (fastembed + HNSW) | +5MB | ❌ |
| `keyword-search` | Full-text keyword search (tantivy BM25) | +2MB | ❌ |

### Offline-First Features (✅ NEW in v0.1.16)

| Feature | Description | Binary Impact | Default |
|---------|-------------|---------------|---------|
| `cqrs-sqlite` | SQLite event store backend | +1MB | ❌ |
| `offline` | Full offline bundle (cqrs + sqlite + di + security) | +1.5MB | ❌ |

**What you get**:
- CommandBus with automatic validation (90% reduction)
- ProjectionRegistry with automatic lifecycle (90% reduction)
- Event Versioning with auto-upcasting (95% reduction)
- Saga Orchestration with automatic compensation (75% reduction)
- Pluggable backends (in-memory, AllSource, custom)

### MCP Server (Separate Crate)

MCP (Model Context Protocol) is now a **separate crate** for 100% zero bloat:

```toml
# Only add if you need LLM integration
[dependencies]
allframe-mcp = "0.1.16"
```

**Benefits:**
- ✅ **100% zero overhead** when not used
- ✅ No feature flags needed
- ✅ No compilation impact on core
- ✅ Independent versioning

See [MCP Zero-Bloat Strategy](docs/phases/MCP_ZERO_BLOAT_COMPLETE.md) for details.

### Tauri Desktop Plugin (Separate Crate - NEW!)

```toml
# For Tauri 2.x desktop applications
[dependencies]
allframe-tauri = "0.1.15"
```

**Benefits:**
- ✅ IPC handler dispatch for Tauri commands
- ✅ Works fully offline with `cqrs-sqlite`
- ✅ In-process handler execution (no HTTP server needed)

See [allframe-tauri README](crates/allframe-tauri/README.md) for details.

**💡 Tip:** Start minimal and add features as needed. See [docs/guides/FEATURE_FLAGS.md](docs/guides/FEATURE_FLAGS.md) for detailed guidance.

### Examples

**Minimal REST API:**
```toml
allframe = { version = "0.1.16", default-features = false, features = ["router"] }
```

**Production GraphQL API:**
```toml
allframe = { version = "0.1.16", features = ["router-graphql"] }
```

**Multi-Protocol Gateway:**
```toml
allframe = { version = "0.1.16", features = ["router-full"] }
```

---

## Documentation

### Project Documentation
- 📊 **[Project Status](docs/PROJECT_STATUS.md)** - Current status, roadmap, metrics
- 🚀 **[ROADMAP.md](docs/current/ROADMAP.md)** - Complete roadmap to v1.0
- 🔮 **[IGNITE_VISION.md](docs/current/IGNITE_VISION.md)** - Microservice generator vision
- 📋 **[Original PRD](docs/current/PRD_01.md)** - Product requirements (PRIMARY SOURCE)
- 📋 **[Router + Docs PRD](docs/current/PRD_ROUTER_DOCS.md)** - Phase 6 planning (Next major phase)
- 📑 **[Documentation Index](docs/INDEX.md)** - Complete documentation catalog

### API Documentation (✅ Complete - All Protocols!)
- 🎉 **[Scalar Integration Complete](docs/phases/SCALAR_INTEGRATION_COMPLETE.md)** - 10x smaller than Swagger!
- 📘 **[Scalar Documentation Guide](docs/guides/SCALAR_DOCUMENTATION.md)** - Complete REST API docs guide
- 📘 [Example: Scalar Docs](crates/allframe-core/examples/scalar_docs.rs) - Production-ready example
- 🎉 **[GraphQL Documentation Guide](docs/guides/GRAPHQL_DOCUMENTATION.md)** - Interactive GraphiQL playground
- 📘 [Example: GraphQL Docs](crates/allframe-core/examples/graphql_docs.rs) - Complete GraphQL setup
- 🎉 **gRPC Service Explorer** - Interactive gRPC documentation
- 📘 [Example: gRPC Docs](crates/allframe-core/examples/grpc_docs.rs) - Complete gRPC setup with Tonic
- 📘 [Binary Size Monitoring](docs/phases/BINARY_SIZE_MONITORING_COMPLETE.md) - All binaries < 2MB

### CQRS Infrastructure (✅ Complete)
- 🎉 **[CQRS Complete Announcement](docs/announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)** - 85% avg reduction!
- 📘 [Phase 1: AllSource Integration](docs/phases/PHASE1_COMPLETE.md)
- 📘 [Phase 2: CommandBus](docs/phases/PHASE2_COMPLETE.md) - 90% reduction
- 📘 [Phase 3: ProjectionRegistry](docs/phases/PHASE3_COMPLETE.md) - 90% reduction
- 📘 [Phase 4: Event Versioning](docs/phases/PHASE4_COMPLETE.md) - 95% reduction
- 📘 [Phase 5: Saga Orchestration](docs/phases/PHASE5_COMPLETE.md) - 75% reduction

### Development Guides
- 🧪 [TDD Workflow](/.claude/TDD_CHECKLIST.md)
- 🏛️ [Clean Architecture Guide](/.claude/skills/rust-clean-architecture.md)
- 🎯 [Feature Flags Guide](docs/guides/FEATURE_FLAGS.md) - Minimize binary size
- 🔧 [API Reference](https://docs.rs/allframe) *(coming soon)*

---

## Contributing

AllFrame is **100% TDD-driven**. Before contributing:

1. Read the [PRD](docs/current/PRD_01.md)
2. Review the [TDD Checklist](/.claude/TDD_CHECKLIST.md)
3. Ensure **100% test coverage** for all changes
4. Follow the [Clean Architecture Guide](/.claude/skills/rust-clean-architecture.md)

**Every commit must contain at least one new failing test.**

```bash
# Clone the repo
git clone https://github.com/all-source-os/all-frame.git
cd all-frame

# Run tests (must pass)
cargo test

# Check coverage (must be 100%)
cargo llvm-cov

# Run quality checks
cargo clippy -- -D warnings
cargo fmt -- --check
```

See [CONTRIBUTING.md](CONTRIBUTING.md) for detailed guidelines. *(coming soon)*

---

## Roadmap

See **[ROADMAP.md](docs/current/ROADMAP.md)** for the complete vision and **[Project Status](docs/PROJECT_STATUS.md)** for current progress.

### In Progress

- [ ] **gRPC Documentation** - Interactive service explorer with grpc-web client (config/HTML generator done, browser JS is a stub)
- [ ] **`allframe forge`** - LLM-powered code generation CLI (command exists, implementation pending)

### Planned

- TechEmpower benchmarks (JSON serialization, query performance)
- Production runtime integration (Axum, Actix, Rocket)
- VS Code extension
- API versioning
- Security audit and 1.0 release preparation

**[Read the full roadmap →](docs/current/ROADMAP.md)** | **[View Ignite Vision →](docs/current/IGNITE_VISION.md)**

---

## Philosophy

### Test-Driven Development (TDD)

**We do not write a single line of implementation until a test fails for it.**

```rust
// 1. RED - Write failing test
#[test]
fn test_user_creation() {
    let user = User::new("test@example.com");
    assert!(user.is_ok());
}

// 2. GREEN - Minimal implementation
pub struct User { email: String }
impl User {
    pub fn new(email: impl Into<String>) -> Result<Self, Error> {
        Ok(Self { email: email.into() })
    }
}

// 3. REFACTOR - Improve while keeping tests passing
```

### Zero Runtime Dependencies

AllFrame only depends on:
- **Tokio** - Async runtime
- **Hyper** - HTTP server
- **std** - Rust standard library

**No hidden bloat. No dependency hell.**

### Clean Architecture

Dependencies flow inward only:

```
Presentation → Application → Domain ← Infrastructure
```

This is **enforced at compile time** by AllFrame's macros.

---

## Performance

AllFrame targets **TechEmpower Round 23** benchmarks in future releases:

| Metric | Target | Status |
|--------|--------|--------|
| JSON serialization | > 500k req/s | 📋 Planned |
| Single query | > 100k req/s | 📋 Planned |
| Multiple queries | > 50k req/s | 📋 Planned |
| Binary size | < 8 MB | ✅ Achieved (<2 MB) |

**Note**: Performance benchmarking is planned for Q2 2025. Current focus is on feature completeness and correctness. All functionality is production-ready with comprehensive test coverage (500+ tests passing).

---

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.

---

## Community

- 💬 [Discussions](https://github.com/all-source-os/all-frame/discussions)
- 🐛 [Issue Tracker](https://github.com/all-source-os/all-frame/issues)
- 📢 [Twitter](https://twitter.com/allframe_rs) *(coming soon)*
- 📧 [Discord](https://discord.gg/allframe) *(coming soon)*

---

## Acknowledgments

AllFrame is inspired by:
- **Axum** - Ergonomic Rust web framework
- **Actix** - High-performance actor framework
- **NestJS** - Architectural patterns for Node.js
- **Clean Architecture** - Uncle Bob Martin
- **Transformers (Cybertron)** - The inspiration for our "transform, compose, ignite" tagline

---

**AllFrame. One frame. Infinite transformations.**

*Built with TDD, from day zero.*
