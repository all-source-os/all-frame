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
//! # Debug Mode
//!
//! Enable debug logging with environment variables:
//!
//! ```bash
//! # Basic debug output to stderr
//! ALLFRAME_MCP_DEBUG=1 cargo run --example mcp_stdio_server
//!
//! # Full tracing with file output (when built with tracing feature)
//! RUST_LOG=debug ALLFRAME_MCP_LOG_FILE=/tmp/allframe-mcp.log \
//!     cargo run --example mcp_stdio_server --features tracing
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
//!       "args": [],
//!       "env": {
//!         "ALLFRAME_MCP_DEBUG": "1"
//!       }
//!     }
//!   }
//! }
//! ```
//!
//! # Testing Manually
//!
//! ```bash
//! # Test initialize
//! echo '{"jsonrpc":"2.0","method":"initialize","params":{},"id":1}' | \
//!     cargo run --example mcp_stdio_server
//!
//! # Test tools/list
//! echo '{"jsonrpc":"2.0","method":"tools/list","params":{},"id":2}' | \
//!     cargo run --example mcp_stdio_server
//! ```

use allframe_core::router::Router;
use allframe_mcp::{init_tracing, McpServer, StdioConfig, StdioTransport};

#[tokio::main]
async fn main() {
    // Initialize tracing (works with or without the feature)
    init_tracing();

    // Create AllFrame router with example handlers
    let router = create_router();

    // Create MCP server
    let mcp = McpServer::new(router);

    // Configure the stdio transport
    let config = StdioConfig::default()
        .with_debug_tool(true)  // Enable allframe/debug tool
        .with_server_name("allframe-example");

    // Run the server
    StdioTransport::new(mcp, config).serve().await;
}

/// Create router with example handlers
fn create_router() -> Router {
    let mut router = Router::new();

    // Example 1: Get user information
    router.register("get_user", || async {
        let user_id = uuid::Uuid::new_v4();
        format!(
            "{{\"id\": \"{}\", \"name\": \"User {}\", \"email\": \"user@example.com\"}}",
            user_id, user_id
        )
    });

    // Example 2: Create an order
    router.register("create_order", || async {
        let order_id = uuid::Uuid::new_v4();
        format!(
            "{{\"order_id\": \"{}\", \"product\": \"Widget\", \"status\": \"created\"}}",
            order_id
        )
    });

    // Example 3: Search products
    router.register("search_products", || async {
        r#"{"query": "search", "results": [{"id": "1", "name": "Product A"}, {"id": "2", "name": "Product B"}]}"#.to_string()
    });

    // Example 4: Calculate shipping
    router.register("calculate_shipping", || async {
        let weight = 10.0;
        let cost = weight * 2.5 + 5.0;
        format!("{{\"weight\": {}, \"cost\": {:.2}}}", weight, cost)
    });

    router
}
