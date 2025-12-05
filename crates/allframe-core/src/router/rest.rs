//! REST/HTTP protocol adapter
//!
//! Provides REST/HTTP support for the protocol-agnostic router.

use std::{future::Future, pin::Pin};

use super::ProtocolAdapter;

/// REST route definition
#[derive(Debug, Clone)]
pub struct RestRoute {
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// URL path pattern (e.g., "/users/:id")
    pub path: String,
    /// Handler name to call
    pub handler: String,
}

impl RestRoute {
    /// Create a new REST route
    pub fn new(
        method: impl Into<String>,
        path: impl Into<String>,
        handler: impl Into<String>,
    ) -> Self {
        Self {
            method: method.into(),
            path: path.into(),
            handler: handler.into(),
        }
    }

    /// Check if this route matches a given method and path
    pub fn matches(&self, method: &str, path: &str) -> bool {
        if self.method != method {
            return false;
        }

        // Simple exact match for now (no path params yet)
        self.path == path
    }

    /// Check if path matches with parameter support
    pub fn matches_path(&self, path: &str) -> bool {
        // Split both paths into segments
        let route_segments: Vec<&str> = self.path.split('/').filter(|s| !s.is_empty()).collect();
        let path_segments: Vec<&str> = path.split('/').filter(|s| !s.is_empty()).collect();

        // Must have same number of segments
        if route_segments.len() != path_segments.len() {
            return false;
        }

        // Check each segment
        for (route_seg, path_seg) in route_segments.iter().zip(path_segments.iter()) {
            // If route segment is a parameter (starts with :), it matches anything
            if route_seg.starts_with(':') {
                continue;
            }
            // Otherwise must match exactly
            if route_seg != path_seg {
                return false;
            }
        }

        true
    }
}

/// REST adapter for HTTP requests
///
/// Handles REST/HTTP protocol-specific request/response transformation.
pub struct RestAdapter {
    routes: Vec<RestRoute>,
}

impl RestAdapter {
    /// Create a new REST adapter
    pub fn new() -> Self {
        Self { routes: Vec::new() }
    }

    /// Register a REST route
    pub fn route(&mut self, method: &str, path: &str, handler: &str) -> &mut Self {
        self.routes.push(RestRoute::new(method, path, handler));
        self
    }

    /// Find a matching route for the given method and path
    pub fn match_route(&self, method: &str, path: &str) -> Option<&RestRoute> {
        self.routes
            .iter()
            .find(|r| r.method == method && r.matches_path(path))
    }

    /// Parse an HTTP request string
    ///
    /// Format: "METHOD /path [body]"
    /// Example: "GET /users", "POST /users {\"name\":\"John\"}"
    pub fn parse_request(&self, request: &str) -> Result<(String, String, Option<String>), String> {
        let parts: Vec<&str> = request.splitn(3, ' ').collect();

        if parts.len() < 2 {
            return Err("Invalid request format. Expected: METHOD /path [body]".to_string());
        }

        let method = parts[0].to_string();
        let path = parts[1].to_string();
        let body = parts.get(2).map(|s| s.to_string());

        Ok((method, path, body))
    }

    /// Format an HTTP response
    pub fn format_response(&self, status: u16, body: &str) -> String {
        format!("HTTP {} {}", status, body)
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
        // Parse the HTTP request before async block
        let parse_result = self.parse_request(request);

        // Clone routes for async block
        let routes = self.routes.clone();

        Box::pin(async move {
            // Handle parse error
            let (method, path, _body) = match parse_result {
                Ok(parsed) => parsed,
                Err(e) => {
                    let response = format!("HTTP 400 {}", e);
                    return Ok(response);
                }
            };

            // Find matching route
            let matched_route = routes
                .iter()
                .find(|r| r.method == method && r.matches_path(&path));

            match matched_route {
                Some(route) => {
                    // In full implementation, would call handler here
                    // For now, return success with handler name
                    let response_body = format!(
                        "{{\"handler\":\"{}\",\"method\":\"{}\",\"path\":\"{}\"}}",
                        route.handler, method, path
                    );
                    let response = format!("HTTP 200 {}", response_body);
                    Ok(response)
                }
                None => {
                    // 404 Not Found
                    let error = format!("{{\"error\":\"Not Found\",\"path\":\"{}\"}}", path);
                    let response = format!("HTTP 404 {}", error);
                    Ok(response)
                }
            }
        })
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

    #[test]
    fn test_route_registration() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users", "list_users");
        adapter.route("POST", "/users", "create_user");

        assert_eq!(adapter.routes.len(), 2);
        assert_eq!(adapter.routes[0].method, "GET");
        assert_eq!(adapter.routes[0].path, "/users");
        assert_eq!(adapter.routes[0].handler, "list_users");
    }

    #[test]
    fn test_route_matching_exact() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users", "list_users");

        let matched = adapter.match_route("GET", "/users");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().handler, "list_users");
    }

    #[test]
    fn test_route_matching_not_found() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users", "list_users");

        let matched = adapter.match_route("GET", "/posts");
        assert!(matched.is_none());
    }

    #[test]
    fn test_route_matching_wrong_method() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users", "list_users");

        let matched = adapter.match_route("POST", "/users");
        assert!(matched.is_none());
    }

    #[test]
    fn test_route_matching_with_params() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users/:id", "get_user");

        let matched = adapter.match_route("GET", "/users/42");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().handler, "get_user");

        let matched2 = adapter.match_route("GET", "/users/123");
        assert!(matched2.is_some());
    }

    #[test]
    fn test_route_matching_params_wrong_length() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users/:id", "get_user");

        // Too few segments
        let matched = adapter.match_route("GET", "/users");
        assert!(matched.is_none());

        // Too many segments
        let matched2 = adapter.match_route("GET", "/users/42/posts");
        assert!(matched2.is_none());
    }

    #[test]
    fn test_parse_request_get() {
        let adapter = RestAdapter::new();
        let result = adapter.parse_request("GET /users");

        assert!(result.is_ok());
        let (method, path, body) = result.unwrap();
        assert_eq!(method, "GET");
        assert_eq!(path, "/users");
        assert!(body.is_none());
    }

    #[test]
    fn test_parse_request_post_with_body() {
        let adapter = RestAdapter::new();
        let result = adapter.parse_request("POST /users {\"name\":\"John\"}");

        assert!(result.is_ok());
        let (method, path, body) = result.unwrap();
        assert_eq!(method, "POST");
        assert_eq!(path, "/users");
        assert_eq!(body.unwrap(), "{\"name\":\"John\"}");
    }

    #[test]
    fn test_parse_request_invalid() {
        let adapter = RestAdapter::new();
        let result = adapter.parse_request("INVALID");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid request format"));
    }

    #[test]
    fn test_format_response_200() {
        let adapter = RestAdapter::new();
        let response = adapter.format_response(200, "{\"success\":true}");

        assert!(response.contains("HTTP 200"));
        assert!(response.contains("{\"success\":true}"));
    }

    #[test]
    fn test_format_response_404() {
        let adapter = RestAdapter::new();
        let response = adapter.format_response(404, "{\"error\":\"Not Found\"}");

        assert!(response.contains("HTTP 404"));
        assert!(response.contains("Not Found"));
    }

    #[tokio::test]
    async fn test_handle_request_success() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users", "list_users");

        let result = adapter.handle("GET /users").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("HTTP 200"));
        assert!(response.contains("list_users"));
    }

    #[tokio::test]
    async fn test_handle_request_not_found() {
        let adapter = RestAdapter::new();
        let result = adapter.handle("GET /users").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("HTTP 404"));
        assert!(response.contains("Not Found"));
    }

    #[tokio::test]
    async fn test_handle_request_invalid() {
        let adapter = RestAdapter::new();
        let result = adapter.handle("INVALID").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("HTTP 400"));
    }

    #[tokio::test]
    async fn test_handle_request_with_params() {
        let mut adapter = RestAdapter::new();
        adapter.route("GET", "/users/:id", "get_user");

        let result = adapter.handle("GET /users/42").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains("HTTP 200"));
        assert!(response.contains("get_user"));
        assert!(response.contains("/users/42"));
    }

    #[test]
    fn test_rest_route_new() {
        let route = RestRoute::new("GET", "/users", "list_users");
        assert_eq!(route.method, "GET");
        assert_eq!(route.path, "/users");
        assert_eq!(route.handler, "list_users");
    }

    #[test]
    fn test_rest_route_matches() {
        let route = RestRoute::new("GET", "/users", "list_users");
        assert!(route.matches("GET", "/users"));
        assert!(!route.matches("POST", "/users"));
        assert!(!route.matches("GET", "/posts"));
    }

    #[test]
    fn test_rest_route_matches_path_with_params() {
        let route = RestRoute::new("GET", "/users/:id/posts/:post_id", "handler");

        assert!(route.matches_path("/users/42/posts/100"));
        assert!(route.matches_path("/users/123/posts/456"));
        assert!(!route.matches_path("/users/42"));
        assert!(!route.matches_path("/users/42/posts"));
        assert!(!route.matches_path("/users/42/posts/100/extra"));
    }
}
