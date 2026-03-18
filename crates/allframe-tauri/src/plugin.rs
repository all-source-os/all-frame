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

/// Managed state for tracking active streams (for cancellation).
struct ActiveStreams {
    /// Maps stream_id -> JoinHandle abort handle
    handles: Mutex<HashMap<String, tokio::task::AbortHandle>>,
}

impl ActiveStreams {
    fn new() -> Self {
        Self {
            handles: Mutex::new(HashMap::new()),
        }
    }
}

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

/// Start a streaming handler. Returns a stream_id immediately.
/// Stream items are emitted as Tauri events:
/// - `allframe:stream:{handler}:{stream_id}` — each item
/// - `allframe:stream:{handler}:{stream_id}:complete` — final result
/// - `allframe:stream:{handler}:{stream_id}:error` — handler error
#[tauri::command]
async fn allframe_stream<R: Runtime>(
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
        let event_base = format!("allframe:stream:{}:{}", handler_name, sid);

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
async fn allframe_stream_cancel<R: Runtime>(
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
                &format!("allframe:stream:unknown:{}:cancelled", stream_id),
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
    PluginBuilder::new("allframe")
        .invoke_handler(tauri::generate_handler![
            allframe_list,
            allframe_call,
            allframe_stream,
            allframe_stream_cancel,
        ])
        .setup(move |app, _api| {
            let mut router = router;
            router.inject_state(app.app_handle().clone());
            app.manage(TauriServer::new(router));
            app.manage(Arc::new(ActiveStreams::new()));
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
