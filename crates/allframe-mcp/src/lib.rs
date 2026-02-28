//! Model Context Protocol (MCP) Server
//!
//! Automatically exposes AllFrame Router handlers as LLM-callable tools.
//!
//! # Router-based MCP Server
//!
//! ```rust,no_run
//! use allframe_core::router::Router;
//! use allframe_mcp::McpServer;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let mut router = Router::new();
//! router.register("get_user", || async { "User data".to_string() });
//! router.register("create_user", || async { "Created".to_string() });
//!
//! let mcp_server = McpServer::with_router(router);
//! // mcp_server.serve_stdio().await?;
//! # Ok(())
//! # }
//! ```
//!
//! # Forge MCP Server (Code Generation)
//!
//! ```rust,no_run
//! use allframe_mcp::forge::ForgeMcpServer;
//! use std::path::PathBuf;
//!
//! fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     let server = ForgeMcpServer::new(PathBuf::from("./my-project"))?;
//!     server.serve_stdio();
//!     Ok(())
//! }
//! ```

pub mod forge;
pub mod schema;
pub mod server;
pub mod stdio;
pub mod tools;

pub use schema::{coerce_type, extract_enum_values, openapi_to_json_schema, validate_input};
pub use server::McpServer;
pub use stdio::{init_tracing, StdioConfig, StdioTransport};
pub use tools::McpTool;

#[cfg(test)]
mod tests {
    use allframe_core::router::Router;

    use super::*;

    // Phase 1 Tests: Core MCP Server Functionality

    #[test]
    fn test_mcp_server_creation() {
        let router = Router::new();
        let mcp_server = McpServer::with_router(router);

        // Should create server successfully
        assert_eq!(mcp_server.tool_count(), 0);
    }

    #[test]
    fn test_mcp_server_discovers_handlers() {
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });
        router.register("create_user", || async { "Created".to_string() });
        router.register("update_user", || async { "Updated".to_string() });

        let mcp_server = McpServer::with_router(router);

        // Should discover all 3 handlers as tools
        assert_eq!(mcp_server.tool_count(), 3);
    }

    #[tokio::test]
    async fn test_mcp_server_lists_tools() {
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });
        router.register("create_user", || async { "Created".to_string() });

        let mcp_server = McpServer::with_router(router);
        let tools = mcp_server.list_tools();

        // Should return 2 tools
        assert_eq!(tools.len(), 2);

        // Should contain expected tool names
        let tool_names: Vec<String> = tools.iter().map(|t| t.name.clone()).collect();
        assert!(tool_names.contains(&"get_user".to_string()));
        assert!(tool_names.contains(&"create_user".to_string()));
    }

    #[tokio::test]
    async fn test_mcp_tool_has_required_fields() {
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        let mcp_server = McpServer::with_router(router);
        let tools = mcp_server.list_tools();
        let tool = &tools[0];

        // Tool should have name
        assert_eq!(tool.name, "get_user");

        // Tool should have description (can be auto-generated)
        assert!(!tool.description.is_empty());

        // Tool should have input schema
        assert!(tool.input_schema.contains("type"));
    }

    #[tokio::test]
    async fn test_mcp_server_calls_tool_successfully() {
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        let mcp_server = McpServer::with_router(router);

        // Call tool without arguments
        let result = mcp_server
            .call_tool("get_user", serde_json::json!({}))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), serde_json::json!("User data"));
    }

    #[tokio::test]
    async fn test_mcp_server_calls_tool_with_arguments() {
        let mut router = Router::new();
        router.register("echo", || async { "echoed".to_string() });

        let mcp_server = McpServer::with_router(router);

        // Call tool with arguments
        let args = serde_json::json!({
            "message": "Hello, World!"
        });
        let result = mcp_server.call_tool("echo", args).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_mcp_server_error_on_unknown_tool() {
        let router = Router::new();
        let mcp_server = McpServer::with_router(router);

        // Try to call non-existent tool
        let result = mcp_server
            .call_tool("unknown_tool", serde_json::json!({}))
            .await;

        // Should return error
        assert!(result.is_err());
        let error_msg = result.unwrap_err();
        assert!(error_msg.contains("unknown_tool") || error_msg.contains("not found"));
    }

    #[tokio::test]
    async fn test_mcp_server_handles_handler_errors() {
        let mut router = Router::new();
        // Handler that returns error in future (simulated by returning empty for now)
        router.register("failing_handler", || async { "".to_string() });

        let mcp_server = McpServer::with_router(router);

        // Call should succeed but we can test error propagation later
        let result = mcp_server
            .call_tool("failing_handler", serde_json::json!({}))
            .await;
        assert!(result.is_ok()); // For now, handlers don't fail
    }

    #[test]
    fn test_mcp_tool_creation() {
        let tool = McpTool::new("test_tool", "A test tool", r#"{"type": "object"}"#);

        assert_eq!(tool.name, "test_tool");
        assert_eq!(tool.description, "A test tool");
        assert_eq!(tool.input_schema, r#"{"type": "object"}"#);
    }

    #[test]
    fn test_mcp_tool_from_handler_name() {
        let tool = McpTool::from_handler_name("get_user");

        // Should auto-generate description
        assert!(!tool.description.is_empty());

        // Should have basic schema
        assert!(tool.input_schema.contains("object"));
    }

    #[tokio::test]
    async fn test_mcp_server_multiple_tool_calls() {
        let mut router = Router::new();
        router.register("tool1", || async { "result1".to_string() });
        router.register("tool2", || async { "result2".to_string() });

        let mcp_server = McpServer::with_router(router);

        // Call first tool
        let result1 = mcp_server.call_tool("tool1", serde_json::json!({})).await;
        assert!(result1.is_ok());
        assert_eq!(result1.unwrap(), serde_json::json!("result1"));

        // Call second tool
        let result2 = mcp_server.call_tool("tool2", serde_json::json!({})).await;
        assert!(result2.is_ok());
        assert_eq!(result2.unwrap(), serde_json::json!("result2"));
    }

    #[tokio::test]
    async fn test_mcp_server_tool_isolation() {
        let mut router = Router::new();
        router.register("tool_a", || async { "A".to_string() });
        router.register("tool_b", || async { "B".to_string() });

        let mcp_server = McpServer::with_router(router);

        // Calling tool_a should not affect tool_b
        let _ = mcp_server.call_tool("tool_a", serde_json::json!({})).await;
        let result_b = mcp_server.call_tool("tool_b", serde_json::json!({})).await;

        assert_eq!(result_b.unwrap(), serde_json::json!("B"));
    }

    #[test]
    fn test_mcp_server_empty_router() {
        let router = Router::new();
        let mcp_server = McpServer::with_router(router);

        // Should handle empty router gracefully
        assert_eq!(mcp_server.tool_count(), 0);
    }

    #[tokio::test]
    async fn test_mcp_server_list_tools_empty() {
        let router = Router::new();
        let mcp_server = McpServer::with_router(router);
        let tools = mcp_server.list_tools();

        // Should return empty list
        assert_eq!(tools.len(), 0);
    }
}
