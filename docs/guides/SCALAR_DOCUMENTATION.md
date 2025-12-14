# Scalar API Documentation Guide

**Status**: Production Ready
**Features**: OpenAPI 3.1, Interactive "Try It", CDN Configuration, CORS Handling
**Last Updated**: 2025-12-01

---

## Table of Contents

1. [Overview](#overview)
2. [Quick Start](#quick-start)
3. [Features](#features)
4. [Configuration](#configuration)
5. [OpenAPI Server Configuration](#openapi-server-configuration)
6. [Framework Integration](#framework-integration)
7. [Advanced Usage](#advanced-usage)
8. [Troubleshooting](#troubleshooting)
9. [Best Practices](#best-practices)
10. [API Reference](#api-reference)

---

## Overview

AllFrame's Scalar integration provides beautiful, interactive API documentation powered by [Scalar](https://scalar.com). Scalar offers a modern alternative to Swagger UI with enhanced features like a lightweight bundle (<50KB), dark mode by default, and powerful "Try It" functionality.

### Key Benefits

- **Beautiful UI**: Modern, responsive design with dark mode
- **Interactive Testing**: "Try It" button for every endpoint
- **Lightweight**: <50KB JavaScript bundle
- **Type-Safe**: Generated from your OpenAPI 3.1 specification
- **Zero Configuration**: Works out of the box with sensible defaults
- **Highly Customizable**: Themes, CSS, CDN, proxy configuration

---

## Quick Start

### 1. Enable Features

Add to your `Cargo.toml`:

```toml
[dependencies]
allframe = { version = "0.1", features = ["router", "openapi"] }
```

### 2. Create Router and Generate Docs

```rust
use allframe_core::router::{Router, OpenApiGenerator};

let mut router = Router::new();

// Register your routes
router.get("/users", || async {
    r#"[{"id":1,"name":"Alice"}]"#.to_string()
});

router.post("/users", || async {
    r#"{"id":2,"name":"Bob"}"#.to_string()
});

// Generate OpenAPI spec
let spec = OpenApiGenerator::new("My API", "1.0.0")
    .with_description("A sample API")
    .with_server("http://localhost:3000", Some("Development"))
    .generate(&router);

let openapi_json = serde_json::to_string_pretty(&spec).unwrap();
```

### 3. Generate Scalar HTML

```rust
use allframe_core::router::{ScalarConfig, scalar_html};

let config = ScalarConfig::new();
let html = scalar_html(&config, "My API", &openapi_json);
```

### 4. Serve the Documentation

See [Framework Integration](#framework-integration) for examples with Axum, Actix, and Rocket.

---

## Features

### Core Features

✅ **OpenAPI 3.1 Generation**
- Automatic spec generation from routes
- Request/response schemas
- Path parameters
- Multiple HTTP methods

✅ **Interactive "Try It" Functionality**
- Test endpoints directly in the browser
- Server selection dropdown
- Request/response preview
- CORS proxy support

✅ **CDN Configuration**
- Version pinning for stability
- SRI (Subresource Integrity) hashes
- Fallback CDN support
- Custom CDN URLs

✅ **UI Customization**
- Dark/Light/Auto themes
- Custom CSS injection
- Modern or Classic layouts
- Sidebar navigation

✅ **Security**
- SRI hash verification
- CORS proxy configuration
- Secure CDN delivery

---

## Configuration

### Basic Configuration

```rust
use allframe_core::router::{ScalarConfig, ScalarTheme, ScalarLayout};

let config = ScalarConfig::new()
    .theme(ScalarTheme::Dark)
    .layout(ScalarLayout::Modern)
    .show_sidebar(true);
```

### Full Configuration Example

```rust
let config = ScalarConfig::new()
    // UI Customization
    .theme(ScalarTheme::Auto)
    .layout(ScalarLayout::Modern)
    .show_sidebar(true)
    .hide_download_button(false)
    .hide_models(false)

    // CDN Configuration
    .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
    .sri_hash("sha384-...")
    .fallback_cdn_url("https://unpkg.com/@scalar/api-reference")

    // Try It Functionality
    .proxy_url("https://proxy.scalar.com")

    // Custom Styling
    .custom_css(r#"
        body {
            font-family: 'Inter', sans-serif;
        }
        .api-reference {
            --scalar-color-accent: #4f46e5;
        }
    "#);
```

### Configuration Options

| Option | Type | Default | Description |
|--------|------|---------|-------------|
| `spec_url` | `String` | `/docs/openapi.json` | URL to OpenAPI spec |
| `theme` | `ScalarTheme` | `Dark` | UI theme (Dark, Light, Auto) |
| `layout` | `ScalarLayout` | `Modern` | Layout style (Modern, Classic) |
| `show_sidebar` | `bool` | `true` | Show sidebar navigation |
| `hide_download_button` | `bool` | `false` | Hide OpenAPI download button |
| `hide_models` | `bool` | `false` | Hide models/schemas section |
| `cdn_url` | `String` | jsdelivr latest | CDN URL for Scalar JS |
| `sri_hash` | `Option<String>` | `None` | SRI hash for CDN integrity |
| `fallback_cdn_url` | `Option<String>` | `None` | Fallback CDN if primary fails |
| `proxy_url` | `Option<String>` | `None` | Proxy for CORS handling |
| `custom_css` | `Option<String>` | `None` | Custom CSS to inject |

---

## OpenAPI Server Configuration

Servers are required for the "Try It" functionality to know where to send requests.

### Single Server

```rust
let spec = OpenApiGenerator::new("API", "1.0.0")
    .with_server("http://localhost:3000", Some("Development"))
    .generate(&router);
```

### Multiple Servers

```rust
let spec = OpenApiGenerator::new("API", "1.0.0")
    .with_server("http://localhost:3000", Some("Local Development"))
    .with_server("https://staging.example.com", Some("Staging"))
    .with_server("https://api.example.com", Some("Production"))
    .generate(&router);
```

The "Try It" button will show a dropdown allowing users to select which server to use.

### Using OpenApiServer Struct

```rust
use allframe_core::router::OpenApiServer;

let servers = vec![
    OpenApiServer::new("http://localhost:3000")
        .with_description("Local Development"),
    OpenApiServer::new("https://api.example.com")
        .with_description("Production"),
];

let spec = OpenApiGenerator::new("API", "1.0.0")
    .with_servers(servers)
    .generate(&router);
```

---

## Framework Integration

### Axum

```rust
use axum::{
    routing::get,
    Router,
    response::{Html, Json},
};
use allframe_core::router::{
    Router as AllFrameRouter,
    OpenApiGenerator,
    ScalarConfig,
    scalar_html,
};

#[tokio::main]
async fn main() {
    // Create AllFrame router and generate docs
    let mut af_router = AllFrameRouter::new();
    af_router.get("/users", || async {
        r#"[{"id":1,"name":"Alice"}]"#.to_string()
    });

    let spec = OpenApiGenerator::new("My API", "1.0.0")
        .with_server("http://localhost:3000", Some("Development"))
        .generate(&af_router);

    let openapi_json = serde_json::to_string_pretty(&spec).unwrap();
    let scalar_html_content = scalar_html(
        &ScalarConfig::new(),
        "My API",
        &openapi_json
    );

    // Create Axum app
    let app = Router::new()
        .route("/docs", get(|| async move {
            Html(scalar_html_content)
        }))
        .route("/docs/openapi.json", get(|| async move {
            Json(spec)
        }))
        .route("/users", get(get_users));

    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();

    println!("📚 Documentation: http://localhost:3000/docs");

    axum::serve(listener, app).await.unwrap();
}

async fn get_users() -> String {
    r#"[{"id":1,"name":"Alice"}]"#.to_string()
}
```

### Actix-web

```rust
use actix_web::{web, App, HttpResponse, HttpServer};
use allframe_core::router::{
    Router as AllFrameRouter,
    OpenApiGenerator,
    ScalarConfig,
    scalar_html,
};

#[actix_web::main]
async fn main() -> std::io::Result<()> {
    // Create AllFrame router and generate docs
    let mut af_router = AllFrameRouter::new();
    af_router.get("/users", || async {
        r#"[{"id":1,"name":"Alice"}]"#.to_string()
    });

    let spec = OpenApiGenerator::new("My API", "1.0.0")
        .with_server("http://localhost:3000", Some("Development"))
        .generate(&af_router);

    let openapi_json = serde_json::to_string_pretty(&spec).unwrap();
    let scalar_html_content = scalar_html(
        &ScalarConfig::new(),
        "My API",
        &openapi_json
    );

    HttpServer::new(move || {
        let spec_clone = spec.clone();
        let html_clone = scalar_html_content.clone();

        App::new()
            .route("/docs", web::get().to(move || {
                let html = html_clone.clone();
                async move { HttpResponse::Ok().body(html) }
            }))
            .route("/docs/openapi.json", web::get().to(move || {
                let spec = spec_clone.clone();
                async move { HttpResponse::Ok().json(spec) }
            }))
            .route("/users", web::get().to(get_users))
    })
    .bind(("0.0.0.0", 3000))?
    .run()
    .await
}

async fn get_users() -> HttpResponse {
    HttpResponse::Ok().body(r#"[{"id":1,"name":"Alice"}]"#)
}
```

### Rocket

```rust
use rocket::{get, routes, State};
use rocket::response::content::{RawHtml, RawJson};
use allframe_core::router::{
    Router as AllFrameRouter,
    OpenApiGenerator,
    ScalarConfig,
    scalar_html,
};

#[get("/docs")]
fn serve_docs(html: &State<String>) -> RawHtml<String> {
    RawHtml(html.inner().clone())
}

#[get("/docs/openapi.json")]
fn serve_openapi(spec: &State<String>) -> RawJson<String> {
    RawJson(spec.inner().clone())
}

#[get("/users")]
fn get_users() -> String {
    r#"[{"id":1,"name":"Alice"}]"#.to_string()
}

#[launch]
fn rocket() -> _ {
    // Create AllFrame router and generate docs
    let mut af_router = AllFrameRouter::new();
    af_router.get("/users", || async {
        r#"[{"id":1,"name":"Alice"}]"#.to_string()
    });

    let spec = OpenApiGenerator::new("My API", "1.0.0")
        .with_server("http://localhost:8000", Some("Development"))
        .generate(&af_router);

    let openapi_json = serde_json::to_string_pretty(&spec).unwrap();
    let scalar_html_content = scalar_html(
        &ScalarConfig::new(),
        "My API",
        &openapi_json
    );

    rocket::build()
        .manage(scalar_html_content)
        .manage(openapi_json)
        .mount("/", routes![serve_docs, serve_openapi, get_users])
}
```

---

## Advanced Usage

### CDN Version Pinning

For production stability, pin the Scalar version:

```rust
let config = ScalarConfig::new()
    .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0");
```

### SRI (Subresource Integrity) Hashes

Add integrity verification for CDN resources:

```rust
let config = ScalarConfig::new()
    .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
    .sri_hash("sha384-oqVuAfXRKap7fdgcCY5uykM6+R9GqQ8K/uxy9rx7HNQlGYl1kPzQho1wx4JwY8wC");
```

To generate SRI hashes, use [srihash.org](https://www.srihash.org/) or:

```bash
curl https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0 | \
  openssl dgst -sha384 -binary | \
  openssl base64 -A
```

### CDN Fallback

Provide a fallback CDN if the primary fails:

```rust
let config = ScalarConfig::new()
    .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
    .fallback_cdn_url("https://unpkg.com/@scalar/api-reference@1.25.0");
```

The fallback will automatically load if the primary CDN is unavailable.

### CORS Proxy Configuration

For "Try It" functionality with CORS restrictions:

```rust
let config = ScalarConfig::new()
    .proxy_url("https://proxy.scalar.com");
```

Or use your own proxy:

```rust
let config = ScalarConfig::new()
    .proxy_url("https://your-proxy.example.com");
```

### Custom Theming

```rust
let config = ScalarConfig::new()
    .theme(ScalarTheme::Auto)
    .custom_css(r#"
        :root {
            --scalar-color-accent: #4f46e5;
            --scalar-color-1: #18181b;
            --scalar-color-2: #27272a;
            --scalar-color-3: #3f3f46;
        }

        body {
            font-family: 'Inter', -apple-system, BlinkMacSystemFont, sans-serif;
        }

        .api-reference {
            --scalar-border-radius: 8px;
        }
    "#);
```

---

## Troubleshooting

### "Try It" Button Not Working

**Problem**: Clicking "Try It" doesn't send requests.

**Solutions**:
1. **Add servers to OpenAPI spec**:
   ```rust
   .with_server("http://localhost:3000", Some("Development"))
   ```

2. **Configure CORS proxy**:
   ```rust
   .proxy_url("https://proxy.scalar.com")
   ```

3. **Check browser console** for CORS errors

### CDN Loading Failures

**Problem**: Scalar UI doesn't load.

**Solutions**:
1. **Check CDN URL** is correct and accessible
2. **Add fallback CDN**:
   ```rust
   .fallback_cdn_url("https://unpkg.com/@scalar/api-reference")
   ```
3. **Verify SRI hash** matches the CDN resource
4. **Check browser network tab** for loading errors

### OpenAPI Spec Not Showing

**Problem**: Documentation shows "No spec found".

**Solutions**:
1. **Verify spec_url** matches your endpoint:
   ```rust
   .spec_url("/docs/openapi.json")
   ```

2. **Check OpenAPI JSON** is valid:
   ```rust
   let spec = serde_json::to_string_pretty(&spec).unwrap();
   println!("{}", spec); // Verify it's valid JSON
   ```

3. **Ensure content-type** is `application/json`

### Styling Issues

**Problem**: Custom CSS not applied.

**Solutions**:
1. **Use !important** for overrides:
   ```css
   .api-reference { --scalar-color-accent: #4f46e5 !important; }
   ```

2. **Check browser DevTools** for CSS specificity conflicts

3. **Verify CSS syntax** is correct

---

## Best Practices

### 1. Version Pinning in Production

Always pin CDN versions in production:

```rust
// ✅ Good
.cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")

// ❌ Bad (unpredictable updates)
.cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference")
```

### 2. Add SRI Hashes

Use SRI hashes for security:

```rust
.cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
.sri_hash("sha384-...")
```

### 3. Configure Multiple Servers

Provide server options for different environments:

```rust
.with_server("http://localhost:3000", Some("Local"))
.with_server("https://staging.example.com", Some("Staging"))
.with_server("https://api.example.com", Some("Production"))
```

### 4. Use Proxy for CORS

Configure a proxy for reliable "Try It" functionality:

```rust
.proxy_url("https://proxy.scalar.com")
```

### 5. Add API Descriptions

Enhance documentation with descriptions:

```rust
OpenApiGenerator::new("My API", "1.0.0")
    .with_description("Comprehensive API for user management with CRUD operations")
```

### 6. Organize Documentation Routes

Keep documentation routes consistent:

```
/docs              - Scalar HTML UI
/docs/openapi.json - OpenAPI specification
```

### 7. Cache Documentation

Cache generated HTML and JSON for performance:

```rust
lazy_static! {
    static ref DOCS_HTML: String = /* generate once */;
    static ref OPENAPI_SPEC: serde_json::Value = /* generate once */;
}
```

---

## API Reference

### ScalarConfig

**Constructor:**
```rust
pub fn new() -> Self
```

**Builder Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `spec_url` | `(url: impl Into<String>) -> Self` | Set OpenAPI spec URL |
| `theme` | `(theme: ScalarTheme) -> Self` | Set UI theme |
| `show_sidebar` | `(show: bool) -> Self` | Toggle sidebar |
| `layout` | `(layout: ScalarLayout) -> Self` | Set layout style |
| `custom_css` | `(css: impl Into<String>) -> Self` | Inject custom CSS |
| `hide_download_button` | `(hide: bool) -> Self` | Hide download button |
| `hide_models` | `(hide: bool) -> Self` | Hide models section |
| `cdn_url` | `(url: impl Into<String>) -> Self` | Set CDN URL |
| `sri_hash` | `(hash: impl Into<String>) -> Self` | Set SRI hash |
| `fallback_cdn_url` | `(url: impl Into<String>) -> Self` | Set fallback CDN |
| `proxy_url` | `(url: impl Into<String>) -> Self` | Set CORS proxy |

### OpenApiGenerator

**Constructor:**
```rust
pub fn new(title: impl Into<String>, version: impl Into<String>) -> Self
```

**Builder Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_description` | `(description: impl Into<String>) -> Self` | Set API description |
| `with_server` | `(url: impl Into<String>, description: Option<impl Into<String>>) -> Self` | Add server URL |
| `with_servers` | `(servers: Vec<OpenApiServer>) -> Self` | Set multiple servers |
| `generate` | `(&self, router: &Router) -> Value` | Generate OpenAPI spec |

### OpenApiServer

**Constructor:**
```rust
pub fn new(url: impl Into<String>) -> Self
```

**Builder Methods:**

| Method | Signature | Description |
|--------|-----------|-------------|
| `with_description` | `(description: impl Into<String>) -> Self` | Set server description |

### Functions

**scalar_html:**
```rust
pub fn scalar_html(
    config: &ScalarConfig,
    title: &str,
    openapi_spec_json: &str
) -> String
```

Generates complete HTML page with Scalar UI.

---

## Example

See the complete working example at `examples/scalar_docs.rs`:

```bash
cargo run --example scalar_docs --features "router,openapi"
```

---

## Resources

- [Scalar Documentation](https://guides.scalar.com/)
- [OpenAPI 3.1 Specification](https://spec.openapis.org/oas/v3.1.0)
- [SRI Hash Generator](https://www.srihash.org/)
- [AllFrame Examples](../examples/)

---

**AllFrame. One frame. Infinite transformations.**
*Beautiful documentation. Powerful by design.* 🦀
