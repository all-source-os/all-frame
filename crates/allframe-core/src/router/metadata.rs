//! Route metadata for documentation generation
//!
//! This module provides the types and functionality for extracting and storing
//! metadata about registered routes, which can then be used to generate
//! OpenAPI specifications, GraphQL schemas, and gRPC reflection data.

use serde::{Deserialize, Serialize};

/// Metadata about a registered route
///
/// Contains all information needed to generate documentation
/// for a route across different protocols (REST, GraphQL, gRPC).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct RouteMetadata {
    /// The route path (e.g., "/users", "/users/{id}")
    pub path: String,

    /// HTTP method for REST routes (e.g., "GET", "POST")
    /// Empty for non-REST protocols
    pub method: String,

    /// Protocol this route belongs to (e.g., "rest", "graphql", "grpc")
    pub protocol: String,

    /// Optional description from doc comments
    pub description: Option<String>,

    /// Request schema as JSON Schema (if available)
    pub request_schema: Option<serde_json::Value>,

    /// Response schema as JSON Schema (if available)
    pub response_schema: Option<serde_json::Value>,
}

impl RouteMetadata {
    /// Create a new RouteMetadata
    pub fn new(path: impl Into<String>, method: impl Into<String>, protocol: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
            protocol: protocol.into(),
            description: None,
            request_schema: None,
            response_schema: None,
        }
    }

    /// Set the description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Set the request schema
    pub fn with_request_schema(mut self, schema: serde_json::Value) -> Self {
        self.request_schema = Some(schema);
        self
    }

    /// Set the response schema
    pub fn with_response_schema(mut self, schema: serde_json::Value) -> Self {
        self.response_schema = Some(schema);
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_route_metadata_creation() {
        let metadata = RouteMetadata::new("/users", "GET", "rest");

        assert_eq!(metadata.path, "/users");
        assert_eq!(metadata.method, "GET");
        assert_eq!(metadata.protocol, "rest");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.request_schema, None);
        assert_eq!(metadata.response_schema, None);
    }

    #[test]
    fn test_route_metadata_with_description() {
        let metadata = RouteMetadata::new("/users", "POST", "rest")
            .with_description("Create a new user");

        assert_eq!(metadata.description, Some("Create a new user".to_string()));
    }

    #[test]
    fn test_route_metadata_with_schemas() {
        let request_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let response_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"}
            }
        });

        let metadata = RouteMetadata::new("/users", "POST", "rest")
            .with_request_schema(request_schema.clone())
            .with_response_schema(response_schema.clone());

        assert_eq!(metadata.request_schema, Some(request_schema));
        assert_eq!(metadata.response_schema, Some(response_schema));
    }

    #[test]
    fn test_route_metadata_builder_pattern() {
        let metadata = RouteMetadata::new("/users/{id}", "GET", "rest")
            .with_description("Get user by ID")
            .with_response_schema(serde_json::json!({"type": "object"}));

        assert_eq!(metadata.path, "/users/{id}");
        assert_eq!(metadata.method, "GET");
        assert_eq!(metadata.protocol, "rest");
        assert!(metadata.description.is_some());
        assert!(metadata.response_schema.is_some());
        assert!(metadata.request_schema.is_none());
    }

    #[test]
    fn test_route_metadata_graphql_protocol() {
        let metadata = RouteMetadata::new("users", "query", "graphql")
            .with_description("Query users");

        assert_eq!(metadata.protocol, "graphql");
        assert_eq!(metadata.method, "query");
    }

    #[test]
    fn test_route_metadata_grpc_protocol() {
        let metadata = RouteMetadata::new("UserService.CreateUser", "unary", "grpc")
            .with_description("Create a user via gRPC");

        assert_eq!(metadata.protocol, "grpc");
        assert_eq!(metadata.method, "unary");
    }

    #[test]
    fn test_route_metadata_clone() {
        let metadata1 = RouteMetadata::new("/test", "GET", "rest")
            .with_description("Test route");

        let metadata2 = metadata1.clone();

        assert_eq!(metadata1, metadata2);
    }

    #[test]
    fn test_route_metadata_serialization() {
        let metadata = RouteMetadata::new("/users", "POST", "rest")
            .with_description("Create user");

        let json = serde_json::to_string(&metadata).unwrap();
        let deserialized: RouteMetadata = serde_json::from_str(&json).unwrap();

        assert_eq!(metadata, deserialized);
    }
}
