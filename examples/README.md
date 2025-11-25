# AllFrame Examples

Welcome to the AllFrame examples! These examples demonstrate how to use AllFrame's protocol-agnostic routing system to build APIs that work across multiple protocols.

## Overview

AllFrame's core value proposition is: **Write your handlers once, expose them via multiple protocols.**

These examples show you how to:
- Build REST APIs
- Build GraphQL APIs
- Expose the same handlers via both REST and GraphQL
- Handle errors consistently across protocols
- Generate schemas automatically

## Running the Examples

Each example is a standalone Rust binary that you can run with:

```bash
cargo run --example <example_name>
```

## Available Examples

### 1. REST API (`rest_api.rs`)

Demonstrates how to build a REST API using AllFrame's REST adapter.

**Run it:**
```bash
cargo run --example rest_api
```

**What you'll learn:**
- Router setup with REST adapter
- Handler registration for different endpoints
- GET/POST request handling
- Query parameter handling
- Error handling
- RestResponse structure
- Integration points for OpenAPI (Milestone 0.2)

**Key concepts:**
```rust
// Create a router
let mut router = Router::new();

// Register handlers
router.register("get_user", || async move {
    r#"{"id": 42, "name": "John Doe"}"#.to_string()
});

// Create REST adapter
let adapter = RestAdapter::new();

// Build requests
let request = adapter.build_request("GET", "/users/42", None, None);

// Execute handlers
let response = router.execute("get_user").await.unwrap();
```

### 2. gRPC API (`grpc_api.rs`)

Demonstrates how to build a gRPC service using AllFrame's gRPC adapter.

**Run it:**
```bash
cargo run --example grpc_api
```

**What you'll learn:**
- Router setup with gRPC adapter
- RPC method handlers (unary calls)
- Protocol Buffer (.proto) schema generation
- gRPC status codes
- Service registration
- Error handling for unimplemented methods

**Key concepts:**
```rust
// Create a gRPC adapter
let adapter = GrpcAdapter::new();

// Generate .proto schema
let proto = adapter.generate_proto();

// Build RPC requests
let request = adapter.build_request("GetUser", r#"{"id": 42}"#);

// Execute RPC methods
let response = adapter.execute("GetUser", "{}").await.unwrap();

// gRPC status codes
println!("{}", GrpcStatus::Ok.code_name()); // "OK"
println!("{}", GrpcStatus::InvalidArgument.code_name()); // "INVALID_ARGUMENT"
```

### 3. GraphQL API (`graphql_api.rs`)

Demonstrates how to build a GraphQL API using AllFrame's GraphQL adapter.

**Run it:**
```bash
cargo run --example graphql_api
```

**What you'll learn:**
- Router setup with GraphQL adapter
- Query and mutation handlers
- GraphQL schema generation
- Query execution and validation
- Both explicit and shorthand query syntax
- Nested type support (MVP)
- Error handling for invalid queries

**Key concepts:**
```rust
// Create a GraphQL adapter
let adapter = GraphQLAdapter::new();

// Generate schema
let schema = adapter.generate_schema();

// Execute queries
let query = r#"
    query {
        user(id: 42) {
            id
            name
        }
    }
"#;
let response = adapter.execute(query).await.unwrap();

// Execute mutations
let mutation = r#"
    mutation {
        createUser(name: "Charlie", email: "charlie@example.com") {
            id
            name
        }
    }
"#;
let response = adapter.execute(mutation).await.unwrap();
```

### 4. Multi-Protocol Router (`multi_protocol.rs`)

**⭐ This is the flagship example!** It demonstrates AllFrame's core value proposition: writing handlers once and exposing them via multiple protocols.

**Run it:**
```bash
cargo run --example multi_protocol
```

**What you'll learn:**
- Single handler registration
- Multiple protocol adapters (REST, GraphQL, and gRPC)
- Protocol-agnostic handler design
- Accessing the same handler via different protocols
- Unified error handling across protocols
- Preview of config-driven protocol selection (Phase 5)

**Key concepts:**
```rust
// Step 1: Register handlers ONCE
let mut router = Router::new();
router.register("get_user", || async move {
    r#"{"id": 42, "name": "John Doe"}"#.to_string()
});

// Step 2: Register multiple protocol adapters
router.add_adapter(Box::new(RestAdapter::new()));
router.add_adapter(Box::new(GraphQLAdapter::new()));
router.add_adapter(Box::new(GrpcAdapter::new()));

// Step 3: Access via REST
let rest = RestAdapter::new();
let request = rest.build_request("GET", "/users/42", None, None);
let response = router.execute("get_user").await.unwrap();

// Step 4: Access via GraphQL
let graphql = GraphQLAdapter::new();
let query = "query { user(id: 42) { name } }";
let response = graphql.execute(query).await.unwrap();

// Step 5: Access via gRPC
let grpc = GrpcAdapter::new();
let response = grpc.execute("GetUser", "{}").await.unwrap();
```

## Example Output

When you run these examples, you'll see detailed output showing:
- Router initialization
- Handler registration
- Request building
- Response handling
- Error cases
- Schema generation (GraphQL)

Each example includes explanatory comments and clear section markers to help you understand what's happening at each step.

## Current Implementation Status

These examples demonstrate **Milestone 0.3 - Protocol-Agnostic Routing** features:

✅ **Phase 1: Core Router** (Complete)
- Handler registration and execution
- Protocol adapter trait
- Handler trait with async support

✅ **Phase 2: REST Adapter** (MVP Complete)
- Basic REST request/response types
- Request building
- Method and path handling
- Query parameter support (basic)

✅ **Phase 3: GraphQL Adapter** (MVP Complete)
- Query and mutation execution
- Schema generation (GraphQL SDL)
- Query validation
- Shorthand query syntax

✅ **Phase 4: gRPC Adapter** (MVP Complete)
- Unary RPC calls
- Protocol Buffer (.proto) generation
- gRPC status codes
- Service registration

🚧 **Phase 5: Configuration System** (Coming Soon)
- YAML-based protocol configuration
- Dynamic protocol enabling/disabling
- Route mapping configuration

## MVP Notes

The current implementation is an **MVP (Minimum Viable Product)**. This means:

- **REST adapter**: Simplified request/response handling. Full HTTP integration, parameter extraction, and middleware coming in future phases.

- **GraphQL adapter**: Basic query parsing and validation. Full AST parsing, resolver system, and field selection coming in future phases.

- **gRPC adapter**: JSON-based message encoding. Full protobuf encoding/decoding, streaming RPCs (client/server/bidirectional), and reflection API coming in future phases.

- **Handlers**: Simple async functions returning strings. Rich type support, dependency injection, and request context coming in future phases.

The MVP approach allows us to validate the architecture and provide working examples while we build toward a production-ready system.

## Integration with Previous Milestones

These examples build on previous AllFrame milestones:

- **Milestone 0.1**: Foundation and core architecture
- **Milestone 0.2**: Dependency Injection and OpenAPI schema generation
  - The REST adapter will integrate with OpenAPI in future iterations
  - DI will enable rich handler dependencies in future phases

## Next Steps

After exploring these examples, you can:

1. **Experiment**: Modify the handlers to return different data
2. **Add handlers**: Register your own handlers and expose them via both protocols
3. **Explore tests**: Check `tests/04_router_*.rs` for detailed test cases
4. **Read the code**: Dive into `crates/allframe-core/src/router/` to see the implementation

## Getting Help

- **Documentation**: Check the inline code documentation with `cargo doc --open`
- **Tests**: Look at the test files in `tests/` for more examples
- **Issues**: Report issues or ask questions on GitHub

## Future Examples (Coming Soon)

- `config_driven.rs` - YAML-configured multi-protocol API (Phase 5)
- `full_stack.rs` - Complete application with DI, OpenAPI, and multi-protocol support
- `streaming_grpc.rs` - gRPC streaming RPCs (client, server, and bidirectional)

---

**One frame. Infinite transformations.**

Welcome to AllFrame!
