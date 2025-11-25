//! tests/04_router_rest.rs
//!
//! RED PHASE: This test MUST fail initially
//!
//! Tests for REST/HTTP protocol adapter.
//! Builds on the OpenAPI work from Milestone 0.2.
//!
//! Acceptance criteria:
//! - REST adapter can transform HTTP requests to handler calls
//! - Path parameters are extracted correctly
//! - Query parameters are extracted correctly
//! - Request body is parsed to handler arguments
//! - Response is serialized to HTTP response
//! - HTTP status codes are set correctly

/// Test basic REST GET request
#[test]
fn test_rest_get_request() {
    // This test will fail because REST adapter doesn't exist yet
    //
    // use allframe::router::{Router, RestAdapter};
    //
    // let mut router = Router::new();
    // router.register("get_user", |id: i32| async move {
    //     format!("User {}", id)
    // });
    //
    // let adapter = RestAdapter::new();
    // let request = adapter.build_request("GET", "/users/42", None, None);
    //
    // let response = adapter.handle(request, &router).await.unwrap();
    // assert_eq!(response.status(), 200);
    // assert_eq!(response.body(), "User 42");

    panic!("REST adapter not implemented yet - RED PHASE");
}

/// Test REST POST request with JSON body
#[test]
fn test_rest_post_with_body() {
    // This test will fail because request body parsing isn't implemented
    //
    // use allframe::router::{Router, RestAdapter};
    // use serde::{Deserialize, Serialize};
    //
    // #[derive(Serialize, Deserialize)]
    // struct CreateUserRequest {
    //     name: String,
    //     email: String,
    // }
    //
    // let mut router = Router::new();
    // router.register("create_user", |req: CreateUserRequest| async move {
    //     format!("Created user: {}", req.name)
    // });
    //
    // let adapter = RestAdapter::new();
    // let body = r#"{"name": "John", "email": "john@example.com"}"#;
    // let request = adapter.build_request("POST", "/users", Some(body), None);
    //
    // let response = adapter.handle(request, &router).await.unwrap();
    // assert_eq!(response.status(), 201);

    panic!("REST POST with body not implemented yet - RED PHASE");
}

/// Test REST with query parameters
#[test]
fn test_rest_query_parameters() {
    // This test will fail because query parameter extraction isn't implemented
    //
    // use allframe::router::{Router, RestAdapter};
    //
    // let mut router = Router::new();
    // router.register("search_users", |query: String, limit: i32| async move {
    //     format!("Search: {} (limit: {})", query, limit)
    // });
    //
    // let adapter = RestAdapter::new();
    // let request = adapter.build_request("GET", "/users/search?query=john&limit=10", None, None);
    //
    // let response = adapter.handle(request, &router).await.unwrap();
    // assert!(response.body().contains("Search: john"));
    // assert!(response.body().contains("limit: 10"));

    panic!("REST query parameters not implemented yet - RED PHASE");
}

/// Test REST error handling
#[test]
fn test_rest_error_handling() {
    // This test will fail because error handling isn't implemented
    //
    // use allframe::router::{Router, RestAdapter};
    //
    // let mut router = Router::new();
    // router.register("get_user", |id: i32| async move {
    //     if id > 0 {
    //         Ok(format!("User {}", id))
    //     } else {
    //         Err("Invalid ID".to_string())
    //     }
    // });
    //
    // let adapter = RestAdapter::new();
    // let request = adapter.build_request("GET", "/users/-1", None, None);
    //
    // let response = adapter.handle(request, &router).await.unwrap();
    // assert_eq!(response.status(), 400);
    // assert!(response.body().contains("Invalid ID"));

    panic!("REST error handling not implemented yet - RED PHASE");
}

/// Test REST route matching
#[test]
fn test_rest_route_matching() {
    // This test will fail because route matching isn't implemented
    //
    // use allframe::router::{Router, RestAdapter};
    //
    // let mut router = Router::new();
    // router.register_with_route("GET", "/users/{id}", "get_user", |id: i32| async move {
    //     format!("User {}", id)
    // });
    // router.register_with_route("POST", "/users", "create_user", |name: String| async move {
    //     format!("Created: {}", name)
    // });
    //
    // let adapter = RestAdapter::new();
    //
    // // Test GET route
    // let get_request = adapter.build_request("GET", "/users/42", None, None);
    // let get_response = adapter.handle(get_request, &router).await.unwrap();
    // assert_eq!(get_response.status(), 200);
    //
    // // Test POST route
    // let post_request = adapter.build_request("POST", "/users", Some(r#""John""#), None);
    // let post_response = adapter.handle(post_request, &router).await.unwrap();
    // assert_eq!(post_response.status(), 201);

    panic!("REST route matching not implemented yet - RED PHASE");
}
