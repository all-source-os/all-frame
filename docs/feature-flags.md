# AllFrame Feature Flags

AllFrame uses Cargo feature flags to allow you to opt into only the functionality you need, minimizing binary size and compilation time.

## Feature Flag Overview

### Core Features

#### `default`
**Includes:** `di`, `openapi`, `router`

The default feature set provides:
- Dependency injection system
- OpenAPI documentation generation
- Core router with TOML configuration support

```toml
[dependencies]
allframe-core = "0.1"  # Uses default features
```

#### `di`
**Dependencies:** `allframe-macros`

Enables the compile-time dependency injection system.

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["di"] }
```

#### `openapi`
**Dependencies:** None

Enables OpenAPI documentation generation from handler definitions.

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["openapi"] }
```

### Router Features

#### `router`
**Dependencies:** `toml`

Enables the core router with configuration file support. Includes:
- Protocol-agnostic handler registration
- MVP adapters (REST, GraphQL, gRPC) - lightweight, no external protocol deps
- TOML-based configuration
- Config-driven protocol selection

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router"] }
```

**What you get:**
- Core `Router` type
- `RestAdapter`, `GraphQLAdapter`, `GrpcAdapter` (MVP implementations)
- `RouterConfig` for TOML configuration
- Handler registration and execution
- Protocol adapter system

**Binary size impact:** ~50KB (just TOML parser)

#### `router-graphql`
**Dependencies:** `async-graphql`, `async-graphql-parser`
**Implies:** `router`

Enables production-ready GraphQL support with full AST parsing.

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router-graphql"] }
```

**What you get:**
- `GraphQLProductionAdapter` - full GraphQL support
- Full AST parsing and validation using async-graphql v7.0
- GraphiQL playground
- Query, mutation, and subscription support
- Schema introspection

**Binary size impact:** ~2MB

**When to use:**
- You need production-ready GraphQL support
- You want GraphiQL playground
- You need full GraphQL spec compliance
- You're building a GraphQL API

**When NOT to use:**
- You only need simple REST endpoints
- You want minimal binary size
- GraphQL is not part of your API

#### `router-grpc`
**Dependencies:** `tonic`, `tonic-reflection`, `prost`, `prost-types`, `tokio-stream`, `futures`
**Implies:** `router`

Enables production-ready gRPC support with streaming and reflection.

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router-grpc"] }
```

**What you get:**
- `GrpcProductionAdapter` - full gRPC support
- HTTP/2 transport using tonic v0.14
- Protobuf encoding using prost v0.14
- Server streaming, client streaming, bidirectional streaming
- gRPC reflection API for service discovery
- Support for tools like grpcurl

**Binary size impact:** ~3MB

**When to use:**
- You need production-ready gRPC support
- You want streaming RPCs
- You need service reflection
- You're building microservices with gRPC

**When NOT to use:**
- You only need REST or GraphQL
- You want minimal binary size
- gRPC is not part of your architecture

#### `router-full`
**Implies:** `router-graphql`, `router-grpc`

Enables both production GraphQL and gRPC adapters.

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router-full"] }
```

**Binary size impact:** ~5MB

**When to use:**
- You need both GraphQL and gRPC in production
- You're building a multi-protocol API gateway
- Binary size is not a concern

## Usage Examples

### Minimal Setup (No Dependencies)

For the absolute minimal build with no external dependencies:

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false }
```

**Available:** Core types only, no router, no DI, no OpenAPI

### REST-Only API

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router"] }
```

```rust
use allframe_core::router::Router;

let mut router = Router::new();
router.register("get_user", || async {
    r#"{"id": 42, "name": "John Doe"}"#.to_string()
});
```

**Binary overhead:** ~50KB for TOML support

### Production GraphQL API

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router-graphql"] }
```

```rust
use allframe_core::router::GraphQLProductionAdapter;

let adapter = GraphQLProductionAdapter::new("/graphql");

// Full GraphQL AST parsing
let query = "query { user(id: 42) { name } }";
GraphQLProductionAdapter::parse_query(query).unwrap();

// GraphiQL playground
let playground = adapter.graphiql_source();
```

**Binary overhead:** ~2MB for async-graphql

### Production gRPC Service

```toml
[dependencies]
allframe-core = { version = "0.1", default-features = false, features = ["router-grpc"] }
```

```rust
use allframe_core::router::{GrpcProductionAdapter, streaming};

let adapter = GrpcProductionAdapter::new();

// Server streaming
let items = vec![1, 2, 3, 4, 5];
let stream = streaming::from_vec(items);

// Enable reflection for grpcurl
let reflection_server = streaming::enable_reflection();
```

**Binary overhead:** ~3MB for tonic + prost

### Multi-Protocol Gateway

```toml
[dependencies]
allframe-core = { version = "0.1", features = ["router-full"] }
```

```rust
use allframe_core::router::{Router, RouterConfig};

let config = RouterConfig::from_str(r#"
    [server]
    protocols = ["rest", "graphql", "grpc"]
"#).unwrap();

let mut router = Router::with_config(config);

// Register once, expose via all protocols
router.register("get_user", || async {
    r#"{"id": 42, "name": "John Doe"}"#.to_string()
});
```

**Binary overhead:** ~5MB for all production features

## Binary Size Comparison

| Configuration | Binary Size | What's Included |
|--------------|-------------|-----------------|
| No features | ~500KB | Core types only |
| `router` | ~550KB | + TOML config |
| `router-graphql` | ~2.5MB | + Full GraphQL |
| `router-grpc` | ~3.5MB | + Full gRPC |
| `router-full` | ~5.5MB | + GraphQL + gRPC |

*Approximate sizes for release builds on x86_64 Linux*

## Compile Time Comparison

| Configuration | Clean Build Time |
|--------------|------------------|
| No features | ~10s |
| `router` | ~12s |
| `router-graphql` | ~45s |
| `router-grpc` | ~60s |
| `router-full` | ~90s |

*Times for M1 MacBook Pro with 8 cores*

## Migration Guide

### From MVP to Production

If you started with MVP adapters and want to upgrade:

**Before:**
```rust
use allframe_core::router::GraphQLAdapter;

let adapter = GraphQLAdapter::new();
```

**After:**
```toml
# Cargo.toml
[dependencies]
allframe-core = { version = "0.1", features = ["router-graphql"] }
```

```rust
use allframe_core::router::GraphQLProductionAdapter;

let adapter = GraphQLProductionAdapter::new("/graphql");
```

## Best Practices

### 1. Start Minimal, Add as Needed
Begin with just the features you need:
```toml
allframe-core = { version = "0.1", default-features = false, features = ["router"] }
```

Add production features when you need them:
```toml
allframe-core = { version = "0.1", default-features = false, features = ["router", "router-graphql"] }
```

### 2. Use MVP Adapters for Prototyping
MVP adapters are great for:
- Learning the framework
- Proof of concepts
- Internal tools with simple needs

Production adapters are essential for:
- Public-facing APIs
- Compliance requirements
- Full protocol spec support

### 3. Consider Binary Size Budget
If binary size matters (embedded systems, serverless, edge):
- Avoid `router-full`
- Choose only protocols you actually need
- Consider MVP adapters for simple use cases

### 4. Separate Services by Protocol
Instead of one service with `router-full`, consider:
- REST-only service: `features = ["router"]`
- GraphQL-only service: `features = ["router-graphql"]`
- gRPC-only service: `features = ["router-grpc"]`

Each service is smaller and deploys faster.

## Feature Detection in Code

You can conditionally compile code based on features:

```rust
#[cfg(feature = "router")]
use allframe_core::router::RouterConfig;

#[cfg(feature = "router-graphql")]
use allframe_core::router::GraphQLProductionAdapter;

#[cfg(feature = "router-grpc")]
use allframe_core::router::GrpcProductionAdapter;
```

## Testing with Features

Run tests with specific features:

```bash
# Test with no features
cargo test --no-default-features

# Test with router only
cargo test --no-default-features --features router

# Test with GraphQL
cargo test --no-default-features --features router-graphql

# Test with all features
cargo test --all-features
```

## Summary

AllFrame's feature flag system gives you control over:
- **Binary size**: Pay only for what you use
- **Compilation time**: Smaller builds compile faster
- **Dependencies**: Minimize your dependency tree
- **Deployment**: Smaller containers, faster deployments

Choose features based on your needs:
- Prototyping? Use MVP adapters (default `router` feature)
- Production GraphQL? Add `router-graphql`
- Production gRPC? Add `router-grpc`
- Multi-protocol gateway? Use `router-full`

**Remember:** You can always start minimal and add features later. AllFrame is designed to grow with your needs.
