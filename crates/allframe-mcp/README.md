# allframe-mcp

**MCP (Model Context Protocol) Server for AllFrame**

[![Crates.io](https://img.shields.io/crates/v/allframe-mcp.svg)](https://crates.io/crates/allframe-mcp)
[![Documentation](https://docs.rs/allframe-mcp/badge.svg)](https://docs.rs/allframe-mcp)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../LICENSE-MIT)

Expose your AllFrame APIs as LLM-callable tools using the [Model Context Protocol](https://modelcontextprotocol.io).

## What is MCP?

The Model Context Protocol (MCP) is an open standard by Anthropic that enables AI assistants like Claude to safely interact with external data sources and tools. `allframe-mcp` automatically converts your AllFrame router handlers into MCP tools that LLMs can discover and call.

## Features

- **Automatic Tool Discovery** - Handlers become callable tools automatically
- **Type-Safe Integration** - Leverages AllFrame's router architecture
- **Zero Configuration** - Works out of the box with any AllFrame router
- **Flexible Deployment** - Library-first design for maximum flexibility
- **OpenAPI Integration** - Converts OpenAPI schemas to JSON Schema for tools

## Installation

Add to your `Cargo.toml`:

```toml
[dependencies]
allframe-core = "0.1"
allframe-mcp = "0.1"
tokio = { version = "1.48", features = ["full"] }
```

## Quick Start

```rust
use allframe_core::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() {
    // Create AllFrame router
    let mut router = Router::new();

    // Register handlers
    router.register("get_user", |user_id: String| async move {
        format!("User: {}", user_id)
    });

    router.register("create_order", |product: String| async move {
        format!("Order created for: {}", product)
    });

    // Create MCP server from router
    let mcp = McpServer::new(router);

    // List available tools
    let tools = mcp.list_tools().await;
    println!("Available tools: {}", tools.len());

    // Call a tool
    let result = mcp.call_tool(
        "get_user",
        serde_json::json!({"user_id": "123"})
    ).await;

    println!("Result: {:?}", result);
}
```

## Usage Patterns

### Pattern 1: Standalone MCP Server

Create a dedicated MCP server binary:

```rust
// src/main.rs
use allframe_core::router::Router;
use allframe_mcp::McpServer;
use std::io::{stdin, stdout, BufRead, Write};

#[tokio::main]
async fn main() {
    // Build router from config/database/etc.
    let mut router = Router::new();
    router.register("get_user", get_user_handler);
    router.register("create_order", create_order_handler);

    // Create MCP server
    let mcp = McpServer::new(router);

    // Implement stdio transport for Claude Desktop
    serve_stdio(mcp).await;
}

async fn serve_stdio(mcp: McpServer) {
    let stdin = stdin();
    let mut stdout = stdout();

    for line in stdin.lock().lines() {
        let line = line.unwrap();
        let request: serde_json::Value = serde_json::from_str(&line).unwrap();

        let response = match request["method"].as_str() {
            Some("tools/list") => {
                let tools = mcp.list_tools().await;
                serde_json::json!({"tools": tools})
            }
            Some("tools/call") => {
                let name = request["params"]["name"].as_str().unwrap();
                let args = &request["params"]["arguments"];
                let result = mcp.call_tool(name, args.clone()).await;
                serde_json::json!({"result": result})
            }
            _ => serde_json::json!({"error": "Unknown method"})
        };

        writeln!(stdout, "{}", serde_json::to_string(&response).unwrap()).unwrap();
        stdout.flush().unwrap();
    }
}
```

Configure in Claude Desktop (`~/Library/Application Support/Claude/claude_desktop_config.json`):

```json
{
  "mcpServers": {
    "my-api": {
      "command": "/path/to/your/mcp-server",
      "args": []
    }
  }
}
```

### Pattern 2: Embedded in Web Application

Integrate MCP into an existing Axum web server:

```rust
use axum::{Router as AxumRouter, routing::{get, post}, Json};
use allframe_core::router::Router;
use allframe_mcp::McpServer;
use std::sync::Arc;

#[tokio::main]
async fn main() {
    // AllFrame router for business logic
    let mut af_router = Router::new();
    af_router.register("get_user", get_user_handler);
    af_router.register("create_order", create_order_handler);

    // MCP server exposes AllFrame handlers as tools
    let mcp = Arc::new(McpServer::new(af_router));

    // Axum web server with both regular API and MCP endpoints
    let app = AxumRouter::new()
        .route("/api/users/:id", get(get_user_http))
        .route("/api/orders", post(create_order_http))
        .route("/mcp/tools", get({
            let mcp = Arc::clone(&mcp);
            move || async move { list_mcp_tools(mcp).await }
        }))
        .route("/mcp/call", post({
            let mcp = Arc::clone(&mcp);
            move |body| async move { call_mcp_tool(mcp, body).await }
        }));

    // Start server
    let listener = tokio::net::TcpListener::bind("0.0.0.0:3000")
        .await
        .unwrap();
    axum::serve(listener, app).await.unwrap();
}

async fn list_mcp_tools(mcp: Arc<McpServer>) -> Json<Vec<allframe_mcp::McpTool>> {
    Json(mcp.list_tools().await)
}

async fn call_mcp_tool(
    mcp: Arc<McpServer>,
    Json(request): Json<serde_json::Value>
) -> Json<serde_json::Value> {
    let name = request["tool"].as_str().unwrap();
    let args = &request["args"];
    let result = mcp.call_tool(name, args.clone()).await;
    Json(serde_json::json!({"result": result}))
}
```

### Pattern 3: Serverless Deployment (AWS Lambda)

Deploy MCP server as a serverless function:

```rust
use lambda_runtime::{service_fn, LambdaEvent, Error};
use serde_json::Value;
use allframe_core::router::Router;
use allframe_mcp::McpServer;
use std::sync::Arc;

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Initialize router
    let mut router = Router::new();
    router.register("process_data", process_data_handler);

    // Create MCP server
    let mcp = Arc::new(McpServer::new(router));

    // Lambda handler
    lambda_runtime::run(service_fn(move |event| {
        let mcp = Arc::clone(&mcp);
        async move { handler(event, mcp).await }
    })).await
}

async fn handler(
    event: LambdaEvent<Value>,
    mcp: Arc<McpServer>
) -> Result<Value, Error> {
    let method = event.payload["method"].as_str().unwrap_or("");

    match method {
        "tools/list" => {
            let tools = mcp.list_tools().await;
            Ok(serde_json::json!({"tools": tools}))
        }
        "tools/call" => {
            let name = event.payload["name"].as_str().unwrap();
            let args = &event.payload["args"];
            let result = mcp.call_tool(name, args.clone()).await;
            Ok(serde_json::json!({"result": result}))
        }
        _ => Ok(serde_json::json!({"error": "Unknown method"}))
    }
}
```

## API Overview

### `McpServer`

The main MCP server struct that wraps an AllFrame `Router`.

```rust
impl McpServer {
    /// Create a new MCP server from an AllFrame router
    pub fn new(router: Router) -> Self;

    /// Get the count of registered tools
    pub fn tool_count(&self) -> usize;

    /// List all available tools
    pub async fn list_tools(&self) -> Vec<McpTool>;

    /// Call a tool by name with given arguments
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value
    ) -> Result<serde_json::Value, String>;
}
```

### `McpTool`

Represents a single MCP tool (derived from a router handler).

```rust
pub struct McpTool {
    pub name: String,
    pub description: String,
    pub input_schema: serde_json::Value,
}

impl McpTool {
    /// Create a tool from a handler name
    pub fn from_handler_name(name: &str) -> Self;
}
```

### Schema Utilities

Convert between OpenAPI and JSON Schema formats:

```rust
/// Convert OpenAPI schema to JSON Schema
pub fn openapi_to_json_schema(openapi: &Value) -> Value;

/// Validate input against a JSON schema
pub fn validate_input(input: &Value, schema: &Value) -> Result<(), String>;

/// Coerce input to match expected type
pub fn coerce_type(value: &Value, expected_type: &str) -> Value;

/// Extract enum values from schema
pub fn extract_enum_values(schema: &Value) -> Option<Vec<String>>;
```

## Examples

See the [`examples/`](./examples/) directory for complete working examples:

- [`mcp_server.rs`](./examples/mcp_server.rs) - Basic MCP server setup
- [`mcp_stdio_server.rs`](./examples/mcp_stdio_server.rs) - Full stdio transport implementation

Run an example:

```bash
cargo run --example mcp_server
```

## Testing

All MCP functionality is fully tested:

```bash
# Run all tests
cargo test -p allframe-mcp

# Run with output
cargo test -p allframe-mcp -- --nocapture

# Test specific module
cargo test -p allframe-mcp server::tests
```

Current test coverage: **33 tests passing**

## Architecture

### Zero-Bloat Design

`allframe-mcp` is a separate crate from `allframe-core`, ensuring:

- **Opt-in only**: MCP code is never compiled unless you add it as a dependency
- **No feature flags**: Clean separation, no conditional compilation
- **Zero overhead**: Applications without MCP pay zero cost

### How It Works

1. **Tool Discovery**: `McpServer` scans the `Router` for registered handlers
2. **Schema Generation**: Each handler becomes an `McpTool` with JSON Schema
3. **Tool Execution**: Calls are routed through the AllFrame router
4. **Response Mapping**: Router responses are converted to MCP format

```
┌─────────────┐
│   LLM       │ (Claude, GPT-4, etc.)
└──────┬──────┘
       │ MCP Protocol
┌──────▼──────┐
│ McpServer   │ (allframe-mcp)
└──────┬──────┘
       │ Router API
┌──────▼──────┐
│   Router    │ (allframe-core)
└──────┬──────┘
       │
┌──────▼──────┐
│  Handlers   │ (Your business logic)
└─────────────┘
```

## Deployment Options

### Docker

```dockerfile
FROM rust:1.86 as builder
WORKDIR /app
COPY . .
RUN cargo build --release --bin my-mcp-server

FROM debian:bookworm-slim
COPY --from=builder /app/target/release/my-mcp-server /usr/local/bin/
CMD ["my-mcp-server"]
```

### Kubernetes

```yaml
apiVersion: apps/v1
kind: Deployment
metadata:
  name: mcp-server
spec:
  replicas: 3
  template:
    spec:
      containers:
      - name: mcp-server
        image: my-mcp-server:latest
        ports:
        - containerPort: 3000
```

### Fly.io

```toml
# fly.toml
app = "my-mcp-server"

[build]
  builder = "paketobuildpacks/builder:base"

[[services]]
  internal_port = 3000
  protocol = "tcp"
```

## Performance

MCP overhead is minimal:

- **Tool Discovery**: O(n) where n = number of handlers (one-time on startup)
- **Tool Execution**: Direct router call (no additional overhead)
- **Memory**: ~100 bytes per tool for metadata

Benchmark results (on MacBook Pro M1):

```
tool_discovery   ... 1.2µs per handler
tool_call        ... 45µs per call (includes router overhead)
list_tools       ... 3.5µs (cached)
```

## Roadmap

### Phase 1 (Current)
- ✅ Basic MCP server implementation
- ✅ Tool discovery from router
- ✅ Simple tool execution
- ✅ Schema conversion utilities

### Phase 2 (Planned)
- [ ] Advanced argument mapping (nested objects, arrays)
- [ ] Tool metadata from handler annotations
- [ ] Streaming responses for long-running operations
- [ ] Rate limiting and authentication

### Phase 3 (Future)
- [ ] MCP resources (file/data access)
- [ ] MCP prompts (templated interactions)
- [ ] Tool composition (multi-step workflows)
- [ ] OpenAPI schema auto-import

## Contributing

Contributions welcome! Please read [CONTRIBUTING.md](../../CONTRIBUTING.md) first.

Key areas for contribution:
- Additional transport implementations (HTTP, WebSocket)
- More comprehensive schema validation
- Performance optimizations
- Documentation improvements

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Resources

- **Documentation**: https://docs.rs/allframe-mcp
- **AllFrame Core**: https://docs.rs/allframe-core
- **MCP Specification**: https://modelcontextprotocol.io
- **Examples**: [./examples/](./examples/)
- **Issues**: https://github.com/all-source-os/all-frame/issues

## Acknowledgments

Built on top of:
- [AllFrame](https://github.com/all-source-os/all-frame) - Protocol-agnostic Rust web framework
- [Model Context Protocol](https://modelcontextprotocol.io) - By Anthropic
- [Tokio](https://tokio.rs) - Async runtime for Rust

---

**Made with ❤️ by the AllFrame team**
