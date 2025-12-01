//! All features example for binary size measurement
//!
//! Measures binary size with all features enabled
//!
//! This example demonstrates:
//! - REST API with multiple endpoints
//! - OpenAPI generation with server configuration
//! - Scalar UI with advanced features:
//!   - CDN version pinning
//!   - CORS proxy support
//!   - Custom theme and styling
//!   - "Try It" functionality

use allframe_core::prelude::*;
use allframe_core::router::{OpenApiGenerator, ScalarConfig, ScalarTheme};

#[tokio::main]
async fn main() {
    // Create a router with multiple endpoints
    let mut router = Router::new();
    router.get("/users", || async { "Users list".to_string() });
    router.post("/users", || async { "User created".to_string() });
    router.get("/health", || async { "OK".to_string() });

    // Generate OpenAPI spec with servers for "Try It" functionality
    let openapi = OpenApiGenerator::new("AllFrame API", "1.0.0")
        .with_description("Full-featured API with Scalar documentation")
        .with_server("http://localhost:3000", Some("Development"))
        .with_server("https://api.example.com", Some("Production"))
        .with_server("https://staging.example.com", Some("Staging"))
        .generate(&router);

    let openapi_json = serde_json::to_string_pretty(&openapi)
        .expect("Failed to serialize OpenAPI spec");

    // Configure Scalar UI with all features
    let scalar_config = ScalarConfig::new()
        .theme(ScalarTheme::Dark)
        .show_sidebar(true)
        .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
        .proxy_url("https://proxy.scalar.com")
        .custom_css(
            r#"
            body {
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            }
            .api-reference {
                --scalar-color-accent: #4f46e5;
            }
        "#,
        );

    // Generate Scalar HTML documentation
    let _scalar_html = allframe_core::router::scalar_html(
        &scalar_config,
        "AllFrame API",
        &openapi_json,
    );

    // Print info to prevent optimizations
    println!("Router handlers: {}", router.handlers_count());

    // Display server configuration
    if let Some(servers) = openapi.get("servers").and_then(|s| s.as_array()) {
        println!("Servers configured: {}", servers.len());
        for server in servers {
            let url = server["url"].as_str().unwrap_or("unknown");
            let desc = server
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("No description");
            println!("  - {} ({})", url, desc);
        }
    }

    println!("All features example complete");
    println!("Scalar features: CDN pinning, CORS proxy, custom theme, 'Try It' enabled");
}
