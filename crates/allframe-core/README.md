# allframe-core

**The composable Rust API framework**

[![Crates.io](https://img.shields.io/crates/v/allframe-core.svg)](https://crates.io/crates/allframe-core)
[![Documentation](https://docs.rs/allframe-core/badge.svg)](https://docs.rs/allframe-core)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)
[![Rust](https://img.shields.io/badge/rust-1.86%2B-orange.svg)](https://www.rust-lang.org)

AllFrame is a protocol-agnostic Rust web framework built with Test-Driven Development. Write your handler once, expose it via REST, GraphQL, and gRPC.

## Features

- ✅ **Protocol-Agnostic Routing** - One handler, multiple protocols (REST, GraphQL, gRPC)
- ✅ **CQRS + Event Sourcing** - Production-ready CQRS infrastructure with 90% less boilerplate
- ✅ **Compile-time DI** - Dependency injection resolved at compile time
- ✅ **Auto Documentation** - OpenAPI 3.1, GraphQL schemas, and gRPC reflection
- ✅ **Zero Runtime Dependencies** - Just Tokio and Hyper
- ✅ **100% TDD** - Every feature has tests before implementation

## Quick Start

```toml
[dependencies]
allframe-core = "0.1"
tokio = { version = "1.48", features = ["full"] }
```

```rust
use allframe_core::router::Router;

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    router.register("get_user", |user_id: String| async move {
        format!("User: {}", user_id)
    });

    // Handler now available via REST, GraphQL, or gRPC!
}
```

## Protocol-Agnostic Example

```rust
use allframe_core::router::{Router, RestAdapter, GraphQLAdapter, GrpcAdapter};

let mut router = Router::new();
router.register("get_user", get_user_handler);

// REST
let rest = RestAdapter::new();
rest.route("GET", "/users/:id", "get_user");

// GraphQL
let graphql = GraphQLAdapter::new();
graphql.query("user", "get_user");

// gRPC
let grpc = GrpcAdapter::new();
grpc.unary("Users", "GetUser", "get_user");
```

## Features

### Default Features
```toml
allframe-core = "0.1"
# Includes: di, openapi, router, otel
```

### All Features
```toml
allframe-core = { version = "0.1", features = [
    "di",              # Dependency injection
    "openapi",         # OpenAPI documentation
    "router",          # Protocol-agnostic routing
    "router-graphql",  # GraphQL support
    "router-grpc",     # gRPC support
    "router-full",     # All protocols
    "cqrs",            # CQRS + Event Sourcing
    "otel",            # OpenTelemetry tracing
] }
```

### Optional CQRS Features
```toml
allframe-core = { version = "0.1", features = [
    "cqrs",            # Core CQRS infrastructure
    "cqrs-allsource",  # AllSource Core event store
] }
```

## CQRS Example

```rust
use allframe_core::cqrs::{CommandBus, Event, EventStore};

#[derive(Command)]
struct CreateUser {
    name: String,
    email: String,
}

#[derive(Event)]
struct UserCreated {
    id: Uuid,
    name: String,
}

// 90% less boilerplate with automatic dispatch!
let mut bus = CommandBus::new();
bus.register(create_user_handler);
bus.execute(CreateUser { /* ... */ }).await?;
```

## Documentation

- **API Docs**: https://docs.rs/allframe-core
- **Guide**: See [examples/](../../examples/) directory
- **MCP Server**: https://crates.io/crates/allframe-mcp

## Examples

```bash
# REST API
cargo run --example rest_api

# GraphQL API
cargo run --example graphql_api

# gRPC API
cargo run --example grpc_api

# Multi-protocol
cargo run --example multi_protocol
```

## Why AllFrame?

| Feature | AllFrame | Actix | Axum | Rocket |
|---------|----------|-------|------|--------|
| TDD-First | ✅ 100% | ❌ | ❌ | ❌ |
| Protocol-Agnostic | ✅ | ❌ | ❌ | ❌ |
| Built-in CQRS | ✅ | ❌ | ❌ | ❌ |
| Compile-time DI | ✅ | ❌ | ❌ | ❌ |
| Zero Runtime Deps | ✅ | ❌ | ✅ | ❌ |

## Contributing

Contributions welcome! See [CONTRIBUTING.md](../../CONTRIBUTING.md).

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Resources

- **Documentation**: https://docs.rs/allframe-core
- **Repository**: https://github.com/all-source-os/all-frame
- **CLI Tool**: https://crates.io/crates/allframe-forge
- **MCP Server**: https://crates.io/crates/allframe-mcp
