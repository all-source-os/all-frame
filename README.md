# AllFrame

**The Composable Rust API Framework**

> *One frame to rule them all. Transform, compose, ignite.*

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.80%2B-orange.svg)](https://www.rust-lang.org)
[![TDD](https://img.shields.io/badge/TDD-100%25-green.svg)](docs/current/PRD_01.md)
[![CQRS](https://img.shields.io/badge/CQRS-Complete-success.svg)](docs/announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)
[![Tests](https://img.shields.io/badge/tests-81%20passing-brightgreen.svg)](docs/PROJECT_STATUS.md)

---

## What is AllFrame?

AllFrame is the **first Rust web framework designed, built, and evolved exclusively through Test-Driven Development (TDD)**. Every feature, macro, and public API has a failing test before it is written.

We ship **one crate** (`allframe-core`) that gives you, out of the box and with **zero external runtime dependencies**:

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
- ✅ **gRPC Documentation** - Interactive service explorer **[NEW!]**
  - gRPC reflection for auto-discovery
  - Service and method browser
  - Stream testing (unary, server, client, bidirectional)
  - TLS/SSL support
  - Custom metadata headers
  - Production-ready with 7 tests
- 🚧 **Protocol-agnostic routing** - REST ↔ GraphQL ↔ gRPC (v0.3 - in progress)
- 📋 **Contract Testing** - Built-in contract test generators (Phase 6 - planned)
- 📋 **Native MCP server** - LLMs can call your API as tools (v0.5 - planned)
- 📋 **LLM-powered code generation** - `allframe forge` CLI (v0.6 - planned)

**Target**: Binaries < 8 MB, > 500k req/s (TechEmpower parity with Actix), and **100% test coverage enforced by CI**.

**Current Status**: **Complete API Documentation Suite!** 145 tests passing. Beautiful docs for REST, GraphQL & gRPC.
**Latest**: [gRPC Service Explorer](crates/allframe-core/examples/grpc_docs.rs) - Interactive gRPC documentation!

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

```rust
// Your API is automatically available to Claude/GPT
// LLMs can discover and call your endpoints as tools

// No additional configuration needed!
```

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
| Protocol-agnostic | ✅ | ❌ | ❌ | ❌ |
| MCP Server | 📋 Planned | ❌ | ❌ | ❌ |
| Zero runtime deps | ✅ | ❌ | ✅ | ❌ |
| Binary size | < 8 MB | ~12 MB | ~6 MB | ~10 MB |

---

## Installation

### As a Library

```toml
[dependencies]
allframe = "0.1"
```

### As a CLI Tool

```bash
cargo install allframe-cli
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
allframe-core = { version = "0.1", features = ["di", "openapi"] }
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

### CQRS Features (✅ Complete - Phases 1-5)

| Feature | Description | Reduction | Default |
|---------|-------------|-----------|---------|
| `cqrs` | CQRS + Event Sourcing infrastructure | 85% avg | ✅ |
| `cqrs-allsource` | AllSource backend (embedded DB) | - | ❌ |
| `cqrs-postgres` | PostgreSQL backend (planned) | - | ❌ |
| `cqrs-rocksdb` | RocksDB backend (planned) | - | ❌ |

**What you get**:
- CommandBus with automatic validation (90% reduction)
- ProjectionRegistry with automatic lifecycle (90% reduction)
- Event Versioning with auto-upcasting (95% reduction)
- Saga Orchestration with automatic compensation (75% reduction)
- Pluggable backends (in-memory, AllSource, custom)

### Planned Features

| Feature | Description | Default |
|---------|-------------|---------|
| `otel` | OpenTelemetry auto-instrumentation | ✅ |
| `mcp` | Model Context Protocol server | ❌ |

**💡 Tip:** Start minimal and add features as needed. See [docs/feature-flags.md](docs/feature-flags.md) for detailed guidance.

### Examples

**Minimal REST API:**
```toml
allframe-core = { version = "0.1", default-features = false, features = ["router"] }
```

**Production GraphQL API:**
```toml
allframe-core = { version = "0.1", features = ["router-graphql"] }
```

**Multi-Protocol Gateway:**
```toml
allframe-core = { version = "0.1", features = ["router-full"] }
```

---

## Documentation

### Project Documentation
- 📊 **[Project Status](docs/PROJECT_STATUS.md)** - Current status, roadmap, metrics
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

See **[Project Status](docs/PROJECT_STATUS.md)** for detailed roadmap and current progress.

### Completed ✅

- [x] **Phase 6.4: gRPC Documentation** ✅ (Dec 2025)
  - Interactive gRPC service explorer
  - gRPC reflection for auto-discovery
  - Stream testing (unary, server, client, bidirectional)
  - TLS/SSL support with custom headers
  - Dark/Light themes with custom CSS
  - **First Rust framework with web-based gRPC docs!**

- [x] **Phase 6.3: GraphQL Documentation** ✅ (Dec 2025)
  - Interactive GraphiQL 3.0 playground (<100KB bundle)
  - Schema explorer with documentation
  - WebSocket subscription support
  - Query history persistence
  - Dark/Light themes with custom CSS
  - **Modern alternative to deprecated GraphQL Playground!**

- [x] **Track A: Scalar Integration** ✅ (Dec 2025)
  - Beautiful OpenAPI 3.1 documentation (<50KB bundle)
  - CDN version pinning + SRI hashes
  - CORS proxy for "Try It" functionality
  - Custom theming and CSS
  - **10x smaller than Swagger UI!**

- [x] **Track B: Binary Size Monitoring** ✅ (Dec 2025)
  - Automated CI/CD workflow
  - Local development scripts
  - cargo-make integration
  - **All binaries < 2MB (exceeded targets!)**

- [x] **Phases 1-5: CQRS Infrastructure** ✅ (Nov 2025)
  - AllSource Integration (pluggable backends)
  - CommandBus (90% reduction)
  - ProjectionRegistry (90% reduction)
  - Event Versioning (95% reduction)
  - Saga Orchestration (75% reduction)
  - **85% average boilerplate reduction!**

- [x] **v0.0** - Repository setup, documentation migration ✅
- [x] **v0.1** - `allframe ignite` + hello world ✅
- [x] **v0.2** - Compile-time DI + OpenAPI ✅
- [x] **v0.3** - OpenTelemetry tracing ✅

### Active 🚧

- [ ] **Phase 6: Router + API Documentation** 🚧 (Q1 2025)
  - ✅ Router Core (protocol-agnostic)
  - ✅ REST Documentation (Scalar)
  - ✅ GraphQL Documentation (GraphiQL)
  - ✅ gRPC Documentation (Service Explorer)
  - 📋 Contract Testing (built-in)

### Planned 📋

**Q2 2025**: Performance + Ecosystem
- TechEmpower benchmarks
- VS Code extension
- Ecosystem integration (Axum, Actix)

**Q3 2025**: Advanced Features
- API versioning
- Multi-language examples
- Analytics

**Q4 2025**: Production Hardening
- Security audit
- 1.0 release preparation

**[Read the full roadmap →](docs/PROJECT_STATUS.md)**

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

AllFrame targets **TechEmpower Round 23** benchmarks:

| Metric | Target | Status |
|--------|--------|--------|
| JSON serialization | > 500k req/s | 🚧 |
| Single query | > 100k req/s | 🚧 |
| Multiple queries | > 50k req/s | 🚧 |
| Binary size | < 8 MB | 🚧 |

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
