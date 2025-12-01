//! REST/HTTP protocol adapter
//!
//! Provides REST/HTTP support for the protocol-agnostic router.

use std::{future::Future, pin::Pin};

use super::ProtocolAdapter;

/// REST adapter for HTTP requests
///
/// Handles REST/HTTP protocol-specific request/response transformation.
pub struct RestAdapter {
    // Future: Add route table, middleware, etc.
}

impl RestAdapter {
    /// Create a new REST adapter
    pub fn new() -> Self {
        Self {}
    }

    /// Build a simulated HTTP request for testing
    ///
    /// In a real implementation, this would parse actual HTTP requests.
    /// For MVP, we use a simple string-based representation.
    pub fn build_request(
        &self,
        method: &str,
        path: &str,
        _body: Option<&str>,
        _headers: Option<&str>,
    ) -> RestRequest {
        RestRequest {
            method: method.to_string(),
            path: path.to_string(),
        }
    }
}

impl Default for RestAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolAdapter for RestAdapter {
    fn name(&self) -> &str {
        "rest"
    }

    fn handle(
        &self,
        request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        // For MVP, we return a simple success response
        // Full implementation will parse the request and call appropriate handlers
        let response = format!("REST handled: {}", request);
        Box::pin(async move { Ok(response) })
    }
}

/// Simplified HTTP request representation
///
/// For MVP testing purposes. Full implementation will use proper HTTP types.
#[derive(Debug, Clone)]
pub struct RestRequest {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Request path
    pub path: String,
}

/// Simplified HTTP response representation
///
/// For MVP testing purposes. Full implementation will use proper HTTP types.
#[derive(Debug, Clone)]
pub struct RestResponse {
    status: u16,
    body: String,
}

impl RestResponse {
    /// Create a new response
    pub fn new(status: u16, body: String) -> Self {
        Self { status, body }
    }

    /// Get the HTTP status code
    pub fn status(&self) -> u16 {
        self.status
    }

    /// Get the response body
    pub fn body(&self) -> &str {
        &self.body
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rest_adapter_creation() {
        let adapter = RestAdapter::new();
        assert_eq!(adapter.name(), "rest");
    }

    #[test]
    fn test_build_request() {
        let adapter = RestAdapter::new();
        let request = adapter.build_request("GET", "/users/42", None, None);
        assert_eq!(request.method, "GET");
        assert_eq!(request.path, "/users/42");
    }

    #[tokio::test]
    async fn test_handle_request() {
        let adapter = RestAdapter::new();
        let result = adapter.handle("test request").await;
        assert!(result.is_ok());
    }
}
