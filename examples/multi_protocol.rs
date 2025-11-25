//! Multi-Protocol Router Example
//!
//! This example demonstrates AllFrame's core value proposition:
//! Write your handlers once, expose them via multiple protocols.
//!
//! Key concepts:
//! - Single handler registration
//! - Multiple protocol adapters (REST, GraphQL, and gRPC)
//! - Protocol-agnostic handler design
//! - Config-driven protocol selection (preview)
//! - Unified error handling across protocols
//!
//! Run this example:
//! ```bash
//! cargo run --example multi_protocol
//! ```

use allframe_core::prelude::*;

#[tokio::main]
async fn main() {
    println!("=== AllFrame Multi-Protocol Router Example ===");
    println!("One frame. Infinite transformations.\n");

    // Create a new router
    let mut router = Router::new();

    // ============================================
    // STEP 1: Register handlers ONCE
    // ============================================
    println!("--- Step 1: Register Handlers Once ---");

    // Handler 1: Get user information
    // This single handler will be accessible via REST, GraphQL, AND gRPC
    router.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string()
    });

    // Handler 2: List all users
    router.register("list_users", || async move {
        r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}, {"id": 3, "name": "Charlie"}]"#
            .to_string()
    });

    // Handler 3: Create a new user
    router.register("create_user", || async move {
        r#"{"id": 4, "name": "David", "email": "david@example.com", "created": true}"#
            .to_string()
    });

    println!("✓ Registered {} handlers", router.handlers_count());
    println!("  - get_user");
    println!("  - list_users");
    println!("  - create_user\n");

    // ============================================
    // STEP 2: Register protocol adapters
    // ============================================
    println!("--- Step 2: Register Protocol Adapters ---");

    // Add REST adapter
    let rest_adapter = RestAdapter::new();
    router.add_adapter(Box::new(rest_adapter));
    println!("✓ REST adapter registered");

    // Add GraphQL adapter
    let graphql_adapter = GraphQLAdapter::new();
    router.add_adapter(Box::new(graphql_adapter));
    println!("✓ GraphQL adapter registered");

    // Add gRPC adapter
    let grpc_adapter = GrpcAdapter::new();
    router.add_adapter(Box::new(grpc_adapter));
    println!("✓ gRPC adapter registered");

    // Verify adapters are registered
    assert!(router.has_adapter("rest"));
    assert!(router.has_adapter("graphql"));
    assert!(router.has_adapter("grpc"));
    println!();

    // ============================================
    // STEP 3: Access via REST
    // ============================================
    println!("--- Step 3: Access Handlers via REST ---");

    let rest = RestAdapter::new();

    // REST GET request
    println!("\nREST: GET /users/42");
    let get_request = rest.build_request("GET", "/users/42", None, None);
    println!("  Method: {}", get_request.method);
    println!("  Path: {}", get_request.path);

    let response = router.execute("get_user").await.unwrap();
    println!("  Response: {}", response);

    // REST GET all users
    println!("\nREST: GET /users");
    let list_request = rest.build_request("GET", "/users", None, None);
    println!("  Method: {}", list_request.method);
    println!("  Path: {}", list_request.path);

    let response = router.execute("list_users").await.unwrap();
    println!("  Response: {}", response);

    // REST POST request
    println!("\nREST: POST /users");
    let post_body = r#"{"name": "David", "email": "david@example.com"}"#;
    let post_request = rest.build_request("POST", "/users", Some(post_body), None);
    println!("  Method: {}", post_request.method);
    println!("  Path: {}", post_request.path);
    println!("  Body: {}", post_body);

    let response = router.execute("create_user").await.unwrap();
    println!("  Response: {}\n", response);

    // ============================================
    // STEP 4: Access via GraphQL
    // ============================================
    println!("--- Step 4: Access Handlers via GraphQL ---");

    let graphql = GraphQLAdapter::new();

    // GraphQL query for single user
    println!("\nGraphQL: Query for user");
    let query = r#"
        query {
            user(id: 42) {
                id
                name
                email
            }
        }
    "#;
    println!("  Query: {}", query.trim());

    let response = graphql.execute(query).await.unwrap();
    println!("  Response: {}", response);

    // GraphQL shorthand query syntax
    println!("\nGraphQL: Shorthand query");
    let shorthand = "{ user(id: 42) { name } }";
    println!("  Query: {}", shorthand);

    let response = graphql.execute(shorthand).await.unwrap();
    println!("  Response: {}", response);

    // GraphQL mutation
    println!("\nGraphQL: Mutation to create user");
    let mutation = r#"
        mutation {
            createUser(name: "David", email: "david@example.com") {
                id
                name
                email
            }
        }
    "#;
    println!("  Mutation: {}", mutation.trim());

    let response = graphql.execute(mutation).await.unwrap();
    println!("  Response: {}\n", response);

    // ============================================
    // STEP 5: Access via gRPC
    // ============================================
    println!("--- Step 5: Access Handlers via gRPC ---");

    let grpc = GrpcAdapter::new();

    // gRPC unary RPC call for single user
    println!("\ngRPC: GetUser RPC");
    let grpc_request = grpc.build_request("GetUser", r#"{"id": 42}"#);
    println!("  Method: {}", grpc_request.method);
    println!("  Payload: {}", grpc_request.payload);

    let response = grpc.execute("GetUser", "{}").await.unwrap();
    println!("  Response: {}", response);

    // gRPC unary RPC call for list
    println!("\ngRPC: ListUsers RPC");
    let list_request = grpc.build_request("ListUsers", "{}");
    println!("  Method: {}", list_request.method);

    let response = grpc.execute("ListUsers", "{}").await.unwrap();
    println!("  Response: {}", response);

    // gRPC unary RPC call for create
    println!("\ngRPC: CreateUser RPC");
    let create_request = grpc.build_request("CreateUser", r#"{"name": "David", "email": "david@example.com"}"#);
    println!("  Method: {}", create_request.method);

    let response = grpc.execute("CreateUser", "{}").await.unwrap();
    println!("  Response: {}\n", response);

    // ============================================
    // STEP 6: Show Schema Generation
    // ============================================
    println!("--- Step 6: Schema Generation ---");

    println!("\nGraphQL Schema:");
    let graphql_schema = graphql.generate_schema();
    println!("{}", graphql_schema);

    println!("\ngRPC .proto file:");
    let grpc_proto = grpc.generate_proto();
    println!("{}\n", grpc_proto);

    // ============================================
    // STEP 7: Error handling across protocols
    // ============================================
    println!("--- Step 7: Unified Error Handling ---");

    println!("\nREST: Accessing non-existent handler");
    match router.execute("nonexistent").await {
        Ok(response) => println!("  Response: {}", response),
        Err(error) => println!("  Error: {}", error),
    }

    println!("\nGraphQL: Invalid query syntax");
    match graphql.execute("not a valid query").await {
        Ok(response) => println!("  Response: {}", response),
        Err(error) => println!("  Error: {}", error),
    }

    println!("\ngRPC: Unknown RPC method");
    match grpc.execute("UnknownMethod", "{}").await {
        Ok(response) => println!("  Response: {}", response),
        Err(error) => println!("  Error: {}", error),
    }

    println!();

    // ============================================
    // STEP 8: The power of protocol-agnostic design
    // ============================================
    println!("--- Step 8: The AllFrame Value Proposition ---");
    println!();
    println!("✓ Single handler implementation");
    println!("✓ Multiple protocol exposures (REST, GraphQL, gRPC)");
    println!("✓ Zero code duplication");
    println!("✓ Consistent error handling");
    println!("✓ Type-safe routing");
    println!("✓ Async by default");
    println!();
    println!("The same handler 'get_user' was accessed via:");
    println!("  1. REST:    GET /users/42");
    println!("  2. GraphQL: query {{ user(id: 42) {{ ... }} }}");
    println!("  3. gRPC:    GetUser({{id: 42}})");
    println!();
    println!("With AllFrame, you write once and deploy everywhere!");
    println!();

    // ============================================
    // PREVIEW: Config-driven protocol selection
    // ============================================
    println!("--- Preview: Future Config System (Phase 5) ---");
    println!();
    println!("In the future, you'll be able to configure protocols via YAML:");
    println!();
    println!(r#"  protocols:
    rest:
      enabled: true
      port: 8080
      routes:
        - path: /users/:id
          method: GET
          handler: get_user

    graphql:
      enabled: true
      port: 8081
      endpoint: /graphql
      handlers:
        - query: user
          handler: get_user

    grpc:
      enabled: true
      port: 50051
      service: UserService
      methods:
        - rpc: GetUser
          handler: get_user"#);
    println!();
    println!("Stay tuned for Phase 5: Configuration System!");
    println!();

    println!("=== Multi-Protocol Router Example Complete ===");
    println!("\nNext steps:");
    println!("- Explore examples/rest_api.rs for REST-specific features");
    println!("- Explore examples/graphql_api.rs for GraphQL-specific features");
    println!("- Explore examples/grpc_api.rs for gRPC-specific features");
    println!("- Watch for Phase 5: Configuration system");
}
