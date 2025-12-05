//! gRPC protocol adapter
//!
//! Provides gRPC support for the protocol-agnostic router.

use std::{future::Future, pin::Pin};

use super::ProtocolAdapter;

/// gRPC method type (streaming mode)
#[derive(Debug, Clone, PartialEq)]
pub enum GrpcMethodType {
    /// Unary RPC: single request, single response
    Unary,
    /// Client streaming: stream of requests, single response
    ClientStreaming,
    /// Server streaming: single request, stream of responses
    ServerStreaming,
    /// Bidirectional streaming: stream of requests and responses
    BidirectionalStreaming,
}

/// gRPC service method definition
#[derive(Debug, Clone)]
pub struct GrpcMethod {
    /// Service name (e.g., "UserService")
    pub service: String,
    /// Method name (e.g., "GetUser")
    pub method: String,
    /// Method type (streaming mode)
    pub method_type: GrpcMethodType,
    /// Handler name to call
    pub handler: String,
}

impl GrpcMethod {
    /// Create a new gRPC method
    pub fn new(
        service: impl Into<String>,
        method: impl Into<String>,
        method_type: GrpcMethodType,
        handler: impl Into<String>,
    ) -> Self {
        Self {
            service: service.into(),
            method: method.into(),
            method_type,
            handler: handler.into(),
        }
    }

    /// Get the fully qualified method name
    ///
    /// Format: "ServiceName.MethodName" (gRPC convention)
    pub fn full_name(&self) -> String {
        format!("{}.{}", self.service, self.method)
    }
}

/// gRPC adapter for gRPC services and RPCs
///
/// Handles gRPC protocol-specific request/response transformation.
pub struct GrpcAdapter {
    methods: Vec<GrpcMethod>,
}

impl GrpcAdapter {
    /// Create a new gRPC adapter
    pub fn new() -> Self {
        Self {
            methods: Vec::new(),
        }
    }

    /// Register a unary RPC method
    pub fn unary(&mut self, service: &str, method: &str, handler: &str) -> &mut Self {
        self.methods.push(GrpcMethod::new(
            service,
            method,
            GrpcMethodType::Unary,
            handler,
        ));
        self
    }

    /// Register a client streaming RPC method
    pub fn client_streaming(&mut self, service: &str, method: &str, handler: &str) -> &mut Self {
        self.methods.push(GrpcMethod::new(
            service,
            method,
            GrpcMethodType::ClientStreaming,
            handler,
        ));
        self
    }

    /// Register a server streaming RPC method
    pub fn server_streaming(&mut self, service: &str, method: &str, handler: &str) -> &mut Self {
        self.methods.push(GrpcMethod::new(
            service,
            method,
            GrpcMethodType::ServerStreaming,
            handler,
        ));
        self
    }

    /// Register a bidirectional streaming RPC method
    pub fn bidirectional_streaming(
        &mut self,
        service: &str,
        method: &str,
        handler: &str,
    ) -> &mut Self {
        self.methods.push(GrpcMethod::new(
            service,
            method,
            GrpcMethodType::BidirectionalStreaming,
            handler,
        ));
        self
    }

    /// Find a matching method by fully qualified name
    ///
    /// Format: "ServiceName.MethodName"
    pub fn match_method(&self, full_name: &str) -> Option<&GrpcMethod> {
        self.methods.iter().find(|m| m.full_name() == full_name)
    }

    /// Parse a gRPC request string
    ///
    /// Format: "ServiceName.MethodName:payload"
    /// Example: "UserService.GetUser:{\"id\":42}"
    pub fn parse_request(&self, request: &str) -> Result<(String, String), String> {
        if request.is_empty() {
            return Err("Empty gRPC request".to_string());
        }

        if let Some((method, payload)) = request.split_once(':') {
            Ok((method.to_string(), payload.to_string()))
        } else {
            Err("Invalid gRPC request format. Expected: ServiceName.MethodName:payload".to_string())
        }
    }

    /// Generate .proto file for registered methods
    ///
    /// Generates Protocol Buffer definition from registered methods.
    pub fn generate_proto(&self) -> String {
        if self.methods.is_empty() {
            return String::new();
        }

        let mut proto = String::from("syntax = \"proto3\";\n\n");
        proto.push_str("package allframe;\n\n");

        // Group methods by service
        let mut services: std::collections::HashMap<String, Vec<&GrpcMethod>> =
            std::collections::HashMap::new();
        for method in &self.methods {
            services
                .entry(method.service.clone())
                .or_default()
                .push(method);
        }

        // Generate service definitions
        for (service_name, methods) in services {
            proto.push_str(&format!("service {} {{\n", service_name));
            for method in methods {
                let method_proto = match method.method_type {
                    GrpcMethodType::Unary => {
                        format!(
                            "  rpc {}({}Request) returns ({}Response);\n",
                            method.method, method.method, method.method
                        )
                    }
                    GrpcMethodType::ClientStreaming => {
                        format!(
                            "  rpc {}(stream {}Request) returns ({}Response);\n",
                            method.method, method.method, method.method
                        )
                    }
                    GrpcMethodType::ServerStreaming => {
                        format!(
                            "  rpc {}({}Request) returns (stream {}Response);\n",
                            method.method, method.method, method.method
                        )
                    }
                    GrpcMethodType::BidirectionalStreaming => {
                        format!(
                            "  rpc {}(stream {}Request) returns (stream {}Response);\n",
                            method.method, method.method, method.method
                        )
                    }
                };
                proto.push_str(&method_proto);
            }
            proto.push_str("}\n");
        }

        proto.trim().to_string()
    }

    /// Build a gRPC request
    pub fn build_request(&self, method: &str, payload: &str) -> GrpcRequest {
        GrpcRequest {
            method: method.to_string(),
            payload: payload.to_string(),
        }
    }

    /// Format a gRPC response
    pub fn format_response(&self, status: GrpcStatus, message: &str) -> String {
        format!("grpc-status: {} {}", status as u32, message)
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
        // Parse request before async block
        let parse_result = self.parse_request(request);
        let methods = self.methods.clone();

        Box::pin(async move {
            // Handle parse error
            let (method_name, _payload) = match parse_result {
                Ok(parsed) => parsed,
                Err(e) => {
                    let response =
                        format!("grpc-status: {} {}", GrpcStatus::InvalidArgument as u32, e);
                    return Ok(response);
                }
            };

            // Find matching method
            let matched_method = methods.iter().find(|m| m.full_name() == method_name);

            match matched_method {
                Some(method) => {
                    // In full implementation, would call handler here
                    // For now, return success with handler info
                    let response = format!(
                        "grpc-status: {} {{\"handler\":\"{}\",\"method\":\"{}\",\"type\":\"{}\"}}",
                        GrpcStatus::Ok as u32,
                        method.handler,
                        method.full_name(),
                        match method.method_type {
                            GrpcMethodType::Unary => "unary",
                            GrpcMethodType::ClientStreaming => "client_streaming",
                            GrpcMethodType::ServerStreaming => "server_streaming",
                            GrpcMethodType::BidirectionalStreaming => "bidirectional_streaming",
                        }
                    );
                    Ok(response)
                }
                None => {
                    // Method not found
                    let response = format!(
                        "grpc-status: {} Method not found: {}",
                        GrpcStatus::Unimplemented as u32,
                        method_name
                    );
                    Ok(response)
                }
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
/// Based on <https://grpc.io/docs/guides/status-codes/>
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

    #[test]
    fn test_method_registration_unary() {
        let mut adapter = GrpcAdapter::new();
        adapter.unary("UserService", "GetUser", "get_user_handler");

        assert_eq!(adapter.methods.len(), 1);
        assert_eq!(adapter.methods[0].service, "UserService");
        assert_eq!(adapter.methods[0].method, "GetUser");
        assert_eq!(adapter.methods[0].method_type, GrpcMethodType::Unary);
        assert_eq!(adapter.methods[0].handler, "get_user_handler");
    }

    #[test]
    fn test_method_registration_client_streaming() {
        let mut adapter = GrpcAdapter::new();
        adapter.client_streaming("UserService", "CreateUsers", "create_users_handler");

        assert_eq!(adapter.methods.len(), 1);
        assert_eq!(
            adapter.methods[0].method_type,
            GrpcMethodType::ClientStreaming
        );
    }

    #[test]
    fn test_method_registration_server_streaming() {
        let mut adapter = GrpcAdapter::new();
        adapter.server_streaming("UserService", "ListUsers", "list_users_handler");

        assert_eq!(adapter.methods.len(), 1);
        assert_eq!(
            adapter.methods[0].method_type,
            GrpcMethodType::ServerStreaming
        );
    }

    #[test]
    fn test_method_registration_bidirectional() {
        let mut adapter = GrpcAdapter::new();
        adapter.bidirectional_streaming("ChatService", "Chat", "chat_handler");

        assert_eq!(adapter.methods.len(), 1);
        assert_eq!(
            adapter.methods[0].method_type,
            GrpcMethodType::BidirectionalStreaming
        );
    }

    #[test]
    fn test_method_registration_multiple() {
        let mut adapter = GrpcAdapter::new();
        adapter.unary("UserService", "GetUser", "get_user");
        adapter.unary("UserService", "DeleteUser", "delete_user");
        adapter.server_streaming("UserService", "ListUsers", "list_users");

        assert_eq!(adapter.methods.len(), 3);
    }

    #[test]
    fn test_grpc_method_full_name() {
        let method = GrpcMethod::new("UserService", "GetUser", GrpcMethodType::Unary, "handler");
        assert_eq!(method.full_name(), "UserService.GetUser");
    }

    #[test]
    fn test_match_method_found() {
        let mut adapter = GrpcAdapter::new();
        adapter.unary("UserService", "GetUser", "get_user_handler");

        let matched = adapter.match_method("UserService.GetUser");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().handler, "get_user_handler");
    }

    #[test]
    fn test_match_method_not_found() {
        let adapter = GrpcAdapter::new();
        let matched = adapter.match_method("UserService.GetUser");
        assert!(matched.is_none());
    }

    #[test]
    fn test_parse_request_valid() {
        let adapter = GrpcAdapter::new();
        let result = adapter.parse_request("UserService.GetUser:{\"id\":42}");

        assert!(result.is_ok());
        let (method, payload) = result.unwrap();
        assert_eq!(method, "UserService.GetUser");
        assert_eq!(payload, r#"{"id":42}"#);
    }

    #[test]
    fn test_parse_request_empty() {
        let adapter = GrpcAdapter::new();
        let result = adapter.parse_request("");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty"));
    }

    #[test]
    fn test_parse_request_invalid() {
        let adapter = GrpcAdapter::new();
        let result = adapter.parse_request("InvalidRequest");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid gRPC request format"));
    }

    #[test]
    fn test_proto_generation_empty() {
        let adapter = GrpcAdapter::new();
        let proto = adapter.generate_proto();
        assert_eq!(proto, "");
    }

    #[test]
    fn test_proto_generation_unary() {
        let mut adapter = GrpcAdapter::new();
        adapter.unary("UserService", "GetUser", "get_user");
        adapter.unary("UserService", "DeleteUser", "delete_user");

        let proto = adapter.generate_proto();
        assert!(proto.contains("syntax = \"proto3\";"));
        assert!(proto.contains("service UserService {"));
        assert!(proto.contains("rpc GetUser(GetUserRequest) returns (GetUserResponse);"));
        assert!(proto.contains("rpc DeleteUser(DeleteUserRequest) returns (DeleteUserResponse);"));
    }

    #[test]
    fn test_proto_generation_streaming() {
        let mut adapter = GrpcAdapter::new();
        adapter.client_streaming("UserService", "CreateUsers", "create_users");
        adapter.server_streaming("UserService", "ListUsers", "list_users");
        adapter.bidirectional_streaming("ChatService", "Chat", "chat");

        let proto = adapter.generate_proto();
        assert!(proto
            .contains("rpc CreateUsers(stream CreateUsersRequest) returns (CreateUsersResponse);"));
        assert!(
            proto.contains("rpc ListUsers(ListUsersRequest) returns (stream ListUsersResponse);")
        );
        assert!(proto.contains("rpc Chat(stream ChatRequest) returns (stream ChatResponse);"));
    }

    #[test]
    fn test_proto_generation_multiple_services() {
        let mut adapter = GrpcAdapter::new();
        adapter.unary("UserService", "GetUser", "get_user");
        adapter.unary("PostService", "GetPost", "get_post");

        let proto = adapter.generate_proto();
        assert!(proto.contains("service UserService {"));
        assert!(proto.contains("service PostService {"));
    }

    #[test]
    fn test_build_request() {
        let adapter = GrpcAdapter::new();
        let request = adapter.build_request("UserService.GetUser", r#"{"id":42}"#);

        assert_eq!(request.method, "UserService.GetUser");
        assert_eq!(request.payload, r#"{"id":42}"#);
    }

    #[test]
    fn test_format_response() {
        let adapter = GrpcAdapter::new();
        let response = adapter.format_response(GrpcStatus::Ok, "success");

        assert!(response.contains("grpc-status: 0"));
        assert!(response.contains("success"));
    }

    #[tokio::test]
    async fn test_handle_unary_success() {
        let mut adapter = GrpcAdapter::new();
        adapter.unary("UserService", "GetUser", "get_user_handler");

        let result = adapter.handle("UserService.GetUser:{\"id\":42}").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("grpc-status: 0"));
        assert!(response.contains("get_user_handler"));
        assert!(response.contains("unary"));
    }

    #[tokio::test]
    async fn test_handle_streaming_success() {
        let mut adapter = GrpcAdapter::new();
        adapter.server_streaming("UserService", "ListUsers", "list_users_handler");

        let result = adapter.handle("UserService.ListUsers:{}").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("grpc-status: 0"));
        assert!(response.contains("list_users_handler"));
        assert!(response.contains("server_streaming"));
    }

    #[tokio::test]
    async fn test_handle_method_not_found() {
        let adapter = GrpcAdapter::new();
        let result = adapter.handle("UserService.GetUser:{}").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("grpc-status: 12")); // UNIMPLEMENTED
        assert!(response.contains("Method not found"));
    }

    #[tokio::test]
    async fn test_handle_invalid_request() {
        let adapter = GrpcAdapter::new();
        let result = adapter.handle("InvalidRequest").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("grpc-status: 3")); // INVALID_ARGUMENT
    }

    #[test]
    fn test_grpc_status_codes() {
        assert_eq!(GrpcStatus::Ok.code_name(), "OK");
        assert_eq!(GrpcStatus::InvalidArgument.code_name(), "INVALID_ARGUMENT");
        assert_eq!(GrpcStatus::NotFound.code_name(), "NOT_FOUND");
        assert_eq!(GrpcStatus::Unimplemented.code_name(), "UNIMPLEMENTED");
        assert_eq!(GrpcStatus::Internal.code_name(), "INTERNAL");
    }

    #[test]
    fn test_grpc_method_new() {
        let method = GrpcMethod::new("UserService", "GetUser", GrpcMethodType::Unary, "handler");
        assert_eq!(method.service, "UserService");
        assert_eq!(method.method, "GetUser");
        assert_eq!(method.method_type, GrpcMethodType::Unary);
        assert_eq!(method.handler, "handler");
    }
}
