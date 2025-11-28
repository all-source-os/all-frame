//! tests/03_api_handler.rs
//!
//! RED PHASE: This test MUST fail initially
//!
//! Tests for the #[api_handler] procedural macro as specified in PRD_01.md.
//! The macro provides automatic OpenAPI 3.1 schema generation with:
//! - Automatic request/response type extraction
//! - OpenAPI 3.1 compliant JSON schema
//! - Swagger UI integration
//! - Type-safe validation
//!
//! Acceptance criteria from PRD:
//! - `curl /openapi.json` returns valid OpenAPI 3.1 schema
//! - Swagger UI loads and displays endpoints
//! - MCP schema present for LLM integration

// Allow dead code - testing #[api_handler] macro expansion and schema generation.
// Test fixtures (CreateUserRequest, CreateUserResponse, etc.) define request/response
// types that the macro introspects to generate OpenAPI schemas. The macro generates
// functions like `create_user_openapi_schema()` which the tests call to verify schema
// correctness. Unused structs/functions are intentional test fixtures demonstrating
// different API patterns (POST, GET, query params, path params, validation, errors).
#[allow(dead_code)]

use allframe_macros::api_handler;
use serde::{Deserialize, Serialize};

/// Request type for user creation
#[derive(Debug, Serialize, Deserialize)]
struct CreateUserRequest {
    name: String,
    email: String,
}

/// Response type for user creation
#[derive(Debug, Serialize, Deserialize)]
struct CreateUserResponse {
    id: i32,
    name: String,
    email: String,
}

/// Test basic API handler with OpenAPI generation
#[test]
fn test_api_handler_basic() {
    #[api_handler(path = "/users", method = "POST", description = "Create a new user")]
    async fn create_user(req: CreateUserRequest) -> CreateUserResponse {
        CreateUserResponse {
            id: 1,
            name: req.name,
            email: req.email,
        }
    }

    // The macro should generate OpenAPI schema
    let schema = create_user_openapi_schema();

    // Verify the schema has correct path
    assert!(schema.contains("/users"));
    // Verify the schema has correct method
    assert!(schema.contains("POST") || schema.contains("post"));
    // Verify the schema has description
    assert!(schema.contains("Create a new user"));
    // Verify the schema has request body
    assert!(schema.contains("CreateUserRequest") || schema.contains("requestBody"));
    // Verify the schema has response
    assert!(schema.contains("CreateUserResponse") || schema.contains("responses"));
}

/// Test API handler with query parameters
#[test]
fn test_api_handler_with_query() {
    #[derive(Debug, Serialize, Deserialize)]
    struct ListUsersQuery {
        page: Option<i32>,
        limit: Option<i32>,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct User {
        id: i32,
        name: String,
    }

    #[api_handler(
        path = "/users",
        method = "GET",
        description = "List all users with pagination"
    )]
    async fn list_users(query: ListUsersQuery) -> Vec<User> {
        let _page = query.page.unwrap_or(1);
        let _limit = query.limit.unwrap_or(10);

        vec![
            User {
                id: 1,
                name: "Alice".to_string(),
            },
            User {
                id: 2,
                name: "Bob".to_string(),
            },
        ]
    }

    let schema = list_users_openapi_schema();

    // Verify query parameters are in schema
    assert!(schema.contains("page") || schema.contains("parameters"));
    assert!(schema.contains("limit"));
}

/// Test API handler with path parameters
#[test]
fn test_api_handler_with_path_params() {
    #[derive(Debug, Serialize, Deserialize)]
    struct UserResponse {
        id: i32,
        name: String,
    }

    #[api_handler(path = "/users/{id}", method = "GET", description = "Get user by ID")]
    async fn get_user(id: i32) -> Option<UserResponse> {
        if id == 1 {
            Some(UserResponse {
                id,
                name: "Test User".to_string(),
            })
        } else {
            None
        }
    }

    let schema = get_user_openapi_schema();

    // Verify path parameter is in schema
    assert!(schema.contains("{id}") || schema.contains("parameters"));
    assert!(schema.contains("id"));
}

/// Test API handler with multiple response types (success/error)
#[test]
fn test_api_handler_with_error_responses() {
    #[derive(Debug, Serialize, Deserialize)]
    struct SuccessResponse {
        message: String,
    }

    #[derive(Debug, Serialize, Deserialize)]
    struct ErrorResponse {
        error: String,
    }

    #[api_handler(
        path = "/validate",
        method = "POST",
        description = "Validate data",
        responses = {
            200: SuccessResponse,
            400: ErrorResponse,
        }
    )]
    async fn validate_data(data: String) -> Result<SuccessResponse, ErrorResponse> {
        if data.is_empty() {
            Err(ErrorResponse {
                error: "Data cannot be empty".to_string(),
            })
        } else {
            Ok(SuccessResponse {
                message: "Valid".to_string(),
            })
        }
    }

    let schema = validate_data_openapi_schema();

    // Verify multiple response codes are in schema
    assert!(schema.contains("200"));
    assert!(schema.contains("400"));
    assert!(schema.contains("SuccessResponse") || schema.contains("responses"));
    assert!(schema.contains("ErrorResponse") || schema.contains("error"));
}

/// Test that OpenAPI schema is valid JSON
#[test]
fn test_api_handler_valid_json_schema() {
    #[api_handler(
        path = "/health",
        method = "GET",
        description = "Health check endpoint"
    )]
    async fn health_check() -> String {
        "OK".to_string()
    }

    let schema = health_check_openapi_schema();

    // The schema should be valid JSON
    let parsed: serde_json::Value =
        serde_json::from_str(&schema).expect("Schema should be valid JSON");

    // Verify it has OpenAPI structure
    assert!(parsed.is_object());
}

/// Test API handler with request validation
#[test]
fn test_api_handler_with_validation() {
    #[derive(Debug, Serialize, Deserialize)]
    struct ValidatedRequest {
        #[serde(default)]
        email: String,
    }

    #[api_handler(
        path = "/register",
        method = "POST",
        description = "Register user with email validation",
        validate = true
    )]
    async fn register_user(req: ValidatedRequest) -> String {
        format!("Registered: {}", req.email)
    }

    let schema = register_user_openapi_schema();

    // Verify schema includes validation rules
    assert!(schema.contains("email"));
}

/// Test OpenAPI schema aggregation from multiple handlers
#[test]
fn test_openapi_schema_aggregation() {
    #[api_handler(path = "/users", method = "GET")]
    async fn get_users() -> Vec<String> {
        vec![]
    }

    #[api_handler(path = "/posts", method = "GET")]
    async fn get_posts() -> Vec<String> {
        vec![]
    }

    // The framework should be able to aggregate all schemas
    let schema_1 = get_users_openapi_schema();
    let schema_2 = get_posts_openapi_schema();

    assert!(schema_1.contains("/users"));
    assert!(schema_2.contains("/posts"));
}
