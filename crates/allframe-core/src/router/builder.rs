//! Route builder for fluent route configuration
//!
//! This module provides a builder pattern for configuring routes with
//! metadata, tags, descriptions, and other OpenAPI properties.

use crate::router::{Method, RouteMetadata};
use serde_json::Value;

/// Builder for configuring a route with metadata
///
/// Provides a fluent interface for setting route properties
/// like description, tags, request/response schemas, etc.
pub struct RouteBuilder {
    path: String,
    method: Method,
    metadata: RouteMetadata,
}

impl RouteBuilder {
    /// Create a new route builder
    pub fn new(path: impl Into<String>, method: Method) -> Self {
        let path = path.into();
        let metadata = RouteMetadata::new(&path, method, "rest");

        Self {
            path,
            method,
            metadata,
        }
    }

    /// Set the route description
    pub fn description(mut self, description: impl Into<String>) -> Self {
        self.metadata = self.metadata.with_description(description);
        self
    }

    /// Set the request schema
    pub fn request_schema(mut self, schema: Value) -> Self {
        self.metadata = self.metadata.with_request_schema(schema);
        self
    }

    /// Set the response schema
    pub fn response_schema(mut self, schema: Value) -> Self {
        self.metadata = self.metadata.with_response_schema(schema);
        self
    }

    /// Build the route metadata
    pub fn build(self) -> RouteMetadata {
        self.metadata
    }

    /// Get the path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Get the method
    pub fn method(&self) -> Method {
        self.method
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;

    #[test]
    fn test_route_builder_basic() {
        let builder = RouteBuilder::new("/users", Method::GET);
        let metadata = builder.build();

        assert_eq!(metadata.path, "/users");
        assert_eq!(metadata.method, "GET");
        assert_eq!(metadata.protocol, "rest");
    }

    #[test]
    fn test_route_builder_with_description() {
        let builder = RouteBuilder::new("/users", Method::GET)
            .description("Get all users");
        let metadata = builder.build();

        assert_eq!(metadata.description, Some("Get all users".to_string()));
    }

    #[test]
    fn test_route_builder_with_request_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let builder = RouteBuilder::new("/users", Method::POST)
            .request_schema(schema.clone());
        let metadata = builder.build();

        assert_eq!(metadata.request_schema, Some(schema));
    }

    #[test]
    fn test_route_builder_with_response_schema() {
        let schema = json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"}
            }
        });

        let builder = RouteBuilder::new("/users", Method::GET)
            .response_schema(schema.clone());
        let metadata = builder.build();

        assert_eq!(metadata.response_schema, Some(schema));
    }

    #[test]
    fn test_route_builder_fluent_chain() {
        let request = json!({"type": "object"});
        let response = json!({"type": "array"});

        let metadata = RouteBuilder::new("/users", Method::POST)
            .description("Create a new user")
            .request_schema(request.clone())
            .response_schema(response.clone())
            .build();

        assert_eq!(metadata.path, "/users");
        assert_eq!(metadata.method, "POST");
        assert_eq!(metadata.description, Some("Create a new user".to_string()));
        assert_eq!(metadata.request_schema, Some(request));
        assert_eq!(metadata.response_schema, Some(response));
    }

    #[test]
    fn test_route_builder_all_http_methods() {
        let methods = vec![
            Method::GET,
            Method::POST,
            Method::PUT,
            Method::DELETE,
            Method::PATCH,
            Method::HEAD,
            Method::OPTIONS,
        ];

        for method in methods {
            let builder = RouteBuilder::new("/test", method);
            assert_eq!(builder.method(), method);
        }
    }

    #[test]
    fn test_route_builder_path_accessor() {
        let builder = RouteBuilder::new("/users/{id}", Method::GET);
        assert_eq!(builder.path(), "/users/{id}");
    }

    #[test]
    fn test_route_builder_method_accessor() {
        let builder = RouteBuilder::new("/users", Method::POST);
        assert_eq!(builder.method(), Method::POST);
    }

    #[test]
    fn test_route_builder_multiple_routes() {
        let route1 = RouteBuilder::new("/users", Method::GET)
            .description("List users")
            .build();

        let route2 = RouteBuilder::new("/users", Method::POST)
            .description("Create user")
            .build();

        assert_eq!(route1.path, "/users");
        assert_eq!(route1.method, "GET");
        assert_eq!(route2.path, "/users");
        assert_eq!(route2.method, "POST");
    }

    #[test]
    fn test_route_builder_minimal_configuration() {
        let metadata = RouteBuilder::new("/health", Method::GET).build();

        assert_eq!(metadata.path, "/health");
        assert_eq!(metadata.method, "GET");
        assert_eq!(metadata.description, None);
        assert_eq!(metadata.request_schema, None);
        assert_eq!(metadata.response_schema, None);
    }
}
