//! MCP Server with STDIO Transport
//!
//! This example demonstrates a complete MCP server implementation
//! using stdio transport, suitable for use with Claude Desktop.
//!
//! # Usage
//!
//! ```bash
//! cargo run --example mcp_stdio_server
//! ```
//!
//! # Claude Desktop Configuration
//!
//! Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "allframe-example": {
//!       "command": "/path/to/target/debug/examples/mcp_stdio_server",
//!       "args": []
//!     }
//!   }
//! }
//! ```

use std::io::{stdin, stdout, BufRead, Write};

use allframe_core::router::Router;
use allframe_mcp::McpServer;
use serde_json::{json, Value};

#[tokio::main]
async fn main() {
    // Initialize logging (optional)
    env_logger::init();

    // Create AllFrame router with example handlers
    let router = create_router();

    // Create MCP server
    let mcp = McpServer::new(router);

    eprintln!("MCP Server started with {} tools", mcp.tool_count());
    eprintln!("Listening on stdio...");

    // Run stdio transport
    serve_stdio(mcp).await;
}

/// Create router with example handlers
fn create_router() -> Router {
    let mut router = Router::new();

    // Example 1: Get user information
    router.register("get_user", |user_id: String| async move {
        format!(
            "{{\"id\": \"{}\", \"name\": \"User {}\", \"email\": \"user{}@example.com\"}}",
            user_id, user_id, user_id
        )
    });

    // Example 2: Create an order
    router.register("create_order", |product: String| async move {
        let order_id = uuid::Uuid::new_v4();
        format!(
            "{{\"order_id\": \"{}\", \"product\": \"{}\", \"status\": \"created\"}}",
            order_id, product
        )
    });

    // Example 3: Search products
    router.register("search_products", |query: String| async move {
        format!(
            "{{\"query\": \"{}\", \"results\": [{{\"id\": \"1\", \"name\": \"Product A\"}}, \
             {{\"id\": \"2\", \"name\": \"Product B\"}}]}}",
            query
        )
    });

    // Example 4: Calculate shipping
    router.register("calculate_shipping", |weight: String| async move {
        let w: f64 = weight.parse().unwrap_or(0.0);
        let cost = w * 2.5 + 5.0;
        format!("{{\"weight\": {}, \"cost\": {:.2}}}", w, cost)
    });

    router
}

/// Serve MCP protocol over stdio
async fn serve_stdio(mcp: McpServer) {
    let stdin = stdin();
    let mut stdout = stdout();

    for line in stdin.lock().lines() {
        let line = match line {
            Ok(l) => l,
            Err(e) => {
                eprintln!("Error reading line: {}", e);
                continue;
            }
        };

        // Skip empty lines
        if line.trim().is_empty() {
            continue;
        }

        // Parse request
        let request: Value = match serde_json::from_str(&line) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("Error parsing JSON: {}", e);
                let error = json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32700,
                        "message": "Parse error"
                    },
                    "id": null
                });
                writeln!(stdout, "{}", serde_json::to_string(&error).unwrap()).unwrap();
                stdout.flush().unwrap();
                continue;
            }
        };

        // Handle request
        let response = handle_request(&mcp, request).await;

        // Write response
        match serde_json::to_string(&response) {
            Ok(json_str) => {
                writeln!(stdout, "{}", json_str).unwrap();
                stdout.flush().unwrap();
            }
            Err(e) => {
                eprintln!("Error serializing response: {}", e);
            }
        }
    }
}

/// Handle MCP request
async fn handle_request(mcp: &McpServer, request: Value) -> Value {
    let method = request["method"].as_str().unwrap_or("");
    let id = request.get("id").cloned();

    let result = match method {
        // Initialize
        "initialize" => {
            json!({
                "protocolVersion": "0.1.0",
                "capabilities": {
                    "tools": {}
                },
                "serverInfo": {
                    "name": "allframe-mcp",
                    "version": "0.1.0"
                }
            })
        }

        // List available tools
        "tools/list" => {
            let tools = mcp.list_tools().await;
            json!({
                "tools": tools.iter().map(|t| {
                    json!({
                        "name": t.name,
                        "description": t.description,
                        "inputSchema": t.input_schema
                    })
                }).collect::<Vec<_>>()
            })
        }

        // Call a tool
        "tools/call" => {
            let params = &request["params"];
            let name = params["name"].as_str().unwrap_or("");
            let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

            match mcp.call_tool(name, arguments).await {
                Ok(result) => {
                    json!({
                        "content": [{
                            "type": "text",
                            "text": result.to_string()
                        }]
                    })
                }
                Err(e) => {
                    json!({
                        "isError": true,
                        "content": [{
                            "type": "text",
                            "text": format!("Error: {}", e)
                        }]
                    })
                }
            }
        }

        // Ping
        "ping" => {
            json!({})
        }

        // Unknown method
        _ => {
            return json!({
                "jsonrpc": "2.0",
                "error": {
                    "code": -32601,
                    "message": format!("Method not found: {}", method)
                },
                "id": id
            });
        }
    };

    // Return successful response
    json!({
        "jsonrpc": "2.0",
        "result": result,
        "id": id
    })
}

// Note: You'll need to add uuid to Cargo.toml for this example
// [dev-dependencies]
// uuid = { version = "1.0", features = ["v4"] }
// env_logger = "0.11"
