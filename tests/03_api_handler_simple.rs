//! Simplified API Handler Tests for v0.2 MVP
//!
//! These tests verify basic OpenAPI schema generation.
//! Full type introspection will come in v0.3.

use allframe_macros::api_handler;
use serde::{Deserialize, Serialize};

/// Test basic API handler with minimal schema
#[test]
fn test_api_handler_generates_schema() {
    #[api_handler(path = "/health", method = "GET", description = "Health check")]
    async fn health_check() -> String {
        "OK".to_string()
    }

    // The macro should generate a schema function
    let schema = health_check_openapi_schema();

    // Verify it's valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&schema).expect("Schema should be valid JSON");

    // Verify basic OpenAPI structure
    assert!(parsed.is_object());
    assert_eq!(parsed["openapi"], "3.1.0");

    // Verify the path is in the schema
    let schema_str = schema.to_lowercase();
    assert!(schema_str.contains("/health"));
    assert!(schema_str.contains("get"));
}

/// Test API handler with POST method
#[test]
fn test_api_handler_post_method() {
    #[derive(Serialize, Deserialize)]
    struct CreateRequest {
        name: String,
    }

    #[api_handler(path = "/users", method = "POST", description = "Create user")]
    async fn create_user(_req: CreateRequest) -> String {
        "Created".to_string()
    }

    let schema = create_user_openapi_schema();

    // Verify POST method is in schema
    let schema_str = schema.to_lowercase();
    assert!(schema_str.contains("post"));
    assert!(schema_str.contains("/users"));
    assert!(schema_str.contains("create user"));
}

/// Test API handler without description
#[test]
fn test_api_handler_default_description() {
    #[api_handler(path = "/status", method = "GET")]
    async fn status() -> String {
        "Running".to_string()
    }

    let schema = status_openapi_schema();

    // Should have a default description
    assert!(schema.contains("status"));
}
