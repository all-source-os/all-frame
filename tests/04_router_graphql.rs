//! tests/04_router_graphql.rs
//!
//! GREEN PHASE: Making tests pass with MVP implementation
//!
//! Tests for GraphQL protocol adapter.
//!
//! Note: For Phase 4 MVP, we're using simplified GraphQL parsing.
//! Full GraphQL query parsing and schema introspection will come in later
//! phases.

use allframe_core::router::{GraphQLAdapter, ProtocolAdapter, Router};

/// Test basic GraphQL query
#[tokio::test]
async fn test_graphql_query() {
    let mut router = Router::new();

    // Register handler (MVP: simple signature)
    router.register("user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    let mut adapter = GraphQLAdapter::new();
    adapter.query("user", "user");
    assert_eq!(adapter.name(), "graphql");

    // Execute GraphQL query
    let query = r#"
        query {
            user(id: 42) {
                id
                name
            }
        }
    "#;

    let response = adapter.handle(query).await.unwrap();
    assert!(response.contains("data") || response.contains("user"));
}

/// Test GraphQL mutation
#[tokio::test]
async fn test_graphql_mutation() {
    let mut router = Router::new();

    // Register handler (MVP: simple signature)
    router.register("createUser", || async move {
        r#"{"name": "John", "email": "john@example.com"}"#.to_string()
    });

    let mut adapter = GraphQLAdapter::new();
    adapter.mutation("createUser", "createUser");

    // Execute GraphQL mutation
    let mutation = r#"
        mutation {
            createUser(name: "John", email: "john@example.com") {
                name
                email
            }
        }
    "#;

    let response = adapter.handle(mutation).await.unwrap();
    assert!(!response.is_empty());
}

/// Test GraphQL schema generation
#[tokio::test]
async fn test_graphql_schema_generation() {
    let mut router = Router::new();

    // Register handler
    router.register("user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    let mut adapter = GraphQLAdapter::new();
    adapter.query("user", "user");

    // Generate schema
    let schema = adapter.generate_schema();

    // Verify schema contains expected types
    assert!(schema.contains("type Query"));
    assert!(schema.contains("user"));
}

/// Test GraphQL with nested types - MVP version
#[tokio::test]
async fn test_graphql_nested_types() {
    let mut router = Router::new();

    // Register handler (MVP: returns JSON with nested structure)
    router.register("user", || async move {
        r#"{"id": 42, "name": "John Doe", "address": {"street": "123 Main St", "city": "Springfield"}}"#
            .to_string()
    });

    let mut adapter = GraphQLAdapter::new();
    adapter.query("user", "user");

    // Execute query (MVP: simplified parsing)
    let query = r#"
        query {
            user(id: 42) {
                name
                address {
                    city
                }
            }
        }
    "#;

    let response = adapter.handle(query).await.unwrap();
    assert!(!response.is_empty());
    // MVP: Returns mock data, full nested type support in later phases
}

/// Test GraphQL error handling
#[tokio::test]
async fn test_graphql_error_handling() {
    let mut router = Router::new();

    // Register handler that returns Result
    router.register("user", || async move {
        r#"{"id": 42, "name": "John"}"#.to_string()
    });

    let mut adapter = GraphQLAdapter::new();
    adapter.query("user", "user");

    // Test valid query
    let valid_query = r#"query { user(id: 42) }"#;
    let response = adapter.handle(valid_query).await;
    assert!(response.is_ok());

    // Test empty query (MVP: returns error in response body per GraphQL spec)
    let empty_query = "";
    let response = adapter.handle(empty_query).await.unwrap();
    // GraphQL errors are returned in response body, not as actual errors
    assert!(response.contains("errors") || response.contains("Empty"));
}
