# Protocol-Agnostic Routing - Completion Plan

**Status**: 🚧 In Progress (60% complete)
**Target**: v0.3
**Goal**: Write handlers once, expose via REST, GraphQL, AND gRPC

---

## Current State

### ✅ What's Already Done

1. **Router Core** (Phase 6.1) ✅
   - Protocol-agnostic routing foundation
   - Route registration and metadata
   - Handler execution
   - 60 tests passing

2. **Protocol Adapters (Stubs)** ✅
   - `RestAdapter` - Basic implementation
   - `GraphQLAdapter` - Basic implementation
   - `GrpcAdapter` - Basic implementation
   - `ProtocolAdapter` trait defined

3. **Example Code** ✅
   - `examples/multi_protocol.rs` - Complete demonstration
   - `examples/rest_api.rs` - REST-specific
   - `examples/graphql_api.rs` - GraphQL-specific
   - `examples/grpc_api.rs` - gRPC-specific

4. **Documentation Systems** (Phase 6.2-6.4) ✅
   - REST documentation (Scalar)
   - GraphQL documentation (GraphiQL)
   - gRPC documentation (Explorer)

### 🚧 What's Missing

To complete protocol-agnostic routing, we need:

1. **Router Adapter Management**
   - `Router::add_adapter()` method
   - `Router::has_adapter()` method
   - `Router::get_adapter()` method
   - Adapter storage and lifecycle

2. **Protocol Adapter Implementations**
   - Full `RestAdapter` implementation
   - Full `GraphQLAdapter` implementation
   - Full `GrpcAdapter` implementation
   - Request/response transformation
   - Error handling per protocol

3. **Handler Registration Enhancement**
   - Protocol-specific route registration
   - Metadata per protocol (REST paths, GraphQL queries, gRPC methods)
   - Automatic schema generation from handlers

4. **Tests**
   - Adapter registration tests
   - Multi-protocol routing tests
   - Request transformation tests
   - Error handling tests

---

## Detailed Implementation Plan

### Phase 1: Adapter Management (2-3 days)

#### Add Adapter Storage to Router

```rust
// In src/router/mod.rs
pub struct Router {
    handlers: HashMap<String, BoxedHandler>,
    routes: Vec<RouteMetadata>,
    adapters: HashMap<String, Box<dyn ProtocolAdapter>>, // NEW
}
```

#### Implement Adapter Methods

```rust
impl Router {
    /// Add a protocol adapter to the router
    pub fn add_adapter(&mut self, adapter: Box<dyn ProtocolAdapter>) {
        let name = adapter.name().to_string();
        self.adapters.insert(name, adapter);
    }

    /// Check if an adapter is registered
    pub fn has_adapter(&self, name: &str) -> bool {
        self.adapters.contains_key(name)
    }

    /// Get an adapter by name
    pub fn get_adapter(&self, name: &str) -> Option<&Box<dyn ProtocolAdapter>> {
        self.adapters.get(name)
    }

    /// Route a request through the appropriate adapter
    pub async fn route_request(
        &self,
        protocol: &str,
        request: &str
    ) -> Result<String, String> {
        let adapter = self.get_adapter(protocol)
            .ok_or_else(|| format!("Adapter not found: {}", protocol))?;

        adapter.handle(request).await
    }
}
```

**Tests Required** (5-7 tests):
- `test_add_adapter`
- `test_has_adapter`
- `test_get_adapter`
- `test_multiple_adapters`
- `test_route_request_success`
- `test_route_request_unknown_adapter`
- `test_adapter_lifecycle`

---

### Phase 2: RestAdapter Implementation (2-3 days)

#### Full REST Protocol Support

```rust
// In src/router/rest.rs
pub struct RestAdapter {
    routes: Vec<RestRoute>,
    middleware: Vec<Box<dyn Middleware>>,
}

impl RestAdapter {
    pub fn new() -> Self {
        Self {
            routes: Vec::new(),
            middleware: Vec::new(),
        }
    }

    /// Register a REST route
    pub fn route(&mut self, method: &str, path: &str, handler_name: &str) {
        self.routes.push(RestRoute {
            method: method.to_string(),
            path: path.to_string(),
            handler: handler_name.to_string(),
        });
    }

    /// Match a request to a route
    fn match_route(&self, method: &str, path: &str) -> Option<&RestRoute> {
        self.routes.iter()
            .find(|r| r.method == method && r.matches_path(path))
    }
}

impl ProtocolAdapter for RestAdapter {
    fn name(&self) -> &str {
        "rest"
    }

    fn handle(
        &self,
        request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        Box::pin(async move {
            // Parse HTTP request
            let (method, path, body) = parse_http_request(request)?;

            // Match to route
            let route = self.match_route(&method, &path)
                .ok_or_else(|| format!("No route found: {} {}", method, path))?;

            // Transform to handler call
            // Execute handler
            // Transform response to HTTP

            Ok(format_http_response(200, &response_body))
        })
    }
}
```

**Tests Required** (8-10 tests):
- `test_rest_adapter_creation`
- `test_rest_route_registration`
- `test_rest_route_matching`
- `test_rest_path_params`
- `test_rest_query_params`
- `test_rest_request_parsing`
- `test_rest_response_formatting`
- `test_rest_error_handling`
- `test_rest_404_not_found`
- `test_rest_method_not_allowed`

---

### Phase 3: GraphQLAdapter Implementation (2-3 days)

#### Full GraphQL Protocol Support

```rust
// In src/router/graphql.rs
pub struct GraphQLAdapter {
    schema: GraphQLSchema,
    queries: HashMap<String, String>, // query_name -> handler_name
    mutations: HashMap<String, String>,
}

impl GraphQLAdapter {
    pub fn new() -> Self {
        Self {
            schema: GraphQLSchema::new(),
            queries: HashMap::new(),
            mutations: HashMap::new(),
        }
    }

    /// Register a GraphQL query
    pub fn query(&mut self, name: &str, handler_name: &str) {
        self.queries.insert(name.to_string(), handler_name.to_string());
    }

    /// Register a GraphQL mutation
    pub fn mutation(&mut self, name: &str, handler_name: &str) {
        self.mutations.insert(name.to_string(), handler_name.to_string());
    }

    /// Generate GraphQL schema from registered handlers
    pub fn generate_schema(&self) -> String {
        let mut schema = String::from("type Query {\n");
        for (query_name, _) in &self.queries {
            schema.push_str(&format!("  {}: String\n", query_name));
        }
        schema.push_str("}\n\ntype Mutation {\n");
        for (mutation_name, _) in &self.mutations {
            schema.push_str(&format!("  {}: String\n", mutation_name));
        }
        schema.push_str("}\n");
        schema
    }
}

impl ProtocolAdapter for GraphQLAdapter {
    fn name(&self) -> &str {
        "graphql"
    }

    fn handle(
        &self,
        request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        Box::pin(async move {
            // Parse GraphQL query/mutation
            let (operation_type, operation_name, variables) =
                parse_graphql_request(request)?;

            // Find handler
            let handler_name = match operation_type {
                "query" => self.queries.get(&operation_name),
                "mutation" => self.mutations.get(&operation_name),
                _ => None,
            }.ok_or_else(|| format!("Unknown operation: {}", operation_name))?;

            // Execute handler
            // Format GraphQL response

            Ok(format_graphql_response(&data))
        })
    }
}
```

**Tests Required** (8-10 tests):
- `test_graphql_adapter_creation`
- `test_graphql_query_registration`
- `test_graphql_mutation_registration`
- `test_graphql_schema_generation`
- `test_graphql_query_parsing`
- `test_graphql_mutation_parsing`
- `test_graphql_variables`
- `test_graphql_error_handling`
- `test_graphql_invalid_syntax`
- `test_graphql_unknown_operation`

---

### Phase 4: GrpcAdapter Implementation (3-4 days)

#### Full gRPC Protocol Support

```rust
// In src/router/grpc.rs
pub struct GrpcAdapter {
    services: HashMap<String, GrpcService>,
}

pub struct GrpcService {
    name: String,
    methods: HashMap<String, String>, // method_name -> handler_name
}

impl GrpcAdapter {
    pub fn new() -> Self {
        Self {
            services: HashMap::new(),
        }
    }

    /// Register a gRPC service
    pub fn service(&mut self, name: &str) -> &mut GrpcService {
        self.services.entry(name.to_string())
            .or_insert_with(|| GrpcService {
                name: name.to_string(),
                methods: HashMap::new(),
            })
    }

    /// Generate .proto file from registered services
    pub fn generate_proto(&self) -> String {
        let mut proto = String::from("syntax = \"proto3\";\n\n");

        for (service_name, service) in &self.services {
            proto.push_str(&format!("service {} {{\n", service_name));
            for (method_name, _) in &service.methods {
                proto.push_str(&format!(
                    "  rpc {}(Request) returns (Response);\n",
                    method_name
                ));
            }
            proto.push_str("}\n\n");
        }

        proto
    }
}

impl ProtocolAdapter for GrpcAdapter {
    fn name(&self) -> &str {
        "grpc"
    }

    fn handle(
        &self,
        request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        Box::pin(async move {
            // Parse gRPC request
            let (service_name, method_name, payload) =
                parse_grpc_request(request)?;

            // Find service and method
            let service = self.services.get(&service_name)
                .ok_or_else(|| format!("Unknown service: {}", service_name))?;

            let handler_name = service.methods.get(&method_name)
                .ok_or_else(|| format!("Unknown method: {}", method_name))?;

            // Execute handler
            // Format gRPC response

            Ok(format_grpc_response(&data))
        })
    }
}

impl GrpcService {
    /// Register a gRPC method
    pub fn method(&mut self, name: &str, handler_name: &str) {
        self.methods.insert(name.to_string(), handler_name.to_string());
    }
}
```

**Tests Required** (10-12 tests):
- `test_grpc_adapter_creation`
- `test_grpc_service_registration`
- `test_grpc_method_registration`
- `test_grpc_proto_generation`
- `test_grpc_unary_call`
- `test_grpc_request_parsing`
- `test_grpc_response_formatting`
- `test_grpc_error_handling`
- `test_grpc_unknown_service`
- `test_grpc_unknown_method`
- `test_grpc_service_discovery`
- `test_grpc_reflection`

---

### Phase 5: Integration Tests (2 days)

#### End-to-End Multi-Protocol Tests

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[tokio::test]
    async fn test_multi_protocol_same_handler() {
        let mut router = Router::new();

        // Register handler once
        router.register("get_user", || async {
            r#"{"id": 42, "name": "John"}"#.to_string()
        });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.service("UserService").method("GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        // Test REST
        let rest_response = router.route_request(
            "rest",
            "GET /users/42"
        ).await.unwrap();
        assert!(rest_response.contains("John"));

        // Test GraphQL
        let graphql_response = router.route_request(
            "graphql",
            "{ user(id: 42) { name } }"
        ).await.unwrap();
        assert!(graphql_response.contains("John"));

        // Test gRPC
        let grpc_response = router.route_request(
            "grpc",
            "UserService.GetUser {\"id\": 42}"
        ).await.unwrap();
        assert!(grpc_response.contains("John"));
    }
}
```

**Tests Required** (8-10 tests):
- `test_multi_protocol_same_handler`
- `test_protocol_specific_features`
- `test_error_handling_across_protocols`
- `test_adapter_isolation`
- `test_concurrent_protocol_requests`
- `test_handler_not_found_all_protocols`
- `test_schema_generation_integration`
- `test_real_world_scenario`

---

## Timeline and Effort

### Estimated Timeline
- **Phase 1** (Adapter Management): 2-3 days, ~300 lines + 7 tests
- **Phase 2** (RestAdapter): 2-3 days, ~500 lines + 10 tests
- **Phase 3** (GraphQLAdapter): 2-3 days, ~400 lines + 10 tests
- **Phase 4** (GrpcAdapter): 3-4 days, ~500 lines + 12 tests
- **Phase 5** (Integration Tests): 2 days, ~400 lines + 10 tests

**Total**: 11-15 days, ~2,100 lines, ~49 tests

### Code Statistics
```
Production code:     ~1,700 lines
Test code:           ~400 lines (49 tests)
────────────────────────────────
Total:               ~2,100 lines
```

---

## Success Criteria

### Functionality
- ✅ Single handler accessible via REST, GraphQL, AND gRPC
- ✅ Automatic schema generation for all protocols
- ✅ Protocol-specific error handling
- ✅ Request/response transformation
- ✅ All examples working

### Quality
- ✅ All 49+ integration tests passing
- ✅ Zero breaking changes
- ✅ 100% TDD (tests first)
- ✅ Zero clippy warnings
- ✅ Comprehensive documentation

### Performance
- ✅ Minimal overhead (<5% vs direct protocol implementation)
- ✅ Zero-copy where possible
- ✅ Async throughout

---

## Dependencies

### Required
- `tokio` - Async runtime (already used)
- `serde` / `serde_json` - Serialization (already used)

### Optional (for full implementations)
- `http` - HTTP types for REST
- `async-graphql` - GraphQL schema/execution
- `tonic` - gRPC support

---

## Migration Path

### For Existing Users

No breaking changes! The protocol-agnostic routing is additive:

```rust
// Before (still works)
let router = Router::new();
router.register("handler", handler_fn);

// After (opt-in to multi-protocol)
let mut router = Router::new();
router.register("handler", handler_fn);

// Add REST
let mut rest = RestAdapter::new();
rest.route("GET", "/api", "handler");
router.add_adapter(Box::new(rest));

// Add GraphQL
let mut graphql = GraphQLAdapter::new();
graphql.query("api", "handler");
router.add_adapter(Box::new(graphql));
```

---

## Documentation Requirements

### User-Facing Docs
1. **Guide**: "Protocol-Agnostic Routing"
2. **Guide**: "Building Multi-Protocol APIs"
3. **Example**: Update `examples/multi_protocol.rs` to use real implementations
4. **README**: Update feature status to ✅

### Developer Docs
1. **Architecture**: Protocol adapter design
2. **Contributing**: Adding new protocol adapters
3. **Testing**: Integration test patterns

---

## Future Enhancements (Post-v0.3)

1. **Configuration System** (v0.4)
   - YAML-based protocol configuration
   - Environment-based protocol selection
   - Dynamic protocol enabling/disabling

2. **Advanced Features** (v0.5+)
   - Protocol-specific middleware
   - Request streaming
   - Bi-directional protocols (WebSocket, gRPC streaming)
   - Protocol translation (REST → GraphQL, etc.)

3. **Performance** (v0.6+)
   - Zero-copy transformations
   - Protocol-specific optimizations
   - Benchmark suite

---

## Current Status Summary

**What's Done**:
- ✅ Router core (60 tests)
- ✅ Protocol adapter trait
- ✅ Basic adapter stubs
- ✅ Example code
- ✅ Documentation systems

**What's Needed**:
- 🚧 Adapter management (Router methods)
- 🚧 Full RestAdapter implementation
- 🚧 Full GraphQLAdapter implementation
- 🚧 Full GrpcAdapter implementation
- 🚧 Integration tests

**Completion**: ~60% done, ~40% remaining

**Next Step**: Begin Phase 1 (Adapter Management) with TDD approach

---

## Related Documentation

- [Phase 6.1 Complete](./PHASE6_1_ROUTER_COMPLETE.md) - Router core foundation
- [Multi-Protocol Example](../../examples/multi_protocol.rs) - Usage demonstration
- [Router Module](../../src/router/mod.rs) - Current implementation

---

**Ready to complete v0.3 Protocol-Agnostic Routing!** 🚀

This feature will make AllFrame the **first Rust framework** where you truly write once and deploy everywhere (REST, GraphQL, gRPC).
