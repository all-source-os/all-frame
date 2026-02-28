//! TauriServer wraps a Router for IPC dispatch
//!
//! Mirrors the `McpServer` pattern: discovers handlers at construction,
//! provides list/call methods that don't require a Tauri runtime.

use std::sync::Arc;

use allframe_core::router::Router;

use crate::error::TauriServerError;
use crate::types::{CallResponse, HandlerInfo};

/// Wraps an AllFrame `Router` for Tauri IPC dispatch.
///
/// Constructed once at app startup and managed as Tauri state.
/// Provides in-process `call_local` for zero-overhead dispatch
/// (useful for local LLM integration without network).
pub struct TauriServer {
    router: Arc<Router>,
    handlers: Vec<HandlerInfo>,
}

impl TauriServer {
    /// Create a new TauriServer from a Router
    pub fn new(router: Router) -> Self {
        let handlers = router
            .list_handlers()
            .into_iter()
            .map(|name| HandlerInfo {
                description: format!("Handler: {name}"),
                name,
            })
            .collect();

        Self {
            router: Arc::new(router),
            handlers,
        }
    }

    /// List all registered handlers
    pub fn list_handlers(&self) -> &[HandlerInfo] {
        &self.handlers
    }

    /// Number of registered handlers
    pub fn handler_count(&self) -> usize {
        self.handlers.len()
    }

    /// Call a handler by name (in-process, no Tauri runtime needed).
    ///
    /// This enables zero-overhead dispatch for local LLM integration
    /// (e.g., Ollama) without opening a network port.
    pub async fn call_handler(
        &self,
        name: &str,
        args: &str,
    ) -> Result<CallResponse, TauriServerError> {
        if !self.handlers.iter().any(|h| h.name == name) {
            return Err(TauriServerError::HandlerNotFound(name.to_string()));
        }

        match self.router.call_handler(name, args).await {
            Ok(result) => Ok(CallResponse { result }),
            Err(e) => Err(TauriServerError::ExecutionFailed(e)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_server_creation_empty() {
        let router = Router::new();
        let server = TauriServer::new(router);
        assert_eq!(server.handler_count(), 0);
        assert!(server.list_handlers().is_empty());
    }

    #[test]
    fn test_server_discovers_handlers() {
        let mut router = Router::new();
        router.register("get_user", || async { "user".to_string() });
        router.register("list_items", || async { "items".to_string() });

        let server = TauriServer::new(router);
        assert_eq!(server.handler_count(), 2);

        let names: Vec<&str> = server.list_handlers().iter().map(|h| h.name.as_str()).collect();
        assert!(names.contains(&"get_user"));
        assert!(names.contains(&"list_items"));
    }

    #[tokio::test]
    async fn test_call_handler_success() {
        let mut router = Router::new();
        router.register("echo", || async { "echoed".to_string() });

        let server = TauriServer::new(router);
        let result = server.call_handler("echo", "{}").await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().result, "echoed");
    }

    #[tokio::test]
    async fn test_call_handler_not_found() {
        let router = Router::new();
        let server = TauriServer::new(router);

        let result = server.call_handler("nonexistent", "{}").await;
        assert!(result.is_err());
        match result.unwrap_err() {
            TauriServerError::HandlerNotFound(name) => assert_eq!(name, "nonexistent"),
            other => panic!("Expected HandlerNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_call_multiple_handlers() {
        let mut router = Router::new();
        router.register("a", || async { "result_a".to_string() });
        router.register("b", || async { "result_b".to_string() });

        let server = TauriServer::new(router);

        let a = server.call_handler("a", "{}").await.unwrap();
        let b = server.call_handler("b", "{}").await.unwrap();

        assert_eq!(a.result, "result_a");
        assert_eq!(b.result, "result_b");
    }

    #[tokio::test]
    async fn test_handler_isolation() {
        let mut router = Router::new();
        router.register("x", || async { "X".to_string() });
        router.register("y", || async { "Y".to_string() });

        let server = TauriServer::new(router);

        // Calling x should not affect y
        let _ = server.call_handler("x", "{}").await;
        let y = server.call_handler("y", "{}").await.unwrap();
        assert_eq!(y.result, "Y");
    }
}
