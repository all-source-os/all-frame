//! Protocol-Agnostic Routing Example
//!
//! This example demonstrates AllFrame's protocol-agnostic routing capability:
//! Write a handler ONCE, expose it via REST, GraphQL, and gRPC.
//!
//! Run with:
//! ```bash
//! cargo run --example protocol_agnostic
//! ```

use allframe_core::router::{GraphQLAdapter, GrpcAdapter, RestAdapter, Router};

#[tokio::main]
async fn main() {
    println!("🚀 AllFrame Protocol-Agnostic Routing Example\n");
    println!("═══════════════════════════════════════════════\n");

    // Create router
    let mut router = Router::new();

    // Register handlers ONCE
    println!("📝 Registering handlers...\n");
    router.register("get_user", || async { "User data".to_string() });
    router.register("list_users", || async { "Users list".to_string() });
    router.register("create_user", || async { "Created user".to_string() });
    router.register("update_user", || async { "Updated user".to_string() });
    router.register("delete_user", || async { "Deleted user".to_string() });

    // Configure REST adapter
    println!("🔧 Configuring REST adapter...");
    let mut rest = RestAdapter::new();
    rest.route("GET", "/users/:id", "get_user");
    rest.route("GET", "/users", "list_users");
    rest.route("POST", "/users", "create_user");
    rest.route("PUT", "/users/:id", "update_user");
    rest.route("DELETE", "/users/:id", "delete_user");
    router.add_adapter(Box::new(rest));
    println!("   ✅ REST adapter registered with 5 routes\n");

    // Configure GraphQL adapter (keep reference for schema generation)
    println!("🔧 Configuring GraphQL adapter...");
    let mut graphql = GraphQLAdapter::new();
    graphql.query("user", "get_user");
    graphql.query("users", "list_users");
    graphql.mutation("createUser", "create_user");
    graphql.mutation("updateUser", "update_user");
    graphql.mutation("deleteUser", "delete_user");
    let graphql_schema = graphql.generate_schema();
    router.add_adapter(Box::new(graphql));
    println!("   ✅ GraphQL adapter registered with 2 queries, 3 mutations\n");

    // Configure gRPC adapter (keep reference for proto generation)
    println!("🔧 Configuring gRPC adapter...");
    let mut grpc = GrpcAdapter::new();
    grpc.unary("UserService", "GetUser", "get_user");
    grpc.server_streaming("UserService", "ListUsers", "list_users");
    grpc.unary("UserService", "CreateUser", "create_user");
    grpc.unary("UserService", "UpdateUser", "update_user");
    grpc.unary("UserService", "DeleteUser", "delete_user");
    let grpc_proto = grpc.generate_proto();
    router.add_adapter(Box::new(grpc));
    println!("   ✅ gRPC adapter registered with 5 methods\n");

    println!("═══════════════════════════════════════════════\n");

    // Test REST
    println!("🌐 Testing REST Protocol:\n");
    test_rest(&router).await;

    println!("\n═══════════════════════════════════════════════\n");

    // Test GraphQL
    println!("📊 Testing GraphQL Protocol:\n");
    test_graphql(&router, &graphql_schema).await;

    println!("\n═══════════════════════════════════════════════\n");

    // Test gRPC
    println!("⚡ Testing gRPC Protocol:\n");
    test_grpc(&router, &grpc_proto).await;

    println!("\n═══════════════════════════════════════════════\n");
    println!("✨ Summary:\n");
    println!("   • 5 handlers written ONCE");
    println!("   • Exposed via 3 protocols (REST, GraphQL, gRPC)");
    println!("   • 15 total endpoints (5 REST + 5 GraphQL + 5 gRPC)");
    println!("   • Zero code duplication!");
    println!("\n🎉 Protocol-Agnostic Routing is AMAZING!\n");
}

async fn test_rest(router: &Router) {
    println!("   GET /users/42");
    match router.route_request("rest", "GET /users/42").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   GET /users");
    match router.route_request("rest", "GET /users").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   POST /users");
    match router.route_request("rest", "POST /users").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   PUT /users/42");
    match router.route_request("rest", "PUT /users/42").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   DELETE /users/42");
    match router.route_request("rest", "DELETE /users/42").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }
}

async fn test_graphql(router: &Router, schema: &str) {
    println!("   Query: {{ user }}");
    match router.route_request("graphql", "query { user }").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   Query: {{ users }}");
    match router.route_request("graphql", "query { users }").await {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   Mutation: {{ createUser }}");
    match router
        .route_request("graphql", "mutation { createUser }")
        .await
    {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   📋 Generated GraphQL Schema:\n");
    for line in schema.lines() {
        println!("      {}", line);
    }
    println!();
}

async fn test_grpc(router: &Router, proto: &str) {
    println!("   UserService.GetUser");
    match router
        .route_request("grpc", "UserService.GetUser:{\"id\":42}")
        .await
    {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   UserService.ListUsers");
    match router
        .route_request("grpc", "UserService.ListUsers:{}")
        .await
    {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   UserService.CreateUser");
    match router
        .route_request("grpc", "UserService.CreateUser:{\"name\":\"John\"}")
        .await
    {
        Ok(response) => println!("   → {}\n", response),
        Err(e) => println!("   ✗ Error: {}\n", e),
    }

    println!("   📋 Generated Protocol Buffer (.proto):\n");
    for line in proto.lines() {
        println!("      {}", line);
    }
    println!();
}
