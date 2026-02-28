//! MCP Server implementation

use std::collections::HashMap;
use std::future::Future;
use std::pin::Pin;
use std::sync::{Arc, RwLock};

use allframe_core::router::Router;

use super::tools::McpTool;

/// Type alias for local tool handlers.
type ToolHandler = Arc<
    dyn Fn(serde_json::Value) -> Pin<Box<dyn Future<Output = Result<serde_json::Value, String>> + Send>>
        + Send
        + Sync,
>;

/// MCP Server that exposes Router handlers as LLM-callable tools
pub struct McpServer {
    router: Option<Arc<Router>>,
    tools: Vec<McpTool>,
    local_tools: RwLock<HashMap<String, ToolHandler>>,
    listening: bool,
}

impl McpServer {
    /// Create a new MCP server without a router (local-only mode).
    pub fn new() -> Self {
        Self {
            router: None,
            tools: Vec::new(),
            local_tools: RwLock::new(HashMap::new()),
            listening: false,
        }
    }

    /// Create a new MCP server from a Router.
    pub fn with_router(router: Router) -> Self {
        let tools = Self::discover_tools(&router);
        Self {
            router: Some(Arc::new(router)),
            tools,
            local_tools: RwLock::new(HashMap::new()),
            listening: false,
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

    /// Register a local tool handler.
    pub fn register_tool<F, Fut>(&self, name: &str, handler: F)
    where
        F: Fn(serde_json::Value) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<serde_json::Value, String>> + Send + 'static,
    {
        let handler: ToolHandler = Arc::new(move |args| Box::pin(handler(args)));
        let mut local = self.local_tools.write().unwrap();
        local.insert(name.to_string(), handler);
    }

    /// Call a locally registered tool by name.
    pub async fn call_tool_local(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        let handler = {
            let local = self.local_tools.read().unwrap();
            local
                .get(name)
                .cloned()
                .ok_or_else(|| format!("Local tool not found: {}", name))?
        };
        handler(args).await
    }

    /// Returns whether the server is listening on a network port.
    pub fn is_listening(&self) -> bool {
        self.listening
    }

    /// Get the count of registered tools (router + local).
    pub fn tool_count(&self) -> usize {
        let local_count = self.local_tools.read().unwrap().len();
        self.tools.len() + local_count
    }

    /// List all available tools (router-discovered + locally registered).
    pub fn list_tools(&self) -> Vec<McpTool> {
        let mut all_tools = self.tools.clone();
        let local = self.local_tools.read().unwrap();
        for name in local.keys() {
            all_tools.push(McpTool::from_handler_name(name));
        }
        all_tools
    }

    /// Call a tool by name with given arguments (router-based).
    pub async fn call_tool(
        &self,
        name: &str,
        args: serde_json::Value,
    ) -> Result<serde_json::Value, String> {
        // Try local tools first
        let local_handler = {
            let local = self.local_tools.read().unwrap();
            local.get(name).cloned()
        };
        if let Some(handler) = local_handler {
            return handler(args).await;
        }

        // Check router tools
        if !self.tools.iter().any(|t| t.name == name) {
            return Err(format!("Tool not found: {}", name));
        }

        let router = self
            .router
            .as_ref()
            .ok_or_else(|| "No router configured".to_string())?;

        let request = format!("{}", args);
        match router.call_handler(name, &request).await {
            Ok(response) => Ok(serde_json::Value::String(response)),
            Err(e) => Err(format!("Tool execution failed: {}", e)),
        }
    }
}

impl Default for McpServer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation() {
        let router = Router::new();
        let server = McpServer::with_router(router);
        assert_eq!(server.tool_count(), 0);
    }

    #[test]
    fn test_server_creation_no_router() {
        let server = McpServer::new();
        assert_eq!(server.tool_count(), 0);
        assert!(!server.is_listening());
    }

    #[test]
    fn test_server_tool_discovery() {
        let mut router = Router::new();
        router.register("test1", || async { "result1".to_string() });
        router.register("test2", || async { "result2".to_string() });

        let server = McpServer::with_router(router);
        assert_eq!(server.tool_count(), 2);
    }

    #[test]
    fn test_server_list_tools() {
        let mut router = Router::new();
        router.register("handler1", || async { "r1".to_string() });

        let server = McpServer::with_router(router);
        let tools = server.list_tools();

        assert_eq!(tools.len(), 1);
        assert_eq!(tools[0].name, "handler1");
    }

    #[tokio::test]
    async fn test_server_call_tool() {
        let mut router = Router::new();
        router.register("echo", || async { "echoed".to_string() });

        let server = McpServer::with_router(router);
        let result = server.call_tool("echo", serde_json::json!({})).await;

        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_server_call_unknown_tool() {
        let router = Router::new();
        let server = McpServer::with_router(router);
        let result = server.call_tool("unknown", serde_json::json!({})).await;

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("not found"));
    }

    #[tokio::test]
    async fn test_server_local_tool() {
        let server = McpServer::new();
        server.register_tool("echo", |args| async move { Ok(args) });

        let result = server
            .call_tool_local("echo", serde_json::json!({"msg": "hi"}))
            .await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap()["msg"], "hi");
    }
}
