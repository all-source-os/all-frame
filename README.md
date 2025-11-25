# AllFrame

**The Composable Rust API Framework**

> *One frame to rule them all. Transform, compose, ignite.*

[![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Rust](https://img.shields.io/badge/rust-1.75%2B-orange.svg)](https://www.rust-lang.org)
[![TDD](https://img.shields.io/badge/TDD-100%25-green.svg)](docs/current/PRD_01.md)

---

## What is AllFrame?

AllFrame is the **first Rust web framework designed, built, and evolved exclusively through Test-Driven Development (TDD)**. Every feature, macro, and public API has a failing test before it is written.

We ship **one crate** (`allframe-core`) that gives you, out of the box and with **zero external runtime dependencies**:

- ✅ **Project Scaffolding** - `allframe ignite` creates Clean Architecture projects (v0.1)
- ✅ **Compile-time DI** - Dependency injection resolved at compile time (v0.2 MVP)
- ✅ **Auto OpenAPI 3.1** - API documentation generated automatically (v0.2 MVP)
- 📋 **OpenTelemetry auto-instrumentation** - Observability built-in (v0.4 - planned)
- 🚧 **Protocol-agnostic routing** - REST ↔ GraphQL ↔ gRPC ↔ WebSockets via config (v0.3 - in progress)
- 📋 **Enforced Clean Architecture + CQRS/ES** - Architectural patterns enforced at compile time (v0.4 - planned)
- 📋 **Native MCP server** - LLMs can call your API as tools (v0.5 - planned)
- 📋 **LLM-powered code generation** - `allframe forge` CLI (v0.6 - planned)

**Target**: Binaries < 8 MB, > 500k req/s (TechEmpower parity with Actix), and **100% test coverage enforced by CI**.

**Current Status**: v0.3 In Progress! (44/50 tests passing - REST, GraphQL & gRPC adapters complete)

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

### 📝 Auto OpenAPI 3.1 Generation

```rust
#[api_handler]
async fn get_user(id: Path<String>) -> Result<Json<User>, ApiError> {
    // OpenAPI schema auto-generated
    // Swagger UI available at /swagger-ui
    // MCP schema exported for LLM tool calling
}
```

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

### 🏛️ Clean Architecture Enforced

```rust
// ✅ This compiles
domain_entity.apply_business_logic();

// ❌ This fails at compile time
handler.direct_database_access(); // Compilation error!
```

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
| Protocol-agnostic | ✅ | ❌ | ❌ | ❌ |
| Clean Arch enforced | ✅ | ❌ | ❌ | ❌ |
| MCP Server | ✅ | ❌ | ❌ | ❌ |
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

### Planned Features

| Feature | Description | Default |
|---------|-------------|---------|
| `otel` | OpenTelemetry auto-instrumentation | ❌ |
| `cqrs` | CQRS + Event Sourcing support | ❌ |
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

- 📖 [Getting Started Guide](docs/guides/getting-started.md) *(coming soon)*
- 🎯 [Feature Flags Guide](docs/feature-flags.md) - Minimize binary size
- 📋 [Product Requirements Document](docs/current/PRD_01.md)
- 🧪 [TDD Workflow](/.claude/TDD_CHECKLIST.md)
- 🏛️ [Clean Architecture Guide](/.claude/skills/rust-clean-architecture.md)
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
git clone https://github.com/yourusername/allframe.git
cd allframe

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

See [PRD_01.md](docs/current/PRD_01.md) for detailed roadmap.

### MVP Milestones (Q1 2026)

- [x] **0.0** - Repository setup, documentation migration ✅
- [x] **0.1** - `allframe ignite` + hello world ✅ (RED-GREEN-REFACTOR complete)
- [x] **0.2** - Compile-time DI + OpenAPI ✅ (MVP complete, 10/10 tests passing)
- [ ] **0.3** - Protocol router + Advanced DI/OpenAPI 🚧 (44/50 tests passing, REST/GraphQL/gRPC complete)
- [ ] **0.4** - OTEL + CQRS + Clean Arch enforcement 📋
- [ ] **0.5** - MCP server (LLMs can call handlers) 📋
- [ ] **0.6** - `allframe forge` CLI (LLM code gen) 📋
- [ ] **1.0** - Production release, benchmarks, docs 📋

**Detailed Status**: See [MILESTONE_0.2_COMPLETE.md](docs/MILESTONE_0.2_COMPLETE.md)

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

- 💬 [Discussions](https://github.com/yourusername/allframe/discussions)
- 🐛 [Issue Tracker](https://github.com/yourusername/allframe/issues)
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
