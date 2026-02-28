//! AllFrame Tauri Desktop Example
//!
//! Minimal Tauri 2.x app demonstrating AllFrame's offline-first IPC:
//! - Router handlers exposed as Tauri commands
//! - No HTTP server needed
//! - Works fully offline
//!
//! Run with: cd examples/tauri-desktop && cargo tauri dev

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

use allframe_core::router::Router;

fn main() {
    let mut router = Router::new();

    // Register application handlers
    router.register("list_notes", || async {
        serde_json::json!([
            {"id": "note-1", "title": "Meeting Notes", "offline": true},
            {"id": "note-2", "title": "Architecture Design", "offline": true},
            {"id": "note-3", "title": "Sprint Planning", "offline": true}
        ])
        .to_string()
    });

    router.register("get_note", || async {
        serde_json::json!({
            "id": "note-1",
            "title": "Meeting Notes",
            "content": "Discussed offline-first architecture with AllFrame.\n\nKey points:\n- SQLite event store for persistence\n- TauriServer for in-process IPC\n- No network required",
            "created_at": "2025-01-15T10:00:00Z"
        })
        .to_string()
    });

    router.register("health_check", || async {
        serde_json::json!({
            "status": "ok",
            "offline": true,
            "backend": "sqlite"
        })
        .to_string()
    });

    tauri::Builder::default()
        .plugin(allframe_tauri::init(router))
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
