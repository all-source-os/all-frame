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
//! const handlers = await invoke("plugin:allframe-tauri|allframe_list");
//!
//! // Call a handler
//! const result = await invoke("plugin:allframe-tauri|allframe_call", {
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

pub mod boot;
pub mod commands;
pub mod error;
pub mod plugin;
pub mod server;
pub mod types;

pub use allframe_core::router::StreamReceiver;
pub use boot::{BootBuilder, BootContext, BootError, BootProgress};
pub use error::TauriServerError;
pub use plugin::{builder, init, init_with_state, PLUGIN_NAME};
pub use server::TauriServer;
pub use types::{CallResponse, HandlerInfo, HandlerKind, StreamStartResponse};

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
            kind: HandlerKind::RequestResponse,
        };
        let json = serde_json::to_string(&info).unwrap();
        assert!(json.contains("test"));
        assert!(json.contains("A test handler"));
        assert!(json.contains("request_response"));
    }

    #[test]
    fn test_error_serialization() {
        let err = TauriServerError::HandlerNotFound("missing".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("missing"));
    }

    #[tokio::test]
    async fn test_typed_args_flow_through_tauri_server() {
        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        let mut router = Router::new();
        router.register_with_args("greet", |args: Input| async move {
            format!(r#"{{"greeting":"Hello {}"}}"#, args.name)
        });

        let server = TauriServer::new(router);
        let result = server
            .call_handler("greet", r#"{"name":"Alice"}"#)
            .await
            .unwrap();

        assert_eq!(result.result, r#"{"greeting":"Hello Alice"}"#);
    }

    #[tokio::test]
    async fn test_tauri_compat_handler_through_server() {
        use allframe_macros::tauri_compat;

        #[tauri_compat]
        async fn greet(name: String, age: u32) -> String {
            format!(r#"{{"greeting":"Hello {}, age {}"}}"#, name, age)
        }

        let mut router = Router::new();
        router.register_with_args::<GreetArgs, _, _>("greet", greet);

        let server = TauriServer::new(router);
        let result = server
            .call_handler("greet", r#"{"name":"Alice","age":30}"#)
            .await
            .unwrap();

        assert_eq!(result.result, r#"{"greeting":"Hello Alice, age 30"}"#);
    }

    #[tokio::test]
    async fn test_tauri_compat_optional_param() {
        use allframe_macros::tauri_compat;

        #[tauri_compat]
        async fn greet_optional(name: String, title: Option<String>) -> String {
            match title {
                Some(t) => format!("{} {}", t, name),
                None => name,
            }
        }

        let mut router = Router::new();
        router
            .register_with_args::<GreetOptionalArgs, _, _>("greet_optional", greet_optional);

        let server = TauriServer::new(router);

        // With optional param
        let result = server
            .call_handler("greet_optional", r#"{"name":"Alice","title":"Dr."}"#)
            .await
            .unwrap();
        assert_eq!(result.result, "Dr. Alice");

        // Without optional param
        let result = server
            .call_handler("greet_optional", r#"{"name":"Bob"}"#)
            .await
            .unwrap();
        assert_eq!(result.result, "Bob");
    }

    #[tokio::test]
    async fn test_typed_return_struct() {
        #[derive(serde::Serialize)]
        struct UserResponse {
            id: u32,
            name: String,
        }

        let mut router = Router::new();
        router.register_typed("get_user", || async {
            UserResponse {
                id: 1,
                name: "Alice".to_string(),
            }
        });

        let server = TauriServer::new(router);
        let result = server.call_handler("get_user", "{}").await.unwrap();
        assert_eq!(result.result, r#"{"id":1,"name":"Alice"}"#);
    }

    #[tokio::test]
    async fn test_typed_return_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            x: i32,
        }

        #[derive(serde::Serialize)]
        struct Output {
            doubled: i32,
        }

        let mut router = Router::new();
        router.register_typed_with_args("double", |args: Input| async move {
            Output { doubled: args.x * 2 }
        });

        let server = TauriServer::new(router);
        let result = server.call_handler("double", r#"{"x":21}"#).await.unwrap();
        assert_eq!(result.result, r#"{"doubled":42}"#);
    }

    #[tokio::test]
    async fn test_result_handler_ok() {
        #[derive(serde::Serialize)]
        struct Data {
            value: i32,
        }

        let mut router = Router::new();
        router.register_result("get_data", || async {
            Ok::<_, String>(Data { value: 42 })
        });

        let server = TauriServer::new(router);
        let result = server.call_handler("get_data", "{}").await.unwrap();
        assert_eq!(result.result, r#"{"value":42}"#);
    }

    #[tokio::test]
    async fn test_result_handler_err() {
        #[derive(serde::Serialize)]
        struct Data {
            value: i32,
        }

        let mut router = Router::new();
        router.register_result("fail", || async {
            Err::<Data, String>("not found".to_string())
        });

        let server = TauriServer::new(router);
        let result = server.call_handler("fail", "{}").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_typed_return_with_state() {
        use allframe_core::router::State;
        use std::sync::Arc;

        struct AppState {
            prefix: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        #[derive(serde::Serialize)]
        struct Greeting {
            message: String,
        }

        let mut router = Router::new().with_state(AppState {
            prefix: "Hey".to_string(),
        });
        router.register_typed_with_state::<AppState, Input, Greeting, _, _>(
            "greet",
            |state: State<Arc<AppState>>, args: Input| async move {
                Greeting {
                    message: format!("{} {}", state.prefix, args.name),
                }
            },
        );

        let server = TauriServer::new(router);
        let result = server
            .call_handler("greet", r#"{"name":"Dave"}"#)
            .await
            .unwrap();
        assert_eq!(result.result, r#"{"message":"Hey Dave"}"#);
    }

    #[tokio::test]
    async fn test_tauri_compat_no_args() {
        use allframe_macros::tauri_compat;

        #[tauri_compat]
        async fn health_check() -> String {
            "ok".to_string()
        }

        let mut router = Router::new();
        router.register("health_check", health_check);

        let server = TauriServer::new(router);
        let result = server.call_handler("health_check", "{}").await.unwrap();
        assert_eq!(result.result, "ok");
    }

    #[tokio::test]
    async fn test_state_injection_through_tauri_server() {
        use allframe_core::router::State;
        use std::sync::Arc;

        struct AppState {
            prefix: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        let mut router = Router::new().with_state(AppState {
            prefix: "Hey".to_string(),
        });
        router.register_with_state::<AppState, Input, _, _>(
            "greet",
            |state: State<Arc<AppState>>, args: Input| async move {
                format!("{} {}", state.prefix, args.name)
            },
        );

        let server = TauriServer::new(router);
        let result = server
            .call_handler("greet", r#"{"name":"Bob"}"#)
            .await
            .unwrap();

        assert_eq!(result.result, "Hey Bob");
    }

    #[tokio::test]
    async fn test_tauri_compat_streaming_with_args() {
        use allframe_core::router::StreamSender;
        use allframe_macros::tauri_compat;

        #[tauri_compat(streaming)]
        async fn stream_greet(name: String, count: u32, tx: StreamSender) -> String {
            for i in 0..count {
                tx.send(format!("Hello {} #{}", name, i)).await.ok();
            }
            "done".to_string()
        }

        let mut router = Router::new();
        router.register_streaming_with_args::<StreamGreetArgs, _, _, _>(
            "stream_greet",
            stream_greet,
        );

        let server = TauriServer::new(router);
        let (mut rx, handle) = server
            .call_streaming_handler("stream_greet", r#"{"name":"Alice","count":2}"#)
            .unwrap();

        let msg1 = rx.recv().await.unwrap();
        assert_eq!(msg1, "Hello Alice #0");
        let msg2 = rx.recv().await.unwrap();
        assert_eq!(msg2, "Hello Alice #1");

        let response = handle.await.unwrap().unwrap();
        assert_eq!(response.result, "done");
    }

    /// Tests the inject_state path that the Tauri plugin uses to inject AppHandle.
    /// We simulate it with a plain struct since we can't construct AppHandle in tests.
    #[tokio::test]
    async fn test_inject_state_simulates_app_handle_injection() {
        use allframe_core::router::State;
        use std::sync::Arc;

        // Simulates what plugin::init does: inject_state after router construction
        struct FakeAppHandle {
            app_name: String,
        }
        struct DbPool {
            url: String,
        }

        let mut router = Router::new().with_state(DbPool {
            url: "sqlite://db".to_string(),
        });

        // Register handler that needs the "app handle"
        router.register_with_state_only::<FakeAppHandle, _, _>(
            "emit_event",
            |app: State<Arc<FakeAppHandle>>| async move {
                format!("emitted from {}", app.app_name)
            },
        );

        // Register handler that needs the db pool
        router.register_with_state_only::<DbPool, _, _>(
            "db_url",
            |db: State<Arc<DbPool>>| async move { db.url.clone() },
        );

        // Simulate what the plugin setup closure does
        router.inject_state(FakeAppHandle {
            app_name: "MyTauriApp".to_string(),
        });

        let server = TauriServer::new(router);

        let result = server.call_handler("emit_event", "{}").await.unwrap();
        assert_eq!(result.result, "emitted from MyTauriApp");

        let result = server.call_handler("db_url", "{}").await.unwrap();
        assert_eq!(result.result, "sqlite://db");
    }

    #[tokio::test]
    async fn test_tauri_compat_streaming_no_args() {
        use allframe_core::router::StreamSender;
        use allframe_macros::tauri_compat;

        #[tauri_compat(streaming)]
        async fn stream_ping(tx: StreamSender) -> String {
            tx.send("pong".to_string()).await.ok();
            "done".to_string()
        }

        let mut router = Router::new();
        router.register_streaming("stream_ping", stream_ping);

        let server = TauriServer::new(router);
        let (mut rx, handle) = server
            .call_streaming_handler("stream_ping", "{}")
            .unwrap();

        let msg = rx.recv().await.unwrap();
        assert_eq!(msg, "pong");

        let response = handle.await.unwrap().unwrap();
        assert_eq!(response.result, "done");
    }
}
