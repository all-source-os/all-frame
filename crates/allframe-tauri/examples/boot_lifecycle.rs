//! Example: Async boot lifecycle for Tauri apps
//!
//! Demonstrates how to use `allframe_tauri::builder` with `on_boot` to run
//! async initialization before the app renders. This solves the common problem
//! where Tauri 2's `setup()` closure is synchronous but your app needs to:
//!
//! 1. Open an event store or database (async)
//! 2. Backfill projections from stored events (async)
//! 3. Initialize a command bus or other services
//! 4. Only then allow the frontend to render
//!
//! # Running this example
//!
//! This example runs without a Tauri runtime — it exercises the boot lifecycle
//! using the same `TauriServer` in-process dispatch pattern.
//!
//! ```sh
//! cargo run -p allframe-tauri --example boot_lifecycle
//! ```
//!
//! # Using in a real Tauri app
//!
//! ```rust,ignore
//! use allframe_core::router::Router;
//! use allframe_tauri::{builder, BootError};
//!
//! fn main() {
//!     let mut router = Router::new();
//!
//!     // Register handlers that depend on boot state.
//!     // State is resolved lazily — no need to inject before registration.
//!     router.register_with_state_only::<AppDatabase, _, _>(
//!         "query_notes",
//!         |db| async move { db.query_all().await },
//!     );
//!
//!     tauri::Builder::default()
//!         .plugin(
//!             builder(router)
//!                 .on_boot(3, |ctx| async move {
//!                     let db = AppDatabase::open(&ctx.data_dir()?).await
//!                         .map_err(|e| BootError::Failed(e.to_string()))?;
//!                     ctx.inject_state(db);
//!                     ctx.emit_progress("Database ready");
//!                     // ... more steps
//!                     Ok(())
//!                 })
//!                 .build(),
//!         )
//!         .run(tauri::generate_context!())
//!         .unwrap();
//! }
//! ```
//!
//! # Frontend: listening for boot progress
//!
//! ```typescript
//! import { listen } from "@tauri-apps/api/event";
//!
//! // Show a splash screen while boot runs
//! const unlisten = await listen("allframe:boot-progress", (event) => {
//!     const { step, total, label } = event.payload;
//!     updateSplashScreen(`${label} (${step}/${total})`);
//! });
//!
//! // Once boot completes, setup() returns and the window renders.
//! // You can safely call handlers now.
//! const notes = await invoke("plugin:allframe|allframe_call", {
//!     handler: "query_notes",
//!     args: {}
//! });
//! ```

use std::any::TypeId;
use std::sync::Arc;

use allframe_core::router::{Router, SharedStateMap, State};
use allframe_tauri::TauriServer;

// ─── Simulated app types ────────────────────────────────────────────────────

/// Simulates an async database connection.
struct AppDatabase {
    notes: Vec<String>,
}

impl AppDatabase {
    async fn open() -> Result<Self, String> {
        // In a real app: open SQLite, connect to AllSource, etc.
        println!("  [async] Opening database...");
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(Self {
            notes: vec![
                "Buy groceries".to_string(),
                "Review PR #42".to_string(),
                "Ship v0.1.20".to_string(),
            ],
        })
    }
}

/// Simulates a projection rebuilt from stored events.
struct NoteStats {
    total_notes: usize,
}

impl NoteStats {
    async fn rebuild_from(db: &AppDatabase) -> Result<Self, String> {
        println!("  [async] Rebuilding projections...");
        tokio::time::sleep(std::time::Duration::from_millis(30)).await;
        Ok(Self {
            total_notes: db.notes.len(),
        })
    }
}

// ─── Main ───────────────────────────────────────────────────────────────────

fn main() {
    println!("=== AllFrame Boot Lifecycle Example ===\n");

    // Step 1: Create router and register handlers BEFORE state exists.
    // Thanks to deferred state resolution, handlers declared now will
    // find their state at call time (after boot injects it).
    let mut router = Router::new();

    router.register_with_state_only::<AppDatabase, _, _>(
        "list_notes",
        |db: State<Arc<AppDatabase>>| async move {
            serde_json::to_string(&db.notes).unwrap()
        },
    );

    router.register_with_state_only::<NoteStats, _, _>(
        "note_count",
        |stats: State<Arc<NoteStats>>| async move {
            format!("{}", stats.total_notes)
        },
    );

    router.register("health", || async { r#"{"status":"ok"}"#.to_string() });

    println!("Registered {} handlers (state not yet injected)\n", router.handlers_count());

    // Step 2: Simulate what BootBuilder::build() does internally.
    //
    // In a real Tauri app, you'd write:
    //
    //   allframe_tauri::builder(router)
    //       .on_boot(2, |ctx| async move { ... })
    //       .build()
    //
    // Here we manually run the same pattern to demonstrate without Tauri.

    let states = router.shared_states();

    println!("Boot phase (current-thread Tokio runtime):");
    let rt = tokio::runtime::Builder::new_current_thread()
        .enable_all()
        .build()
        .expect("failed to create boot runtime");

    rt.block_on(async {
        // Boot step 1: Open database
        let db = AppDatabase::open().await.expect("db open failed");
        inject_state(&states, db);
        println!("  ✓ Step 1/2: Database opened");

        // Boot step 2: Rebuild projections from events
        // Access the just-injected database to rebuild
        let db_ref = {
            let map = states.read().unwrap();
            map.get(&TypeId::of::<AppDatabase>())
                .unwrap()
                .clone()
                .downcast::<AppDatabase>()
                .unwrap()
        };
        let stats = NoteStats::rebuild_from(&db_ref)
            .await
            .expect("rebuild failed");
        inject_state(&states, stats);
        println!("  ✓ Step 2/2: Projections rebuilt");
    });
    drop(rt); // Ephemeral runtime — same as BootBuilder::build()
    println!();

    // Step 3: Create TauriServer and call handlers.
    // In a real app, this happens automatically inside build().

    let server = TauriServer::new(router);
    println!(
        "TauriServer ready with {} handlers:\n",
        server.handler_count()
    );

    let rt = tokio::runtime::Runtime::new().unwrap();

    let notes = rt
        .block_on(server.call_handler("list_notes", "{}"))
        .unwrap();
    println!("  list_notes → {}", notes.result);

    let count = rt
        .block_on(server.call_handler("note_count", "{}"))
        .unwrap();
    println!("  note_count → {}", count.result);

    let health = rt
        .block_on(server.call_handler("health", "{}"))
        .unwrap();
    println!("  health     → {}", health.result);

    println!("\nBoot lifecycle complete.");
}

/// Helper: inject state into a SharedStateMap (same as BootContext::inject_state).
fn inject_state<S: Send + Sync + 'static>(states: &SharedStateMap, state: S) {
    let mut map = states.write().expect("state lock poisoned");
    map.insert(TypeId::of::<S>(), Arc::new(state));
}
