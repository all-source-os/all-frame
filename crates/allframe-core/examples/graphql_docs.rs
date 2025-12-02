//! GraphQL API Documentation Example
//!
//! This example demonstrates how to use AllFrame's GraphiQL integration
//! to generate beautiful, interactive GraphQL API documentation.
//!
//! Features demonstrated:
//! - GraphiQL playground configuration
//! - Multiple theme options
//! - Subscription support via WebSocket
//! - Custom headers configuration
//! - Query history persistence
//! - Schema explorer
//!
//! Run with: cargo run --example graphql_docs --features "router,openapi"

use allframe_core::router::{graphiql_html, GraphiQLConfig, GraphiQLTheme, Router};

fn main() {
    println!("🚀 AllFrame GraphQL Documentation Example");
    println!("==========================================\n");

    // Create router (in a real app, this would have GraphQL handlers)
    let router = Router::new();
    println!("✅ Router created\n");

    // Configure GraphiQL playground with all features
    println!("🎨 Configuring GraphiQL playground...");

    let graphiql_config = GraphiQLConfig::new()
        .endpoint_url("/graphql")
        .subscription_url("ws://localhost:3000/graphql")
        .theme(GraphiQLTheme::Dark)
        .enable_explorer(true)
        .enable_history(true)
        .add_header("Authorization", "Bearer your-token-here")
        .add_header("X-API-Version", "v1")
        .cdn_url("https://unpkg.com/graphiql@3.0.0/graphiql.min.css")
        .custom_css(
            r#"
            body {
                font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
            }
            .graphiql-container {
                --color-primary: 60, 76, 231;
            }
        "#,
        );

    println!("   • Endpoint: /graphql");
    println!("   • Subscriptions: ws://localhost:3000/graphql");
    println!("   • Theme: Dark mode");
    println!("   • Explorer: Enabled");
    println!("   • History: Enabled");
    println!("   • Custom headers: 2 configured");
    println!("   • Custom CSS: Applied\n");

    // Generate GraphiQL HTML
    println!("🎭 Generating GraphiQL HTML documentation...");

    let graphiql_html = graphiql_html(&graphiql_config, "AllFrame GraphQL API");

    println!("✅ GraphiQL HTML generated ({} bytes)\n", graphiql_html.len());

    // Show what the HTML contains
    println!("📦 Generated documentation includes:");
    println!("   ✅ Interactive GraphQL playground");
    println!("   ✅ Schema explorer sidebar");
    println!("   ✅ Query editor with syntax highlighting");
    println!("   ✅ Variables editor");
    println!("   ✅ Headers configuration");
    println!("   ✅ Query history persistence");
    println!("   ✅ Subscription support via WebSocket");
    println!("   ✅ Dark theme by default");
    println!();

    // Usage instructions
    println!("🚀 Next Steps:");
    println!("   1. Integrate with your web framework (Axum, Actix, etc.)");
    println!("   2. Serve the HTML at /graphql/playground");
    println!("   3. Implement GraphQL endpoint at /graphql");
    println!("   4. Optionally: Add WebSocket support for subscriptions");
    println!();

    // Example integration code
    println!("💡 Example Integration (Axum with async-graphql):");
    println!(
        r#"
    use axum::{{routing::{{get, post}}, Router, response::Html}};
    use async_graphql::{{Schema, EmptyMutation, EmptySubscription}};
    use async_graphql_axum::GraphQL;

    // Define your GraphQL schema
    struct Query;

    #[Object]
    impl Query {{
        async fn hello(&self) -> &str {{
            "Hello, GraphQL!"
        }}

        async fn users(&self) -> Vec<User> {{
            vec![
                User {{ id: 1, name: "Alice".to_string() }},
                User {{ id: 2, name: "Bob".to_string() }},
            ]
        }}
    }}

    #[derive(SimpleObject)]
    struct User {{
        id: i32,
        name: String,
    }}

    #[tokio::main]
    async fn main() {{
        // Build GraphQL schema
        let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

        let app = Router::new()
            // GraphQL endpoint
            .route("/graphql", post(GraphQL::new(schema.clone())))
            // GraphiQL playground
            .route("/graphql/playground", get(|| async {{
                Html(graphiql_html)
            }}));

        let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
            .await
            .unwrap();

        println!("📚 GraphiQL Playground: http://localhost:3000/graphql/playground");
        println!("🔌 GraphQL Endpoint: http://localhost:3000/graphql");

        axum::serve(listener, app).await.unwrap();
    }}
    "#
    );

    // Example GraphQL queries
    println!("\n💡 Example GraphQL Queries to Try:");
    println!(
        r#"
    # Simple query
    query {{
        hello
    }}

    # Query with fields
    query {{
        users {{
            id
            name
        }}
    }}

    # Query with variables
    query GetUser($id: Int!) {{
        user(id: $id) {{
            id
            name
            email
        }}
    }}

    # Variables (in Variables panel):
    {{
        "id": 1
    }}

    # Mutation
    mutation {{
        createUser(name: "Charlie", email: "charlie@example.com") {{
            id
            name
            email
        }}
    }}

    # Subscription (requires WebSocket)
    subscription {{
        userCreated {{
            id
            name
        }}
    }}
    "#
    );

    // Feature comparison
    println!("\n📊 GraphiQL vs GraphQL Playground:");
    println!("   GraphiQL 3.0      | GraphQL Playground");
    println!("   ─────────────────────────────────────");
    println!("   Modern React UI   | Older UI");
    println!("   Active development| Deprecated");
    println!("   Built-in explorer | Plugin required");
    println!("   ~100KB bundle     | ~200KB bundle");
    println!("   Dark mode native  | Theme switching");
    println!();

    // Configuration options
    println!("⚙️ Available Configuration Options:");
    println!("   • endpoint_url: GraphQL endpoint URL");
    println!("   • subscription_url: WebSocket URL for subscriptions");
    println!("   • theme: Light or Dark");
    println!("   • enable_explorer: Toggle schema explorer sidebar");
    println!("   • enable_history: Toggle query history persistence");
    println!("   • add_header: Add custom HTTP headers");
    println!("   • cdn_url: Pin GraphiQL version");
    println!("   • custom_css: Inject custom styling");
    println!();

    println!("✨ Example complete!");
    println!("\n🎯 Key Takeaways:");
    println!("   1. GraphiQL provides beautiful, interactive GraphQL documentation");
    println!("   2. Full subscription support via WebSocket");
    println!("   3. Schema explorer for easy API discovery");
    println!("   4. Query history for development productivity");
    println!("   5. Customizable theming and styling");
    println!("   6. Production-ready with CDN version pinning");
}
