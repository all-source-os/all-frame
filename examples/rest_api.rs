//! REST API Example
//!
//! This example demonstrates how to use AllFrame's REST adapter to build a REST
//! API.
//!
//! Key concepts:
//! - Router setup with REST adapter
//! - Handler registration for different endpoints
//! - GET/POST request handling
//! - Error handling
//! - Integration with OpenAPI schema generation (from Milestone 0.2)
//!
//! Run this example:
//! ```bash
//! cargo run --example rest_api
//! ```

use allframe_core::prelude::*;

#[tokio::main]
async fn main() {
    println!("=== AllFrame REST API Example ===\n");

    // Create a new router
    let mut router = Router::new();

    // Register handlers for different endpoints
    // Each handler is a simple async function that returns a String

    // GET /users - List all users
    router.register("list_users", || async move {
        r#"{"users": [{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]}"#.to_string()
    });

    // GET /users/:id - Get a specific user
    router.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string()
    });

    // POST /users - Create a new user
    router.register("create_user", || async move {
        r#"{"id": 3, "name": "Charlie", "email": "charlie@example.com", "created": true}"#
            .to_string()
    });

    // PUT /users/:id - Update a user
    router.register("update_user", || async move {
        r#"{"id": 42, "name": "John Updated", "email": "john.new@example.com", "updated": true}"#
            .to_string()
    });

    // DELETE /users/:id - Delete a user
    router.register("delete_user", || async move {
        r#"{"id": 42, "deleted": true}"#.to_string()
    });

    // Create and register the REST adapter
    let adapter = RestAdapter::new();
    router.add_adapter(Box::new(adapter));

    println!(
        "✓ Router initialized with {} handlers",
        router.handlers_count()
    );
    println!("✓ REST adapter registered\n");

    // Demonstrate the REST adapter capabilities
    let rest_adapter = RestAdapter::new();

    println!("--- Example 1: GET Request ---");
    // Build a GET request
    let get_request = rest_adapter.build_request("GET", "/users/42", None, None);
    println!("Request: {} {}", get_request.method, get_request.path);

    // Execute the handler
    let response = router.execute("get_user").await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 2: POST Request with Body ---");
    // Build a POST request with a body
    let post_body = r#"{"name": "Charlie", "email": "charlie@example.com"}"#;
    let post_request = rest_adapter.build_request("POST", "/users", Some(post_body), None);
    println!("Request: {} {}", post_request.method, post_request.path);
    println!("Body: {}", post_body);

    // Execute the handler
    let response = router.execute("create_user").await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 3: GET Request with Query Parameters ---");
    // Build a GET request with query parameters
    let query_request =
        rest_adapter.build_request("GET", "/users/search?name=John&limit=10", None, None);
    println!("Request: {} {}", query_request.method, query_request.path);
    println!("Note: Full query parameter parsing coming in future phases\n");

    println!("--- Example 4: Error Handling ---");
    // Try to execute a non-existent handler
    match router.execute("nonexistent_handler").await {
        Ok(response) => println!("Response: {}", response),
        Err(error) => println!("Error: {}\n", error),
    }

    println!("--- Example 5: RestResponse Structure ---");
    // Create a structured REST response
    let success_response = RestResponse::new(200, r#"{"status": "ok"}"#.to_string());
    println!("Status: {}", success_response.status());
    println!("Body: {}\n", success_response.body());

    let error_response = RestResponse::new(404, r#"{"error": "Not Found"}"#.to_string());
    println!("Status: {}", error_response.status());
    println!("Body: {}\n", error_response.body());

    println!("--- Example 6: List All Handlers ---");
    let users_response = router.execute("list_users").await.unwrap();
    println!("GET /users");
    println!("Response: {}\n", users_response);

    println!("=== Key Takeaways ===");
    println!("1. Handlers are simple async functions that return String");
    println!("2. RestAdapter provides protocol-specific request building");
    println!("3. Router executes handlers by name");
    println!("4. RestResponse provides structured HTTP responses");
    println!("5. Error handling is built-in with Result types");
    println!("\n✓ REST API example complete!");
    println!("\nNext steps:");
    println!("- See examples/graphql_api.rs for GraphQL support");
    println!("- See examples/multi_protocol.rs for multi-protocol routing");
    println!("- Integration with Milestone 0.2 OpenAPI schema generation coming soon");
}
