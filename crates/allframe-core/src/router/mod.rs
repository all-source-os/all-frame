//! Router module for protocol-agnostic request handling
//!
//! This module provides the core abstractions for routing requests across
//! multiple protocols (REST, GraphQL, gRPC) using a unified handler interface.

use std::collections::HashMap;
use std::future::Future;

pub mod adapter;
#[cfg(feature = "router")]
pub mod config;
pub mod graphql;
pub mod grpc;
pub mod handler;
pub mod rest;

// Production adapters (optional features)
#[cfg(feature = "router-graphql")]
pub mod graphql_prod;
#[cfg(feature = "router-grpc")]
pub mod grpc_prod;

pub use adapter::ProtocolAdapter;
#[cfg(feature = "router")]
pub use config::{GrpcConfig, GraphQLConfig, RestConfig, RouterConfig, ServerConfig};
pub use graphql::GraphQLAdapter;
pub use grpc::{GrpcAdapter, GrpcRequest, GrpcStatus};
pub use handler::{Handler, HandlerFn};
pub use rest::{RestAdapter, RestRequest, RestResponse};

// Re-export production adapters when features are enabled
#[cfg(feature = "router-graphql")]
pub use graphql_prod::GraphQLProductionAdapter;
#[cfg(feature = "router-grpc")]
pub use grpc_prod::{GrpcProductionAdapter, GrpcService, protobuf, status, streaming};

/// Router manages handler registration and protocol adapters
///
/// The router allows you to register handlers once and expose them via
/// multiple protocols based on configuration.
pub struct Router {
    handlers: HashMap<String, Box<dyn Handler>>,
    adapters: HashMap<String, Box<dyn ProtocolAdapter>>,
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
}
