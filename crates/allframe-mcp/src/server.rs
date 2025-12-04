//! MCP Server implementation

use allframe_core::router::Router;
use super::tools::McpTool;
use std::sync::Arc;

/// MCP Server that exposes Router handlers as LLM-callable tools
pub struct McpServer {
    router: Arc<Router>,
    tools: Vec<McpTool>,
}

impl McpServer {
    /// Create a new MCP server from a Router
    pub fn new(router: Router) -> Self {
        let tools = Self::discover_tools(&router);
        Self {
            router: Arc::new(router),
            tools,
        }
    }

    /// Discover tools from Router handlers
    fn discover_tools(router: &Router) -> Vec<McpTool> {
        router
            .list_handlers()
            .iter()
            .map(|name| McpTool::from_handler_name(name))
            .collect()
    }

    /// Get the count of registered tools
    pub fn tool_count(&self) -> usize {
        self.tools.len()
    }

    /// List all available tools
    pub async fn list_tools(&self) -> Vec<McpTool> {
        self.tools.clone()
    }

    /// Call a tool by name with given arguments
    pub async fn call_tool(&self, name: &str, args: serde_json::Value) -> Result<serde_json::Value, String> {
        // Check if tool exists
        if !self.tools.iter().any(|t| t.name == name) {
            return Err(format!("Tool not found: {}", name));
        }

        // For now, we route through the router using a simple request format
        // In Phase 2, we'll add proper argument mapping
        let request = format!("{}", args);

        // Call the handler through the router
        match self.router.call_handler(name, &request).await {
            Ok(response) => {
                // Convert response string to JSON value
                Ok(serde_json::Value::String(response))
            }
            Err(e) => Err(format!("Tool execution failed: {}", e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let router = Router::new();
        let server = McpServer::new(router);
        assert_eq!(server.tool_count(), 0);
    }

    #[test]
    fn test_server_tool_discovery() {
        let mut router = Router::new();
        router.register("test1", || async { "result1".to_string() });
        router.register("test2", || async { "result2".to_string() });

        let server = McpServer::new(router);
        assert_eq!(server.tool_count(), 2);
    }

    #[tokio::test]
    async fn test_server_list_tools() {
        let mut router = Router::new();
        router.register("handler1", || async { "r1".to_string() });

        let server = McpServer::new(router);
        let tools = server.list_tools().await;

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "handler1");
    }

    #[tokio::test]
    async fn test_server_call_tool() {
        let mut router = Router::new();
        router.register("echo", || async { "echoed".to_string() });

        let server = McpServer::new(router);
        let result = server.call_tool("echo", serde_json::json!({})).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_call_unknown_tool() {
        let router = Router::new();
        let server = McpServer::new(router);
        let result = server.call_tool("unknown", serde_json::json!({})).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }
}
