# PRD: Router + API Documentation System

**Status**: 📝 DRAFT
**Created**: 2025-11-26
**Owner**: AllFrame Core Team
**Priority**: P0 (Next Major Phase)

---

## Executive Summary

Build a **best-in-class, protocol-agnostic routing and documentation system** that automatically generates beautiful, interactive API documentation for REST, GraphQL, and gRPC endpoints. This system must be **lighter weight and better than anything on the market** while maintaining AllFrame's zero-boilerplate philosophy.

### Key Goals

1. **Unified Router**: Single routing abstraction that works across REST, GraphQL, gRPC
2. **Best-in-Class Docs**:
   - REST → [Scalar](https://scalar.com/) (modern, lightweight OpenAPI docs)
   - GraphQL → GraphiQL playground + auto-generated documentation
   - gRPC → Protocol documentation + reflection
3. **Contract Testing**: Built-in contract testing helpers for all protocols
4. **Zero Config**: Documentation generated automatically from code
5. **Developer Experience**: Better than Swagger/ReDoc/Postman combined

---

## Problem Statement

### Current Pain Points

**1. Documentation Fragmentation**
- REST uses Swagger UI (outdated, heavy, slow)
- GraphQL has multiple tools (GraphiQL, Apollo Studio, etc.)
- gRPC has no standard documentation solution
- No unified experience across protocols

**2. Manual Documentation Effort**
- Developers manually maintain OpenAPI specs
- Schema drift between code and docs
- Documentation becomes outdated quickly
- No single source of truth

**3. Poor Developer Experience**
- Swagger UI is slow and outdated (built 2015)
- No modern dark mode, poor mobile support
- Lacks modern features (search, examples, validation)
- Heavy JavaScript bundles (100KB+ gzipped)

**4. Testing Complexity**
- Contract testing requires separate tools
- No built-in testing UI
- Hard to verify API contracts
- Manual test case generation

### Why This Matters

**For API Consumers**:
- Need beautiful, accurate, interactive documentation
- Want to try APIs without writing code
- Expect modern UX (dark mode, search, mobile)

**For API Developers**:
- Don't want to maintain separate docs
- Need docs to stay in sync with code
- Want contract testing built-in

**For AllFrame**:
- Router is core to framework value proposition
- Documentation is often the first touchpoint
- Best-in-class docs = competitive advantage

---

## Success Criteria

### Must Have (P0)

1. ✅ **Unified Router Core**
   - Protocol-agnostic routing abstraction
   - Zero-cost abstraction overhead
   - Type-safe route registration
   - Compile-time route validation

2. ✅ **REST Documentation (Scalar)**
   - Automatic OpenAPI 3.1 generation from code
   - Beautiful, modern UI via Scalar
   - Interactive API testing
   - <50KB bundle size (vs 100KB+ for Swagger)

3. ✅ **GraphQL Documentation**
   - GraphiQL playground embedded
   - Auto-generated schema documentation
   - Interactive query builder
   - Subscription support

4. ✅ **gRPC Documentation**
   - Protocol documentation generation
   - gRPC reflection API
   - Service explorer UI
   - Request/response examples

5. ✅ **Contract Testing**
   - Built-in contract test generators
   - Schema validation helpers
   - Mock server generation
   - Test report generation

### Should Have (P1)

1. **Advanced Features**
   - Multi-language code examples (cURL, JavaScript, Python, Rust)
   - Request/response recording
   - API versioning support
   - Deprecation warnings

2. **Developer Tools**
   - VS Code extension for route preview
   - CLI for documentation export
   - Offline documentation mode
   - PDF/Markdown export

3. **Performance**
   - Server-side rendering option
   - Static site generation
   - CDN-friendly assets
   - Progressive web app (PWA)

### Nice to Have (P2)

1. **Ecosystem Integration**
   - Postman collection export
   - Insomnia workspace export
   - OpenAPI tools compatibility
   - GraphQL Playground compatibility

2. **Analytics**
   - API usage tracking
   - Popular endpoints highlighting
   - Deprecation analytics
   - Error rate tracking

---

## Technical Architecture

### 1. Router Core

```rust
// Core routing abstraction
pub trait Router: Send + Sync {
    /// Register a route handler
    fn route<H: Handler>(&mut self, path: &str, handler: H);

    /// Get route metadata for documentation
    fn routes(&self) -> Vec<RouteMetadata>;

    /// Generate protocol-specific config
    fn to_protocol<P: Protocol>(&self) -> P::Config;
}

// Protocol abstraction
pub trait Protocol {
    type Config;
    type Handler;

    fn name(&self) -> &str;
    fn generate_docs(&self, config: Self::Config) -> Documentation;
}
```

**Key Design Principles**:
- **Protocol-agnostic**: Router knows nothing about HTTP, GraphQL, gRPC
- **Zero-cost**: Compile-time protocol selection, no runtime overhead
- **Type-safe**: Routes validated at compile time
- **Extensible**: Third-party protocols via trait implementation

---

### 2. REST Documentation (Scalar)

**Why Scalar?**
- ✅ Modern, beautiful UI (built 2023)
- ✅ Lightweight (<50KB vs 100KB+ for Swagger UI)
- ✅ Dark mode by default
- ✅ Mobile-friendly
- ✅ Better search and navigation
- ✅ OpenAPI 3.1 support
- ✅ Free and open source

**Architecture**:
```rust
#[api_handler]
async fn create_user(req: CreateUserRequest) -> Result<User, ApiError> {
    // Business logic
}

// Automatic OpenAPI generation:
// POST /users
// Request: CreateUserRequest (JSON schema)
// Response: User (JSON schema)
// Errors: ApiError variants
```

**Features**:
- Automatic OpenAPI 3.1 spec generation
- JSON Schema derivation from Rust types
- Example generation from doc comments
- Request/response validation
- Interactive "Try It" functionality

**Example Output**:
```yaml
openapi: 3.1.0
info:
  title: My API
  version: 1.0.0
paths:
  /users:
    post:
      summary: Create a new user
      requestBody:
        content:
          application/json:
            schema:
              $ref: '#/components/schemas/CreateUserRequest'
      responses:
        '200':
          content:
            application/json:
              schema:
                $ref: '#/components/schemas/User'
```

---

### 3. GraphQL Documentation

**Stack**:
- **GraphiQL**: Interactive playground (industry standard)
- **Schema introspection**: Auto-generated schema docs
- **Explorer**: Visual query builder

**Architecture**:
```rust
#[graphql_object]
impl User {
    /// Get the user's email
    async fn email(&self) -> String {
        self.email.clone()
    }

    /// Get the user's posts
    async fn posts(&self) -> Vec<Post> {
        // Logic here
    }
}

// Automatic schema generation:
// type User {
//   email: String!
//   posts: [Post!]!
// }
```

**Features**:
- GraphiQL playground embedded
- Schema documentation sidebar
- Query auto-completion
- Subscription testing
- Mutation testing
- Query history

---

### 4. gRPC Documentation

**Challenge**: gRPC has no standard documentation solution

**AllFrame Solution**:
1. **gRPC Reflection**: Expose service definitions at runtime
2. **Custom UI**: Build lightweight gRPC explorer
3. **Proto Documentation**: Generate docs from .proto files

**Architecture**:
```rust
#[grpc_service]
impl UserService {
    /// Create a new user
    #[grpc_method]
    async fn create_user(&self, req: CreateUserRequest) -> Result<User, Status> {
        // Logic here
    }
}

// Automatic .proto generation + reflection API
// service UserService {
//   rpc CreateUser(CreateUserRequest) returns (User);
// }
```

**Features**:
- gRPC reflection API (industry standard)
- Interactive service explorer UI
- Request builder with syntax highlighting
- Response viewer
- Stream testing (server/client/bidirectional)

**Inspiration**:
- Postman gRPC client
- grpcurl CLI tool
- BloomRPC UI

---

### 5. Contract Testing

**Goal**: Make API contract testing effortless

**Architecture**:
```rust
#[test]
async fn test_create_user_contract() {
    // Automatic contract test generation
    let contract = Router::generate_contract_test("POST /users");

    // Validates:
    // - Request schema matches OpenAPI spec
    // - Response schema matches OpenAPI spec
    // - Status codes match spec
    // - Headers match spec

    contract.run().await.expect("Contract test failed");
}
```

**Features**:
1. **Schema Validation**
   - Request/response against OpenAPI spec
   - GraphQL query against schema
   - gRPC message against proto

2. **Mock Generation**
   - Generate mock servers from specs
   - Realistic fake data
   - Error scenario testing

3. **Test Reports**
   - Coverage report (% of endpoints tested)
   - Schema drift detection
   - Breaking change detection

4. **CI/CD Integration**
   - `cargo test --contract-tests`
   - JUnit XML output
   - GitHub Actions workflow

---

## Implementation Plan

### Phase 6.1: Router Core ✅ COMPLETE (2025-11-27)

**Goal**: Build protocol-agnostic routing foundation

**Delivered**:
1. ✅ Route metadata extraction (`RouteMetadata`)
2. ✅ Type-safe route registration (`router.get()`, etc.)
3. ✅ JSON Schema generation (`ToJsonSchema` trait)
4. ✅ OpenAPI 3.1 spec generation
5. ✅ Route builder API
6. ✅ Documentation serving

**Results**:
- 60 tests added (100% coverage)
- Zero runtime overhead achieved
- Zero breaking changes
- All success metrics exceeded

**Documentation**: [PHASE6_1_COMPLETE.md](../phases/PHASE6_1_COMPLETE.md)

---

### Phase 6.2: REST + Scalar Integration (2 weeks)

**Goal**: Best-in-class REST API documentation

**Deliverables**:
1. OpenAPI 3.1 generation from Rust types
2. Scalar UI integration (<50KB bundle)
3. Interactive "Try It" functionality
4. JSON Schema derivation
5. Example generation from doc comments

**Tests**:
- OpenAPI spec validation
- JSON Schema correctness
- UI integration tests
- Bundle size assertions (<50KB)

**Success Metrics**:
- 100% automatic spec generation
- <50KB JavaScript bundle
- Mobile-friendly UI
- Dark mode by default

---

### Phase 6.3: GraphQL Documentation (2 weeks)

**Goal**: Beautiful GraphQL API documentation

**Deliverables**:
1. GraphiQL playground integration
2. Schema introspection API
3. Query auto-completion
4. Subscription testing UI
5. Schema documentation sidebar

**Tests**:
- Schema generation correctness
- GraphiQL integration tests
- Subscription testing
- Query validation

**Success Metrics**:
- 100% schema auto-generation
- Interactive playground works
- Subscription support
- Query history

---

### Phase 6.4: gRPC Documentation (2 weeks)

**Goal**: First-class gRPC documentation

**Deliverables**:
1. gRPC reflection API
2. Service explorer UI
3. Proto file generation
4. Request builder UI
5. Stream testing (all types)

**Tests**:
- Reflection API correctness
- UI integration tests
- Stream handling tests
- Proto generation tests

**Success Metrics**:
- gRPC reflection working
- Interactive UI for all RPC types
- Stream testing supported
- Proto docs generated

---

### Phase 6.5: Contract Testing (2 weeks)

**Goal**: Effortless API contract testing

**Deliverables**:
1. Contract test generators
2. Schema validation helpers
3. Mock server generation
4. Test report generation
5. CI/CD integration

**Tests**:
- Contract validation tests
- Mock server tests
- Breaking change detection
- Report generation tests

**Success Metrics**:
- 1-line contract test generation
- 100% schema validation
- Automatic mock servers
- CI/CD ready

---

## User Experience

### REST API Developer Flow

```rust
use allframe::prelude::*;

#[tokio::main]
async fn main() {
    let mut router = Router::new();

    // Type-safe route registration (Phase 6.1 ✅)
    router.get("/users", || async { "User list".to_string() });
    router.post("/users", || async { "User created".to_string() });
    router.get("/users/{id}", || async { "User details".to_string() });

    // Generate OpenAPI 3.1 spec (Phase 6.1 ✅)
    let spec = router.to_openapi("My API", "1.0.0");

    // Serve OpenAPI JSON (Phase 6.1 ✅)
    let json = router.openapi_json("My API", "1.0.0");

    // Serve basic HTML docs (Phase 6.1 ✅)
    let config = router.docs_config("/docs", "My API", "1.0.0");
    let html = router.docs_html(&config);

    // Future: Scalar integration (Phase 6.2)
    // router.serve_with_scalar("0.0.0.0:3000", "/docs").await;
}
```

**Developer sees**:
1. Navigate to `http://localhost:3000/docs`
2. Beautiful Scalar UI with dark mode
3. All routes auto-documented
4. "Try It" button to test endpoints
5. JSON schemas auto-generated
6. Example requests from doc comments

**Zero configuration. Zero manual work.**

---

### GraphQL API Developer Flow

```rust
// Future: Phase 6.3 - GraphQL Documentation

use allframe::prelude::*;

#[graphql_object]
impl Query {
    /// Get a user by ID
    async fn user(&self, id: String) -> Option<User> {
        // Logic here
    }
}

#[tokio::main]
async fn main() {
    let schema = Schema::build(Query, Mutation, Subscription).finish();

    let router = Router::new();
    router.graphql("/graphql", schema);

    // Serve GraphQL + GraphiQL at /graphql (Phase 6.3)
    router.serve_with_graphiql("0.0.0.0:3000", "/graphql").await;
}
```

**Developer sees**:
1. Navigate to `http://localhost:3000/graphql`
2. GraphiQL playground loads
3. Schema docs in sidebar
4. Query auto-completion works
5. "Run" button to test queries
6. Subscription testing UI

**Zero configuration. Zero manual work.**

---

### gRPC API Developer Flow

```rust
// Future: Phase 6.4 - gRPC Documentation

use allframe::prelude::*;

#[grpc_service]
impl UserService {
    /// Create a new user
    async fn create_user(&self, req: CreateUserRequest) -> Result<User, Status> {
        // Logic here
    }
}

#[tokio::main]
async fn main() {
    let router = Router::new();
    router.grpc("/", UserService);

    // Serve gRPC + reflection + docs (Phase 6.4)
    router.serve_with_grpc_reflection("0.0.0.0:50051", "/docs").await;
}
```

**Developer sees**:
1. Navigate to `http://localhost:50051/docs`
2. gRPC service explorer UI
3. All methods documented
4. Request builder with syntax highlighting
5. "Send" button to test RPCs
6. Stream testing (server/client/bidirectional)

**Zero configuration. Zero manual work.**

---

### Contract Testing Flow

```rust
#[test]
async fn contract_tests() {
    let router = Router::new();
    router.post("/users", create_user);

    // Generate contract tests automatically
    let tests = router.generate_contract_tests();

    // Run all contract tests
    tests.run().await.expect("Contract tests failed");
}
```

**Tests validate**:
- ✅ Request schemas match OpenAPI spec
- ✅ Response schemas match OpenAPI spec
- ✅ Status codes are correct
- ✅ Headers are correct
- ✅ Error responses match spec

**Output**:
```
✅ POST /users - Request schema valid
✅ POST /users - Response schema valid
✅ POST /users - Status codes correct
✅ GET /users/{id} - Request schema valid
✅ GET /users/{id} - Response schema valid

Contract Tests: 5/5 passed (100%)
Schema Coverage: 100%
```

---

## Competitive Analysis

### Swagger UI vs Scalar

| Feature | Swagger UI | Scalar | AllFrame + Scalar |
|---------|-----------|--------|-------------------|
| **Bundle Size** | ~100KB gzipped | <50KB gzipped | <50KB gzipped |
| **UI Design** | Outdated (2015) | Modern (2023) | Modern (2023) |
| **Dark Mode** | Addon required | Built-in | Built-in |
| **Mobile** | Poor | Excellent | Excellent |
| **Performance** | Slow on large APIs | Fast | Fast |
| **Search** | Basic | Advanced | Advanced |
| **Auto-generation** | Manual | Manual | **Automatic** |
| **Contract Tests** | ❌ | ❌ | **✅** |

**Verdict**: Scalar is objectively better, AllFrame makes it automatic.

---

### GraphQL Playground vs GraphiQL

| Feature | Playground | GraphiQL | AllFrame + GraphiQL |
|---------|-----------|----------|---------------------|
| **Maintenance** | Deprecated | Active | Active |
| **UI** | Modern but heavy | Classic, lighter | Classic, lighter |
| **Bundle Size** | ~200KB | ~100KB | ~100KB |
| **Features** | Rich | Essential | Essential + Auto-gen |
| **Auto-generation** | Manual | Manual | **Automatic** |
| **Contract Tests** | ❌ | ❌ | **✅** |

**Verdict**: GraphiQL is industry standard, AllFrame makes it automatic.

---

### gRPC Documentation

| Feature | grpcurl | BloomRPC | Postman | AllFrame |
|---------|---------|----------|---------|----------|
| **Type** | CLI | Desktop app | Desktop app | Web UI |
| **Bundle** | Binary | ~100MB | ~500MB | <50KB |
| **Auto-docs** | ❌ | ❌ | ❌ | **✅** |
| **Reflection** | ✅ | ✅ | ✅ | ✅ |
| **Stream Testing** | ✅ | ✅ | ✅ | ✅ |
| **Contract Tests** | ❌ | ❌ | ❌ | **✅** |

**Verdict**: No good web-based solution exists. AllFrame fills the gap.

---

## Risk Mitigation

### Risk 1: Bundle Size Creep

**Risk**: Documentation UIs become too heavy

**Mitigation**:
- Strict bundle size budgets (<50KB for REST, <100KB for GraphQL)
- CI/CD checks for bundle size
- Tree-shaking optimization
- Lazy-loading for heavy components
- CDN delivery for static assets

---

### Risk 2: Scalar/GraphiQL Breaking Changes

**Risk**: Third-party dependencies break

**Mitigation**:
- Pin exact versions in Cargo.toml
- Version compatibility tests in CI/CD
- Fallback to basic HTML if UI fails
- Vendor critical assets if needed

---

### Risk 3: gRPC Documentation Complexity

**Risk**: Building gRPC UI from scratch is hard

**Mitigation**:
- Phase 6.4 is P1, not P0 (can defer if needed)
- Start with reflection API only (easy)
- Build minimal UI incrementally
- Consider embedding grpcurl CLI as fallback

---

### Risk 4: OpenAPI Generation Accuracy

**Risk**: Auto-generated OpenAPI specs are wrong

**Mitigation**:
- Extensive test coverage (>100 tests)
- Property-based testing for schema generation
- OpenAPI validation tools in CI/CD
- Manual review of generated specs
- User feedback loop

---

## Success Metrics

### Technical Metrics

1. **Performance**
   - Router overhead: <10ns per request
   - OpenAPI generation: <1ms for 100 routes
   - Bundle sizes: REST <50KB, GraphQL <100KB, gRPC <50KB

2. **Quality**
   - Test coverage: >90% for all router code
   - Contract test coverage: 100% of endpoints
   - Zero breaking changes to existing APIs

3. **Developer Experience**
   - Zero configuration required
   - 100% automatic documentation generation
   - 1-line contract test generation

### Adoption Metrics

1. **Documentation Usage**
   - % of AllFrame users serving docs
   - Time spent in documentation UI
   - "Try It" usage rate

2. **Contract Testing**
   - % of projects using contract tests
   - Number of breaking changes caught
   - CI/CD integration rate

---

## Open Questions

1. **Q**: Should we support OpenAPI 2.0 (Swagger 2.0)?
   **A**: No. OpenAPI 3.1 only. 2.0 is legacy (2014).

2. **Q**: Should we build our own GraphQL playground?
   **A**: No. GraphiQL is industry standard and actively maintained.

3. **Q**: How do we handle API versioning?
   **A**: Phase 6.6 (P1). Support `/v1/users`, `/v2/users` routing.

4. **Q**: Should we support AsyncAPI for async/event APIs?
   **A**: Phase 7 (P2). Focus on REST/GraphQL/gRPC first.

5. **Q**: How do we handle authentication in docs?
   **A**: Support OAuth2, API keys, JWT in "Try It" functionality.

---

## Timeline

| Phase | Duration | Deliverable |
|-------|----------|-------------|
| 6.1 | 3 weeks | Router Core |
| 6.2 | 2 weeks | REST + Scalar |
| 6.3 | 2 weeks | GraphQL Docs |
| 6.4 | 2 weeks | gRPC Docs |
| 6.5 | 2 weeks | Contract Testing |
| **Total** | **11 weeks** | **Complete Router + Docs** |

**Target Completion**: Q1 2025

---

## Appendix

### A. Technology Stack

**Router Core**:
- Pure Rust, no external dependencies
- Proc macros for route registration
- Compile-time route validation

**REST Documentation**:
- Scalar (https://scalar.com/)
- OpenAPI 3.1 spec generation
- JSON Schema derivation

**GraphQL Documentation**:
- GraphiQL (https://github.com/graphql/graphiql)
- Schema introspection
- async-graphql integration

**gRPC Documentation**:
- gRPC reflection API (tonic)
- Custom web UI (Svelte or vanilla JS)
- Proto documentation generation

**Contract Testing**:
- JSON Schema validation
- GraphQL query validation
- gRPC message validation
- JUnit XML output

---

### B. Prior Art

**REST**:
- Swagger UI: https://swagger.io/tools/swagger-ui/
- ReDoc: https://redocly.com/redoc
- Scalar: https://scalar.com/ ← **Our choice**
- RapiDoc: https://rapidocweb.com/

**GraphQL**:
- GraphiQL: https://github.com/graphql/graphiql ← **Our choice**
- GraphQL Playground: https://github.com/graphql/graphql-playground (deprecated)
- Apollo Studio: https://studio.apollographql.com/ (commercial)

**gRPC**:
- grpcurl: https://github.com/fullstorydev/grpcurl (CLI only)
- BloomRPC: https://github.com/bloomrpc/bloomrpc (desktop app)
- Postman: https://www.postman.com/ (commercial)
- **No good web solution exists** ← **Opportunity for AllFrame**

**Contract Testing**:
- Pact: https://pact.io/ (complex setup)
- Dredd: https://dredd.org/ (OpenAPI only)
- Prism: https://stoplight.io/open-source/prism (mock server only)
- **No Rust-native solution** ← **Opportunity for AllFrame**

---

### C. References

- OpenAPI 3.1 Spec: https://spec.openapis.org/oas/v3.1.0
- GraphQL Spec: https://spec.graphql.org/
- gRPC Reflection: https://github.com/grpc/grpc/blob/master/doc/server-reflection.md
- Scalar Documentation: https://github.com/scalar/scalar
- GraphiQL Documentation: https://graphiql-test.netlify.app/

---

**END OF PRD**

---

## Approval

- [ ] Engineering Lead
- [ ] Product Owner
- [ ] Architecture Review
- [ ] Security Review

**Next Steps**:
1. Review and approve PRD
2. Create Phase 6.1 tasks
3. Begin Router Core implementation
