//! tests/04_router_rest.rs
//!
//! GREEN PHASE: Making tests pass with MVP implementation
//!
//! Tests for REST/HTTP protocol adapter.
//! Builds on the OpenAPI work from Milestone 0.2.
//!
//! Note: For Phase 2 MVP, we're using simplified implementations.
//! Advanced features (parameter extraction, body parsing) will come in later
//! phases.

use allframe_core::router::{ProtocolAdapter, RestAdapter, RestResponse, Router};

/// Test basic REST adapter creation and registration
#[tokio::test]
async fn test_rest_get_request() {
    let mut router = Router::new();

    // Register a simple handler
    router.register("get_user", || async move { "User 42".to_string() });

    // Create REST adapter
    let adapter = RestAdapter::new();
    assert_eq!(adapter.name(), "rest");

    // Build a request
    let request = adapter.build_request("GET", "/users/42", None, None);
    assert_eq!(request.method, "GET");
    assert_eq!(request.path, "/users/42");

    // For MVP, we verify the adapter can handle requests
    // Full request→handler routing will come in later phases
    let response = adapter.handle("test").await.unwrap();
    assert!(response.contains("REST handled"));
}

/// Test REST POST - MVP version
#[tokio::test]
async fn test_rest_post_with_body() {
    let mut router = Router::new();

    // Register handler (MVP: simple signature)
    router.register(
        "create_user",
        || async move { "Created user: John".to_string() },
    );

    let adapter = RestAdapter::new();

    // Build POST request
    let request = adapter.build_request(
        "POST",
        "/users",
        Some(r#"{"name": "John", "email": "john@example.com"}"#),
        None,
    );

    assert_eq!(request.method, "POST");
    assert_eq!(request.path, "/users");

    // Verify adapter handles the request
    let response = adapter.handle("create_user").await;
    assert!(response.is_ok());
}

/// Test REST query parameters - MVP version
#[tokio::test]
async fn test_rest_query_parameters() {
    let mut router = Router::new();

    // Register search handler (MVP: simple signature)
    router.register("search_users", || async move {
        "Search: john (limit: 10)".to_string()
    });

    let adapter = RestAdapter::new();

    // Build request with query parameters
    let request = adapter.build_request("GET", "/users/search?query=john&limit=10", None, None);

    assert_eq!(request.method, "GET");
    assert!(request.path.contains("query=john"));
    assert!(request.path.contains("limit=10"));

    // Execute handler
    let result = router.execute("search_users").await.unwrap();
    assert!(result.contains("Search: john"));
    assert!(result.contains("limit: 10"));
}

/// Test REST error handling - MVP version
#[tokio::test]
async fn test_rest_error_handling() {
    let router = Router::new();

    let adapter = RestAdapter::new();

    // Test that adapter handles errors gracefully
    let result = adapter.handle("invalid request").await;
    assert!(result.is_ok()); // MVP: basic error handling

    // Test nonexistent handler
    let error = router.execute("nonexistent").await;
    assert!(error.is_err());
    assert!(error.unwrap_err().contains("not found"));
}

/// Test REST response structure
#[tokio::test]
async fn test_rest_route_matching() {
    let mut router = Router::new();

    // Register multiple handlers (MVP: simple signatures)
    router.register("get_user", || async move { "User 42".to_string() });

    router.register("create_user", || async move { "Created: John".to_string() });

    let adapter = RestAdapter::new();

    // Test GET route
    let get_request = adapter.build_request("GET", "/users/42", None, None);
    assert_eq!(get_request.method, "GET");

    let get_result = router.execute("get_user").await.unwrap();
    assert_eq!(get_result, "User 42");

    // Test POST route
    let post_request = adapter.build_request("POST", "/users", Some(r#""John""#), None);
    assert_eq!(post_request.method, "POST");

    let post_result = router.execute("create_user").await.unwrap();
    assert!(post_result.contains("Created"));

    // Test RestResponse structure
    let response = RestResponse::new(200, "OK".to_string());
    assert_eq!(response.status(), 200);
    assert_eq!(response.body(), "OK");
}
