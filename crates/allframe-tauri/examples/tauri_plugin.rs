//! Example: AllFrame Tauri plugin for an offline-first desktop app
//!
//! NOTE: This example cannot be `cargo run` directly — it requires a full
//! Tauri application with `tauri.conf.json`. It demonstrates the integration
//! pattern you would use in your Tauri app's `main.rs`.
//!
//! # Setup in your Tauri app
//!
//! ```rust,no_run
//! use allframe_core::router::Router;
//!
//! fn main() {
//!     let mut router = Router::new();
//!     router.register("get_user", || async {
//!         r#"{"id":1,"name":"Alice"}"#.to_string()
//!     });
//!     router.register("list_items", || async {
//!         r#"[{"id":1,"title":"Item 1"}]"#.to_string()
//!     });
//!     router.register("health_check", || async {
//!         r#"{"status":"ok","offline":true}"#.to_string()
//!     });
//!
//!     tauri::Builder::default()
//!         .plugin(allframe_tauri::init(router))
//!         .run(tauri::generate_context!())
//!         .unwrap();
//! }
//! ```
//!
//! # Frontend usage (TypeScript)
//!
//! ```typescript
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
//! # In-process dispatch (local LLM / Ollama)
//!
//! ```rust
//! use allframe_tauri::TauriServer;
//! use allframe_core::router::Router;
//!
//! # async fn example() {
//! let mut router = Router::new();
//! router.register("skill", || async { "done".to_string() });
//!
//! let server = TauriServer::new(router);
//! // Direct call — no IPC, no serialization overhead
//! let result = server.call_handler("skill", "{}").await.unwrap();
//! assert_eq!(result.result, "done");
//! # }
//! ```

/// This example demonstrates the TauriServer API without requiring a Tauri runtime.
fn main() {
    use allframe_core::router::Router;
    use allframe_tauri::TauriServer;

    // Build the router with your handlers
    let mut router = Router::new();
    router.register("get_user", || async {
        r#"{"id":1,"name":"Alice"}"#.to_string()
    });
    router.register("list_items", || async {
        r#"[{"id":1,"title":"Item 1"},{"id":2,"title":"Item 2"}]"#.to_string()
    });
    router.register("health_check", || async {
        r#"{"status":"ok","offline":true}"#.to_string()
    });

    let server = TauriServer::new(router);
    println!("Registered {} handlers:", server.handler_count());
    for handler in server.list_handlers() {
        println!("  - {} ({})", handler.name, handler.description);
    }

    // In-process call (no Tauri runtime needed)
    let rt = tokio::runtime::Runtime::new().unwrap();
    let result = rt.block_on(server.call_handler("get_user", "{}"));
    println!("Result: {:?}", result.unwrap().result);
}
