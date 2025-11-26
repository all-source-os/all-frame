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
default = ["di", "openapi", "router", "otel"]
```

- `di` - Dependency Injection and Clean Architecture
- `openapi` - OpenAPI schema generation
- `router` - Protocol-agnostic routing (REST, GraphQL, gRPC)
- `otel` - OpenTelemetry observability

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

**Dependencies**: `tonic`, `prost`, `tonic-reflection`

**APIs**:
- `GrpcProductionAdapter` - Full gRPC server
- Protobuf code generation
- Service reflection

**Binary Impact**: +1.2MB (tonic + protobuf)

---

### `router-full` - All Production Features

**Enables**: Both `router-graphql` and `router-grpc`

**Binary Impact**: +2.0MB (combined)

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

## Feature Dependencies

```
di
  └─ allframe-macros

openapi
  └─ (no dependencies)

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
  ├─ tokio-stream
  └─ futures

router-full
  ├─ router-graphql
  └─ router-grpc

cqrs
  └─ allframe-macros

otel
  └─ allframe-macros

mcp
  └─ (placeholder)
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

- `cqrs-chronos` - Chronos event store integration
- `cqrs-postgres` - PostgreSQL persistence
- `cqrs-sqlite` - SQLite persistence
- `mcp` - Model Context Protocol integration
- `auth` - Authentication/authorization helpers
- `rate-limit` - Rate limiting middleware
- `websockets` - WebSocket support

---

## See Also

- [CQRS + Chronos Assessment](./CQRS_CHRONOS_ASSESSMENT.md)
- [Milestone 0.4 Complete](./MILESTONE_0.4_COMPLETE.md)
- Main README.md
