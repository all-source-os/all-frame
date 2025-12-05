//! Default features example for binary size measurement
//!
//! Measures binary size with default features: di, openapi, router
//!
//! This example demonstrates:
//! - Basic router with REST endpoints
//! - OpenAPI spec generation with server configuration
//! - Scalar documentation with "Try It" functionality

use allframe_core::{prelude::*, router::OpenApiGenerator};

fn main() {
    // Create a router with routes
    let mut router = Router::new();
    router.get("/users", || async { "Users".to_string() });
    router.post("/users", || async { "Created".to_string() });

    // Generate OpenAPI spec with server configuration for "Try It" functionality
    let spec = OpenApiGenerator::new("API", "1.0.0")
        .with_description("Example API with default features")
        .with_server("http://localhost:3000", Some("Development"))
        .with_server("https://api.example.com", Some("Production"))
        .generate(&router);

    // Print info to prevent optimizations
    println!("Router handlers: {}", router.handlers_count());

    // Display server configuration
    if let Some(servers) = spec.get("servers").and_then(|s| s.as_array()) {
        println!("Servers configured: {}", servers.len());
        for server in servers {
            let url = server["url"].as_str().unwrap_or("unknown");
            println!("  - {}", url);
        }
    }
}
