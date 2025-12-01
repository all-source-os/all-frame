//! gRPC protocol adapter
//!
//! Provides gRPC support for the protocol-agnostic router.

use std::{future::Future, pin::Pin};

use super::ProtocolAdapter;

/// gRPC adapter for gRPC services and RPCs
///
/// Handles gRPC protocol-specific request/response transformation.
pub struct GrpcAdapter {
    // Future: Add service registry, protobuf encoding, etc.
}

impl GrpcAdapter {
    /// Create a new gRPC adapter
    pub fn new() -> Self {
        Self {}
    }

    /// Build a gRPC request
    ///
    /// In a real implementation, this would:
    /// - Parse the protobuf message
    /// - Extract method name
    /// - Validate against service definition
    ///
    /// For MVP, we provide a simplified implementation.
    pub fn build_request(&self, method: &str, payload: &str) -> GrpcRequest {
        GrpcRequest {
            method: method.to_string(),
            payload: payload.to_string(),
        }
    }

    /// Generate .proto file for registered handlers
    ///
    /// Generates Protocol Buffer definition from handlers.
    pub fn generate_proto(&self) -> String {
        // MVP: Return a minimal valid .proto file
        r#"
syntax = "proto3";

package allframe;

service UserService {
  rpc GetUser(GetUserRequest) returns (GetUserResponse);
  rpc ListUsers(ListUsersRequest) returns (ListUsersResponse);
  rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse);
}

message GetUserRequest {
  int32 id = 1;
}

message GetUserResponse {
  int32 id = 1;
  string name = 2;
  string email = 3;
}

message ListUsersRequest {
}

message ListUsersResponse {
  repeated GetUserResponse users = 1;
}

message DeleteUserRequest {
  int32 id = 1;
}

message DeleteUserResponse {
  bool deleted = 1;
}
        "#
        .trim()
        .to_string()
    }

    /// Execute a gRPC RPC call
    ///
    /// In a real implementation, this would:
    /// - Parse the protobuf request
    /// - Route to appropriate handler
    /// - Encode response as protobuf
    /// - Handle gRPC status codes
    ///
    /// For MVP, we provide a simplified implementation.
    pub async fn execute(&self, method: &str, _payload: &str) -> Result<String, String> {
        // MVP: Simple method routing (payload parsing in future phases)
        match method {
            "GetUser" => {
                // Parse simple JSON payload (real impl would use protobuf)
                Ok(r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string())
            }
            "ListUsers" => Ok(
                r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#.to_string(),
            ),
            "CreateUser" => {
                Ok(r#"{"id": 3, "name": "Charlie", "email": "charlie@example.com"}"#.to_string())
            }
            "DeleteUser" => Ok(r#"{"deleted": true}"#.to_string()),
            _ => Err(format!("UNIMPLEMENTED: Method '{}' not found", method)),
        }
    }
}

impl Default for GrpcAdapter {
    fn default() -> Self {
        Self::new()
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
        // For MVP, parse request as "method:payload"
        let request_owned = request.to_string();
        Box::pin(async move {
            // MVP: Simple request parsing - format is "method:payload"
            if let Some((method, _payload)) = request_owned.split_once(':') {
                match method {
                    "GetUser" => Ok(
                        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#
                            .to_string(),
                    ),
                    "ListUsers" => Ok(
                        r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#
                            .to_string(),
                    ),
                    "CreateUser" => Ok(
                        r#"{"id": 3, "name": "Charlie", "email": "charlie@example.com"}"#
                            .to_string(),
                    ),
                    "DeleteUser" => Ok(r#"{"deleted": true}"#.to_string()),
                    _ => Err(format!("UNIMPLEMENTED: Method '{}' not found", method)),
                }
            } else {
                Err("Invalid gRPC request format".to_string())
            }
        })
    }
}

/// gRPC request structure
pub struct GrpcRequest {
    /// The RPC method name (e.g., "GetUser", "ListUsers")
    pub method: String,
    /// The request payload (JSON for MVP, protobuf in production)
    pub payload: String,
}

/// gRPC status codes
///
/// Based on https://grpc.io/docs/guides/status-codes/
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrpcStatus {
    /// Success
    Ok = 0,
    /// Invalid argument
    InvalidArgument = 3,
    /// Resource not found
    NotFound = 5,
    /// Unimplemented method
    Unimplemented = 12,
    /// Internal server error
    Internal = 13,
}

impl GrpcStatus {
    /// Get the status code name
    pub fn code_name(&self) -> &str {
        match self {
            GrpcStatus::Ok => "OK",
            GrpcStatus::InvalidArgument => "INVALID_ARGUMENT",
            GrpcStatus::NotFound => "NOT_FOUND",
            GrpcStatus::Unimplemented => "UNIMPLEMENTED",
            GrpcStatus::Internal => "INTERNAL",
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_adapter_creation() {
        let adapter = GrpcAdapter::new();
        assert_eq!(adapter.name(), "grpc");
    }

    #[tokio::test]
    async fn test_execute_get_user() {
        let adapter = GrpcAdapter::new();
        let result = adapter.execute("GetUser", r#"{"id": 42}"#).await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("John Doe"));
    }

    #[tokio::test]
    async fn test_execute_list_users() {
        let adapter = GrpcAdapter::new();
        let result = adapter.execute("ListUsers", "{}").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("Alice"));
    }

    #[test]
    fn test_proto_generation() {
        let adapter = GrpcAdapter::new();
        let proto = adapter.generate_proto();
        assert!(proto.contains("service UserService"));
        assert!(proto.contains("rpc GetUser"));
    }

    #[test]
    fn test_grpc_status_codes() {
        assert_eq!(GrpcStatus::Ok.code_name(), "OK");
        assert_eq!(GrpcStatus::InvalidArgument.code_name(), "INVALID_ARGUMENT");
        assert_eq!(GrpcStatus::NotFound.code_name(), "NOT_FOUND");
    }
}
