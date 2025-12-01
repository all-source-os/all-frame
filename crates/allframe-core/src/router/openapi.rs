//! OpenAPI 3.1 specification generation
//!
//! This module provides functionality to generate OpenAPI 3.1 specifications
//! from router metadata. This enables automatic API documentation for REST
//! endpoints.

use serde_json::{json, Value};

use crate::router::{RouteMetadata, Router};

/// OpenAPI server configuration
///
/// Represents a server that the API can be accessed from.
/// Used in the "Try It" functionality to make actual API calls.
#[derive(Debug, Clone)]
pub struct OpenApiServer {
    /// Server URL (e.g., "https://api.example.com")
    pub url: String,
    /// Optional description (e.g., "Production server")
    pub description: Option<String>,
}

impl OpenApiServer {
    /// Create a new server configuration
    pub fn new(url: impl Into<String>) -> Self {
        Self {
            url: url.into(),
            description: None,
        }
    }

    /// Set the server description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }
}

/// OpenAPI specification generator
///
/// Generates OpenAPI 3.1 compliant specifications from router metadata.
pub struct OpenApiGenerator {
    title: String,
    version: String,
    description: Option<String>,
    servers: Vec<OpenApiServer>,
}

impl OpenApiGenerator {
    /// Create a new OpenAPI generator
    pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            title: title.into(),
            version: version.into(),
            description: None,
            servers: vec![],
        }
    }

    /// Set the API description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Add a server URL
    ///
    /// Servers are used by the "Try It" functionality to make actual API calls.
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::openapi::OpenApiGenerator;
    ///
    /// let generator = OpenApiGenerator::new("API", "1.0.0")
    ///     .with_server("http://localhost:3000", Some("Local development"));
    /// ```
    pub fn with_server(
        mut self,
        url: impl Into<String>,
        description: Option<impl Into<String>>,
    ) -> Self {
        let mut server = OpenApiServer::new(url);
        if let Some(desc) = description {
            server = server.with_description(desc);
        }
        self.servers.push(server);
        self
    }

    /// Add multiple servers
    pub fn with_servers(mut self, servers: Vec<OpenApiServer>) -> Self {
        self.servers = servers;
        self
    }

    /// Generate OpenAPI specification from router
    pub fn generate(&self, router: &Router) -> Value {
        let mut spec = json!({
            "openapi": "3.1.0",
            "info": {
                "title": self.title,
                "version": self.version,
            },
            "paths": {}
        });

        // Add description if present
        if let Some(ref desc) = self.description {
            spec["info"]["description"] = Value::String(desc.clone());
        }

        // Add servers if present (required for "Try It" functionality)
        if !self.servers.is_empty() {
            let servers: Vec<Value> = self
                .servers
                .iter()
                .map(|s| {
                    let mut server = json!({ "url": s.url });
                    if let Some(ref desc) = s.description {
                        server["description"] = Value::String(desc.clone());
                    }
                    server
                })
                .collect();
            spec["servers"] = Value::Array(servers);
        }

        // Build paths from routes
        let paths = self.build_paths(router.routes());
        spec["paths"] = paths;

        spec
    }

    fn build_paths(&self, routes: &[RouteMetadata]) -> Value {
        let mut paths = serde_json::Map::new();

        for route in routes {
            // Only process REST routes
            if route.protocol != "rest" {
                continue;
            }

            let path_item = paths.entry(route.path.clone()).or_insert_with(|| json!({}));

            let method = route.method.to_lowercase();
            let operation = self.build_operation(route);

            if let Value::Object(ref mut map) = path_item {
                map.insert(method, operation);
            }
        }

        Value::Object(paths)
    }

    fn build_operation(&self, route: &RouteMetadata) -> Value {
        let mut operation = json!({
            "responses": {
                "200": {
                    "description": "Successful response"
                }
            }
        });

        // Add description if present
        if let Some(ref desc) = route.description {
            operation["description"] = Value::String(desc.clone());
        }

        // Add request body if schema present
        if let Some(ref schema) = route.request_schema {
            operation["requestBody"] = json!({
                "required": true,
                "content": {
                    "application/json": {
                        "schema": schema
                    }
                }
            });
        }

        // Add response schema if present
        if let Some(ref schema) = route.response_schema {
            operation["responses"]["200"]["content"] = json!({
                "application/json": {
                    "schema": schema
                }
            });
        }

        operation
    }
}

impl Router {
    /// Generate OpenAPI 3.1 specification
    ///
    /// This is a convenience method that creates an OpenAPI specification
    /// for all REST routes registered with this router.
    pub fn to_openapi(&self, title: &str, version: &str) -> Value {
        OpenApiGenerator::new(title, version).generate(self)
    }

    /// Generate OpenAPI 3.1 specification with description
    pub fn to_openapi_with_description(
        &self,
        title: &str,
        version: &str,
        description: &str,
    ) -> Value {
        OpenApiGenerator::new(title, version)
            .with_description(description)
            .generate(self)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::router::RouteMetadata;

    #[tokio::test]
    async fn test_openapi_generator_basic() {
        let generator = OpenApiGenerator::new("Test API", "1.0.0");
        let router = Router::new();

        let spec = generator.generate(&router);

        assert_eq!(spec["openapi"], "3.1.0");
        assert_eq!(spec["info"]["title"], "Test API");
        assert_eq!(spec["info"]["version"], "1.0.0");
        assert!(spec["paths"].is_object());
    }

    #[tokio::test]
    async fn test_openapi_with_description() {
        let generator = OpenApiGenerator::new("Test API", "1.0.0").with_description("A test API");
        let router = Router::new();

        let spec = generator.generate(&router);

        assert_eq!(spec["info"]["description"], "A test API");
    }

    #[tokio::test]
    async fn test_openapi_single_route() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });

        let spec = router.to_openapi("Test API", "1.0.0");

        assert!(spec["paths"]["/users"].is_object());
        assert!(spec["paths"]["/users"]["get"].is_object());
        assert!(spec["paths"]["/users"]["get"]["responses"]["200"].is_object());
    }

    #[tokio::test]
    async fn test_openapi_multiple_routes() {
        let mut router = Router::new();
        router.get("/users", || async { "List".to_string() });
        router.post("/users", || async { "Create".to_string() });
        router.get("/posts", || async { "Posts".to_string() });

        let spec = router.to_openapi("Test API", "1.0.0");

        assert!(spec["paths"]["/users"]["get"].is_object());
        assert!(spec["paths"]["/users"]["post"].is_object());
        assert!(spec["paths"]["/posts"]["get"].is_object());
    }

    #[tokio::test]
    async fn test_openapi_route_with_description() {
        let mut router = Router::new();
        let metadata =
            RouteMetadata::new("/users", "GET", "rest").with_description("Get all users");
        router.add_route(metadata);

        let spec = router.to_openapi("Test API", "1.0.0");

        assert_eq!(
            spec["paths"]["/users"]["get"]["description"],
            "Get all users"
        );
    }

    #[tokio::test]
    async fn test_openapi_route_with_request_schema() {
        let mut router = Router::new();
        let request_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            }
        });

        let metadata = RouteMetadata::new("/users", "POST", "rest")
            .with_request_schema(request_schema.clone());
        router.add_route(metadata);

        let spec = router.to_openapi("Test API", "1.0.0");

        assert_eq!(
            spec["paths"]["/users"]["post"]["requestBody"]["content"]["application/json"]["schema"],
            request_schema
        );
    }

    #[tokio::test]
    async fn test_openapi_route_with_response_schema() {
        let mut router = Router::new();
        let response_schema = serde_json::json!({
            "type": "object",
            "properties": {
                "id": {"type": "string"},
                "name": {"type": "string"}
            }
        });

        let metadata = RouteMetadata::new("/users", "GET", "rest")
            .with_response_schema(response_schema.clone());
        router.add_route(metadata);

        let spec = router.to_openapi("Test API", "1.0.0");

        assert_eq!(
            spec["paths"]["/users"]["get"]["responses"]["200"]["content"]["application/json"]
                ["schema"],
            response_schema
        );
    }

    #[tokio::test]
    async fn test_openapi_filters_non_rest_routes() {
        let mut router = Router::new();
        router.add_route(RouteMetadata::new("/users", "GET", "rest"));
        router.add_route(RouteMetadata::new("users", "query", "graphql"));
        router.add_route(RouteMetadata::new("UserService", "unary", "grpc"));

        let spec = router.to_openapi("Test API", "1.0.0");

        // Only REST route should be in spec
        assert!(spec["paths"]["/users"].is_object());
        assert!(spec["paths"]["users"].is_null());
        assert!(spec["paths"]["UserService"].is_null());
    }

    #[tokio::test]
    async fn test_router_to_openapi_convenience_method() {
        let mut router = Router::new();
        router.get("/test", || async { "Test".to_string() });

        let spec = router.to_openapi("My API", "2.0.0");

        assert_eq!(spec["info"]["title"], "My API");
        assert_eq!(spec["info"]["version"], "2.0.0");
        assert!(spec["paths"]["/test"]["get"].is_object());
    }

    #[tokio::test]
    async fn test_router_to_openapi_with_description() {
        let mut router = Router::new();
        router.get("/test", || async { "Test".to_string() });

        let spec = router.to_openapi_with_description("My API", "2.0.0", "A great API");

        assert_eq!(spec["info"]["description"], "A great API");
    }
}
