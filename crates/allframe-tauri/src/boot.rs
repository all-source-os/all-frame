//! Async boot lifecycle for Tauri plugins.
//!
//! Tauri 2's `setup()` closure is synchronous and runs on the macOS main
//! thread with no Tokio reactor. This module provides [`BootBuilder`] and
//! [`BootContext`] to run async initialization (event stores, projections,
//! command buses) correctly — creating a dedicated Tokio runtime, blocking
//! until boot completes, and emitting progress events to the frontend.
//!
//! # Example
//!
//! ```rust,ignore
//! allframe_tauri::builder(router)
//!     .on_boot(2, |ctx| async move {
//!         let store = open_store(&ctx.data_dir()?).await
//!             .map_err(|e| BootError::Failed(e.to_string()))?;
//!         ctx.inject_state(store);
//!         ctx.emit_progress("Event store opened");
//!
//!         let registry = init_projections().await
//!             .map_err(|e| BootError::Failed(e.to_string()))?;
//!         ctx.inject_state(registry);
//!         ctx.emit_progress("Projections ready");
//!
//!         Ok(())
//!     })
//!     .build()
//! ```

use std::any::TypeId;
use std::future::Future;
use std::pin::Pin;
use std::sync::atomic::{AtomicU32, Ordering};
use std::sync::Arc;

use allframe_core::router::{Router, SharedStateMap};
use serde::Serialize;
use tauri::plugin::TauriPlugin;
use tauri::{Emitter, Manager, Runtime};

use crate::plugin::{boot_progress_event, build_plugin, ActiveStreams};
use crate::server::TauriServer;

// ─── Types ──────────────────────────────────────────────────────────────────

/// Progress event payload emitted during boot as `allframe-tauri:boot-progress`.
///
/// Frontend can listen for this to render a splash/loading screen:
///
/// ```typescript
/// import { listen } from "@tauri-apps/api/event";
/// await listen("allframe-tauri:boot-progress", (e) => {
///     // e.payload: { step: 2, total: 3, label: "Projections ready" }
/// });
/// ```
#[derive(Debug, Clone, Serialize)]
pub struct BootProgress {
    /// Current step number (1-indexed).
    pub step: u32,
    /// Total number of steps declared in `on_boot`.
    pub total: u32,
    /// Human-readable label for this step.
    pub label: String,
}

/// Errors that can occur during the boot phase.
#[derive(Debug, thiserror::Error, Serialize)]
pub enum BootError {
    /// Generic boot failure with a descriptive message.
    #[error("Boot failed: {0}")]
    Failed(String),

    /// Could not resolve the app data directory.
    #[error("Could not resolve data directory: {0}")]
    DataDir(String),

    /// Failed to create the boot Tokio runtime.
    #[error("Failed to create boot runtime: {0}")]
    Runtime(String),
}

// ─── BootContext ─────────────────────────────────────────────────────────────

/// Context passed to the async boot closure.
///
/// Provides access to the Tauri `AppHandle`, state injection into the
/// Router's shared state map, and progress event emission.
pub struct BootContext<R: Runtime> {
    app_handle: tauri::AppHandle<R>,
    states: SharedStateMap,
    total_steps: u32,
    current_step: AtomicU32,
}

impl<R: Runtime> BootContext<R> {
    /// Access the Tauri `AppHandle`.
    pub fn app_handle(&self) -> &tauri::AppHandle<R> {
        &self.app_handle
    }

    /// Convenience: resolve the app data directory.
    pub fn data_dir(&self) -> Result<std::path::PathBuf, BootError> {
        self.app_handle
            .path()
            .app_data_dir()
            .map_err(|e| BootError::DataDir(e.to_string()))
    }

    /// Inject state into the Router so handlers can access it via
    /// `State<Arc<S>>`.
    ///
    /// State is immediately visible to handlers at call time because
    /// the Router resolves state lazily from the shared map.
    pub fn inject_state<S: Send + Sync + 'static>(&self, state: S) {
        let mut map = self.states.write().expect("state lock poisoned");
        map.insert(TypeId::of::<S>(), Arc::new(state));
    }

    /// Emit a progress event to the frontend.
    ///
    /// Increments the internal step counter and emits an
    /// `allframe-tauri:boot-progress` event with the step number, total, and label.
    pub fn emit_progress(&self, label: &str) {
        let step = self.current_step.fetch_add(1, Ordering::Relaxed) + 1;
        let payload = BootProgress {
            step,
            total: self.total_steps,
            label: label.to_string(),
        };
        if let Err(_e) = self.app_handle.emit(&boot_progress_event(), &payload) {
            #[cfg(debug_assertions)]
            eprintln!("allframe: failed to emit boot progress event: {_e}");
        }
    }
}

// ─── BootBuilder ────────────────────────────────────────────────────────────

/// Type-erased boot closure.
type BoxedBootFn<R> =
    Box<dyn FnOnce(BootContext<R>) -> Pin<Box<dyn Future<Output = Result<(), BootError>>>> + Send>;

/// Builder for creating a Tauri plugin with an optional async boot phase.
///
/// # Example
///
/// ```rust,ignore
/// allframe_tauri::builder(router)
///     .on_boot(2, |ctx| async move {
///         // async initialization...
///         Ok(())
///     })
///     .build()
/// ```
pub struct BootBuilder<R: Runtime> {
    router: Router,
    boot_fn: Option<BoxedBootFn<R>>,
    step_count: u32,
}

impl<R: Runtime> BootBuilder<R> {
    /// Returns `true` if an async boot closure has been registered.
    pub fn has_boot(&self) -> bool {
        self.boot_fn.is_some()
    }

    /// Returns the number of progress steps declared via `on_boot`.
    pub fn step_count(&self) -> u32 {
        self.step_count
    }
}

impl<R: Runtime> BootBuilder<R> {
    /// Create a new boot builder with the given router.
    pub fn new(router: Router) -> Self {
        Self {
            router,
            boot_fn: None,
            step_count: 0,
        }
    }

    /// Set the async boot closure.
    ///
    /// `steps` is the total number of progress steps (for the denominator
    /// in `BootProgress`). The closure receives a [`BootContext`] for state
    /// injection and progress reporting.
    ///
    /// The boot closure runs on a dedicated current-thread Tokio runtime
    /// inside Tauri's synchronous `setup()` hook. It blocks until completion,
    /// ensuring all state is ready before the UI renders.
    pub fn on_boot<F, Fut>(mut self, steps: u32, f: F) -> Self
    where
        F: FnOnce(BootContext<R>) -> Fut + Send + 'static,
        Fut: Future<Output = Result<(), BootError>> + Send + 'static,
    {
        self.step_count = steps;
        self.boot_fn = Some(Box::new(move |ctx| Box::pin(f(ctx))));
        self
    }

    /// Build the Tauri plugin.
    ///
    /// If [`on_boot`](Self::on_boot) was called, a current-thread Tokio
    /// runtime is created inside `setup()` to drive the async boot to
    /// completion before the app renders.
    pub fn build(self) -> TauriPlugin<R> {
        let BootBuilder {
            router,
            boot_fn,
            step_count,
        } = self;

        build_plugin(move |app_handle| {
            let mut router = router;
            router.inject_state(app_handle.clone());

            if let Some(boot) = boot_fn {
                let ctx = BootContext {
                    app_handle: app_handle.clone(),
                    states: router.shared_states(),
                    total_steps: step_count,
                    current_step: AtomicU32::new(0),
                };

                // Create a current-thread Tokio runtime for the boot phase.
                // This handles the "no reactor on macOS main thread" problem.
                let rt = tokio::runtime::Builder::new_current_thread()
                    .enable_all()
                    .build()
                    .map_err(|e| {
                        Box::new(BootError::Runtime(e.to_string()))
                            as Box<dyn std::error::Error>
                    })?;

                rt.block_on(boot(ctx)).map_err(|e| {
                    Box::new(e) as Box<dyn std::error::Error>
                })?;
                // rt drops here — boot runtime is ephemeral
            }

            app_handle.manage(TauriServer::new(router));
            app_handle.manage(Arc::new(ActiveStreams::new()));
            Ok(())
        })
    }
}

// ─── Tests ──────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use allframe_core::router::{Router, State};

    #[test]
    fn test_boot_progress_serialization() {
        let progress = BootProgress {
            step: 2,
            total: 3,
            label: "Projections ready".to_string(),
        };
        let json = serde_json::to_string(&progress).unwrap();
        assert!(json.contains("\"step\":2"));
        assert!(json.contains("\"total\":3"));
        assert!(json.contains("Projections ready"));
    }

    #[test]
    fn test_boot_error_serialization() {
        let err = BootError::Failed("store open failed".to_string());
        let json = serde_json::to_string(&err).unwrap();
        assert!(json.contains("store open failed"));
    }

    #[test]
    fn test_boot_error_display() {
        let err = BootError::Failed("oops".to_string());
        assert_eq!(err.to_string(), "Boot failed: oops");

        let err = BootError::DataDir("not found".to_string());
        assert_eq!(err.to_string(), "Could not resolve data directory: not found");

        let err = BootError::Runtime("failed".to_string());
        assert_eq!(err.to_string(), "Failed to create boot runtime: failed");
    }

    #[tokio::test]
    async fn test_boot_state_visible_to_handlers() {
        struct BootState {
            name: String,
        }

        let mut router = Router::new();

        // Register handler BEFORE state is injected (the whole point)
        router.register_with_state_only::<BootState, _, _>(
            "get_name",
            |state: State<Arc<BootState>>| async move { state.name.clone() },
        );

        // Simulate boot: inject state into the shared map
        {
            let states = router.shared_states();
            let mut map = states.write().unwrap();
            map.insert(
                TypeId::of::<BootState>(),
                Arc::new(BootState {
                    name: "booted".to_string(),
                }) as Arc<dyn std::any::Any + Send + Sync>,
            );
        }

        // Handler should see the injected state
        let server = TauriServer::new(router);
        let result = server.call_handler("get_name", "{}").await.unwrap();
        assert_eq!(result.result, "booted");
    }

    #[test]
    fn test_boot_builder_defaults() {
        let router = Router::new();
        let builder: BootBuilder<tauri::Wry> = BootBuilder::new(router);
        assert!(!builder.has_boot());
        assert_eq!(builder.step_count(), 0);
    }

    #[test]
    fn test_boot_builder_on_boot_configures() {
        let router = Router::new();
        let builder: BootBuilder<tauri::Wry> = BootBuilder::new(router)
            .on_boot(3, |_ctx| async move { Ok(()) });
        assert!(builder.has_boot());
        assert_eq!(builder.step_count(), 3);
    }

    /// Integration test: exercises the full boot lifecycle without Tauri.
    ///
    /// Simulates what `BootBuilder::build()` does internally:
    /// 1. Register handlers before state exists
    /// 2. Create a current-thread runtime (same as build() does)
    /// 3. Run boot closure that injects state via SharedStateMap
    /// 4. Verify handlers can access the injected state
    #[test]
    fn test_full_boot_lifecycle_without_tauri() {
        struct DbPool {
            url: String,
        }
        struct AppConfig {
            version: u32,
        }

        let mut router = Router::new();

        // Step 1: Register handlers BEFORE state is injected
        router.register_with_state_only::<DbPool, _, _>(
            "db_url",
            |db: State<Arc<DbPool>>| async move { db.url.clone() },
        );
        router.register_with_state_only::<AppConfig, _, _>(
            "version",
            |cfg: State<Arc<AppConfig>>| async move { format!("{}", cfg.version) },
        );

        // Step 2: Create a current-thread runtime (same as BootBuilder::build)
        let rt = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();

        // Step 3: Run async boot that injects state
        let states = router.shared_states();
        rt.block_on(async {
            // Simulate async work (e.g., opening a database)
            let pool = DbPool {
                url: "sqlite://app.db".to_string(),
            };
            let config = AppConfig { version: 42 };

            // Inject through SharedStateMap (same path as BootContext::inject_state)
            {
                let mut map = states.write().unwrap();
                map.insert(
                    TypeId::of::<DbPool>(),
                    Arc::new(pool) as Arc<dyn std::any::Any + Send + Sync>,
                );
                map.insert(
                    TypeId::of::<AppConfig>(),
                    Arc::new(config) as Arc<dyn std::any::Any + Send + Sync>,
                );
            }
        });
        // rt drops here — ephemeral, same as in BootBuilder::build

        // Step 4: Verify handlers see the boot-injected state
        let server = TauriServer::new(router);
        let rt2 = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .unwrap();
        let db_result = rt2.block_on(server.call_handler("db_url", "{}")).unwrap();
        assert_eq!(db_result.result, "sqlite://app.db");

        let ver_result = rt2.block_on(server.call_handler("version", "{}")).unwrap();
        assert_eq!(ver_result.result, "42");
    }
}
