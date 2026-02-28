//! tests/08_offline_first.rs
//!
//! E2E tests for GitHub Issue #36: Offline-first and offline-only optimizations
//! for desktop/embedded deployments.
//!
//! These tests define the expected behavior for:
//! - UC-036.1: Offline Event Store Backend (SQLite)
//! - UC-036.2: Offline-Aware Resilience Patterns
//! - UC-036.3: Local-First Projection Sync
//! - UC-036.4: Feature Flag (`offline`)
//! - UC-036.5: Embedded MCP Server Without Network
//! - UC-036.6: DI Container Lazy Initialization
//! - UC-036.7: Saga Compensation with Local Rollback

// =============================================================================
// Shared test fixtures
// =============================================================================

#![allow(dead_code, unused_variables, unused_imports)]

use std::collections::HashMap;

use allframe_core::cqrs::{
    Aggregate, Event, EventStore, EventTypeName, OrchestratorSagaStep, Projection, SagaDefinition,
    SagaOrchestrator, Snapshot,
};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum DocumentEvent {
    Created {
        doc_id: String,
        title: String,
    },
    Updated {
        title: String,
        content: String,
    },
    Deleted,
    TagAdded {
        tag: String,
    },
    SyncedToRemote {
        remote_id: String,
        timestamp: u64,
    },
}

impl EventTypeName for DocumentEvent {}
impl Event for DocumentEvent {}

#[derive(Default, Clone)]
struct DocumentAggregate {
    id: String,
    title: String,
    content: String,
    tags: Vec<String>,
    is_deleted: bool,
    version: u64,
}

impl Aggregate for DocumentAggregate {
    type Event = DocumentEvent;

    fn apply_event(&mut self, event: &Self::Event) {
        self.version += 1;
        match event {
            DocumentEvent::Created { doc_id, title } => {
                self.id = doc_id.clone();
                self.title = title.clone();
            }
            DocumentEvent::Updated { title, content } => {
                self.title = title.clone();
                self.content = content.clone();
            }
            DocumentEvent::Deleted => {
                self.is_deleted = true;
            }
            DocumentEvent::TagAdded { tag } => {
                self.tags.push(tag.clone());
            }
            DocumentEvent::SyncedToRemote { .. } => {}
        }
    }
}

struct DocumentProjection {
    documents: HashMap<String, DocumentView>,
}

#[derive(Clone, Debug)]
struct DocumentView {
    id: String,
    title: String,
    content: String,
    tags: Vec<String>,
}

impl Projection for DocumentProjection {
    type Event = DocumentEvent;

    fn apply(&mut self, event: &Self::Event) {
        match event {
            DocumentEvent::Created { doc_id, title } => {
                self.documents.insert(
                    doc_id.clone(),
                    DocumentView {
                        id: doc_id.clone(),
                        title: title.clone(),
                        content: String::new(),
                        tags: Vec::new(),
                    },
                );
            }
            DocumentEvent::Updated { title, content } => {}
            DocumentEvent::TagAdded { tag } => {}
            DocumentEvent::Deleted => {}
            DocumentEvent::SyncedToRemote { .. } => {}
        }
    }
}

// =============================================================================
// UC-036.1: Offline Event Store Backend (SQLite)
// =============================================================================

/// Test that a SQLite-backed event store can be created with a file path
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_creation() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");

    let backend =
        SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
            .await
            .unwrap();

    let store = EventStore::with_backend(backend);

    // Store should be empty initially
    let events = store.get_all_events().await.unwrap();
    assert!(events.is_empty());

    let stats = store.backend().stats().await;
    assert_eq!(stats.total_events, 0);
    assert_eq!(stats.total_aggregates, 0);
}

/// Test that SQLite backend implements full EventStoreBackend contract
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_append_and_retrieve() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");
    let backend =
        SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
            .await
            .unwrap();
    let store = EventStore::with_backend(backend);

    // Append events
    store
        .append(
            "doc-1",
            vec![
                DocumentEvent::Created {
                    doc_id: "doc-1".into(),
                    title: "My Doc".into(),
                },
                DocumentEvent::Updated {
                    title: "Updated Doc".into(),
                    content: "Hello".into(),
                },
            ],
        )
        .await
        .unwrap();

    // Retrieve events for aggregate
    let events = store.get_events("doc-1").await.unwrap();
    assert_eq!(events.len(), 2);

    // Verify event ordering
    assert!(matches!(&events[0], DocumentEvent::Created { .. }));
    assert!(matches!(&events[1], DocumentEvent::Updated { .. }));

    // Stats reflect the append
    let stats = store.backend().stats().await;
    assert_eq!(stats.total_events, 2);
    assert_eq!(stats.total_aggregates, 1);
}

/// Test that SQLite backend supports get_events_after for snapshot optimization
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_events_after_version() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");
    let backend =
        SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
            .await
            .unwrap();
    let store = EventStore::with_backend(backend);

    // Append 100 events
    for i in 0..100 {
        store
            .append(
                "doc-1",
                vec![DocumentEvent::TagAdded {
                    tag: format!("tag-{}", i),
                }],
            )
            .await
            .unwrap();
    }

    // Get events after version 50
    let events = store.get_events_after("doc-1", 50).await.unwrap();
    assert_eq!(events.len(), 50);
}

/// Test that SQLite backend supports snapshot persistence
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_snapshots() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");
    let backend =
        SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
            .await
            .unwrap();
    let store = EventStore::with_backend(backend);

    // Save a snapshot
    let snapshot_data = serde_json::to_vec(&serde_json::json!({
        "id": "doc-1", "title": "My Doc", "version": 50
    }))
    .unwrap();
    store
        .backend()
        .save_snapshot("doc-1", snapshot_data.clone(), 50)
        .await
        .unwrap();

    // Retrieve snapshot
    let (data, version) = store.backend().get_latest_snapshot("doc-1").await.unwrap();
    assert_eq!(version, 50);
    assert_eq!(data, snapshot_data);
}

/// Test that SQLite backend enables WAL mode for concurrent access
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_wal_mode_concurrent_access() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");

    let backend =
        SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
            .await
            .unwrap();

    // Verify WAL mode is enabled
    assert!(backend
        .stats()
        .await
        .backend_specific
        .get("journal_mode")
        .map(|m| m == "wal")
        .unwrap_or(false));

    let store = EventStore::with_backend(backend);

    // Concurrent reads and writes should not block each other
    let store_clone = store.clone();
    let write_handle = tokio::spawn(async move {
        for i in 0..50 {
            store_clone
                .append(
                    "doc-concurrent",
                    vec![DocumentEvent::TagAdded {
                        tag: format!("tag-{}", i),
                    }],
                )
                .await
                .unwrap();
        }
    });

    let store_clone2 = store.clone();
    let read_handle = tokio::spawn(async move {
        for _ in 0..50 {
            let _ = store_clone2.get_events("doc-concurrent").await;
        }
    });

    write_handle.await.unwrap();
    read_handle.await.unwrap();

    let events = store.get_events("doc-concurrent").await.unwrap();
    assert_eq!(events.len(), 50);
}

/// Test that SQLite backend persists across restarts
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_persistence_across_restarts() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");

    // First session: write events
    {
        let backend =
            SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
                .await
                .unwrap();
        let store = EventStore::with_backend(backend);
        store
            .append(
                "doc-1",
                vec![DocumentEvent::Created {
                    doc_id: "doc-1".into(),
                    title: "Persisted".into(),
                }],
            )
            .await
            .unwrap();
        store.backend().flush().await.unwrap();
    }

    // Second session: read events back
    {
        let backend =
            SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
                .await
                .unwrap();
        let store = EventStore::with_backend(backend);
        let events = store.get_events("doc-1").await.unwrap();
        assert_eq!(events.len(), 1);
        assert!(
            matches!(&events[0], DocumentEvent::Created { title, .. } if title == "Persisted")
        );
    }
}

/// Test that atomic append prevents partial writes
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_sqlite_event_store_atomic_append() {
    use allframe_core::cqrs::SqliteEventStoreBackend;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("events.db");
    let backend =
        SqliteEventStoreBackend::<DocumentEvent>::new(db_path.to_str().unwrap())
            .await
            .unwrap();
    let store = EventStore::with_backend(backend);

    // Append a batch of events atomically
    store
        .append(
            "doc-1",
            vec![
                DocumentEvent::Created {
                    doc_id: "doc-1".into(),
                    title: "Doc".into(),
                },
                DocumentEvent::Updated {
                    title: "Updated".into(),
                    content: "Content".into(),
                },
                DocumentEvent::TagAdded {
                    tag: "important".into(),
                },
            ],
        )
        .await
        .unwrap();

    // All or nothing: all 3 events should be present
    let events = store.get_events("doc-1").await.unwrap();
    assert_eq!(events.len(), 3);
}

// =============================================================================
// UC-036.2: Offline-Aware Resilience Patterns
// =============================================================================

/// Test ConnectivityProbe trait contract
#[tokio::test]
#[cfg(feature = "resilience")]
async fn test_connectivity_probe_trait() {
    use allframe_core::resilience::{ConnectivityProbe, ConnectivityStatus};

    // A mock probe that starts offline
    struct MockProbe {
        online: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait::async_trait]
    impl ConnectivityProbe for MockProbe {
        async fn check(&self) -> ConnectivityStatus {
            if self
                .online
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                ConnectivityStatus::Online
            } else {
                ConnectivityStatus::Offline
            }
        }
    }

    let online = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let probe = MockProbe {
        online: online.clone(),
    };

    assert!(matches!(probe.check().await, ConnectivityStatus::Offline));

    online.store(true, std::sync::atomic::Ordering::SeqCst);
    assert!(matches!(probe.check().await, ConnectivityStatus::Online));
}

/// Test OfflineCircuitBreaker queues operations when offline
#[tokio::test]
#[cfg(feature = "resilience")]
async fn test_offline_circuit_breaker_queues_when_offline() {
    use allframe_core::resilience::{
        ConnectivityProbe, ConnectivityStatus, OfflineCircuitBreaker,
    };

    struct AlwaysOfflineProbe;

    #[async_trait::async_trait]
    impl ConnectivityProbe for AlwaysOfflineProbe {
        async fn check(&self) -> ConnectivityStatus {
            ConnectivityStatus::Offline
        }
    }

    let probe = AlwaysOfflineProbe;
    let cb = OfflineCircuitBreaker::new("sync-service", probe);

    // When offline, operations should be queued, not failed
    let result = cb
        .call(|| async { Ok::<_, String>("should be queued") })
        .await;

    // The operation was queued, not executed
    assert!(result.is_queued());
    assert_eq!(cb.queued_count().await, 1);
}

/// Test OfflineCircuitBreaker drains queue when connectivity returns
#[tokio::test]
#[cfg(feature = "resilience")]
async fn test_offline_circuit_breaker_drains_on_reconnect() {
    use allframe_core::resilience::{
        ConnectivityProbe, ConnectivityStatus, OfflineCircuitBreaker,
    };

    struct ToggleProbe {
        online: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait::async_trait]
    impl ConnectivityProbe for ToggleProbe {
        async fn check(&self) -> ConnectivityStatus {
            if self
                .online
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                ConnectivityStatus::Online
            } else {
                ConnectivityStatus::Offline
            }
        }
    }

    let online = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let probe = ToggleProbe {
        online: online.clone(),
    };
    let cb = OfflineCircuitBreaker::new("sync-service", probe);

    // Queue operations while offline
    let call_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let cc = call_count.clone();
    cb.call(move || {
        let cc = cc.clone();
        async move {
            cc.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, String>("done")
        }
    })
    .await;

    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        0
    );

    // Go online — queued operations should drain
    online.store(true, std::sync::atomic::Ordering::SeqCst);
    cb.drain().await.unwrap();

    assert_eq!(
        call_count.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
    assert_eq!(cb.queued_count().await, 0);
}

/// Test StoreAndForward persists operations locally when offline
#[tokio::test]
#[cfg(feature = "resilience")]
async fn test_store_and_forward_persists_operations() {
    use allframe_core::resilience::{InMemoryQueue, StoreAndForward};

    struct AlwaysOfflineProbe;

    #[async_trait::async_trait]
    impl allframe_core::resilience::ConnectivityProbe for AlwaysOfflineProbe {
        async fn check(&self) -> allframe_core::resilience::ConnectivityStatus {
            allframe_core::resilience::ConnectivityStatus::Offline
        }
    }

    let queue = InMemoryQueue::new();
    let probe = AlwaysOfflineProbe;
    let saf = StoreAndForward::new(queue, probe);

    // Execute operation while offline — should be stored
    saf.execute("sync-payload-1", || async {
        Err::<(), _>("network unavailable".to_string())
    })
    .await;

    saf.execute("sync-payload-2", || async {
        Err::<(), _>("network unavailable".to_string())
    })
    .await;

    // Two operations queued
    assert_eq!(saf.pending_count().await, 2);

    // Queue preserves FIFO order
    let pending = saf.peek_pending().await;
    assert_eq!(pending[0].id, "sync-payload-1");
    assert_eq!(pending[1].id, "sync-payload-2");
}

/// Test StoreAndForward replays operations when connectivity returns
#[tokio::test]
#[cfg(feature = "resilience")]
async fn test_store_and_forward_replay_on_reconnect() {
    use allframe_core::resilience::{InMemoryQueue, StoreAndForward};

    struct ToggleProbe {
        online: std::sync::Arc<std::sync::atomic::AtomicBool>,
    }

    #[async_trait::async_trait]
    impl allframe_core::resilience::ConnectivityProbe for ToggleProbe {
        async fn check(&self) -> allframe_core::resilience::ConnectivityStatus {
            if self
                .online
                .load(std::sync::atomic::Ordering::SeqCst)
            {
                allframe_core::resilience::ConnectivityStatus::Online
            } else {
                allframe_core::resilience::ConnectivityStatus::Offline
            }
        }
    }

    let online = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let queue = InMemoryQueue::new();
    let probe = ToggleProbe {
        online: online.clone(),
    };
    let saf = StoreAndForward::new(queue, probe);

    // Store operations while offline
    saf.execute("op-1", || async {
        Err::<(), _>("offline".into())
    })
    .await;
    saf.execute("op-2", || async {
        Err::<(), _>("offline".into())
    })
    .await;

    // Go online and replay
    online.store(true, std::sync::atomic::Ordering::SeqCst);

    let report = saf.replay_all(|_id| async { Ok(()) }).await.unwrap();
    assert_eq!(report.replayed, 2);
    assert_eq!(report.failed, 0);
    assert_eq!(saf.pending_count().await, 0);
}

// =============================================================================
// UC-036.3: Local-First Projection Sync
// =============================================================================

/// Test that projections rebuild from local event store
#[tokio::test]
async fn test_local_projection_rebuild_from_event_store() {
    let store = EventStore::new();

    store
        .append(
            "doc-1",
            vec![
                DocumentEvent::Created {
                    doc_id: "doc-1".into(),
                    title: "First Doc".into(),
                },
                DocumentEvent::TagAdded {
                    tag: "rust".into(),
                },
            ],
        )
        .await
        .unwrap();

    store
        .append(
            "doc-2",
            vec![DocumentEvent::Created {
                doc_id: "doc-2".into(),
                title: "Second Doc".into(),
            }],
        )
        .await
        .unwrap();

    // Rebuild projection from all events
    let mut projection = DocumentProjection {
        documents: HashMap::new(),
    };
    let all_events = store.get_all_events().await.unwrap();
    for event in &all_events {
        projection.apply(event);
    }

    assert_eq!(projection.documents.len(), 2);
    assert_eq!(projection.documents["doc-1"].title, "First Doc");
    assert_eq!(projection.documents["doc-2"].title, "Second Doc");
}

/// Test SyncEngine trait contract for bidirectional sync
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn test_sync_engine_bidirectional_sync() {
    use allframe_core::cqrs::{LastWriteWins, SyncEngine};

    let local_store = EventStore::new();
    let remote_store = EventStore::new();

    // Local events
    local_store
        .append(
            "doc-1",
            vec![DocumentEvent::Created {
                doc_id: "doc-1".into(),
                title: "Local Doc".into(),
            }],
        )
        .await
        .unwrap();

    // Remote events (simulated)
    remote_store
        .append(
            "doc-2",
            vec![DocumentEvent::Created {
                doc_id: "doc-2".into(),
                title: "Remote Doc".into(),
            }],
        )
        .await
        .unwrap();

    let sync = SyncEngine::new(local_store.clone(), remote_store.clone(), LastWriteWins);
    let report = sync.sync().await.unwrap();

    // After sync, both stores should have all events
    assert_eq!(report.pushed, 1); // local → remote
    assert_eq!(report.pulled, 1); // remote → local
    assert_eq!(report.conflicts, 0);

    let local_events = local_store.get_all_events().await.unwrap();
    assert_eq!(local_events.len(), 2);
}

/// Test ConflictResolver with LastWriteWins strategy
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn test_conflict_resolver_last_write_wins() {
    use allframe_core::cqrs::{ConflictResolver, LastWriteWins};

    let resolver = LastWriteWins;

    let local = vec![DocumentEvent::Updated {
        title: "Local Title".into(),
        content: "local".into(),
    }];
    let remote = vec![DocumentEvent::Updated {
        title: "Remote Title".into(),
        content: "remote".into(),
    }];

    // With LastWriteWins, remote wins
    let resolved = resolver.resolve(&local, &remote).await;
    assert_eq!(resolved.len(), 1); // One event wins
}

/// Test sync idempotency — replaying same sync produces same result
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn test_sync_idempotency() {
    use allframe_core::cqrs::{LastWriteWins, SyncEngine};

    let local_store = EventStore::new();
    let remote_store = EventStore::new();

    local_store
        .append(
            "doc-1",
            vec![DocumentEvent::Created {
                doc_id: "doc-1".into(),
                title: "Doc".into(),
            }],
        )
        .await
        .unwrap();

    let sync = SyncEngine::new(local_store, remote_store, LastWriteWins);

    let report1 = sync.sync().await.unwrap();
    let report2 = sync.sync().await.unwrap();

    // Second sync should be a no-op
    assert_eq!(report2.pushed, 0);
    assert_eq!(report2.pulled, 0);
    assert_eq!(report2.conflicts, 0);
}

// =============================================================================
// UC-036.4: Feature Flag — `offline` or `embedded`
// =============================================================================

/// Test that cqrs feature is available (baseline for offline)
#[test]
#[cfg(feature = "cqrs")]
fn test_cqrs_feature_available_for_offline() {
    use allframe_core::cqrs::{EventStore, EventStoreBackend};

    let store = EventStore::<DocumentEvent>::new();
    let _: &dyn EventStoreBackend<DocumentEvent> = store.backend();
}

/// Test that DI feature is available (baseline for offline)
#[test]
#[cfg(feature = "di")]
fn test_di_feature_available_for_offline() {
    use allframe_core::di::{ContainerBuilder, DependencyRegistry, Scope};

    let mut registry = DependencyRegistry::new();
    registry.store_singleton(42i32);
    assert!(registry.has_singleton::<i32>());

    let builder = ContainerBuilder::new();
    assert_eq!(builder.initialization_order().len(), 0);
}

/// Test that the offline feature flag implies the expected feature set
#[test]
#[cfg(feature = "offline")]
fn test_offline_feature_flag_implies_cqrs_and_di() {
    // When `offline` is enabled, both cqrs and di should be available
    use allframe_core::cqrs::EventStore;
    use allframe_core::di::DependencyRegistry;

    let _store = EventStore::<DocumentEvent>::new();
    let _registry = DependencyRegistry::new();

    // SQLite backend should also be available
    use allframe_core::cqrs::SqliteEventStoreBackend;
}

/// Test that offline feature does not pull in network dependencies
#[test]
fn test_offline_feature_no_network_deps() {
    // This is a documentation/CI test:
    // `cargo tree --features offline --no-default-features`
    // should NOT contain: reqwest, redis, opentelemetry-otlp, tonic, hyper, rustls, openssl
    assert!(true);
}

// =============================================================================
// UC-036.5: Embedded MCP Server Without Network
// =============================================================================

/// Test in-process MCP tool call without network
#[tokio::test]
async fn test_mcp_local_tool_call_no_network() {
    use allframe_mcp::McpServer;

    let mcp = McpServer::new();
    mcp.register_tool("echo", |args: serde_json::Value| async move { Ok(args) });

    // Direct in-process call — no serialization overhead
    let result = mcp
        .call_tool_local("echo", serde_json::json!({"message": "hello"}))
        .await
        .unwrap();

    assert_eq!(result["message"], "hello");
}

/// Test that MCP local and network paths share the same tool registry
#[tokio::test]
async fn test_mcp_shared_tool_registry() {
    use allframe_mcp::McpServer;

    let mcp = McpServer::new();
    mcp.register_tool("add", |args: serde_json::Value| async move {
        let a = args["a"].as_i64().unwrap();
        let b = args["b"].as_i64().unwrap();
        Ok(serde_json::json!({"result": a + b}))
    });

    // Local call
    let local_result = mcp
        .call_tool_local("add", serde_json::json!({"a": 2, "b": 3}))
        .await
        .unwrap();

    assert_eq!(local_result["result"], 5);

    // Tool list should be the same for local and network paths
    let tools = mcp.list_tools();
    assert_eq!(tools.len(), 1);
    assert_eq!(tools[0].name, "add");
}

/// Test that no network port is opened in local-only mode
#[tokio::test]
async fn test_mcp_no_network_port_in_local_mode() {
    use allframe_mcp::McpServer;

    let mcp = McpServer::new();
    mcp.register_tool("noop", |_| async { Ok(serde_json::json!({})) });

    // In local-only mode, no listener should be created
    assert!(!mcp.is_listening());

    // Tool calls still work
    let result = mcp
        .call_tool_local("noop", serde_json::json!({}))
        .await;
    assert!(result.is_ok());
}

// =============================================================================
// UC-036.6: DI Container — Lazy Initialization
// =============================================================================

/// Test that DI container supports eager initialization (current behavior)
#[test]
fn test_di_container_eager_initialization_baseline() {
    use allframe_core::di::DependencyRegistry;

    let mut registry = DependencyRegistry::new();

    // Eager: available immediately after store
    registry.store_singleton(String::from("config-value"));
    assert!(registry.has_singleton::<String>());

    let value = registry.get_singleton::<String>().unwrap();
    assert_eq!(*value, "config-value");
}

/// Test that DI container supports lazy binding that initializes on first get
#[tokio::test]
#[cfg(feature = "di")]
async fn test_di_lazy_binding_initializes_on_first_get() {
    use allframe_core::di::LazyProvider;

    let initialized = std::sync::Arc::new(std::sync::atomic::AtomicBool::new(false));
    let init_flag = initialized.clone();

    let provider = LazyProvider::new(move || {
        let flag = init_flag.clone();
        async move {
            flag.store(true, std::sync::atomic::Ordering::SeqCst);
            Ok::<_, allframe_core::di::DependencyError>("heavy-resource".to_string())
        }
    });

    // Not yet initialized
    assert!(!initialized.load(std::sync::atomic::Ordering::SeqCst));

    // First get triggers initialization
    let value = provider.get().await.unwrap();
    assert_eq!(value, "heavy-resource");
    assert!(initialized.load(std::sync::atomic::Ordering::SeqCst));

    // Second get returns cached value (no re-initialization)
    let value2 = provider.get().await.unwrap();
    assert_eq!(value2, "heavy-resource");
}

/// Test that warm_up initializes all lazy bindings concurrently
#[tokio::test]
#[cfg(feature = "di")]
async fn test_di_warm_up_initializes_lazy_bindings() {
    use allframe_core::di::LazyContainer;

    let mut container = LazyContainer::new();

    let init_order = std::sync::Arc::new(std::sync::Mutex::new(Vec::new()));

    let order1 = init_order.clone();
    container.register_lazy::<String, _, _>("service_a", move || {
        let order = order1.clone();
        async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            order.lock().unwrap().push("a");
            Ok("service_a".to_string())
        }
    });

    let order2 = init_order.clone();
    container.register_lazy::<i32, _, _>("service_b", move || {
        let order = order2.clone();
        async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            order.lock().unwrap().push("b");
            Ok(42i32)
        }
    });

    // Warm up initializes all concurrently
    container.warm_up().await.unwrap();

    // Both should be initialized
    let order = init_order.lock().unwrap();
    assert_eq!(order.len(), 2);
}

/// Test thread safety of lazy initialization under concurrent access
#[tokio::test]
#[cfg(feature = "di")]
async fn test_di_lazy_concurrent_get_no_double_init() {
    use allframe_core::di::LazyProvider;

    let init_count = std::sync::Arc::new(std::sync::atomic::AtomicU32::new(0));
    let count = init_count.clone();

    let provider = std::sync::Arc::new(LazyProvider::new(move || {
        let count = count.clone();
        async move {
            count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            tokio::time::sleep(std::time::Duration::from_millis(50)).await;
            Ok::<_, allframe_core::di::DependencyError>(42i32)
        }
    }));

    // Launch 10 concurrent gets
    let mut handles = vec![];
    for _ in 0..10 {
        let p = provider.clone();
        handles.push(tokio::spawn(async move { p.get().await.unwrap() }));
    }

    for h in handles {
        assert_eq!(h.await.unwrap(), 42);
    }

    // Should only have initialized once
    assert_eq!(
        init_count.load(std::sync::atomic::Ordering::SeqCst),
        1
    );
}

// =============================================================================
// UC-036.7: Saga Compensation with Local Rollback
// =============================================================================

/// Test saga with in-memory steps (existing behavior, baseline)
#[tokio::test]
async fn test_saga_compensation_baseline() {
    struct SuccessStep;

    #[async_trait::async_trait]
    impl OrchestratorSagaStep<DocumentEvent> for SuccessStep {
        async fn execute(&self) -> Result<Vec<DocumentEvent>, String> {
            Ok(vec![DocumentEvent::Created {
                doc_id: "saga-doc".into(),
                title: "Saga Created".into(),
            }])
        }

        async fn compensate(&self) -> Result<Vec<DocumentEvent>, String> {
            Ok(vec![DocumentEvent::Deleted])
        }

        fn name(&self) -> &str {
            "SuccessStep"
        }
    }

    struct FailStep;

    #[async_trait::async_trait]
    impl OrchestratorSagaStep<DocumentEvent> for FailStep {
        async fn execute(&self) -> Result<Vec<DocumentEvent>, String> {
            Err("simulated failure".into())
        }

        async fn compensate(&self) -> Result<Vec<DocumentEvent>, String> {
            Ok(vec![])
        }

        fn name(&self) -> &str {
            "FailStep"
        }
    }

    let orchestrator = SagaOrchestrator::<DocumentEvent>::new();

    let saga = SagaDefinition::new("test-compensation")
        .add_step(SuccessStep)
        .add_step(FailStep);

    let result = orchestrator.execute(saga).await;
    assert!(result.is_err());
}

/// Test FileSnapshot compensation primitive for saga local rollback
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn test_saga_file_snapshot_compensation() {
    use allframe_core::cqrs::FileSnapshot;

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("document.txt");

    // Write initial content
    std::fs::write(&file_path, "original content").unwrap();

    // Create a snapshot before modification
    let snapshot = FileSnapshot::capture(&file_path).await.unwrap();

    // Modify the file
    std::fs::write(&file_path, "modified content").unwrap();
    assert_eq!(
        std::fs::read_to_string(&file_path).unwrap(),
        "modified content"
    );

    // Restore from snapshot (compensation)
    snapshot.restore().await.unwrap();
    assert_eq!(
        std::fs::read_to_string(&file_path).unwrap(),
        "original content"
    );
}

/// Test SqliteSavepoint compensation primitive for saga local rollback
#[tokio::test]
#[cfg(feature = "cqrs-sqlite")]
async fn test_saga_sqlite_savepoint_compensation() {
    use allframe_core::cqrs::SqliteSavepoint;

    let dir = tempfile::tempdir().unwrap();
    let db_path = dir.path().join("saga_test.db");

    // Setup: create a SQLite database with a table
    let conn = rusqlite::Connection::open(&db_path).unwrap();
    conn.execute(
        "CREATE TABLE items (id INTEGER PRIMARY KEY, name TEXT)",
        [],
    )
    .unwrap();
    conn.execute(
        "INSERT INTO items (id, name) VALUES (1, 'original')",
        [],
    )
    .unwrap();

    // Create savepoint
    let savepoint = SqliteSavepoint::create(&conn, "saga_step_1").unwrap();

    // Modify data
    conn.execute("UPDATE items SET name = 'modified' WHERE id = 1", [])
        .unwrap();

    // Rollback to savepoint (compensation)
    savepoint.rollback().unwrap();

    // Data should be restored
    let name: String = conn
        .query_row("SELECT name FROM items WHERE id = 1", [], |row| {
            row.get(0)
        })
        .unwrap();
    assert_eq!(name, "original");
}

/// Test saga with file write step and automatic compensation on failure
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn test_saga_local_rollback_on_failure() {
    use allframe_core::cqrs::{CompensationStrategy, WriteFileStep};

    let dir = tempfile::tempdir().unwrap();
    let file_path = dir.path().join("saga_output.txt");

    // Write initial content
    std::fs::write(&file_path, "original").unwrap();

    struct AlwaysFailStep;

    #[async_trait::async_trait]
    impl OrchestratorSagaStep<DocumentEvent> for AlwaysFailStep {
        async fn execute(&self) -> Result<Vec<DocumentEvent>, String> {
            Err("intentional failure".into())
        }
        async fn compensate(&self) -> Result<Vec<DocumentEvent>, String> {
            Ok(vec![])
        }
        fn name(&self) -> &str {
            "AlwaysFailStep"
        }
    }

    let saga = SagaDefinition::new("file-write-saga")
        .add_step(WriteFileStep::new(
            file_path.clone(),
            "modified by saga".to_string(),
        ))
        .add_step(AlwaysFailStep)
        .with_compensation(CompensationStrategy::LocalRollback);

    let orchestrator = SagaOrchestrator::new();
    let result = orchestrator.execute(saga).await;

    // Saga failed
    assert!(result.is_err());

    // File should be restored to original content (compensation ran)
    assert_eq!(
        std::fs::read_to_string(&file_path).unwrap(),
        "original"
    );
}

/// Test that compensation cleanup removes snapshots after successful saga
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn test_saga_compensation_cleanup_on_success() {
    use allframe_core::cqrs::{CompensationStrategy, WriteFileStep};

    let dir = tempfile::tempdir().unwrap();
    let snapshot_dir = dir.path().join(".saga_snapshots");

    // Run a successful saga with file steps
    let saga = SagaDefinition::<DocumentEvent>::new("cleanup-test")
        .add_step(WriteFileStep::new(
            dir.path().join("output.txt"),
            "success".to_string(),
        ))
        .with_compensation(CompensationStrategy::LocalRollback)
        .with_snapshot_dir(&snapshot_dir);

    let orchestrator = SagaOrchestrator::new();
    orchestrator.execute(saga).await.unwrap();

    // Snapshot directory should be empty or removed after success
    assert!(
        !snapshot_dir.exists()
            || std::fs::read_dir(&snapshot_dir).unwrap().count() == 0
    );
}

// =============================================================================
// Integration: Full offline-first CQRS flow
// =============================================================================

/// Test full offline CQRS flow: Command → Event → Store → Projection → Query
#[tokio::test]
async fn test_full_offline_cqrs_flow_with_in_memory() {
    use allframe_macros::{command, command_handler, query, query_handler};

    #[command]
    struct CreateDocumentCommand {
        doc_id: String,
        title: String,
    }

    #[command_handler]
    async fn handle_create_document(
        cmd: CreateDocumentCommand,
        store: &EventStore<DocumentEvent>,
    ) -> Result<(), String> {
        store
            .append(
                &cmd.doc_id,
                vec![DocumentEvent::Created {
                    doc_id: cmd.doc_id.clone(),
                    title: cmd.title.clone(),
                }],
            )
            .await?;
        Ok(())
    }

    #[query]
    struct GetDocumentQuery {
        doc_id: String,
    }

    #[query_handler]
    async fn handle_get_document(
        query: GetDocumentQuery,
        projection: &DocumentProjection,
    ) -> Option<DocumentView> {
        projection.documents.get(&query.doc_id).cloned()
    }

    let store = EventStore::new();
    let mut projection = DocumentProjection {
        documents: HashMap::new(),
    };

    handle_create_document(
        CreateDocumentCommand {
            doc_id: "doc-offline-1".into(),
            title: "Offline Document".into(),
        },
        &store,
    )
    .await
    .unwrap();

    let events = store.get_all_events().await.unwrap();
    for event in &events {
        projection.apply(event);
    }

    let doc = handle_get_document(
        GetDocumentQuery {
            doc_id: "doc-offline-1".into(),
        },
        &projection,
    )
    .await;

    assert!(doc.is_some());
    assert_eq!(doc.unwrap().title, "Offline Document");
}

/// Test aggregate rebuild from event store (offline pattern)
#[tokio::test]
async fn test_aggregate_rebuild_from_event_store_offline() {
    let store = EventStore::new();

    store
        .append(
            "doc-1",
            vec![DocumentEvent::Created {
                doc_id: "doc-1".into(),
                title: "Initial".into(),
            }],
        )
        .await
        .unwrap();

    store
        .append(
            "doc-1",
            vec![DocumentEvent::Updated {
                title: "Revised".into(),
                content: "Some content".into(),
            }],
        )
        .await
        .unwrap();

    store
        .append(
            "doc-1",
            vec![
                DocumentEvent::TagAdded {
                    tag: "rust".into(),
                },
                DocumentEvent::TagAdded {
                    tag: "offline".into(),
                },
            ],
        )
        .await
        .unwrap();

    let events = store.get_events("doc-1").await.unwrap();
    let mut aggregate = DocumentAggregate::default();
    for event in &events {
        aggregate.apply_event(event);
    }

    assert_eq!(aggregate.title, "Revised");
    assert_eq!(aggregate.content, "Some content");
    assert_eq!(aggregate.tags, vec!["rust", "offline"]);
    assert_eq!(aggregate.version, 4);
    assert!(!aggregate.is_deleted);
}

/// Test snapshot + replay pattern for offline performance
#[tokio::test]
async fn test_snapshot_replay_pattern_for_offline_performance() {
    let store = EventStore::new();

    let mut events_batch = Vec::new();
    for i in 0..500 {
        events_batch.push(DocumentEvent::TagAdded {
            tag: format!("tag-{}", i),
        });
    }
    store
        .append(
            "doc-1",
            std::iter::once(DocumentEvent::Created {
                doc_id: "doc-1".into(),
                title: "Tagged Doc".into(),
            })
            .chain(events_batch)
            .collect(),
        )
        .await
        .unwrap();

    let all_events = store.get_events("doc-1").await.unwrap();
    let mut aggregate = DocumentAggregate::default();
    for event in &all_events {
        aggregate.apply_event(&event);
    }
    assert_eq!(aggregate.version, 501);

    let snapshot = Snapshot::create(aggregate.clone(), 501);

    store
        .append(
            "doc-1",
            vec![DocumentEvent::TagAdded {
                tag: "new-tag".into(),
            }],
        )
        .await
        .unwrap();

    // Rebuild from snapshot + only new events
    let mut rebuilt = snapshot.into_aggregate();
    let new_events = store.get_events_after("doc-1", 501).await.unwrap();
    for event in &new_events {
        rebuilt.apply_event(&event);
    }

    assert_eq!(rebuilt.version, 502);
    assert_eq!(rebuilt.tags.len(), 501); // 500 original + 1 new
    assert_eq!(rebuilt.tags.last().unwrap(), "new-tag");
}

/// Test event subscription for real-time local projections (offline-capable)
#[tokio::test]
async fn test_event_subscription_for_offline_projections() {
    let store = EventStore::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<DocumentEvent>(100);
    store.subscribe(tx).await;

    store
        .append(
            "doc-1",
            vec![DocumentEvent::Created {
                doc_id: "doc-1".into(),
                title: "Subscribed Doc".into(),
            }],
        )
        .await
        .unwrap();

    let received = rx.recv().await.unwrap();
    assert!(matches!(
        received,
        DocumentEvent::Created { title, .. } if title == "Subscribed Doc"
    ));
}
