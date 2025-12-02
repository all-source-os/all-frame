//! Router module for protocol-agnostic request handling
//!
//! This module provides the core abstractions for routing requests across
//! multiple protocols (REST, GraphQL, gRPC) using a unified handler interface.

use std::{collections::HashMap, future::Future};

pub mod adapter;
pub mod builder;
#[cfg(feature = "router")]
pub mod config;
pub mod contract;
pub mod docs;
pub mod graphql;
pub mod graphiql;
pub mod grpc;
pub mod grpc_explorer;
pub mod handler;
pub mod metadata;
pub mod method;
pub mod openapi;
pub mod rest;
pub mod scalar;
pub mod schema;

// Production adapters (optional features)
#[cfg(feature = "router-graphql")]
pub mod graphql_prod;
#[cfg(feature = "router-grpc")]
pub mod grpc_prod;

pub use adapter::ProtocolAdapter;
pub use builder::RouteBuilder;
#[cfg(feature = "router")]
pub use config::{GraphQLConfig, GrpcConfig, RestConfig, RouterConfig, ServerConfig};
pub use contract::{
    ContractTestConfig, ContractTestResult, ContractTestResults, ContractTestable, ContractTester,
};
pub use docs::DocsConfig;
pub use graphql::{GraphQLAdapter, GraphQLOperation, OperationType};
pub use graphiql::{graphiql_html, GraphiQLConfig, GraphiQLTheme};
// Re-export production adapters when features are enabled
#[cfg(feature = "router-graphql")]
pub use graphql_prod::GraphQLProductionAdapter;
pub use grpc::{GrpcAdapter, GrpcMethod, GrpcMethodType, GrpcRequest, GrpcStatus};
pub use grpc_explorer::{grpc_explorer_html, GrpcExplorerConfig, GrpcExplorerTheme};
#[cfg(feature = "router-grpc")]
pub use grpc_prod::{protobuf, status, streaming, GrpcProductionAdapter, GrpcService};
pub use handler::{Handler, HandlerFn};
pub use metadata::RouteMetadata;
pub use method::Method;
pub use openapi::{OpenApiGenerator, OpenApiServer};
pub use rest::{RestAdapter, RestRequest, RestResponse, RestRoute};
pub use scalar::{scalar_html, ScalarConfig, ScalarLayout, ScalarTheme};
pub use schema::ToJsonSchema;

/// Router manages handler registration and protocol adapters
///
/// The router allows you to register handlers once and expose them via
/// multiple protocols based on configuration.
pub struct Router {
    handlers: HashMap<String, Box<dyn Handler>>,
    adapters: HashMap<String, Box<dyn ProtocolAdapter>>,
    routes: Vec<RouteMetadata>,
    #[cfg(feature = "router")]
    #[allow(dead_code)]
    config: Option<RouterConfig>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            adapters: HashMap::new(),
            routes: Vec::new(),
            #[cfg(feature = "router")]
            config: None,
        }
    }

    /// Create a new router with configuration
    #[cfg(feature = "router")]
    pub fn with_config(config: RouterConfig) -> Self {
        let mut router = Self {
            handlers: HashMap::new(),
            adapters: HashMap::new(),
            routes: Vec::new(),
            config: Some(config.clone()),
        };

        // Auto-register adapters based on config
        if config.has_protocol("rest") {
            router.add_adapter(Box::new(RestAdapter::new()));
        }
        if config.has_protocol("graphql") {
            router.add_adapter(Box::new(GraphQLAdapter::new()));
        }
        if config.has_protocol("grpc") {
            router.add_adapter(Box::new(GrpcAdapter::new()));
        }

        router
    }

    /// Register a handler with a name
    pub fn register<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        self.handlers
            .insert(name.to_string(), Box::new(HandlerFn::new(handler)));
    }

    /// Get the number of registered handlers
    pub fn handlers_count(&self) -> usize {
        self.handlers.len()
    }

    /// Add a protocol adapter
    pub fn add_adapter(&mut self, adapter: Box<dyn ProtocolAdapter>) {
        self.adapters.insert(adapter.name().to_string(), adapter);
    }

    /// Check if an adapter is registered
    pub fn has_adapter(&self, name: &str) -> bool {
        self.adapters.contains_key(name)
    }

    /// Get an adapter by name
    pub fn get_adapter(&self, name: &str) -> Option<&Box<dyn ProtocolAdapter>> {
        self.adapters.get(name)
    }

    /// Route a request through the appropriate protocol adapter
    pub async fn route_request(
        &self,
        protocol: &str,
        request: &str,
    ) -> Result<String, String> {
        let adapter = self
            .get_adapter(protocol)
            .ok_or_else(|| format!("Adapter not found: {}", protocol))?;

        adapter.handle(request).await
    }

    /// Execute a handler by name
    pub async fn execute(&self, name: &str) -> Result<String, String> {
        match self.handlers.get(name) {
            Some(handler) => handler.call().await,
            None => Err(format!("Handler '{}' not found", name)),
        }
    }

    /// Check if handler can be called via REST
    pub fn can_handle_rest(&self, _name: &str) -> bool {
        self.has_adapter("rest")
    }

    /// Check if handler can be called via GraphQL
    pub fn can_handle_graphql(&self, _name: &str) -> bool {
        self.has_adapter("graphql")
    }

    /// Check if handler can be called via gRPC
    pub fn can_handle_grpc(&self, _name: &str) -> bool {
        self.has_adapter("grpc")
    }

    /// Get list of enabled protocols
    pub fn enabled_protocols(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }

    /// Add a route with metadata
    ///
    /// This stores route metadata that can be used to generate
    /// documentation (OpenAPI, GraphQL schemas, gRPC reflection).
    pub fn add_route(&mut self, metadata: RouteMetadata) {
        self.routes.push(metadata);
    }

    /// Get all registered routes
    ///
    /// Returns an immutable reference to all route metadata.
    /// This is used for documentation generation.
    pub fn routes(&self) -> &[RouteMetadata] {
        &self.routes
    }

    /// Register a GET route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a GET request. The handler name is automatically
    /// generated as "GET:{path}".
    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("GET:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::GET, "rest"));
    }

    /// Register a POST route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a POST request. The handler name is automatically
    /// generated as "POST:{path}".
    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("POST:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::POST, "rest"));
    }

    /// Register a PUT route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a PUT request. The handler name is automatically
    /// generated as "PUT:{path}".
    pub fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("PUT:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::PUT, "rest"));
    }

    /// Register a DELETE route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a DELETE request. The handler name is automatically
    /// generated as "DELETE:{path}".
    pub fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("DELETE:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::DELETE, "rest"));
    }

    /// Register a PATCH route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a PATCH request. The handler name is automatically
    /// generated as "PATCH:{path}".
    pub fn patch<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("PATCH:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::PATCH, "rest"));
    }

    /// Register a HEAD route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a HEAD request. The handler name is automatically
    /// generated as "HEAD:{path}".
    pub fn head<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("HEAD:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::HEAD, "rest"));
    }

    /// Register an OPTIONS route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for an OPTIONS request. The handler name is automatically
    /// generated as "OPTIONS:{path}".
    pub fn options<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("OPTIONS:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::OPTIONS, "rest"));
    }

    /// Call handler via REST
    pub async fn call_rest(&self, method: &str, path: &str) -> Result<String, String> {
        let adapter = self
            .adapters
            .get("rest")
            .ok_or_else(|| "REST adapter not enabled".to_string())?;

        let request = format!("{} {}", method, path);
        adapter.handle(&request).await
    }

    /// Call handler via GraphQL
    pub async fn call_graphql(&self, query: &str) -> Result<String, String> {
        let adapter = self
            .adapters
            .get("graphql")
            .ok_or_else(|| "GraphQL adapter not enabled".to_string())?;

        adapter.handle(query).await
    }

    /// Call handler via gRPC
    pub async fn call_grpc(&self, method: &str, request: &str) -> Result<String, String> {
        let adapter = self
            .adapters
            .get("grpc")
            .ok_or_else(|| "gRPC adapter not enabled".to_string())?;

        let grpc_request = format!("{}:{}", method, request);
        adapter.handle(&grpc_request).await
    }

    /// Generate Scalar documentation HTML with default configuration
    ///
    /// This is a convenience method that generates a complete HTML page
    /// with Scalar UI for interactive API documentation.
    ///
    /// # Arguments
    ///
    /// * `title` - API title
    /// * `version` - API version
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::Router;
    ///
    /// let mut router = Router::new();
    /// router.get("/users", || async { "Users".to_string() });
    ///
    /// let html = router.scalar("My API", "1.0.0");
    /// // Serve this HTML at /docs endpoint
    /// ```
    pub fn scalar(&self, title: &str, version: &str) -> String {
        let config = scalar::ScalarConfig::default();
        self.scalar_docs(config, title, version)
    }

    /// Generate Scalar documentation HTML with custom configuration
    ///
    /// This method allows full customization of the Scalar UI appearance
    /// and behavior.
    ///
    /// # Arguments
    ///
    /// * `config` - Scalar configuration
    /// * `title` - API title
    /// * `version` - API version
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::{Router, ScalarConfig, ScalarTheme};
    ///
    /// let mut router = Router::new();
    /// router.get("/users", || async { "Users".to_string() });
    ///
    /// let config = ScalarConfig::new()
    ///     .theme(ScalarTheme::Light)
    ///     .show_sidebar(false);
    ///
    /// let html = router.scalar_docs(config, "My API", "1.0.0");
    /// ```
    pub fn scalar_docs(&self, config: scalar::ScalarConfig, title: &str, version: &str) -> String {
        // Generate OpenAPI spec
        let spec = OpenApiGenerator::new(title, version).generate(self);
        let spec_json = serde_json::to_string(&spec).unwrap_or_else(|_| "{}".to_string());

        // Generate Scalar HTML
        scalar::scalar_html(&config, title, &spec_json)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let router = Router::new();
        assert_eq!(router.handlers_count(), 0);
    }

    #[tokio::test]
    async fn test_handler_registration() {
        let mut router = Router::new();
        router.register("test", || async { "Hello".to_string() });
        assert_eq!(router.handlers_count(), 1);
    }

    #[tokio::test]
    async fn test_handler_execution() {
        let mut router = Router::new();
        router.register("test", || async { "Hello".to_string() });
        let result = router.execute("test").await;
        assert_eq!(result, Ok("Hello".to_string()));
    }

    // New tests for route metadata tracking
    #[tokio::test]
    async fn test_router_starts_with_no_routes() {
        let router = Router::new();
        let routes = router.routes();
        assert_eq!(routes.len(), 0);
    }

    #[tokio::test]
    async fn test_add_route_metadata() {
        let mut router = Router::new();
        let metadata = RouteMetadata::new("/users", "GET", "rest");

        router.add_route(metadata.clone());

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[0].protocol, "rest");
    }

    #[tokio::test]
    async fn test_add_multiple_routes() {
        let mut router = Router::new();

        router.add_route(RouteMetadata::new("/users", "GET", "rest"));
        router.add_route(RouteMetadata::new("/users", "POST", "rest"));
        router.add_route(RouteMetadata::new("/posts", "GET", "rest"));

        let routes = router.routes();
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    async fn test_routes_with_different_protocols() {
        let mut router = Router::new();

        router.add_route(RouteMetadata::new("/users", "GET", "rest"));
        router.add_route(RouteMetadata::new("users", "query", "graphql"));
        router.add_route(RouteMetadata::new("UserService.GetUser", "unary", "grpc"));

        let routes = router.routes();
        assert_eq!(routes.len(), 3);

        assert_eq!(routes[0].protocol, "rest");
        assert_eq!(routes[1].protocol, "graphql");
        assert_eq!(routes[2].protocol, "grpc");
    }

    #[tokio::test]
    async fn test_routes_returns_immutable_reference() {
        let mut router = Router::new();
        router.add_route(RouteMetadata::new("/test", "GET", "rest"));

        let routes1 = router.routes();
        let routes2 = router.routes();

        // Both should have the same data
        assert_eq!(routes1.len(), routes2.len());
        assert_eq!(routes1[0].path, routes2[0].path);
    }

    // Tests for type-safe route registration
    #[tokio::test]
    async fn test_route_get_method() {
        let mut router = Router::new();
        router.get("/users", || async { "User list".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[0].protocol, "rest");
    }

    #[tokio::test]
    async fn test_route_post_method() {
        let mut router = Router::new();
        router.post("/users", || async { "User created".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "POST");
        assert_eq!(routes[0].protocol, "rest");
    }

    #[tokio::test]
    async fn test_route_put_method() {
        let mut router = Router::new();
        router.put("/users/1", || async { "User updated".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "PUT");
    }

    #[tokio::test]
    async fn test_route_delete_method() {
        let mut router = Router::new();
        router.delete("/users/1", || async { "User deleted".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "DELETE");
    }

    #[tokio::test]
    async fn test_route_patch_method() {
        let mut router = Router::new();
        router.patch("/users/1", || async { "User patched".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "PATCH");
    }

    #[tokio::test]
    async fn test_multiple_routes_different_methods() {
        let mut router = Router::new();
        router.get("/users", || async { "List".to_string() });
        router.post("/users", || async { "Create".to_string() });
        router.put("/users/1", || async { "Update".to_string() });
        router.delete("/users/1", || async { "Delete".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 4);

        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[1].method, "POST");
        assert_eq!(routes[2].method, "PUT");
        assert_eq!(routes[3].method, "DELETE");
    }

    #[tokio::test]
    async fn test_route_method_with_path_params() {
        let mut router = Router::new();
        router.get("/users/{id}", || async { "User details".to_string() });
        router.get("/users/{id}/posts/{post_id}", || async {
            "Post details".to_string()
        });

        let routes = router.routes();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/users/{id}");
        assert_eq!(routes[1].path, "/users/{id}/posts/{post_id}");
    }

    #[tokio::test]
    async fn test_route_registration_and_execution() {
        let mut router = Router::new();
        router.get("/test", || async { "GET response".to_string() });
        router.post("/test", || async { "POST response".to_string() });

        // Both route metadata and handler should be registered
        assert_eq!(router.routes().len(), 2);
        assert_eq!(router.handlers_count(), 2);

        // Handlers should be executable
        let result1 = router.execute("GET:/test").await;
        let result2 = router.execute("POST:/test").await;

        assert_eq!(result1, Ok("GET response".to_string()));
        assert_eq!(result2, Ok("POST response".to_string()));
    }

    // Tests for Scalar integration
    #[tokio::test]
    async fn test_scalar_generates_html() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });

        let html = router.scalar("Test API", "1.0.0");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test API - API Documentation</title>"));
        assert!(html.contains("@scalar/api-reference"));
    }

    #[tokio::test]
    async fn test_scalar_contains_openapi_spec() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });
        router.post("/users", || async { "User created".to_string() });

        let html = router.scalar("Test API", "1.0.0");

        // Should contain the OpenAPI spec
        assert!(html.contains("openapi"));
        assert!(html.contains("Test API"));
        assert!(html.contains("1.0.0"));
    }

    #[tokio::test]
    async fn test_scalar_docs_with_custom_config() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });

        let config = scalar::ScalarConfig::new()
            .theme(scalar::ScalarTheme::Light)
            .show_sidebar(false);

        let html = router.scalar_docs(config, "Custom API", "2.0.0");

        assert!(html.contains("Custom API"));
        assert!(html.contains(r#""theme":"light""#));
        assert!(html.contains(r#""showSidebar":false"#));
    }

    #[tokio::test]
    async fn test_scalar_docs_with_custom_css() {
        let mut router = Router::new();
        router.get("/test", || async { "Test".to_string() });

        let config = scalar::ScalarConfig::new().custom_css("body { font-family: 'Inter'; }");

        let html = router.scalar_docs(config, "API", "1.0");

        assert!(html.contains("<style>body { font-family: 'Inter'; }</style>"));
    }

    #[tokio::test]
    async fn test_scalar_with_multiple_routes() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });
        router.post("/users", || async { "Create".to_string() });
        router.get("/users/{id}", || async { "User details".to_string() });
        router.delete("/users/{id}", || async { "Delete".to_string() });

        let html = router.scalar("API", "1.0.0");

        // Should contain all routes in the OpenAPI spec
        assert!(html.contains("/users"));
    }

    // Tests for protocol adapter management
    #[tokio::test]
    async fn test_get_adapter_returns_adapter() {
        let mut router = Router::new();
        router.add_adapter(Box::new(RestAdapter::new()));

        let adapter = router.get_adapter("rest");
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().name(), "rest");
    }

    #[tokio::test]
    async fn test_get_adapter_returns_none_for_missing() {
        let router = Router::new();
        let adapter = router.get_adapter("rest");
        assert!(adapter.is_none());
    }

    #[tokio::test]
    async fn test_route_request_success() {
        let mut router = Router::new();
        router.register("test_handler", || async { "Success!".to_string() });

        // Register adapter with a route
        let mut rest_adapter = RestAdapter::new();
        rest_adapter.route("GET", "/test", "test_handler");
        router.add_adapter(Box::new(rest_adapter));

        let result = router.route_request("rest", "GET /test").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("HTTP 200") || response.contains("test_handler"));
    }

    #[tokio::test]
    async fn test_route_request_unknown_adapter() {
        let router = Router::new();
        let result = router.route_request("unknown", "test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Adapter not found"));
    }

    #[tokio::test]
    async fn test_enabled_protocols_empty() {
        let router = Router::new();
        let protocols = router.enabled_protocols();
        assert_eq!(protocols.len(), 0);
    }

    #[tokio::test]
    async fn test_enabled_protocols_multiple() {
        let mut router = Router::new();
        router.add_adapter(Box::new(RestAdapter::new()));
        router.add_adapter(Box::new(GraphQLAdapter::new()));
        router.add_adapter(Box::new(GrpcAdapter::new()));

        let protocols = router.enabled_protocols();
        assert_eq!(protocols.len(), 3);
        assert!(protocols.contains(&"rest".to_string()));
        assert!(protocols.contains(&"graphql".to_string()));
        assert!(protocols.contains(&"grpc".to_string()));
    }

    #[tokio::test]
    async fn test_can_handle_rest() {
        let mut router = Router::new();
        assert!(!router.can_handle_rest("test"));

        router.add_adapter(Box::new(RestAdapter::new()));
        assert!(router.can_handle_rest("test"));
    }

    #[tokio::test]
    async fn test_can_handle_graphql() {
        let mut router = Router::new();
        assert!(!router.can_handle_graphql("test"));

        router.add_adapter(Box::new(GraphQLAdapter::new()));
        assert!(router.can_handle_graphql("test"));
    }

    #[tokio::test]
    async fn test_can_handle_grpc() {
        let mut router = Router::new();
        assert!(!router.can_handle_grpc("test"));

        router.add_adapter(Box::new(GrpcAdapter::new()));
        assert!(router.can_handle_grpc("test"));
    }

    // ===== Integration Tests: Multi-Protocol Routing =====

    #[tokio::test]
    async fn test_integration_single_handler_rest() {
        // Test: Single handler exposed via REST
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        // Route REST request
        let response = router.route_request("rest", "GET /users/42").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_single_handler_graphql() {
        // Test: Single handler exposed via GraphQL
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        // Route GraphQL request
        let response = router.route_request("graphql", "query { user }").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_single_handler_grpc() {
        // Test: Single handler exposed via gRPC
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        // Route gRPC request
        let response = router
            .route_request("grpc", "UserService.GetUser:{\"id\":42}")
            .await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_single_handler_all_protocols() {
        // Test: Single handler exposed via ALL protocols
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        // Test REST
        let rest_response = router.route_request("rest", "GET /users/42").await;
        assert!(rest_response.is_ok());
        assert!(rest_response.unwrap().contains("get_user"));

        // Test GraphQL
        let graphql_response = router.route_request("graphql", "query { user }").await;
        assert!(graphql_response.is_ok());
        assert!(graphql_response.unwrap().contains("get_user"));

        // Test gRPC
        let grpc_response = router
            .route_request("grpc", "UserService.GetUser:{\"id\":42}")
            .await;
        assert!(grpc_response.is_ok());
        assert!(grpc_response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_multiple_handlers_all_protocols() {
        // Test: Multiple handlers, each exposed via all protocols
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });
        router.register("list_users", || async { "Users list".to_string() });
        router.register("create_user", || async { "Created user".to_string() });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        rest.route("GET", "/users", "list_users");
        rest.route("POST", "/users", "create_user");
        router.add_adapter(Box::new(rest));

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        graphql.query("users", "list_users");
        graphql.mutation("createUser", "create_user");
        router.add_adapter(Box::new(graphql));

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        grpc.unary("UserService", "ListUsers", "list_users");
        grpc.unary("UserService", "CreateUser", "create_user");
        router.add_adapter(Box::new(grpc));

        // Test each handler via each protocol
        assert!(router
            .route_request("rest", "GET /users/42")
            .await
            .unwrap()
            .contains("get_user"));
        assert!(router
            .route_request("graphql", "query { user }")
            .await
            .unwrap()
            .contains("get_user"));
        assert!(router
            .route_request("grpc", "UserService.GetUser:{}")
            .await
            .unwrap()
            .contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_error_handling_rest_404() {
        // Test: REST 404 error
        let mut router = Router::new();

        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        let response = router.route_request("rest", "GET /posts/42").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("HTTP 404"));
    }

    #[tokio::test]
    async fn test_integration_error_handling_graphql_not_found() {
        // Test: GraphQL operation not found
        let mut router = Router::new();

        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        let response = router.route_request("graphql", "query { post }").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("errors"));
    }

    #[tokio::test]
    async fn test_integration_error_handling_grpc_unimplemented() {
        // Test: gRPC method not implemented
        let mut router = Router::new();

        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        let response = router.route_request("grpc", "UserService.GetPost:{}").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("grpc-status: 12")); // UNIMPLEMENTED
    }

    #[tokio::test]
    async fn test_integration_unknown_protocol() {
        // Test: Unknown protocol error
        let router = Router::new();

        let response = router.route_request("unknown", "request").await;
        assert!(response.is_err());
        assert!(response.unwrap_err().contains("Adapter not found"));
    }

    #[tokio::test]
    async fn test_integration_protocol_specific_features_rest_methods() {
        // Test: REST-specific HTTP methods
        let mut router = Router::new();
        router.register("get_users", || async { "Users".to_string() });
        router.register("create_user", || async { "Created".to_string() });
        router.register("update_user", || async { "Updated".to_string() });
        router.register("delete_user", || async { "Deleted".to_string() });

        let mut rest = RestAdapter::new();
        rest.route("GET", "/users", "get_users");
        rest.route("POST", "/users", "create_user");
        rest.route("PUT", "/users/:id", "update_user");
        rest.route("DELETE", "/users/:id", "delete_user");
        router.add_adapter(Box::new(rest));

        // Test different HTTP methods
        assert!(router
            .route_request("rest", "GET /users")
            .await
            .unwrap()
            .contains("get_users"));
        assert!(router
            .route_request("rest", "POST /users")
            .await
            .unwrap()
            .contains("create_user"));
        assert!(router
            .route_request("rest", "PUT /users/42")
            .await
            .unwrap()
            .contains("update_user"));
        assert!(router
            .route_request("rest", "DELETE /users/42")
            .await
            .unwrap()
            .contains("delete_user"));
    }

    #[tokio::test]
    async fn test_integration_protocol_specific_features_graphql_types() {
        // Test: GraphQL-specific query vs mutation
        let mut router = Router::new();
        router.register("get_user", || async { "User".to_string() });
        router.register("create_user", || async { "Created".to_string() });

        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        graphql.mutation("createUser", "create_user");
        router.add_adapter(Box::new(graphql));

        // Test query
        assert!(router
            .route_request("graphql", "query { user }")
            .await
            .unwrap()
            .contains("get_user"));

        // Test mutation
        assert!(router
            .route_request("graphql", "mutation { createUser }")
            .await
            .unwrap()
            .contains("create_user"));
    }

    #[tokio::test]
    async fn test_integration_protocol_specific_features_grpc_streaming() {
        // Test: gRPC-specific streaming modes
        let mut router = Router::new();
        router.register("get_user", || async { "User".to_string() });
        router.register("list_users", || async { "Users".to_string() });

        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        grpc.server_streaming("UserService", "ListUsers", "list_users");
        router.add_adapter(Box::new(grpc));

        // Test unary
        let unary_response = router
            .route_request("grpc", "UserService.GetUser:{}")
            .await
            .unwrap();
        assert!(unary_response.contains("unary"));

        // Test server streaming
        let streaming_response = router
            .route_request("grpc", "UserService.ListUsers:{}")
            .await
            .unwrap();
        assert!(streaming_response.contains("server_streaming"));
    }
}
