//! gRPC API Example
//!
//! This example demonstrates how to use AllFrame's gRPC adapter to build a gRPC service.
//!
//! Key concepts:
//! - Router setup with gRPC adapter
//! - RPC method handlers (unary calls)
//! - Protocol Buffer (.proto) schema generation
//! - gRPC status codes
//! - Service registration
//!
//! Run this example:
//! ```bash
//! cargo run --example grpc_api
//! ```

use allframe_core::prelude::*;

#[tokio::main]
async fn main() {
    println!("=== AllFrame gRPC API Example ===\n");

    // Create a new router
    let mut router = Router::new();

    // Register RPC method handlers
    // Each handler is a simple async function that returns JSON (protobuf in production)

    // RPC: GetUser(GetUserRequest) returns (GetUserResponse)
    router.register("GetUser", || async move {
        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string()
    });

    // RPC: ListUsers(ListUsersRequest) returns (ListUsersResponse)
    router.register("ListUsers", || async move {
        r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#.to_string()
    });

    // RPC: CreateUser(CreateUserRequest) returns (CreateUserResponse)
    router.register("CreateUser", || async move {
        r#"{"id": 3, "name": "Charlie", "email": "charlie@example.com"}"#.to_string()
    });

    // RPC: DeleteUser(DeleteUserRequest) returns (DeleteUserResponse)
    router.register("DeleteUser", || async move {
        r#"{"deleted": true}"#.to_string()
    });

    // Create and register the gRPC adapter
    let adapter = GrpcAdapter::new();
    router.add_adapter(Box::new(adapter));

    println!("✓ Router initialized with {} handlers", router.handlers_count());
    println!("✓ gRPC adapter registered\n");

    // Demonstrate the gRPC adapter capabilities
    let grpc_adapter = GrpcAdapter::new();

    println!("--- Example 1: Protocol Buffer Schema Generation ---");
    // Generate .proto file for the service
    let proto = grpc_adapter.generate_proto();
    println!("Generated .proto file:\n{}\n", proto);

    println!("--- Example 2: Unary RPC Call (GetUser) ---");
    // Build a gRPC request
    let request = grpc_adapter.build_request("GetUser", r#"{"id": 42}"#);
    println!("Method: {}", request.method);
    println!("Payload: {}", request.payload);

    // Execute the RPC method
    let response = grpc_adapter.execute("GetUser", "{}").await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 3: Unary RPC Call (ListUsers) ---");
    // Execute ListUsers RPC
    let list_request = grpc_adapter.build_request("ListUsers", "{}");
    println!("Method: {}", list_request.method);

    let response = grpc_adapter.execute("ListUsers", "{}").await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 4: Unary RPC Call (CreateUser) ---");
    // Execute CreateUser RPC
    let create_request =
        grpc_adapter.build_request("CreateUser", r#"{"name": "Charlie", "email": "charlie@example.com"}"#);
    println!("Method: {}", create_request.method);
    println!("Payload: {}", create_request.payload);

    let response = grpc_adapter.execute("CreateUser", "{}").await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 5: Unary RPC Call (DeleteUser) ---");
    // Execute DeleteUser RPC
    let delete_request = grpc_adapter.build_request("DeleteUser", r#"{"id": 42}"#);
    println!("Method: {}", delete_request.method);

    let response = grpc_adapter.execute("DeleteUser", "{}").await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 6: gRPC Status Codes ---");
    // gRPC uses specific status codes for errors
    println!("Available gRPC status codes:");
    println!("  {:?} = {}", GrpcStatus::Ok as i32, GrpcStatus::Ok.code_name());
    println!(
        "  {:?} = {}",
        GrpcStatus::InvalidArgument as i32,
        GrpcStatus::InvalidArgument.code_name()
    );
    println!(
        "  {:?} = {}",
        GrpcStatus::NotFound as i32,
        GrpcStatus::NotFound.code_name()
    );
    println!(
        "  {:?} = {}",
        GrpcStatus::Unimplemented as i32,
        GrpcStatus::Unimplemented.code_name()
    );
    println!(
        "  {:?} = {}",
        GrpcStatus::Internal as i32,
        GrpcStatus::Internal.code_name()
    );
    println!();

    println!("--- Example 7: Error Handling (Unknown Method) ---");
    // Try to call an unimplemented RPC method
    let unknown_request = grpc_adapter.build_request("UnknownMethod", "{}");
    println!("Method: {}", unknown_request.method);

    match grpc_adapter.execute("UnknownMethod", "{}").await {
        Ok(response) => println!("Response: {}", response),
        Err(error) => println!("Error: {}\n", error),
    }

    println!("--- Example 8: Using ProtocolAdapter Trait ---");
    // The gRPC adapter also implements the ProtocolAdapter trait
    println!("Adapter name: {}", grpc_adapter.name());

    // Use the handle method (format: "method:payload")
    let response = grpc_adapter.handle("GetUser:{}").await.unwrap();
    println!("Via handle(): {}\n", response);

    println!("--- Example 9: Service Registration ---");
    // Multiple RPC methods form a gRPC service
    println!("UserService with {} RPC methods:", router.handlers_count());
    println!("  1. GetUser(GetUserRequest) returns (GetUserResponse)");
    println!("  2. ListUsers(ListUsersRequest) returns (ListUsersResponse)");
    println!("  3. CreateUser(CreateUserRequest) returns (CreateUserResponse)");
    println!("  4. DeleteUser(DeleteUserRequest) returns (DeleteUserResponse)");
    println!();

    println!("=== Key Takeaways ===");
    println!("1. gRPC adapter handles RPC method routing");
    println!("2. .proto file generation creates Protocol Buffer definitions");
    println!("3. Unary RPC calls are the simplest form (request → response)");
    println!("4. gRPC status codes provide standardized error handling");
    println!("5. Multiple RPC methods combine to form a gRPC service");
    println!("6. MVP uses JSON; full protobuf encoding coming in future phases");
    println!("\n✓ gRPC API example complete!");
    println!("\nNext steps:");
    println!("- See examples/rest_api.rs for REST support");
    println!("- See examples/graphql_api.rs for GraphQL support");
    println!("- See examples/multi_protocol.rs for multi-protocol routing");
    println!("- Full protobuf encoding, streaming RPCs, and bidirectional communication coming in future phases");
}
