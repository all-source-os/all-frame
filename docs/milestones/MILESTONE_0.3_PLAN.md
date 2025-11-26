# Milestone 0.3 - Protocol-Agnostic Routing

**Date**: 2025-11-25
**Status**: Planning Phase
**Previous Milestone**: v0.2 Complete (Compile-time DI + OpenAPI)

---

## Vision

**"Same handler works as REST, GraphQL, gRPC via config"**

The goal of Milestone 0.3 is to implement **protocol-agnostic routing** that allows a single handler function to be exposed as REST, GraphQL, or gRPC endpoints through configuration alone.

## Core Concept

```rust
// Write once
#[allframe_handler]
async fn get_user(id: i32) -> Result<User, AppError> {
    // implementation
}

// Use anywhere via config:
// REST:    GET /users/{id}
// GraphQL: query { user(id: 1) { ... } }
// gRPC:    UserService.GetUser(request)
```

## Acceptance Criteria (from PRD)

From PRD_01.md line 55:
> **Acceptance Criteria**: Same handler works as REST, GraphQL, gRPC via config

### Must Pass:
1. ✅ Single handler function can be called via REST HTTP
2. ✅ Same handler can be called via GraphQL query
3. ✅ Same handler can be called via gRPC RPC
4. ✅ Protocol selection via configuration (not code changes)
5. ✅ Request/response transformation automatic
6. ✅ Type safety maintained across all protocols
7. ✅ OpenAPI schema reflects REST endpoints
8. ✅ GraphQL schema generated automatically
9. ✅ gRPC .proto files generated automatically

## Technical Architecture

### 1. Handler Abstraction Layer

```rust
// Universal handler trait
trait ProtocolHandler {
    type Request;
    type Response;
    type Error;

    async fn handle(&self, req: Self::Request) -> Result<Self::Response, Self::Error>;
}
```

### 2. Protocol Adapters

Each protocol needs an adapter that:
- Parses incoming protocol-specific requests
- Transforms to universal handler format
- Calls the handler
- Transforms response back to protocol format

```
┌─────────────────────────────────────────────────┐
│                  Protocols                      │
├─────────────┬─────────────┬─────────────────────┤
│   REST      │  GraphQL    │      gRPC           │
│   HTTP      │   Query     │   Protobuf          │
└──────┬──────┴──────┬──────┴──────┬──────────────┘
       │             │              │
       ▼             ▼              ▼
┌─────────────────────────────────────────────────┐
│            Protocol Adapters                    │
│  (Parse → Transform → Validate)                 │
└──────┬──────────────┬──────────────┬────────────┘
       │              │               │
       ▼              ▼               ▼
┌─────────────────────────────────────────────────┐
│          Universal Handler Interface            │
│         (Type-safe function call)               │
└──────┬──────────────┬──────────────┬────────────┘
       │              │               │
       ▼              ▼               ▼
┌─────────────────────────────────────────────────┐
│              User Handler                       │
│      async fn get_user(id: i32) -> User        │
└─────────────────────────────────────────────────┘
```

### 3. Configuration-Driven Protocol Selection

```toml
# allframe.toml
[server]
protocols = ["rest", "graphql", "grpc"]

[server.rest]
port = 8080
path_prefix = "/api/v1"

[server.graphql]
port = 8081
path = "/graphql"
playground = true

[server.grpc]
port = 9090
reflection = true
```

### 4. Schema Generation

- **REST**: OpenAPI 3.1 (already done in v0.2)
- **GraphQL**: SDL schema from Rust types
- **gRPC**: .proto files from Rust types

## Implementation Plan

### Phase 1: Core Abstractions (Week 1)

**Goal**: Define the universal handler interface and protocol adapter traits

**Tests to write**:
- `tests/04_router_core.rs` - Core router abstractions
- Handler registration
- Protocol adapter trait
- Request/response transformation

**Implementation**:
- `crates/allframe-core/src/router/mod.rs`
- `crates/allframe-core/src/router/handler.rs`
- `crates/allframe-core/src/router/adapter.rs`

### Phase 2: REST Adapter (Week 1)

**Goal**: Implement REST HTTP adapter with existing OpenAPI integration

**Tests to write**:
- `tests/04_router_rest.rs` - REST-specific tests
- GET, POST, PUT, DELETE methods
- Path parameters
- Query parameters
- Request body parsing
- Response serialization
- Status codes

**Implementation**:
- `crates/allframe-core/src/router/rest.rs`
- Integration with existing `#[api_handler]` macro

### Phase 3: GraphQL Adapter (Week 2)

**Goal**: Implement GraphQL query/mutation adapter

**Tests to write**:
- `tests/04_router_graphql.rs` - GraphQL-specific tests
- Query execution
- Mutation execution
- Type mapping (Rust → GraphQL)
- Schema generation
- Introspection support
- Error handling

**Implementation**:
- `crates/allframe-core/src/router/graphql.rs`
- `crates/allframe-macros/src/graphql.rs` - Schema generation

**Dependencies to consider**:
- May need lightweight GraphQL parser (or build minimal one)
- Schema generation from Rust types

### Phase 4: gRPC Adapter (Week 2)

**Goal**: Implement gRPC service adapter

**Tests to write**:
- `tests/04_router_grpc.rs` - gRPC-specific tests
- Unary RPC calls
- Service definition
- Type mapping (Rust → Protobuf)
- .proto file generation
- Error handling (gRPC status codes)

**Implementation**:
- `crates/allframe-core/src/router/grpc.rs`
- `crates/allframe-macros/src/grpc.rs` - Proto generation

**Dependencies to consider**:
- May need protobuf codec (or minimal implementation)
- gRPC wire format handling

### Phase 5: Config-Driven Routing (Week 3)

**Goal**: Configuration system for protocol selection

**Tests to write**:
- `tests/04_router_config.rs` - Configuration tests
- TOML parsing
- Protocol enablement
- Port configuration
- Multi-protocol server startup
- Runtime protocol switching

**Implementation**:
- `crates/allframe-core/src/config/mod.rs`
- `crates/allframe-core/src/config/router.rs`
- Server initialization with multiple protocols

### Phase 6: Unified Handler Macro (Week 3)

**Goal**: Single macro that works across all protocols

**Tests to write**:
- `tests/04_router_macro.rs` - Macro tests
- Handler registration
- Protocol metadata generation
- Schema generation for all protocols

**Implementation**:
- `crates/allframe-macros/src/router.rs`
- `#[allframe_handler]` macro that supersedes `#[api_handler]`

## Test Structure

### Simple Tests (MVP - Week 1)
```rust
// tests/04_router_simple.rs

#[test]
fn test_rest_handler_basic() {
    // Register handler
    // Call via REST
    // Verify response
}

#[test]
fn test_handler_with_multiple_protocols() {
    // Register same handler for REST and GraphQL
    // Call via both protocols
    // Verify same result
}
```

### Advanced Tests (Complete - Week 3)
```rust
// tests/04_router_advanced.rs

#[test]
fn test_protocol_switching_via_config() {
    // Load config with protocol selection
    // Start server with multiple protocols
    // Call same handler via different protocols
    // Verify consistent behavior
}

#[test]
fn test_graphql_query_execution() {
    // Register handler
    // Execute GraphQL query
    // Verify response matches schema
}

#[test]
fn test_grpc_unary_call() {
    // Register handler
    // Make gRPC call
    // Verify protobuf response
}
```

## Success Metrics

| Metric | Target | How to Measure |
|--------|--------|---------------|
| Protocol Coverage | 3 protocols (REST, GraphQL, gRPC) | All protocol tests passing |
| Single Handler | Same code for all protocols | No protocol-specific handler code |
| Config-Driven | Zero code changes to switch protocols | Toggle via TOML only |
| Schema Generation | Automatic for all protocols | OpenAPI, GraphQL SDL, .proto generated |
| Type Safety | 100% type-safe transformations | Compile-time guarantees |
| Test Coverage | 100% line + branch coverage | cargo llvm-cov |
| Performance | < 10% overhead vs raw protocol | Benchmark suite |

## Dependencies Analysis

### Required:
- **HTTP**: Already have hyper/tokio
- **JSON**: serde_json (already used)

### To Evaluate:
- **GraphQL**:
  - Option 1: async-graphql (full-featured, 3rd party)
  - Option 2: Build minimal GraphQL parser/executor
  - **Decision**: Start with minimal implementation for MVP

- **gRPC**:
  - Option 1: tonic (full-featured, adds deps)
  - Option 2: Build minimal protobuf + gRPC wire format handler
  - **Decision**: Start with minimal implementation for MVP

### Strategy:
- Begin with REST (already mostly done)
- Add minimal GraphQL support (query parsing + type mapping)
- Add minimal gRPC support (protobuf codec + unary RPC)
- Avoid heavy external dependencies where possible
- Can always add full-featured libraries later as optional features

## File Structure

```
crates/allframe-core/src/
├── router/
│   ├── mod.rs           # Core router + registration
│   ├── handler.rs       # Handler trait + universal interface
│   ├── adapter.rs       # Protocol adapter trait
│   ├── rest.rs          # REST/HTTP adapter
│   ├── graphql.rs       # GraphQL adapter
│   └── grpc.rs          # gRPC adapter
├── config/
│   ├── mod.rs           # Config loading
│   └── router.rs        # Router config structures
└── server/
    └── multi_protocol.rs # Server that runs multiple protocols

crates/allframe-macros/src/
├── router.rs            # #[allframe_handler] macro
├── graphql.rs           # GraphQL schema generation
└── grpc.rs              # Protobuf schema generation

tests/
├── 04_router_core.rs         # Core abstractions
├── 04_router_rest.rs          # REST adapter
├── 04_router_graphql.rs       # GraphQL adapter
├── 04_router_grpc.rs          # gRPC adapter
├── 04_router_config.rs        # Config-driven routing
├── 04_router_macro.rs         # Unified macro
└── 04_router_integration.rs   # End-to-end multi-protocol
```

## Risks & Mitigations

| Risk | Impact | Mitigation |
|------|--------|------------|
| GraphQL complexity | High | Start with minimal query support, add features incrementally |
| gRPC wire format | High | Focus on unary RPC first, streams later |
| Type mapping challenges | Medium | Leverage existing OpenAPI type system |
| Performance overhead | Medium | Benchmark early, optimize protocol adapters |
| Schema generation complexity | Medium | Reuse patterns from OpenAPI generation |
| External dependencies | Low | Build minimal implementations first |

## TDD Workflow

### Week 1 (Days 1-7):
1. **Day 1**: Write failing tests for core router abstractions
2. **Day 2**: Implement core router (make tests pass)
3. **Day 3**: Write failing tests for REST adapter
4. **Day 4**: Implement REST adapter (make tests pass)
5. **Day 5**: Write failing tests for basic GraphQL
6. **Day 6-7**: Implement basic GraphQL support

### Week 2 (Days 8-14):
1. **Day 8-9**: Complete GraphQL adapter + tests
2. **Day 10**: Write failing tests for gRPC adapter
3. **Day 11-12**: Implement gRPC adapter (make tests pass)
4. **Day 13-14**: Write failing tests for config system

### Week 3 (Days 15-21):
1. **Day 15-16**: Implement config system (make tests pass)
2. **Day 17**: Write failing tests for unified macro
3. **Day 18-19**: Implement unified macro
4. **Day 20**: Integration tests + benchmarks
5. **Day 21**: Documentation + examples

## Deliverables

### Code:
- [ ] Core router abstractions
- [ ] REST adapter (enhanced from v0.2)
- [ ] GraphQL adapter (query + mutation support)
- [ ] gRPC adapter (unary RPC support)
- [ ] Config system for protocol selection
- [ ] `#[allframe_handler]` macro

### Tests:
- [ ] 40+ tests covering all protocols
- [ ] Integration tests with multi-protocol server
- [ ] 100% line + branch coverage

### Documentation:
- [ ] Protocol adapter guide
- [ ] Config reference for routing
- [ ] Migration guide from `#[api_handler]` to `#[allframe_handler]`
- [ ] Examples for each protocol

### Schemas:
- [ ] OpenAPI 3.1 (enhanced from v0.2)
- [ ] GraphQL SDL generation
- [ ] .proto file generation

## Examples to Build

```rust
// examples/multi_protocol_api.rs

use allframe::prelude::*;

#[derive(Serialize, Deserialize)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// Write once, use everywhere
#[allframe_handler]
async fn get_user(id: i32) -> Result<User, AppError> {
    Ok(User {
        id,
        name: "John Doe".to_string(),
        email: "john@example.com".to_string(),
    })
}

#[allframe_handler]
async fn create_user(user: User) -> Result<User, AppError> {
    // Save to database
    Ok(user)
}

#[tokio::main]
async fn main() {
    // Load config (determines which protocols are enabled)
    let config = AllFrameConfig::from_file("allframe.toml")?;

    // Start server with all configured protocols
    AllFrame::new()
        .with_config(config)
        .handler(get_user)
        .handler(create_user)
        .run()
        .await?;
}
```

**Configuration** (`allframe.toml`):
```toml
[server]
protocols = ["rest", "graphql", "grpc"]

[server.rest]
port = 8080

[server.graphql]
port = 8081

[server.grpc]
port = 9090
```

**Usage**:
```bash
# REST
curl http://localhost:8080/users/1

# GraphQL
curl -X POST http://localhost:8081/graphql \
  -d '{"query": "{ user(id: 1) { id name email } }"}'

# gRPC
grpcurl -plaintext localhost:9090 UserService.GetUser
```

## Next Steps

1. **Create test file structure** for router tests
2. **Write failing tests** for core router (RED phase)
3. **Implement core router** to make tests pass (GREEN phase)
4. **Refactor** and iterate (REFACTOR phase)
5. Repeat for each protocol adapter

## Questions to Answer

- [ ] Should we support WebSockets in this milestone or defer to 0.4?
  - **Decision**: Defer to 0.4, focus on REST/GraphQL/gRPC first

- [ ] Do we need full GraphQL subscriptions or just queries/mutations?
  - **Decision**: Queries + Mutations for 0.3, subscriptions in 0.4

- [ ] Should gRPC support streaming or just unary RPCs?
  - **Decision**: Unary only for 0.3, streaming in 0.4

- [ ] How do we handle authentication across protocols?
  - **Decision**: Defer to 0.4 (OTEL + middleware)

## References

- PRD_01.md lines 17, 55, 68
- Milestone 0.2 completion: docs/MILESTONE_0.2_COMPLETE.md
- TDD checklist: .claude/TDD_CHECKLIST.md

---

**Ready to begin RED phase**: Writing failing tests for core router abstractions.

🚀 **Milestone 0.3 - Protocol-Agnostic Routing - Let's ignite it!**
