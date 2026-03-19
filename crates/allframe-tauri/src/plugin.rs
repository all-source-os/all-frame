//! Tauri 2.x plugin builder

use std::collections::HashMap;
use std::sync::Arc;

use allframe_core::router::Router;
use tauri::plugin::{Builder as PluginBuilder, TauriPlugin};
use tauri::{Emitter, Manager, Runtime};
use tokio::sync::Mutex;

use crate::error::TauriServerError;
use crate::server::TauriServer;
use crate::types::{CallResponse, HandlerInfo, StreamStartResponse};

/// The plugin identifier used for Tauri 2 ACL resolution and IPC routing.
///
/// This **must** match the identifier that `tauri_plugin::Builder` in `build.rs`
/// derives from the crate name (`allframe-tauri`). A mismatch causes:
///
/// ```text
/// allframe.allframe_call not allowed. Plugin not found
/// ```
///
/// Frontend invocations use this identifier:
///
/// ```typescript
/// invoke("plugin:allframe-tauri|allframe_call", { handler: "greet", args: {} });
/// ```
pub const PLUGIN_NAME: &str = "allframe-tauri";

/// Format a stream event name for a given handler and stream ID.
///
/// Returns `allframe-tauri:stream:{handler}:{stream_id}`.
pub(crate) fn stream_event(handler: &str, stream_id: &str) -> String {
    format!("{PLUGIN_NAME}:stream:{handler}:{stream_id}")
}

/// Format a boot-progress event name.
///
/// Returns `allframe-tauri:boot-progress`.
pub(crate) fn boot_progress_event() -> String {
    format!("{PLUGIN_NAME}:boot-progress")
}

/// Managed state for tracking active streams (for cancellation).
pub(crate) struct ActiveStreams {
    /// Maps stream_id -> JoinHandle abort handle
    handles: Mutex<HashMap<String, tokio::task::AbortHandle>>,
}

impl ActiveStreams {
    pub(crate) fn new() -> Self {
        Self {
            handles: Mutex::new(HashMap::new()),
        }
    }
}

/// List all registered AllFrame handlers.
#[tauri::command]
pub(crate) async fn allframe_list(
    server: tauri::State<'_, TauriServer>,
) -> Result<Vec<HandlerInfo>, TauriServerError> {
    Ok(server.list_handlers().to_vec())
}

/// Call a handler by name with JSON arguments.
#[tauri::command]
pub(crate) async fn allframe_call(
    handler: String,
    args: serde_json::Value,
    server: tauri::State<'_, TauriServer>,
) -> Result<CallResponse, TauriServerError> {
    let args_str = args.to_string();
    server.call_handler(&handler, &args_str).await
}

/// Start a streaming handler. Returns a stream_id immediately.
/// Stream items are emitted as Tauri events:
/// - `allframe-tauri:stream:{handler}:{stream_id}` — each item
/// - `allframe-tauri:stream:{handler}:{stream_id}:complete` — final result
/// - `allframe-tauri:stream:{handler}:{stream_id}:error` — handler error
#[tauri::command]
pub(crate) async fn allframe_stream<R: Runtime>(
    handler: String,
    args: serde_json::Value,
    app: tauri::AppHandle<R>,
    server: tauri::State<'_, TauriServer>,
    active: tauri::State<'_, Arc<ActiveStreams>>,
) -> Result<StreamStartResponse, TauriServerError> {
    let stream_id = uuid::Uuid::new_v4().to_string();
    let args_str = args.to_string();

    let (mut rx, join_handle) = server.call_streaming_handler(&handler, &args_str)?;

    let sid = stream_id.clone();
    let handler_name = handler.clone();
    let app_clone = app.clone();
    let active_inner: Arc<ActiveStreams> = (*active).clone();

    let task = tokio::spawn(async move {
        let event_base = stream_event(&handler_name, &sid);

        // Forward stream items as events
        while let Some(item) = rx.recv().await {
            let _ = app_clone.emit(&event_base, &item);
        }

        // Wait for the handler to complete and emit final event
        match join_handle.await {
            Ok(Ok(response)) => {
                let _ = app_clone.emit(&format!("{event_base}:complete"), &response.result);
            }
            Ok(Err(e)) => {
                let _ = app_clone.emit(&format!("{event_base}:error"), &e.to_string());
            }
            Err(e) => {
                let _ = app_clone.emit(
                    &format!("{event_base}:error"),
                    &format!("Handler task panicked: {e}"),
                );
            }
        }

        // Cleanup from active streams map
        active_inner.handles.lock().await.remove(&sid);
    });

    // Store abort handle for cancellation
    active
        .handles
        .lock()
        .await
        .insert(stream_id.clone(), task.abort_handle());

    Ok(StreamStartResponse { stream_id })
}

/// Cancel an active stream by stream_id.
#[tauri::command]
pub(crate) async fn allframe_stream_cancel<R: Runtime>(
    stream_id: String,
    app: tauri::AppHandle<R>,
    active: tauri::State<'_, Arc<ActiveStreams>>,
) -> Result<(), TauriServerError> {
    let mut handles = active.handles.lock().await;
    match handles.remove(&stream_id) {
        Some(abort_handle) => {
            abort_handle.abort();
            // The stream's StreamReceiver will be dropped when the task is aborted,
            // which auto-cancels the CancellationToken.
            let _ = app.emit(
                &format!("{}:cancelled", stream_event("unknown", &stream_id)),
                &(),
            );
            Ok(())
        }
        None => Err(TauriServerError::ExecutionFailed(format!(
            "Stream not found or already completed: {stream_id}"
        ))),
    }
}

/// Create a Tauri 2.x plugin that exposes AllFrame handlers via IPC.
///
/// The Tauri `AppHandle<R>` is automatically injected as state during plugin
/// setup, so handlers registered with `register_with_state::<AppHandle<R>, …>`
/// (or any `*_with_state*` variant) can access it directly.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::router::{Router, State};
/// use std::sync::Arc;
/// use tauri::AppHandle;
///
/// fn main() {
///     let mut router = Router::new();
///     router.register("greet", || async { "Hello!".to_string() });
///
///     // AppHandle<tauri::Wry> is auto-injected — handlers can request it:
///     router.register_with_state_only::<AppHandle<tauri::Wry>, _, _>(
///         "send_notification",
///         |app: State<Arc<AppHandle<tauri::Wry>>>| async move {
///             // app.emit("event", &payload).unwrap();
///             "sent".to_string()
///         },
///     );
///
///     tauri::Builder::default()
///         .plugin(allframe_tauri::init(router))
///         .run(tauri::generate_context!())
///         .unwrap();
/// }
/// ```
pub fn init<R: Runtime>(router: Router) -> TauriPlugin<R> {
    build_plugin(move |app| {
        let mut router = router;
        router.inject_state(app.app_handle().clone());
        app.manage(TauriServer::new(router));
        app.manage(Arc::new(ActiveStreams::new()));
        Ok(())
    })
}

/// Internal helper: creates the plugin with invoke_handler and a custom setup closure.
/// Used by both `init()` and `BootBuilder::build()`.
pub(crate) fn build_plugin<R, F>(setup: F) -> TauriPlugin<R>
where
    R: Runtime,
    F: FnOnce(&tauri::AppHandle<R>) -> Result<(), Box<dyn std::error::Error>> + Send + 'static,
{
    PluginBuilder::new(PLUGIN_NAME)
        .invoke_handler(tauri::generate_handler![
            allframe_list,
            allframe_call,
            allframe_stream,
            allframe_stream_cancel,
        ])
        .setup(move |app, _api| {
            setup(app.app_handle())?;
            Ok(())
        })
        .build()
}

/// Create a Tauri 2.x plugin with shared state for dependency injection.
///
/// Convenience wrapper that calls `router.with_state(state)` before
/// constructing the plugin. The Tauri `AppHandle<R>` is also automatically
/// injected (see [`init`]).
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::router::Router;
///
/// struct AppState { db: DbPool }
///
/// fn main() {
///     let state = AppState { db: pool };
///     let mut router = Router::new().with_state(state);
///     router.register_with_state_only::<AppState, _, _>("health", |s| async move {
///         format!("ok")
///     });
///
///     tauri::Builder::default()
///         .plugin(allframe_tauri::init_with_state(router))
///         .run(tauri::generate_context!())
///         .unwrap();
/// }
/// ```
pub fn init_with_state<R: Runtime, S: Send + Sync + 'static>(
    router: Router,
    state: S,
) -> TauriPlugin<R> {
    let router = router.with_state(state);
    init(router)
}

/// Create a [`BootBuilder`](crate::boot::BootBuilder) for configuring
/// an async boot phase before the app renders.
///
/// # Example
///
/// ```rust,ignore
/// tauri::Builder::default()
///     .plugin(
///         allframe_tauri::builder(router)
///             .on_boot(2, |ctx| async move {
///                 let store = open_store(&ctx.data_dir()?).await
///                     .map_err(|e| BootError::Failed(e.to_string()))?;
///                 ctx.inject_state(store);
///                 ctx.emit_progress("Store ready");
///                 Ok(())
///             })
///             .build(),
///     )
///     .run(tauri::generate_context!())
///     .unwrap();
/// ```
pub fn builder<R: Runtime>(router: Router) -> crate::boot::BootBuilder<R> {
    crate::boot::BootBuilder::new(router)
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── Plugin name identity ──────────────────────────────────────────

    #[test]
    fn plugin_name_matches_crate_name() {
        // The Tauri 2 ACL system resolves permissions by the crate name
        // (allframe-tauri). The runtime plugin name must match exactly.
        assert_eq!(PLUGIN_NAME, "allframe-tauri");
    }

    #[test]
    fn plugin_name_contains_hyphen() {
        // Regression: the old value "allframe" (no hyphen) caused ACL
        // mismatch because build.rs generates permissions under "allframe-tauri".
        assert!(
            PLUGIN_NAME.contains('-'),
            "Plugin name must include hyphen to match crate-derived ACL identifier"
        );
    }

    // ─── Event name formatting ─────────────────────────────────────────

    #[test]
    fn stream_event_uses_plugin_name_prefix() {
        let event = stream_event("my_handler", "abc-123");
        assert_eq!(event, "allframe-tauri:stream:my_handler:abc-123");
    }

    #[test]
    fn stream_event_complete_suffix() {
        let base = stream_event("handler", "id1");
        let complete = format!("{base}:complete");
        assert_eq!(complete, "allframe-tauri:stream:handler:id1:complete");
    }

    #[test]
    fn stream_event_error_suffix() {
        let base = stream_event("handler", "id1");
        let error = format!("{base}:error");
        assert_eq!(error, "allframe-tauri:stream:handler:id1:error");
    }

    #[test]
    fn boot_progress_event_uses_plugin_name() {
        assert_eq!(boot_progress_event(), "allframe-tauri:boot-progress");
    }

    #[test]
    fn stream_event_does_not_use_old_prefix() {
        // Regression: events previously used "allframe:" prefix
        let event = stream_event("handler", "id");
        assert!(
            !event.starts_with("allframe:"),
            "Event must not use the old 'allframe:' prefix (should be 'allframe-tauri:')"
        );
    }

    #[test]
    fn boot_progress_event_does_not_use_old_prefix() {
        let event = boot_progress_event();
        assert!(
            !event.starts_with("allframe:"),
            "Boot progress event must not use the old 'allframe:' prefix"
        );
    }
}
