//! Scalar API Documentation Example
//!
//! This example demonstrates how to use AllFrame's Scalar integration
//! to generate beautiful, interactive API documentation with "Try It" functionality.
//!
//! Features demonstrated:
//! - REST API with multiple endpoints
//! - OpenAPI 3.1 generation with servers
//! - Scalar UI with custom configuration
//! - Server URLs for "Try It" functionality
//! - CDN configuration
//! - Proxy URL for CORS handling
//!
//! Run with: cargo run --example scalar_docs --features "router,openapi"

use allframe_core::router::{OpenApiGenerator, Router, ScalarConfig, ScalarTheme};

fn main() {
    println!("🚀 AllFrame Scalar Documentation Example");
    println!("=========================================\n");

    // Create router
    let mut router = Router::new();

    // Register API endpoints
    println!("📝 Registering API endpoints...");

    // GET /users - List all users
    router.get("/users", || async {
        r#"[{"id":1,"name":"Alice","email":"alice@example.com"},{"id":2,"name":"Bob","email":"bob@example.com"}]"#.to_string()
    });

    // GET /users/{id} - Get specific user
    router.get("/users/{id}", || async {
        r#"{"id":1,"name":"Alice","email":"alice@example.com"}"#.to_string()
    });

    // POST /users - Create new user
    router.post("/users", || async {
        r#"{"id":3,"name":"Charlie","email":"charlie@example.com"}"#.to_string()
    });

    // PUT /users/{id} - Update user
    router.put("/users/{id}", || async {
        r#"{"id":1,"name":"Alice Updated","email":"alice.updated@example.com"}"#.to_string()
    });

    // DELETE /users/{id} - Delete user
    router.delete("/users/{id}", || async {
        r#"{"success":true,"message":"User deleted successfully"}"#.to_string()
    });

    // GET /health - Health check endpoint
    router.get("/health", || async {
        r#"{"status":"healthy","timestamp":"2025-12-01T00:00:00Z"}"#.to_string()
    });

    println!("✅ Registered {} endpoints\n", router.handlers_count());

    // Generate OpenAPI specification with servers
    println!("📚 Generating OpenAPI specification...");

    let openapi_spec = OpenApiGenerator::new("AllFrame Users API", "1.0.0")
        .with_description(
            "A sample REST API demonstrating AllFrame's Scalar integration. \
             This API provides CRUD operations for user management."
        )
        .with_server("http://localhost:3000", Some("Local Development"))
        .with_server("https://api.example.com", Some("Production"))
        .with_server("https://staging.example.com", Some("Staging"))
        .generate(&router);

    let openapi_json = serde_json::to_string_pretty(&openapi_spec)
        .expect("Failed to serialize OpenAPI spec");

    println!("✅ OpenAPI spec generated ({} bytes)\n", openapi_json.len());

    // Display server configuration
    if let Some(servers) = openapi_spec.get("servers").and_then(|s| s.as_array()) {
        println!("🌐 Configured servers for 'Try It' functionality:");
        for server in servers {
            let url = server["url"].as_str().unwrap_or("unknown");
            let desc = server
                .get("description")
                .and_then(|d| d.as_str())
                .unwrap_or("No description");
            println!("   • {} - {}", url, desc);
        }
        println!();
    }

    // Configure Scalar UI with all features
    println!("🎨 Configuring Scalar UI...");

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

    println!("   • Theme: Dark mode");
    println!("   • Sidebar: Enabled");
    println!("   • CDN: jsdelivr (version pinned)");
    println!("   • Proxy: Configured for CORS handling");
    println!("   • Custom CSS: Applied\n");

    // Generate Scalar HTML
    println!("🎭 Generating Scalar HTML documentation...");

    let scalar_html = allframe_core::router::scalar_html(
        &scalar_config,
        "AllFrame Users API",
        &openapi_json,
    );

    println!("✅ Scalar HTML generated ({} bytes)\n", scalar_html.len());

    // Show what the HTML contains
    println!("📦 Generated documentation includes:");
    println!("   ✅ Interactive API reference");
    println!("   ✅ 'Try It' button for each endpoint");
    println!("   ✅ Request/response examples");
    println!("   ✅ Schema documentation");
    println!("   ✅ Server selection dropdown");
    println!("   ✅ CORS proxy support");
    println!();

    // Usage instructions
    println!("🚀 Next Steps:");
    println!("   1. Integrate with your web framework (Axum, Actix, etc.)");
    println!("   2. Serve the HTML at /docs");
    println!("   3. Serve the OpenAPI JSON at /docs/openapi.json");
    println!("   4. Open http://localhost:3000/docs in your browser");
    println!();

    // Example integration code
    println!("💡 Example Integration (Axum):");
    println!(r#"
    use axum::{{routing::get, Router, response::Html, Json}};

    #[tokio::main]
    async fn main() {{
        let app = Router::new()
            .route("/docs", get(|| async {{
                Html(scalar_html)
            }}))
            .route("/docs/openapi.json", get(|| async {{
                Json(openapi_spec)
            }}))
            .route("/users", get(get_users))
            .route("/users/:id", get(get_user));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .unwrap();

        println!("📚 Documentation: http://localhost:3000/docs");

        axum::serve(listener, app).await.unwrap();
    }}
    "#);

    println!("\n✨ Example complete!");
}
