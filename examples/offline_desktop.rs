//! Offline Desktop Example
//!
//! Demonstrates AllFrame's offline-first architecture combining:
//! - SQLite event store (persistent, zero network deps)
//! - TauriServer for in-process handler dispatch
//! - Embedded MCP server for local LLM tool calling
//! - Offline circuit breaker for connectivity-aware dispatch
//!
//! Run with: cargo run --example offline_desktop --features "offline,resilience"

use allframe_core::cqrs::{Event, EventStore, EventTypeName, SqliteEventStoreBackend};
use allframe_core::resilience::{
    AlwaysOnlineProbe, ConnectivityProbe, ConnectivityStatus, OfflineCircuitBreaker,
};
use allframe_core::router::Router;
use allframe_mcp::McpServer;
use allframe_tauri::TauriServer;

// --- Domain: Note-taking app events ---

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
enum NoteEvent {
    Created { id: String, title: String },
    Updated { id: String, content: String },
}

impl EventTypeName for NoteEvent {}
impl Event for NoteEvent {}

// --- Connectivity probes ---

struct AlwaysOfflineProbe;

#[async_trait::async_trait]
impl ConnectivityProbe for AlwaysOfflineProbe {
    async fn check(&self) -> ConnectivityStatus {
        ConnectivityStatus::Offline
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("=== AllFrame Offline Desktop Demo ===\n");

    // --- 1. SQLite Event Store ---
    println!("1. SQLite Event Store (WAL mode)");
    let db_path = "/tmp/allframe_offline_demo.db";
    let backend = SqliteEventStoreBackend::<NoteEvent>::new(db_path).await?;
    let store = EventStore::with_backend(backend);

    store
        .append(
            "note-1",
            vec![NoteEvent::Created {
                id: "note-1".into(),
                title: "Meeting Notes".into(),
            }],
        )
        .await?;
    store
        .append(
            "note-1",
            vec![NoteEvent::Updated {
                id: "note-1".into(),
                content: "Discussed offline-first architecture".into(),
            }],
        )
        .await?;

    let events = store.get_events("note-1").await?;
    println!("   Stored {} events for note-1", events.len());
    for event in &events {
        println!("   - {:?}", event);
    }

    // --- 2. TauriServer (in-process dispatch) ---
    println!("\n2. TauriServer (in-process handler dispatch)");
    let mut router = Router::new();
    router.register("list_notes", || async {
        r#"[{"id":"note-1","title":"Meeting Notes"}]"#.to_string()
    });
    router.register("search", || async {
        r#"{"results":[],"query":"offline"}"#.to_string()
    });

    let tauri = TauriServer::new(router);
    println!("   {} handlers registered:", tauri.handler_count());
    for h in tauri.list_handlers() {
        println!("   - {}", h.name);
    }

    let result = tauri.call_handler("list_notes", "{}").await?;
    println!("   list_notes -> {}", result.result);

    // --- 3. Embedded MCP (local LLM tool calling) ---
    println!("\n3. Embedded MCP Server (no network port)");
    let mcp = McpServer::new();
    mcp.register_tool("create_note", |args| {
        Box::pin(async move {
            let title = args["title"].as_str().unwrap_or("Untitled");
            Ok(serde_json::json!({ "id": "note-2", "title": title, "status": "created" }))
        })
    });
    mcp.register_tool("search_notes", |args| {
        Box::pin(async move {
            let query = args["query"].as_str().unwrap_or("");
            Ok(serde_json::json!({ "results": [], "query": query }))
        })
    });

    println!("   Listening: {}", mcp.is_listening());
    println!(
        "   Tools: {:?}",
        mcp.list_tools()
            .iter()
            .map(|t| &t.name)
            .collect::<Vec<_>>()
    );

    let result = mcp
        .call_tool_local(
            "create_note",
            serde_json::json!({"title": "Created by LLM"}),
        )
        .await?;
    println!("   create_note -> {}", result);

    // --- 4. Offline Circuit Breaker ---
    println!("\n4. Offline Circuit Breaker (connectivity-aware dispatch)");

    // While offline: operations are queued, not executed
    let breaker = OfflineCircuitBreaker::new("sync", AlwaysOfflineProbe);

    let r1 = breaker
        .call(|| async { Ok::<_, String>("synced note-1") })
        .await;
    let r2 = breaker
        .call(|| async { Ok::<_, String>("synced note-2") })
        .await;
    println!("   r1 queued: {}, r2 queued: {}", r1.is_queued(), r2.is_queued());
    println!("   {} operations queued while offline", breaker.queued_count().await);

    // When online: operations execute immediately
    let breaker_online = OfflineCircuitBreaker::new("sync-online", AlwaysOnlineProbe);

    let r3 = breaker_online
        .call(|| async { Ok::<_, String>("synced note-3") })
        .await;
    println!("   r3 queued: {} (executed immediately)", r3.is_queued());
    println!("   Online queue: {} pending", breaker_online.queued_count().await);

    // Drain queued ops when connectivity returns
    breaker.drain().await?;
    println!("   After drain: {} pending", breaker.queued_count().await);

    // Cleanup
    std::fs::remove_file(db_path).ok();

    println!("\n=== All offline features working without network! ===");
    Ok(())
}
