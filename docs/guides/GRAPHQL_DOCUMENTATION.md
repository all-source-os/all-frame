# GraphQL Documentation with GraphiQL

**Complete guide to AllFrame's GraphQL API documentation with GraphiQL playground**

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Features](#features)
4. [Configuration](#configuration)
5. [Framework Integration](#framework-integration)
6. [Advanced Usage](#advanced-usage)
7. [Subscriptions](#subscriptions)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)
10. [API Reference](#api-reference)

---

## Overview

AllFrame provides beautiful, interactive GraphQL API documentation through GraphiQL 3.0 integration. GraphiQL is the de facto standard for GraphQL API exploration and testing, offering:

- **Interactive Playground**: Test queries, mutations, and subscriptions in real-time
- **Schema Explorer**: Browse your entire GraphQL schema with documentation
- **Auto-completion**: IntelliSense for queries, fields, and arguments
- **Query History**: Persistent storage of previous queries
- **Variables Editor**: JSON editor with syntax validation
- **Dark Mode**: Beautiful dark theme by default
- **Subscription Support**: Real-time subscriptions via WebSocket

### Why GraphiQL?

| Feature | GraphiQL 3.0 | GraphQL Playground | Altair |
|---------|-------------|-------------------|--------|
| Modern UI | ✅ React 18 | ❌ Deprecated | ✅ |
| Active Development | ✅ Yes | ❌ No | ✅ |
| Bundle Size | ~100KB | ~200KB | ~150KB |
| Schema Explorer | ✅ Built-in | 🟡 Plugin | ✅ |
| Subscriptions | ✅ WebSocket | ✅ WebSocket | ✅ |
| Type Safety | ✅ TypeScript | 🟡 Limited | ✅ |

**AllFrame uses GraphiQL 3.0** - the latest and most actively maintained GraphQL IDE.

---

## Quick Start

### Step 1: Add Dependencies

```toml
[dependencies]
allframe-core = { version = "0.1", features = ["router", "openapi"] }
async-graphql = "7.0"  # Or your preferred GraphQL library
```

### Step 2: Configure GraphiQL

```rust
use allframe_core::router::{GraphiQLConfig, GraphiQLTheme, graphiql_html};

let config = GraphiQLConfig::new()
    .endpoint_url("/graphql")
    .theme(GraphiQLTheme::Dark)
    .enable_explorer(true)
    .enable_history(true);

let html = graphiql_html(&config, "My GraphQL API");
```

### Step 3: Serve the Playground

```rust
// Axum example
use axum::{routing::get, Router, response::Html};

let app = Router::new()
    .route("/graphql/playground", get(|| async {
        Html(html)
    }));
```

### Step 4: Access the Playground

Navigate to `http://localhost:3000/graphql/playground` and start exploring your API!

---

## Features

### 1. Interactive Query Editor

- **Syntax Highlighting**: Full GraphQL syntax highlighting
- **Auto-completion**: IntelliSense for all schema types
- **Error Detection**: Real-time validation of queries
- **Formatting**: Automatic query formatting (Ctrl+Shift+P)

### 2. Schema Explorer

Browse your entire GraphQL schema with documentation:

```rust
let config = GraphiQLConfig::new()
    .enable_explorer(true);  // Enable schema explorer sidebar
```

Features:
- Browse all types, queries, mutations, and subscriptions
- View field descriptions and deprecation notices
- Navigate type relationships
- Search schema by name

### 3. Variables Editor

Edit query variables with JSON validation:

```graphql
query GetUser($id: Int!) {
    user(id: $id) {
        id
        name
    }
}
```

Variables panel:
```json
{
    "id": 42
}
```

### 4. Query History

Persistent storage of previous queries:

```rust
let config = GraphiQLConfig::new()
    .enable_history(true);  // Enable query history
```

Features:
- Automatically saves all executed queries
- Search through history
- Re-run previous queries
- Stored in browser localStorage

### 5. Headers Configuration

Add custom HTTP headers for authentication:

```rust
let config = GraphiQLConfig::new()
    .add_header("Authorization", "Bearer your-token")
    .add_header("X-API-Version", "v1");
```

### 6. Subscription Support

Real-time subscriptions via WebSocket:

```rust
let config = GraphiQLConfig::new()
    .endpoint_url("/graphql")
    .subscription_url("ws://localhost:3000/graphql");
```

Test subscriptions:
```graphql
subscription {
    userCreated {
        id
        name
    }
}
```

### 7. Theming

Light or dark theme:

```rust
use allframe_core::router::GraphiQLTheme;

let config = GraphiQLConfig::new()
    .theme(GraphiQLTheme::Dark);  // or GraphiQLTheme::Light
```

### 8. Custom Styling

Inject custom CSS:

```rust
let config = GraphiQLConfig::new()
    .custom_css(r#"
        body {
            font-family: 'Inter', sans-serif;
        }
        .graphiql-container {
            --color-primary: 60, 76, 231;
        }
    "#);
```

---

## Configuration

### GraphiQLConfig

Complete configuration reference:

```rust
use allframe_core::router::{GraphiQLConfig, GraphiQLTheme};

let config = GraphiQLConfig::new()
    // Required
    .endpoint_url("/graphql")              // GraphQL endpoint URL

    // Optional - Subscriptions
    .subscription_url("ws://localhost:3000/graphql")  // WebSocket URL

    // Optional - UI
    .theme(GraphiQLTheme::Dark)            // Light or Dark
    .enable_explorer(true)                 // Schema explorer sidebar
    .enable_history(true)                  // Query history persistence

    // Optional - Headers
    .add_header("Authorization", "Bearer token")
    .add_header("X-Custom-Header", "value")

    // Optional - CDN & Styling
    .cdn_url("https://unpkg.com/graphiql@3.0.0/graphiql.min.css")
    .custom_css("/* your custom CSS */");
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `endpoint_url` | `String` | `/graphql` | GraphQL HTTP endpoint |
| `subscription_url` | `Option<String>` | `None` | WebSocket URL for subscriptions |
| `theme` | `GraphiQLTheme` | `Dark` | UI theme (Light/Dark) |
| `enable_explorer` | `bool` | `true` | Show schema explorer |
| `enable_history` | `bool` | `true` | Enable query history |
| `headers` | `HashMap` | `{}` | Custom HTTP headers |
| `cdn_url` | `String` | unpkg.com | GraphiQL CDN URL |
| `custom_css` | `Option<String>` | `None` | Custom CSS styles |

---

## Framework Integration

### Axum + async-graphql

Complete working example:

```rust
use axum::{
    routing::{get, post},
    Router,
    response::Html,
    extract::Extension,
};
use async_graphql::{
    Schema, Object, SimpleObject, EmptyMutation, EmptySubscription,
    http::GraphiQLSource,
};
use async_graphql_axum::GraphQL;
use allframe_core::router::{GraphiQLConfig, graphiql_html};

// 1. Define your schema
struct Query;

#[Object]
impl Query {
    async fn hello(&self) -> &str {
        "Hello, GraphQL!"
    }

    async fn users(&self) -> Vec<User> {
        vec![
            User { id: 1, name: "Alice".to_string(), email: "alice@example.com".to_string() },
            User { id: 2, name: "Bob".to_string(), email: "bob@example.com".to_string() },
        ]
    }

    async fn user(&self, id: i32) -> Option<User> {
        // Fetch user from database
        Some(User { id, name: "User".to_string(), email: "user@example.com".to_string() })
    }
}

#[derive(SimpleObject)]
struct User {
    id: i32,
    name: String,
    email: String,
}

// 2. Build schema
type MySchema = Schema<Query, EmptyMutation, EmptySubscription>;

#[tokio::main]
async fn main() {
    // Create schema
    let schema = MySchema::new(Query, EmptyMutation, EmptySubscription);

    // Configure GraphiQL
    let graphiql_config = GraphiQLConfig::new()
        .endpoint_url("/graphql")
        .theme(allframe_core::router::GraphiQLTheme::Dark)
        .enable_explorer(true);

    let graphiql_html = graphiql_html(&graphiql_config, "My API");

    // Build app
    let app = Router::new()
        .route("/graphql", post(graphql_handler))
        .route("/graphql/playground", get(|| async move {
            Html(graphiql_html)
        }))
        .layer(Extension(schema));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("🚀 GraphQL endpoint: http://localhost:3000/graphql");
    println!("📚 GraphiQL playground: http://localhost:3000/graphql/playground");

    axum::serve(listener, app).await.unwrap();
}

async fn graphql_handler(
    Extension(schema): Extension<MySchema>,
    req: async_graphql_axum::GraphQLRequest,
) -> async_graphql_axum::GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
```

### Actix-web + async-graphql

```rust
use actix_web::{web, App, HttpServer, HttpResponse, Result};
use async_graphql::{Schema, EmptyMutation, EmptySubscription};
use async_graphql_actix_web::GraphQL;
use allframe_core::router::{GraphiQLConfig, graphiql_html};

// ... (same Query and User from above)

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    let schema = Schema::new(Query, EmptyMutation, EmptySubscription);

    let graphiql_config = GraphiQLConfig::new()
        .endpoint_url("/graphql");

    let graphiql_html = graphiql_html(&graphiql_config, "My API");

    HttpServer::new(move || {
        App::new()
            .app_data(web::Data::new(schema.clone()))
            .route("/graphql", web::post().to(graphql_handler))
            .route("/graphql/playground", web::get().to(move || async move {
                HttpResponse::Ok()
                    .content_type("text/html")
                    .body(graphiql_html.clone())
            }))
    })
    .bind("127.0.0.1:3000")?
    .run()
    .await
}

async fn graphql_handler(
    schema: web::Data<MySchema>,
    req: async_graphql_actix_web::GraphQLRequest,
) -> async_graphql_actix_web::GraphQLResponse {
    schema.execute(req.into_inner()).await.into()
}
```

### Rocket + juniper

```rust
use rocket::{State, response::content::RawHtml};
use juniper::{EmptyMutation, EmptySubscription, RootNode};
use allframe_core::router::{GraphiQLConfig, graphiql_html};

// ... (define your schema with juniper)

#[rocket::get("/graphql/playground")]
fn playground() -> RawHtml<String> {
    let config = GraphiQLConfig::new()
        .endpoint_url("/graphql");

    RawHtml(graphiql_html(&config, "My API"))
}

#[rocket::launch]
fn rocket() -> _ {
    rocket::build()
        .mount("/", routes![playground, graphql_post])
}
```

---

## Advanced Usage

### CDN Version Pinning

Pin GraphiQL to a specific version for production stability:

```rust
let config = GraphiQLConfig::new()
    .cdn_url("https://unpkg.com/graphiql@3.0.10/graphiql.min.css");
```

**Recommended versions:**
- Latest stable: `3.0.0` (current)
- Previous stable: `2.4.7`

**CDN providers:**
- unpkg.com (default) - Fast, reliable
- jsdelivr.net - Alternative with better China access
- cdnjs.com - Cloudflare-backed

### Multiple Environments

Different configs for dev/staging/prod:

```rust
fn get_graphiql_config(env: &str) -> GraphiQLConfig {
    let base = GraphiQLConfig::new()
        .enable_explorer(true)
        .enable_history(true);

    match env {
        "development" => base
            .endpoint_url("http://localhost:3000/graphql")
            .subscription_url("ws://localhost:3000/graphql")
            .theme(GraphiQLTheme::Dark),

        "staging" => base
            .endpoint_url("https://staging-api.example.com/graphql")
            .subscription_url("wss://staging-api.example.com/graphql")
            .add_header("X-Environment", "staging"),

        "production" => base
            .endpoint_url("https://api.example.com/graphql")
            .subscription_url("wss://api.example.com/graphql")
            .enable_history(false)  // Disable in prod for privacy
            .add_header("X-Environment", "production"),

        _ => base
    }
}
```

### Authentication

Add authentication headers:

```rust
// Bearer token
let config = GraphiQLConfig::new()
    .add_header("Authorization", "Bearer eyJhbGciOiJIUzI1NiIs...");

// API key
let config = GraphiQLConfig::new()
    .add_header("X-API-Key", "your-api-key-here");

// Multiple auth headers
let config = GraphiQLConfig::new()
    .add_header("Authorization", "Bearer token")
    .add_header("X-Tenant-ID", "tenant-123");
```

**Note**: Headers are visible in the browser. For sensitive tokens, use environment-specific configs.

### Custom Branding

Customize the appearance:

```rust
let config = GraphiQLConfig::new()
    .custom_css(r#"
        /* Custom colors */
        :root {
            --color-primary: 60, 76, 231;
            --color-secondary: 139, 92, 246;
        }

        /* Custom font */
        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
        }

        /* Custom logo */
        .graphiql-logo::before {
            content: 'My Company API';
            font-weight: bold;
            color: var(--color-primary);
        }

        /* Hide GraphiQL branding */
        .graphiql-logo img {
            display: none;
        }
    "#);
```

---

## Subscriptions

### WebSocket Setup

Enable real-time subscriptions:

```rust
// 1. Configure GraphiQL with subscription URL
let config = GraphiQLConfig::new()
    .endpoint_url("/graphql")
    .subscription_url("ws://localhost:3000/graphql");  // WebSocket

// 2. Implement WebSocket handler (async-graphql example)
use async_graphql::http::WebSocketProtocols;
use async_graphql_axum::GraphQLSubscription;

let app = Router::new()
    .route("/graphql", post(graphql_handler))
    .route("/graphql", get(graphql_subscription))  // WebSocket endpoint
    .route("/graphql/playground", get(playground));

async fn graphql_subscription(
    Extension(schema): Extension<MySchema>,
    protocol: GraphQLProtocols,
    websocket: WebSocketUpgrade,
) -> Response {
    websocket
        .protocols(["graphql-transport-ws", "graphql-ws"])
        .on_upgrade(move |stream| {
            GraphQLSubscription::new(stream, schema, protocol)
                .serve()
        })
}
```

### Testing Subscriptions

Example subscription:

```graphql
subscription OnUserCreated {
    userCreated {
        id
        name
        email
        createdAt
    }
}
```

The GraphiQL playground will:
1. Establish WebSocket connection
2. Send subscription
3. Display real-time updates as they arrive
4. Show connection status

---

## Troubleshooting

### Common Issues

#### 1. GraphiQL Not Loading

**Symptom**: Blank page or "Loading GraphiQL..." never completes

**Solutions**:
```rust
// Check CDN accessibility
let config = GraphiQLConfig::new()
    .cdn_url("https://unpkg.com/graphiql@3.0.0/graphiql.min.css");

// Try alternative CDN
let config = GraphiQLConfig::new()
    .cdn_url("https://cdn.jsdelivr.net/npm/graphiql@3.0.0/graphiql.min.css");
```

#### 2. Subscriptions Not Working

**Symptom**: "Connection failed" or subscriptions don't receive updates

**Check**:
1. WebSocket URL is correct (`ws://` or `wss://`)
2. WebSocket endpoint is implemented
3. CORS headers allow WebSocket upgrade
4. Firewall allows WebSocket connections

```rust
// Correct WebSocket URL format
.subscription_url("ws://localhost:3000/graphql")  // Development
.subscription_url("wss://api.example.com/graphql")  // Production (TLS)
```

#### 3. Authentication Errors

**Symptom**: 401 Unauthorized or 403 Forbidden

**Solution**: Add authentication headers
```rust
let config = GraphiQLConfig::new()
    .add_header("Authorization", "Bearer YOUR_TOKEN");
```

#### 4. CORS Errors

**Symptom**: "CORS policy" errors in browser console

**Solution**: Configure CORS on your GraphQL endpoint
```rust
// Axum example
use tower_http::cors::{CorsLayer, Any};

let app = Router::new()
    .route("/graphql", post(graphql_handler))
    .layer(CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::POST, Method::GET])
        .allow_headers(Any));
```

#### 5. Query History Not Persisting

**Symptom**: History clears on page refresh

**Check**:
1. `enable_history` is `true`
2. Browser localStorage is enabled
3. Not in incognito/private mode

```rust
let config = GraphiQLConfig::new()
    .enable_history(true);  // Ensure this is set
```

---

## Best Practices

### 1. Use Different Configs for Environments

```rust
#[cfg(debug_assertions)]
fn graphiql_config() -> GraphiQLConfig {
    GraphiQLConfig::new()
        .enable_history(true)
        .enable_explorer(true)
}

#[cfg(not(debug_assertions))]
fn graphiql_config() -> GraphiQLConfig {
    GraphiQLConfig::new()
        .enable_history(false)  // Privacy in production
        .enable_explorer(true)
}
```

### 2. Pin CDN Versions in Production

```rust
// Development: Use latest
.cdn_url("https://unpkg.com/graphiql@3/graphiql.min.css")

// Production: Pin version
.cdn_url("https://unpkg.com/graphiql@3.0.10/graphiql.min.css")
```

### 3. Disable in Production (Optional)

```rust
#[cfg(debug_assertions)]
fn playground_route() -> Router {
    Router::new()
        .route("/graphql/playground", get(playground_handler))
}

#[cfg(not(debug_assertions))]
fn playground_route() -> Router {
    Router::new()  // No playground in production
}
```

### 4. Use HTTPS/WSS in Production

```rust
let config = if cfg!(debug_assertions) {
    GraphiQLConfig::new()
        .endpoint_url("http://localhost:3000/graphql")
        .subscription_url("ws://localhost:3000/graphql")
} else {
    GraphiQLConfig::new()
        .endpoint_url("https://api.example.com/graphql")
        .subscription_url("wss://api.example.com/graphql")
};
```

### 5. Add Helpful Defaults

```rust
let config = GraphiQLConfig::new()
    .endpoint_url("/graphql")
    .enable_explorer(true)
    .enable_history(true)
    .theme(GraphiQLTheme::Dark);  // Most developers prefer dark mode
```

### 6. Document Your Schema

Good schema documentation shows up in GraphiQL:

```rust
#[Object]
impl Query {
    /// Get a user by ID
    ///
    /// Returns None if user doesn't exist
    async fn user(&self, #[graphql(desc = "User ID")] id: i32) -> Option<User> {
        // ...
    }
}

#[derive(SimpleObject)]
struct User {
    /// Unique user identifier
    id: i32,

    /// User's display name
    name: String,

    /// User's email address
    #[graphql(desc = "Email (unique)")]
    email: String,
}
```

### 7. Security Considerations

- **Don't expose in public prod** unless necessary
- **Use authentication** for sensitive APIs
- **Disable history** in production for privacy
- **Rate limit** the playground endpoint
- **Use HTTPS/WSS** in production

---

## API Reference

### GraphiQLConfig

```rust
pub struct GraphiQLConfig {
    pub endpoint_url: String,
    pub subscription_url: Option<String>,
    pub theme: GraphiQLTheme,
    pub enable_explorer: bool,
    pub enable_history: bool,
    pub headers: HashMap<String, String>,
    pub cdn_url: String,
    pub custom_css: Option<String>,
}
```

#### Methods

##### `new() -> Self`
Create a new configuration with defaults

##### `endpoint_url(self, url: impl Into<String>) -> Self`
Set the GraphQL endpoint URL

##### `subscription_url(self, url: impl Into<String>) -> Self`
Set the WebSocket URL for subscriptions

##### `theme(self, theme: GraphiQLTheme) -> Self`
Set the UI theme (Light or Dark)

##### `enable_explorer(self, enable: bool) -> Self`
Enable/disable schema explorer sidebar

##### `enable_history(self, enable: bool) -> Self`
Enable/disable query history persistence

##### `add_header(self, key: impl Into<String>, value: impl Into<String>) -> Self`
Add a custom HTTP header

##### `cdn_url(self, url: impl Into<String>) -> Self`
Set CDN URL for version pinning

##### `custom_css(self, css: impl Into<String>) -> Self`
Add custom CSS styling

### GraphiQLTheme

```rust
pub enum GraphiQLTheme {
    Light,
    Dark,
}
```

### graphiql_html()

```rust
pub fn graphiql_html(config: &GraphiQLConfig, title: &str) -> String
```

Generate GraphiQL playground HTML.

**Parameters:**
- `config`: GraphiQL configuration
- `title`: Page title

**Returns:** Complete HTML string ready to serve

---

## Examples

See [`examples/graphql_docs.rs`](../../crates/allframe-core/examples/graphql_docs.rs) for a complete working example.

Run with:
```bash
cargo run --example graphql_docs --features "router,openapi"
```

---

## Resources

- [GraphiQL Official Docs](https://graphiql-test.netlify.app/)
- [async-graphql Documentation](https://async-graphql.github.io/async-graphql/)
- [GraphQL Specification](https://spec.graphql.org/)
- [GraphQL Best Practices](https://graphql.org/learn/best-practices/)

---

**AllFrame. One frame. Infinite transformations.**
*Beautiful GraphQL documentation. Powerful by architecture.* 🦀
