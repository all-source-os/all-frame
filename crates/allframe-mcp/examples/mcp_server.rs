//! MCP Server Example
//!
//! This example demonstrates how to create an MCP (Model Context Protocol)
//! server that exposes AllFrame Router handlers as LLM-callable tools.
//!
//! Run with:
//! ```bash
//! cargo run --example mcp_server
//! ```

use allframe_core::router::Router;
use allframe_mcp::McpServer;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🤖 AllFrame MCP Server Example\n");
    println!("═══════════════════════════════════════════════\n");

    // Create router and register handlers
    let mut router = Router::new();

    println!("📝 Registering handlers...\n");

    // User management handlers
    router.register("get_user", || async {
        serde_json::json!({
            "id": 123,
            "name": "Alice",
            "email": "alice@example.com"
        })
        .to_string()
    });

    router.register("list_users", || async {
        serde_json::json!([
            {"id": 1, "name": "Alice"},
            {"id": 2, "name": "Bob"},
            {"id": 3, "name": "Charlie"}
        ])
        .to_string()
    });

    router.register("create_user", || async {
        serde_json::json!({
            "id": 456,
            "name": "New User",
            "email": "newuser@example.com",
            "created": true
        })
        .to_string()
    });

    router.register("update_user", || async {
        serde_json::json!({
            "id": 123,
            "name": "Alice Updated",
            "updated": true
        })
        .to_string()
    });

    router.register("delete_user", || async {
        serde_json::json!({
            "id": 123,
            "deleted": true
        })
        .to_string()
    });

    println!("   ✅ Registered 5 handlers\n");

    // Create MCP server
    println!("🔧 Creating MCP server...\n");
    let mcp_server = McpServer::new(router);

    println!("   ✅ MCP server created");
    println!("   📊 Tool count: {}\n", mcp_server.tool_count());

    // List available tools
    println!("═══════════════════════════════════════════════\n");
    println!("🛠️  Available MCP Tools:\n");

    let tools = mcp_server.list_tools().await;
    for (i, tool) in tools.iter().enumerate() {
        println!("   {}. {}", i + 1, tool.name);
        println!("      Description: {}", tool.description);
        println!();
    }

    // Test tool invocation
    println!("═══════════════════════════════════════════════\n");
    println!("🧪 Testing Tool Invocation:\n");

    // Test 1: Get user
    println!("   1. Calling 'get_user'...");
    match mcp_server
        .call_tool("get_user", serde_json::json!({}))
        .await
    {
        Ok(result) => println!("      ✅ Result: {}\n", result),
        Err(e) => println!("      ❌ Error: {}\n", e),
    }

    // Test 2: List users
    println!("   2. Calling 'list_users'...");
    match mcp_server
        .call_tool("list_users", serde_json::json!({}))
        .await
    {
        Ok(result) => println!("      ✅ Result: {}\n", result),
        Err(e) => println!("      ❌ Error: {}\n", e),
    }

    // Test 3: Create user
    println!("   3. Calling 'create_user'...");
    let create_args = serde_json::json!({
        "name": "Bob",
        "email": "bob@example.com"
    });
    match mcp_server.call_tool("create_user", create_args).await {
        Ok(result) => println!("      ✅ Result: {}\n", result),
        Err(e) => println!("      ❌ Error: {}\n", e),
    }

    // Test 4: Unknown tool
    println!("   4. Calling 'unknown_tool' (should fail)...");
    match mcp_server
        .call_tool("unknown_tool", serde_json::json!({}))
        .await
    {
        Ok(result) => println!("      ❌ Unexpected success: {}\n", result),
        Err(e) => println!("      ✅ Expected error: {}\n", e),
    }

    println!("═══════════════════════════════════════════════\n");
    println!("📋 Summary:\n");
    println!("   • Registered {} handlers", mcp_server.tool_count());
    println!("   • All handlers automatically exposed as MCP tools");
    println!("   • Tools can be called by Claude Desktop or other MCP clients");
    println!("   • Zero configuration required!\n");

    println!("🎉 MCP Server Ready!\n");
    println!("To use with Claude Desktop, configure:");
    println!(
        r#"
{{
  "mcpServers": {{
    "allframe-api": {{
      "command": "/path/to/your/mcp_server",
      "args": []
    }}
  }}
}}
"#
    );

    Ok(())
}
