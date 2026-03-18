<p align="center">
  <strong>AllFrame</strong><br>
  <em>The Composable Rust API Framework</em><br>
  <em>One frame to rule them all. Transform, compose, ignite.</em>
</p>

<p align="center">
  <a href="https://crates.io/crates/allframe"><img src="https://img.shields.io/crates/v/allframe.svg" alt="crates.io"></a>
  <a href="https://crates.io/crates/allframe"><img src="https://img.shields.io/crates/d/allframe.svg" alt="downloads"></a>
  <a href="https://docs.rs/allframe"><img src="https://img.shields.io/docsrs/allframe" alt="docs.rs"></a>
  <a href="https://github.com/all-source-os/all-frame/actions"><img src="https://img.shields.io/github/actions/workflow/status/all-source-os/all-frame/offline-quality-gates.yml?branch=main&label=CI" alt="CI"></a>
  <br>
  <a href="#license"><img src="https://img.shields.io/crates/l/allframe.svg" alt="license"></a>
  <a href="https://www.rust-lang.org"><img src="https://img.shields.io/badge/MSRV-1.89-orange.svg" alt="MSRV"></a>
  <a href="docs/PROJECT_STATUS.md"><img src="https://img.shields.io/badge/tests-500%2B%20passing-brightgreen.svg" alt="tests"></a>
  <a href="https://all-source-os.github.io/all-frame"><img src="https://img.shields.io/badge/docs-GitHub%20Pages-blue" alt="project docs"></a>
</p>

---

AllFrame is a **complete Rust web framework** with a built-in HTTP/2 server, designed and evolved exclusively through TDD. Write your handler once and expose it via REST, GraphQL, and gRPC from a single codebase.

## Quick Start

```bash
cargo install allframe
allframe ignite my-api
cd my-api && cargo run
# Visit http://localhost:8080/swagger-ui
```

Or add it as a dependency:

```toml
[dependencies]
allframe = "0.1"
```

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

## Crates

AllFrame ships as composable crates -- use only what you need:

| Crate | Description | |
|-------|-------------|-|
| [`allframe`](https://crates.io/crates/allframe) | Re-exports core + CLI (`allframe ignite`) | [![crates.io](https://img.shields.io/crates/v/allframe.svg?label=)](https://crates.io/crates/allframe) |
| [`allframe-core`](https://crates.io/crates/allframe-core) | Router, CQRS, DI, resilience, auth, security | [![crates.io](https://img.shields.io/crates/v/allframe-core.svg?label=)](https://crates.io/crates/allframe-core) |
| [`allframe-macros`](https://crates.io/crates/allframe-macros) | Proc macros (`#[handler]`, `#[di_container]`, `#[retry]`, ...) | [![crates.io](https://img.shields.io/crates/v/allframe-macros.svg?label=)](https://crates.io/crates/allframe-macros) |
| [`allframe-forge`](https://crates.io/crates/allframe-forge) | Project scaffolding & code generation | [![crates.io](https://img.shields.io/crates/v/allframe-forge.svg?label=)](https://crates.io/crates/allframe-forge) |
| [`allframe-mcp`](https://crates.io/crates/allframe-mcp) | MCP server -- expose handlers as LLM tools (zero overhead when unused) | [![crates.io](https://img.shields.io/crates/v/allframe-mcp.svg?label=)](https://crates.io/crates/allframe-mcp) |
| [`allframe-tauri`](https://crates.io/crates/allframe-tauri) | Tauri 2.x plugin for offline-first desktop apps | [![crates.io](https://img.shields.io/crates/v/allframe-tauri.svg?label=)](https://crates.io/crates/allframe-tauri) |

## Features at a Glance

### Protocol-Agnostic Routing

```rust
#[handler(protocols = ["rest", "graphql", "grpc"])]
async fn create_user(input: CreateUserInput) -> Result<User, Error> {
    // Exposed as POST /users, mutation { createUser }, and CreateUser(request)
}
```

78 tests across 5 phases. Full adapters for REST, GraphQL (async-graphql), and gRPC (tonic) with automatic schema generation (OpenAPI 3.1, GraphQL SDL, `.proto`).

### CQRS + Event Sourcing

85% average boilerplate reduction with `CommandBus`, `ProjectionRegistry`, event versioning with auto-upcasting, and saga orchestration with automatic compensation. Pluggable backends: in-memory, SQLite, AllSource.

```rust
#[command_handler]
async fn create_user(cmd: CreateUserCommand) -> CommandResult<UserEvent> {
    Ok(vec![UserEvent::Created { user_id: cmd.user_id, email: cmd.email }])
}
```

[Read the CQRS announcement](docs/announcements/CQRS_INFRASTRUCTURE_COMPLETE.md)

### Compile-Time DI

```rust
#[di_container]
struct AppContainer {
    user_repo: Arc<dyn UserRepository>,
    user_service: Arc<UserService>,
}
// Resolved at compile time -- zero runtime overhead
```

### Resilience Patterns

Retry with exponential backoff, circuit breaker (per-resource with `KeyedCircuitBreaker`), rate limiting (token bucket, Redis-backed for distributed), adaptive retry, and retry budgets. Available as both builder API and attribute macros:

```rust
#[retry(max_retries = 3, initial_interval_ms = 100)]
async fn fetch_user(id: &str) -> Result<User, Error> { /* ... */ }

#[circuit_breaker(failure_threshold = 5, timeout_secs = 30)]
async fn call_payment() -> Result<Payment, Error> { /* ... */ }
```

### MCP Server (LLM Tool Calling)

Handlers automatically become [Model Context Protocol](https://modelcontextprotocol.io) tools. Zero overhead when not used -- opt-in via a separate crate.

```rust
let mcp = McpServer::with_router(router);
let result = mcp.call_tool("get_user", json!({"user_id": "123"})).await;
```

[allframe-mcp docs](crates/allframe-mcp/README.md) | [MCP distribution model](docs/MCP_DISTRIBUTION_MODEL.md)

### Layered Authentication

Protocol-agnostic auth with zero-bloat feature flags: core `Authenticator` trait, JWT validation (HS256, RS256, EdDSA), Axum extractors/middleware, gRPC interceptors, and type-safe `AuthContext<C>`.

### Security Utilities

URL/credential obfuscation, `Sensitive<T>` wrapper, `#[derive(Obfuscate)]` with `#[sensitive]` field attributes, smart header obfuscation.

### API Documentation

Auto-generated interactive docs for all protocols:
- **Scalar UI** for REST (10x smaller than Swagger) -- [guide](docs/guides/SCALAR_DOCUMENTATION.md)
- **GraphiQL 3.0** playground for GraphQL -- [guide](docs/guides/GRAPHQL_DOCUMENTATION.md)
- **gRPC Explorer** for service discovery -- [example](crates/allframe-core/examples/grpc_docs.rs)

### Offline-First Architecture

SQLite event store, offline circuit breaker with operation queuing, store-and-forward for intermittent connectivity, bidirectional projection sync with conflict resolution. Zero network dependencies in offline builds, enforced by CI.

### Streaming Handlers

`StreamSender` with backpressure and cooperative cancellation. Tauri IPC bridge with per-stream events. TypeScript codegen with RxJS adapter.

### Contract Testing

Automatic test generation from router, schema validation, breaking change detection, and coverage reporting.

### Graceful Shutdown

`ShutdownAwareTaskSpawner` for named tasks with automatic cancellation, `GracefulShutdownExt` for cleanup orchestration.

## Feature Flags

Start minimal and add features as needed. Detailed guide: [docs/guides/FEATURE_FLAGS.md](docs/guides/FEATURE_FLAGS.md)

| Feature | Description | Default |
|---------|-------------|:-------:|
| `router` | Protocol-agnostic routing | yes |
| `di` | Compile-time dependency injection | yes |
| `openapi` | Auto OpenAPI 3.1 + Scalar UI | yes |
| `cqrs` | CQRS + Event Sourcing | yes |
| `otel` | OpenTelemetry tracing | yes |
| `health` | Health check endpoints | yes |
| `router-graphql` | GraphQL via async-graphql | -- |
| `router-grpc` | gRPC via tonic | -- |
| `router-full` | GraphQL + gRPC | -- |
| `resilience` | Retry, circuit breaker, rate limiting | -- |
| `resilience-redis` | Distributed rate limiting | -- |
| `auth` / `auth-jwt` / `auth-axum` / `auth-tonic` | Layered auth | -- |
| `security` | Safe logging, obfuscation | -- |
| `cqrs-sqlite` | SQLite event store | -- |
| `offline` | Full offline bundle | -- |
| `vector-search` | Vector similarity search | -- |
| `keyword-search` | Full-text BM25 search | -- |

```toml
# Minimal REST API
allframe = { version = "0.1", default-features = false, features = ["router"] }

# Full multi-protocol gateway
allframe = { version = "0.1", features = ["router-full", "resilience", "auth-jwt"] }
```

## Examples

```bash
cargo run --example rest_api
cargo run --example graphql_api
cargo run --example grpc_api
cargo run --example multi_protocol
cargo run --example resilience --features resilience
cargo run --example security --features security
cargo run -p allframe-core --example graceful_shutdown
```

See [examples/README.md](examples/README.md) for details.

## Why AllFrame?

| | AllFrame | Actix | Axum | Rocket |
|-|:--------:|:-----:|:----:|:------:|
| Protocol-agnostic handlers | **yes** | -- | -- | -- |
| Built-in CQRS + Event Sourcing | **yes** | -- | -- | -- |
| Compile-time DI | **yes** | -- | -- | -- |
| Auto OpenAPI 3.1 | **yes** | manual | manual | manual |
| Resilience patterns | **yes** | ext | ext | -- |
| MCP server (LLM tools) | **yes** | -- | -- | -- |
| Offline-first / Tauri | **yes** | -- | -- | -- |
| Streaming handlers | **yes** | -- | -- | -- |
| Contract testing | **yes** | -- | -- | -- |
| TDD from day zero | **yes** | -- | -- | -- |

## Documentation

| | |
|-|-|
| [API Reference (docs.rs)](https://docs.rs/allframe) | Generated Rust API docs |
| [Project Docs (GitHub Pages)](https://all-source-os.github.io/all-frame) | Guides, architecture, announcements |
| [Project Status](docs/PROJECT_STATUS.md) | Current status and metrics |
| [Roadmap](docs/current/ROADMAP.md) | Path to v1.0 |
| [Feature Flags Guide](docs/guides/FEATURE_FLAGS.md) | Minimize binary size |
| [Documentation Index](docs/INDEX.md) | Full catalog |

### CQRS Deep Dives

- [Phase 1: AllSource Integration](docs/phases/PHASE1_COMPLETE.md)
- [Phase 2: CommandBus (90% reduction)](docs/phases/PHASE2_COMPLETE.md)
- [Phase 3: ProjectionRegistry](docs/phases/PHASE3_COMPLETE.md)
- [Phase 4: Event Versioning (95% reduction)](docs/phases/PHASE4_COMPLETE.md)
- [Phase 5: Saga Orchestration](docs/phases/PHASE5_COMPLETE.md)

## Contributing

AllFrame is **100% TDD-driven**. Every commit must contain at least one new failing test.

```bash
git clone https://github.com/all-source-os/all-frame.git
cd all-frame
cargo test
cargo clippy -- -D warnings
cargo fmt -- --check
```

1. Read the [PRD](docs/current/PRD_01.md)
2. Ensure **100% test coverage** for all changes
3. Follow the [Clean Architecture Guide](/.claude/skills/rust-clean-architecture.md)

## License

Licensed under either of [Apache License 2.0](LICENSE-APACHE) or [MIT](LICENSE-MIT), at your option.

## Community

- [Discussions](https://github.com/all-source-os/all-frame/discussions)
- [Issue Tracker](https://github.com/all-source-os/all-frame/issues)

---

<p align="center"><strong>AllFrame. One frame. Infinite transformations.</strong><br><em>Built with TDD, from day zero.</em></p>
