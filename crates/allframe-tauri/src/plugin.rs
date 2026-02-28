//! Tauri 2.x plugin builder

use allframe_core::router::Router;
use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::Runtime;

use crate::error::TauriServerError;
use crate::server::TauriServer;
use crate::types::{CallResponse, HandlerInfo};

/// List all registered AllFrame handlers.
#[tauri::command]
async fn allframe_list(
    server: tauri::State<'_, TauriServer>,
) -> Result<Vec<HandlerInfo>, TauriServerError> {
    Ok(server.list_handlers().to_vec())
}

/// Call a handler by name with JSON arguments.
#[tauri::command]
async fn allframe_call(
    handler: String,
    args: serde_json::Value,
    server: tauri::State<'_, TauriServer>,
) -> Result<CallResponse, TauriServerError> {
    let args_str = args.to_string();
    server.call_handler(&handler, &args_str).await
}

/// Create a Tauri 2.x plugin that exposes AllFrame handlers via IPC.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::router::Router;
///
/// fn main() {
///     let mut router = Router::new();
///     router.register("greet", || async { "Hello!".to_string() });
///
///     tauri::Builder::default()
///         .plugin(allframe_tauri::init(router))
///         .run(tauri::generate_context!())
///         .unwrap();
/// }
/// ```
pub fn init<R: Runtime>(router: Router) -> TauriPlugin<R> {
    PluginBuilder::new("allframe")
        .invoke_handler(tauri::generate_handler![allframe_list, allframe_call])
        .setup(move |app, _api| {
            use tauri::Manager;
            app.manage(TauriServer::new(router));
            Ok(())
        })
        .build()
}
