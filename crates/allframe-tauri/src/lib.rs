//! Tauri 2.x plugin for AllFrame
//!
//! Exposes AllFrame Router handlers as Tauri IPC commands for desktop apps.
//! Designed for offline-first deployments where HTTP transport is unnecessary.
//!
//! # Rust (Tauri app)
//!
//! ```rust,ignore
//! use allframe_core::router::Router;
//!
//! fn main() {
//!     let mut router = Router::new();
//!     router.register("get_user", || async {
//!         r#"{"id":1,"name":"Alice"}"#.to_string()
//!     });
//!
//!     tauri::Builder::default()
//!         .plugin(allframe_tauri::init(router))
//!         .run(tauri::generate_context!())
//!         .unwrap();
//! }
//! ```
//!
//! # Frontend (TypeScript)
//!
//! ```text
//! import { invoke } from "@tauri-apps/api/core";
//!
//! // List available handlers
//! const handlers = await invoke("plugin:allframe|allframe_list");
//!
//! // Call a handler
//! const result = await invoke("plugin:allframe|allframe_call", {
//!     handler: "get_user",
//!     args: { id: 42 }
//! });
//! ```
//!
//! # In-Process Dispatch (Local LLM / Ollama)
//!
//! `TauriServer` also supports direct in-process calls without Tauri runtime,
//! useful for local LLM integration without opening a network port:
//!
//! ```rust
//! use allframe_core::router::Router;
//! use allframe_tauri::TauriServer;
//!
//! # async fn example() {
//! let mut router = Router::new();
//! router.register("skill", || async { "done".to_string() });
//!
//! let server = TauriServer::new(router);
//! let result = server.call_handler("skill", "{}").await.unwrap();
//! assert_eq!(result.result, "done");
//! # }
//! ```

pub mod commands;
pub mod error;
pub mod plugin;
pub mod server;
pub mod types;

pub use error::TauriServerError;
pub use plugin::init;
pub use server::TauriServer;
pub use types::{CallResponse, HandlerInfo};

#[cfg(test)]
mod tests {
    use allframe_core::router::Router;

    use super::*;

    #[test]
    fn test_server_creation() {
        let router = Router::new();
        let server = TauriServer::new(router);
        assert_eq!(server.handler_count(), 0);
    }

    #[test]
    fn test_server_discovers_handlers() {
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });
        router.register("create_user", || async { "Created".to_string() });
        router.register("delete_user", || async { "Deleted".to_string() });

        let server = TauriServer::new(router);
        assert_eq!(server.handler_count(), 3);
    }

    #[test]
    fn test_handler_info_fields() {
        let mut router = Router::new();
        router.register("my_handler", || async { "result".to_string() });

        let server = TauriServer::new(router);
        let handlers = server.list_handlers();

        assert_eq!(handlers.len(), 1);
        assert_eq!(handlers[0].name, "my_handler");
        assert!(!handlers[0].description.is_empty());
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

        let result = server.call_handler("missing", "{}").await;
        assert!(result.is_err());

        let err = result.unwrap_err();
        match err {
            TauriServerError::HandlerNotFound(name) => {
                assert_eq!(name, "missing");
            }
            other => panic!("Expected HandlerNotFound, got: {other:?}"),
        }
    }

    #[tokio::test]
    async fn test_multiple_calls() {
        let mut router = Router::new();
        router.register("a", || async { "A".to_string() });
        router.register("b", || async { "B".to_string() });

        let server = TauriServer::new(router);

        let a = server.call_handler("a", "{}").await.unwrap();
        let b = server.call_handler("b", "{}").await.unwrap();

        assert_eq!(a.result, "A");
        assert_eq!(b.result, "B");
    }

    #[tokio::test]
    async fn test_handler_isolation() {
        let mut router = Router::new();
        router.register("x", || async { "X".to_string() });
        router.register("y", || async { "Y".to_string() });

        let server = TauriServer::new(router);

        let _ = server.call_handler("x", "{}").await;
        let y = server.call_handler("y", "{}").await.unwrap();
        assert_eq!(y.result, "Y");
    }

    #[test]
    fn test_empty_router() {
        let router = Router::new();
        let server = TauriServer::new(router);
        assert_eq!(server.handler_count(), 0);
        assert!(server.list_handlers().is_empty());
    }

    #[tokio::test]
    async fn test_list_empty() {
        let router = Router::new();
        let server = TauriServer::new(router);
        assert!(server.list_handlers().is_empty());
    }

    #[test]
    fn test_call_response_serialization() {
        let response = CallResponse {
            result: r#"{"id":1}"#.to_string(),
        };
        let json = serde_json::to_string(&response).unwrap();
        assert!(json.contains("id"));
    }

    #[test]
    fn test_handler_info_serialization() {
        let info = HandlerInfo {
            name: "test".to_string(),
            description: "A test handler".to_string(),
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("A test handler"));
    }

    #[test]
    fn test_error_serialization() {
        let err = TauriServerError::HandlerNotFound("missing".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("missing"));
    }
}
