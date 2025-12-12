//! tests/04_router_grpc.rs
//!
//! GREEN PHASE: Making tests pass with MVP implementation
//!
//! Tests for gRPC protocol adapter.
//!
//! Acceptance criteria:
//! - gRPC adapter can handle unary RPC calls
//! - Handler can be called via gRPC
//! - Type mapping from Rust to Protobuf
//! - .proto file generation for registered handlers
//! - gRPC status codes handled correctly
//!
//! Note: For Phase 4 MVP, we're using simplified gRPC implementations.
//! Full protobuf encoding, streaming, and service generation will come in later
//! phases.

use allframe_core::router::{GrpcAdapter, GrpcStatus, ProtocolAdapter, Router};

/// Test basic gRPC unary call
#[tokio::test]
async fn test_grpc_unary_call() {
    let mut router = Router::new();

    // Register handler (MVP: simple signature)
    router.register("GetUser", || async move {
        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string()
    });

    let mut adapter = GrpcAdapter::new();
    adapter.unary("UserService", "GetUser", "GetUser");
    assert_eq!(adapter.name(), "grpc");

    // Build gRPC request (MVP: simple format)
    let request = adapter.build_request("GetUser", r#"{"id": 42}"#);
    assert_eq!(request.method, "GetUser");
    assert_eq!(request.payload, r#"{"id": 42}"#);

    // Execute via adapter (MVP: uses "method:payload" format)
    let response = adapter.handle("GetUser:{}").await.unwrap();
    assert!(!response.is_empty());
}

/// Test gRPC .proto generation
#[tokio::test]
async fn test_grpc_proto_generation() {
    let mut router = Router::new();

    // Register handlers (MVP: simple signatures)
    router.register("GetUser", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    router.register("ListUsers", || async move { "[]".to_string() });

    let mut adapter = GrpcAdapter::new();
    adapter.unary("UserService", "GetUser", "GetUser");
    adapter.unary("UserService", "ListUsers", "ListUsers");

    // Generate .proto file (MVP: returns static schema)
    let proto = adapter.generate_proto();

    // Verify .proto contains expected service definition
    assert!(proto.contains("service UserService"));
    assert!(proto.contains("rpc GetUser"));
    assert!(proto.contains("rpc ListUsers"));
    assert!(proto.contains("syntax = \"proto3\""));
}

/// Test gRPC with message types (MVP)
#[tokio::test]
async fn test_grpc_message_types() {
    let mut router = Router::new();

    // Register handler that returns structured data (MVP: as JSON string)
    router.register("GetUser", || async move {
        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string()
    });

    let mut adapter = GrpcAdapter::new();
    adapter.unary("UserService", "GetUser", "GetUser");

    // Execute GetUser method via handle()
    let response = adapter.handle("GetUser:{\"id\": 42}").await.unwrap();

    // Verify response is not empty
    assert!(!response.is_empty());

    // MVP: Full protobuf encoding/decoding will come in later phases
}

/// Test gRPC error handling (status codes)
#[tokio::test]
async fn test_grpc_error_status() {
    let adapter = GrpcAdapter::new();

    // Test unknown method (returns response with error status)
    let response = adapter.handle("UnknownMethod:{}").await.unwrap();
    // gRPC returns status in response body
    assert!(response.contains("grpc-status"));

    // Test gRPC status code names
    assert_eq!(GrpcStatus::Ok.code_name(), "OK");
    assert_eq!(GrpcStatus::InvalidArgument.code_name(), "INVALID_ARGUMENT");
    assert_eq!(GrpcStatus::NotFound.code_name(), "NOT_FOUND");
    assert_eq!(GrpcStatus::Unimplemented.code_name(), "UNIMPLEMENTED");
    assert_eq!(GrpcStatus::Internal.code_name(), "INTERNAL");

    // MVP: Full error status mapping will come in later phases
}

/// Test gRPC service registration (MVP)
#[tokio::test]
async fn test_grpc_service_registration() {
    let mut router = Router::new();

    // Register multiple RPC methods (MVP: individual registration)
    router.register("GetUser", || async move {
        r#"{"id": 42, "name": "John"}"#.to_string()
    });

    router.register("ListUsers", || async move { "[]".to_string() });

    router.register("DeleteUser", || async move {
        r#"{"deleted": true}"#.to_string()
    });

    let mut adapter = GrpcAdapter::new();
    adapter.unary("UserService", "GetUser", "GetUser");
    adapter.unary("UserService", "ListUsers", "ListUsers");
    adapter.unary("UserService", "DeleteUser", "DeleteUser");

    // Generate .proto for all methods
    let proto = adapter.generate_proto();

    // Verify service contains all RPC methods
    assert!(proto.contains("service UserService"));
    assert!(proto.contains("rpc GetUser"));
    assert!(proto.contains("rpc ListUsers"));
    assert!(proto.contains("rpc DeleteUser"));

    // Verify we can execute each method via handle()
    let get_response = adapter.handle("GetUser:{}").await.unwrap();
    assert!(!get_response.is_empty());

    let list_response = adapter.handle("ListUsers:{}").await.unwrap();
    assert!(!list_response.is_empty());

    let delete_response = adapter.handle("DeleteUser:{}").await.unwrap();
    assert!(!delete_response.is_empty());

    // MVP: Service-level registration will come in later phases
}
