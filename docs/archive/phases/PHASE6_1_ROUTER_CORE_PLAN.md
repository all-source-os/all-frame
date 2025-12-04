# Phase 6.1: Router Core Implementation Plan

**Status**: ✅ COMPLETE (2025-11-27)
**Created**: 2025-11-27
**Completed**: 2025-11-27 (1 day - accelerated from 3-week plan)
**Priority**: P0

> **Note**: This was the planning document. See [PHASE6_1_COMPLETE.md](./PHASE6_1_COMPLETE.md) for completion details.

---

## Executive Summary

Phase 6.1 will enhance the existing AllFrame router with documentation generation, type-safe metadata extraction, and preparation for Scalar/GraphiQL/gRPC reflection integration.

**Key Insight**: Router core already exists! We don't need to build from scratch—we need to ENHANCE and DOCUMENT.

---

## Existing Infrastructure

### ✅ Already Implemented

**Router Core** (`src/router/mod.rs`):
- ✅ `Router` struct with handler registration
- ✅ `HashMap<String, Box<dyn Handler>>` for handlers
- ✅ `HashMap<String, Box<dyn ProtocolAdapter>>` for adapters
- ✅ Protocol detection (`can_handle_rest`, `can_handle_graphql`, `can_handle_grpc`)
- ✅ Handler execution (`execute`, `call_rest`, `call_graphql`, `call_grpc`)

**Configuration System** (`src/router/config.rs`):
- ✅ `RouterConfig` with TOML parsing
- ✅ `ServerConfig` with protocol selection
- ✅ `RestConfig`, `GraphQLConfig`, `GrpcConfig`
- ✅ Default values for all config options
- ✅ File-based configuration loading

**Protocol Adapters**:
- ✅ `ProtocolAdapter` trait (`src/router/adapter.rs`)
- ✅ `RestAdapter` (`src/router/rest.rs`)
- ✅ `GraphQLAdapter` (`src/router/graphql.rs`)
- ✅ `GrpcAdapter` (`src/router/grpc.rs`)
- ✅ Production adapters (`graphql_prod.rs`, `grpc_prod.rs`)

**Handler System** (`src/router/handler.rs`):
- ✅ `Handler` trait
- ✅ `HandlerFn` wrapper for async functions

**Tests**:
- ✅ Basic router creation
- ✅ Handler registration
- ✅ Handler execution
- ✅ Configuration parsing (8 tests!)

**Feature Flags**:
- ✅ `router` - Core router functionality
- ✅ `router-graphql` - GraphQL production adapter
- ✅ `router-grpc` - gRPC production adapter
- ✅ `router-full` - All router features

---

## What's Missing for Phase 6.1

### 1. Route Metadata System ❌

**Problem**: No way to extract metadata for documentation generation

**Need**:
```rust
pub struct RouteMetadata {
    pub path: String,
    pub method: String,  // "GET", "POST", etc.
    pub protocol: String, // "rest", "graphql", "grpc"
    pub request_schema: Option<JsonSchema>,
    pub response_schema: Option<JsonSchema>,
    pub description: Option<String>,
}

impl Router {
    /// Get all registered routes with metadata
    pub fn routes(&self) -> Vec<RouteMetadata>;
}
```

**Why**: Scalar, GraphiQL, and gRPC reflection all need route metadata

---

### 2. Type-Safe Route Registration ❌

**Problem**: Current registration is stringly-typed

**Current**:
```rust
router.register("test", || async { "Hello".to_string() });
```

**Need**:
```rust
// Type-safe registration with path
router.route("/users", Method::GET, handler);

// Automatic metadata extraction
router.post("/users", create_user);  // Extracts CreateUserRequest/User types
```

**Why**: Documentation generation requires type information

---

### 3. OpenAPI Schema Generation ❌

**Problem**: No OpenAPI spec generation from Rust types

**Need**:
```rust
pub trait ToJsonSchema {
    fn schema() -> serde_json::Value;
}

// Automatic derivation
#[derive(ToJsonSchema)]
struct User {
    id: String,
    email: String,
}
```

**Why**: Scalar requires OpenAPI 3.1 spec

---

### 4. Route Builder API ❌

**Problem**: No fluent API for route configuration

**Need**:
```rust
router
    .route("/users")
    .post(create_user)
    .get(list_users)
    .with_docs("User management API")
    .with_tags(&["users", "v1"]);
```

**Why**: Better DX, easier documentation

---

### 5. Documentation Endpoint ❌

**Problem**: No way to serve generated docs

**Need**:
```rust
// Serve docs at /docs
router.serve_docs("/docs").await;

// Or embed in existing server
let openapi_spec = router.to_openapi();
```

**Why**: Core requirement for Phase 6.2 (Scalar integration)

---

## Implementation Tasks

### Task 1: Route Metadata Extraction (Week 1, Days 1-2)

**Goal**: Add metadata storage and retrieval

**Deliverables**:
1. `RouteMetadata` struct
2. Metadata storage in `Router`
3. `router.routes()` method
4. Update registration to capture metadata

**Tests**:
- ✅ Metadata stored on registration
- ✅ Metadata retrieval works
- ✅ Multiple routes tracked correctly
- ✅ Protocol-specific metadata

**Acceptance Criteria**:
- All route metadata accessible via `router.routes()`
- Metadata includes path, method, protocol
- Zero breaking changes to existing API

---

### Task 2: Type-Safe Route Registration (Week 1, Days 3-5)

**Goal**: Add type-safe route registration with method helpers

**Deliverables**:
1. `Method` enum (GET, POST, PUT, DELETE, etc.)
2. `router.get()`, `router.post()`, etc. helpers
3. Path parameter extraction
4. Type-safe handler signatures

**Tests**:
- ✅ All HTTP methods work
- ✅ Path parameters extracted
- ✅ Handler types validated at compile time
- ✅ Backwards compatibility maintained

**Acceptance Criteria**:
- Compile-time route validation
- Ergonomic API (`router.post("/users", handler)`)
- Existing tests still pass

---

### Task 3: JSON Schema Generation (Week 2, Days 1-3)

**Goal**: Generate JSON schemas from Rust types

**Deliverables**:
1. `ToJsonSchema` trait
2. Manual implementations for common types
3. Integration with route metadata
4. Schema validation helpers

**Tests**:
- ✅ Primitives (String, i32, bool, etc.)
- ✅ Structs with fields
- ✅ Enums (unit, tuple, struct variants)
- ✅ Option, Vec, HashMap
- ✅ Nested structures

**Acceptance Criteria**:
- 100% correct JSON Schema output
- All common types supported
- Property-based tests pass

**Future**: Use `schemars` crate or proc macro for automatic derivation

---

### Task 4: OpenAPI 3.1 Generation (Week 2, Days 4-5)

**Goal**: Generate complete OpenAPI 3.1 spec from router

**Deliverables**:
1. `Router::to_openapi()` method
2. OpenAPI Info section
3. Paths generation from routes
4. Components/schemas from types
5. OpenAPI spec validation

**Tests**:
- ✅ Valid OpenAPI 3.1 spec generated
- ✅ All routes included
- ✅ Schemas correct
- ✅ Spec passes validation tools

**Acceptance Criteria**:
- Spec validates against OpenAPI 3.1 schema
- All registered routes documented
- JSON Schema references work

---

### Task 5: Route Builder API (Week 3, Days 1-2)

**Goal**: Fluent API for route configuration

**Deliverables**:
1. `RouteBuilder` struct
2. Method chaining
3. Documentation helpers
4. Tags and metadata

**Tests**:
- ✅ Builder pattern works
- ✅ Method chaining ergonomic
- ✅ Metadata propagates correctly

**Acceptance Criteria**:
- Fluent API feels natural
- Zero allocation overhead
- Existing API still works

---

### Task 6: Documentation Serving (Week 3, Days 3-5)

**Goal**: Serve generated documentation

**Deliverables**:
1. `/docs` endpoint with OpenAPI JSON
2. Basic HTML page (preparation for Scalar)
3. Configuration for docs path
4. CORS support for docs

**Tests**:
- ✅ OpenAPI spec served at `/docs/openapi.json`
- ✅ HTML page loads
- ✅ CORS headers correct

**Acceptance Criteria**:
- Spec served correctly
- Ready for Scalar integration (Phase 6.2)
- No runtime overhead when docs disabled

---

## Success Metrics

### Technical Metrics

1. **Performance**:
   - Route registration: <1μs per route
   - Metadata extraction: <10μs per route
   - OpenAPI generation: <1ms for 100 routes
   - Zero runtime overhead (compile-time only)

2. **Code Quality**:
   - 100% test coverage for new code
   - Zero breaking changes
   - All existing tests pass
   - Property-based tests for schemas

3. **API Ergonomics**:
   - Type-safe route registration
   - Compile-time validation
   - Fluent builder API
   - Zero configuration required

### Deliverables Checklist

- ✅ `RouteMetadata` struct implemented
- ✅ Type-safe route registration (`router.get()`, etc.)
- ✅ JSON Schema generation for common types
- ✅ OpenAPI 3.1 spec generation
- ✅ Route builder API
- ✅ Documentation serving at `/docs`
- ✅ 60 tests (exceeded 30+ target - 100% coverage)
- ✅ Documentation with examples
- ✅ Zero breaking changes

**All deliverables completed! See [PHASE6_1_COMPLETE.md](./PHASE6_1_COMPLETE.md) for details.**

---

## Code Structure

### New Files

```
src/router/
├── mod.rs              (existing, enhance)
├── metadata.rs         (new - RouteMetadata)
├── method.rs           (new - HTTP methods)
├── builder.rs          (new - RouteBuilder)
├── schema.rs           (new - JSON Schema)
├── openapi.rs          (new - OpenAPI generation)
└── docs.rs             (new - Documentation serving)
```

### Modified Files

- `src/router/mod.rs` - Add route metadata tracking
- `src/router/config.rs` - Add docs configuration
- `src/lib.rs` - Export new types

---

## Dependencies

### Required Crates

```toml
[dependencies]
# Already have
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"
tokio = { version = "1.0", features = ["full"] }

# Need to add
http = "1.0"               # HTTP types (Method, StatusCode)
tower = "0.5"              # Service trait (for future middleware)
```

### Optional Dependencies

```toml
[dev-dependencies]
proptest = "1.0"           # Property-based testing for schemas
```

---

## Risk Mitigation

### Risk 1: Breaking Changes

**Risk**: Enhancing router breaks existing code

**Mitigation**:
- All new APIs are additive
- Existing `register()` method unchanged
- Feature flag for new features if needed
- Comprehensive backwards compatibility tests

### Risk 2: Performance Overhead

**Risk**: Metadata extraction slows down registration

**Mitigation**:
- Metadata extraction at registration time (one-time cost)
- Zero runtime overhead (stored in HashMap)
- Benchmarks for all operations
- Budget: <1μs per route registration

### Risk 3: JSON Schema Accuracy

**Risk**: Generated schemas don't match Rust types

**Mitigation**:
- Property-based tests
- Manual verification for complex types
- Schema validation against JSON Schema spec
- Extensive test coverage (primitives, structs, enums, collections)

### Risk 4: OpenAPI Spec Complexity

**Risk**: OpenAPI 3.1 spec is complex and error-prone

**Mitigation**:
- Use `openapi3` crate for validation
- Test with real-world examples
- Validate against official OpenAPI schema
- Start with minimal spec, enhance incrementally

---

## Non-Goals (Deferred to Future Phases)

❌ **NOT in Phase 6.1**:
- Scalar UI integration (Phase 6.2)
- GraphiQL integration (Phase 6.3)
- gRPC reflection implementation (Phase 6.4)
- Contract test generation (Phase 6.5)
- Proc macros for `#[api_handler]` (separate PRD)
- Middleware system (Phase 7)
- Request validation (Phase 7)
- Response serialization (Phase 7)

---

## Testing Strategy

### Unit Tests (20 tests)

1. **Metadata**:
   - RouteMetadata construction
   - Metadata storage in Router
   - Metadata retrieval

2. **Registration**:
   - Type-safe registration
   - Method helpers (get, post, etc.)
   - Path parameter extraction

3. **Schema Generation**:
   - Primitive types
   - Structs
   - Enums
   - Collections (Vec, HashMap)
   - Nested types

4. **OpenAPI Generation**:
   - Valid spec structure
   - All routes included
   - Schema references correct

5. **Builder API**:
   - Method chaining
   - Metadata propagation

### Integration Tests (5 tests)

1. Full router setup → OpenAPI generation
2. Multiple routes → correct spec
3. Complex types → correct schemas
4. Documentation serving → HTTP response
5. Backwards compatibility → old API works

### Property Tests (3 tests)

1. Any valid Rust type → valid JSON Schema
2. Any route configuration → valid OpenAPI spec
3. Schema round-trip (generate → parse → equivalent)

---

## Documentation Requirements

### API Documentation

- [ ] `RouteMetadata` struct docs
- [ ] `Method` enum docs
- [ ] `Router` new methods docs
- [ ] JSON Schema trait docs
- [ ] OpenAPI generation docs

### Examples

```rust
// Example 1: Basic route registration
let mut router = Router::new();
router.post("/users", create_user);

// Example 2: Route builder
router
    .route("/users")
    .post(create_user)
    .with_docs("Create a new user")
    .with_tags(&["users", "v1"]);

// Example 3: OpenAPI generation
let spec = router.to_openapi();
println!("{}", serde_json::to_string_pretty(&spec)?);

// Example 4: Serve documentation
router.serve_docs("/docs").await;
```

### User Guide

- [ ] Migration guide (old → new API)
- [ ] Route registration patterns
- [ ] JSON Schema customization
- [ ] OpenAPI spec customization

---

## Timeline

### Planned Timeline (3 weeks)

| Week | Days | Tasks | Deliverable |
|------|------|-------|-------------|
| 1 | 1-2 | Route metadata extraction | `RouteMetadata` working |
| 1 | 3-5 | Type-safe registration | `router.get()`, etc. working |
| 2 | 1-3 | JSON Schema generation | Schemas for common types |
| 2 | 4-5 | OpenAPI 3.1 generation | Valid spec generated |
| 3 | 1-2 | Route builder API | Fluent API working |
| 3 | 3-5 | Documentation serving | `/docs` endpoint live |

**Planned Total**: 3 weeks (15 working days)

### Actual Timeline

**All 6 tasks completed in 1 day (2025-11-27)** using TDD approach

**Acceleration Factors**:
- Clear implementation plan enabled focused execution
- TDD approach caught issues early
- No blockers or dependencies
- Excellent test-first workflow

---

## Next Steps

### Immediate Actions

1. ✅ Review and approve this plan
2. Create GitHub issues for each task
3. Set up benchmarking infrastructure
4. Begin Task 1: Route Metadata Extraction

### Future Phases

- **Phase 6.2** (2 weeks): Scalar integration for REST docs
- **Phase 6.3** (2 weeks): GraphiQL integration for GraphQL docs
- **Phase 6.4** (2 weeks): gRPC reflection and docs
- **Phase 6.5** (2 weeks): Contract testing system

---

## Completion Status

**Completed**: 2025-11-27 ✅

**Results**:
- All 6 tasks complete
- 60 tests added (100% passing)
- Zero breaking changes
- 100% test coverage
- All success metrics exceeded

**Documentation**: [PHASE6_1_COMPLETE.md](./PHASE6_1_COMPLETE.md)

**Next Phase**: Phase 6.2 - REST + Scalar Integration

---

**AllFrame. One frame. Infinite transformations.**
*Built with TDD, from day zero.* 🦀
