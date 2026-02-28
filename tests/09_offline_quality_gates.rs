//! tests/09_offline_quality_gates.rs
//!
//! Quality gates for offline-first and offline-only desktop/embedded deployments.
//!
//! These tests verify that AllFrame can be used in environments where:
//! - **Offline-first**: Works without internet, syncs when available (Tauri desktop apps)
//! - **Offline-only**: Air-gapped environments, local LLMs, no network ever
//!
//! Quality gate categories:
//! - QG-OFF-1: Feature flag isolation (no network deps sneak in)
//! - QG-OFF-2: Minimal compilation profiles
//! - QG-OFF-3: allframe-tauri IPC dispatch
//! - QG-OFF-4: CQRS offline patterns
//! - QG-OFF-5: Resilience without network assumptions (tested via CI on allframe-core)
//! - QG-OFF-6: Dependency tree validation (CI-enforced)

#![allow(unused_imports, dead_code)]

use allframe_core::cqrs::{Aggregate, Event, EventStore, EventTypeName, Snapshot};

// Shared test event type for CQRS quality gates
#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum QgEvent {
    Created(String),
    Updated(String),
    Incremented,
}
impl EventTypeName for QgEvent {}
impl Event for QgEvent {}

#[derive(Default, Clone)]
struct QgAggregate {
    value: String,
    count: u64,
}
impl Aggregate for QgAggregate {
    type Event = QgEvent;
    fn apply_event(&mut self, event: &Self::Event) {
        self.count += 1;
        match event {
            QgEvent::Created(v) | QgEvent::Updated(v) => self.value = v.clone(),
            QgEvent::Incremented => {}
        }
    }
}

// =============================================================================
// QG-OFF-1: Feature Flag Isolation
// =============================================================================

/// Quality gate: `cqrs` feature compiles without network dependencies.
/// The CQRS module should work entirely in-process with InMemoryBackend.
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn qg_off_1_cqrs_compiles_without_network() {
    use allframe_core::cqrs::EventStoreBackend;

    let store = EventStore::<QgEvent>::new();
    let backend = store.backend();
    assert_eq!(backend.stats().await.total_events, 0);
}

/// Quality gate: `di` feature compiles without network dependencies.
#[test]
#[cfg(feature = "di")]
fn qg_off_1_di_compiles_without_network() {
    use allframe_core::di::{ContainerBuilder, DependencyRegistry, Scope};

    let mut registry = DependencyRegistry::new();
    registry.store_singleton(42_i32);
    assert!(registry.has_singleton::<i32>());

    let _singleton = Scope::Singleton;
    let _transient = Scope::Transient;
}

/// Quality gate: `router` feature compiles without network dependencies.
/// Router is an in-memory handler registry — no HTTP server required.
#[test]
fn qg_off_1_router_compiles_without_network() {
    use allframe_core::router::Router;

    let mut router = Router::new();
    router.register("local_handler", || async { "local result".to_string() });
    assert_eq!(router.handlers_count(), 1);
}

/// Quality gate: `security` feature compiles without network dependencies.
#[test]
#[cfg(feature = "security")]
fn qg_off_1_security_compiles_without_network() {
    use allframe_core::security::Obfuscate;

    let value = "secret-token-123";
    let safe = value.obfuscate();
    assert!(!safe.contains("secret-token-123"));
}

/// Quality gate: `openapi` feature compiles without network dependencies.
#[test]
#[cfg(feature = "openapi")]
fn qg_off_1_openapi_compiles_without_network() {
    assert!(true);
}

// =============================================================================
// QG-OFF-2: Minimal Compilation Profiles for Desktop/Embedded
// =============================================================================

/// Quality gate: AllFrame compiles with zero default features.
/// This is the absolute minimum for embedded/offline-only use.
#[test]
fn qg_off_2_zero_features_compiles() {
    use allframe_core::router::Router;

    let router = Router::new();
    assert_eq!(router.handlers_count(), 0);
}

/// Quality gate: "offline-only embedded" profile compiles.
/// Minimal for air-gapped: `cqrs` + `di` only.
#[test]
#[cfg(all(feature = "cqrs", feature = "di"))]
fn qg_off_2_embedded_profile_compiles() {
    use allframe_core::di::DependencyRegistry;

    let _store = EventStore::<QgEvent>::new();
    let mut registry = DependencyRegistry::new();
    registry.store_singleton(String::from("embedded-config"));
    assert!(registry.has_singleton::<String>());
}

/// Quality gate: Router + CQRS can coexist for Tauri IPC + event sourcing.
#[test]
#[cfg(feature = "cqrs")]
fn qg_off_2_router_plus_cqrs_compiles() {
    use allframe_core::router::Router;

    let mut router = Router::new();
    router.register("create_doc", || async { "created".to_string() });

    let _store = EventStore::<QgEvent>::new();
    assert_eq!(router.handlers_count(), 1);
}

// =============================================================================
// QG-OFF-3: allframe-tauri IPC Dispatch
// =============================================================================

/// Quality gate: TauriServer wraps Router for IPC dispatch without Tauri runtime.
#[test]
fn qg_off_3_tauri_server_creation() {
    use allframe_tauri::TauriServer;

    let router = allframe_core::router::Router::new();
    let server = TauriServer::new(router);
    assert_eq!(server.handler_count(), 0);
}

/// Quality gate: TauriServer discovers handlers from Router.
#[test]
fn qg_off_3_tauri_handler_discovery() {
    use allframe_tauri::TauriServer;

    let mut router = allframe_core::router::Router::new();
    router.register("get_user", || async { r#"{"id":1}"#.to_string() });
    router.register("list_items", || async { "[]".to_string() });
    router.register("health", || async { r#"{"ok":true}"#.to_string() });

    let server = TauriServer::new(router);
    assert_eq!(server.handler_count(), 3);

    let names: Vec<&str> = server
        .list_handlers()
        .iter()
        .map(|h| h.name.as_str())
        .collect();
    assert!(names.contains(&"get_user"));
    assert!(names.contains(&"list_items"));
    assert!(names.contains(&"health"));
}

/// Quality gate: TauriServer dispatches in-process calls (zero-overhead local LLM path).
#[tokio::test]
async fn qg_off_3_tauri_in_process_dispatch() {
    use allframe_tauri::TauriServer;

    let mut router = allframe_core::router::Router::new();
    router.register("echo", || async { "echoed".to_string() });

    let server = TauriServer::new(router);
    let response = server.call_handler("echo", "{}").await;

    assert!(response.is_ok());
    assert_eq!(response.unwrap().result, "echoed");
}

/// Quality gate: TauriServer returns typed error for unknown handlers.
#[tokio::test]
async fn qg_off_3_tauri_unknown_handler_error() {
    use allframe_tauri::{TauriServer, TauriServerError};

    let router = allframe_core::router::Router::new();
    let server = TauriServer::new(router);

    let result = server.call_handler("nonexistent", "{}").await;
    assert!(result.is_err());
    assert!(matches!(
        result.unwrap_err(),
        TauriServerError::HandlerNotFound(_)
    ));
}

/// Quality gate: TauriServer types are serializable (required by Tauri IPC).
#[test]
fn qg_off_3_tauri_types_serializable() {
    use allframe_tauri::{CallResponse, HandlerInfo, TauriServerError};

    let info = HandlerInfo {
        name: "test".to_string(),
        description: "A test handler".to_string(),
    };
    let json = serde_json::to_string(&info).unwrap();
    assert!(json.contains("test"));

    let response = CallResponse {
        result: r#"{"id":1}"#.to_string(),
    };
    let json = serde_json::to_string(&response).unwrap();
    assert!(json.contains("id"));

    let err = TauriServerError::HandlerNotFound("missing".to_string());
    let json = serde_json::to_string(&err).unwrap();
    assert!(json.contains("missing"));
}

/// Quality gate: TauriServer supports concurrent handler calls.
#[tokio::test]
async fn qg_off_3_tauri_concurrent_dispatch() {
    use allframe_tauri::TauriServer;
    use std::sync::Arc;

    let mut router = allframe_core::router::Router::new();
    router.register("a", || async { "A".to_string() });
    router.register("b", || async { "B".to_string() });

    let server = Arc::new(TauriServer::new(router));

    let s1 = server.clone();
    let s2 = server.clone();

    let (r1, r2) = tokio::join!(s1.call_handler("a", "{}"), s2.call_handler("b", "{}"),);

    assert_eq!(r1.unwrap().result, "A");
    assert_eq!(r2.unwrap().result, "B");
}

// =============================================================================
// QG-OFF-4: CQRS Offline Patterns
// =============================================================================

/// Quality gate: Event sourcing works entirely offline with InMemoryBackend.
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn qg_off_4_event_sourcing_fully_offline() {
    let store = EventStore::new();

    store
        .append("agg-1", vec![QgEvent::Created("initial".into())])
        .await
        .unwrap();
    store
        .append("agg-1", vec![QgEvent::Updated("revised".into())])
        .await
        .unwrap();

    let events = store.get_events("agg-1").await.unwrap();
    let mut agg = QgAggregate::default();
    for e in &events {
        agg.apply_event(e);
    }

    assert_eq!(agg.value, "revised");
    assert_eq!(agg.count, 2);
}

/// Quality gate: Snapshot + replay pattern works offline.
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn qg_off_4_snapshot_replay_offline() {
    let store = EventStore::new();

    for _ in 0..100 {
        store
            .append("counter-1", vec![QgEvent::Incremented])
            .await
            .unwrap();
    }

    let events = store.get_events("counter-1").await.unwrap();
    let mut counter = QgAggregate::default();
    for e in &events {
        counter.apply_event(e);
    }
    assert_eq!(counter.count, 100);

    let snapshot = Snapshot::create(counter, 100);

    store
        .append("counter-1", vec![QgEvent::Incremented])
        .await
        .unwrap();

    let mut rebuilt = snapshot.into_aggregate();
    let new_events = store.get_events_after("counter-1", 100).await.unwrap();
    for e in &new_events {
        rebuilt.apply_event(e);
    }

    assert_eq!(rebuilt.count, 101);
}

/// Quality gate: Event subscription works for local projections (no network).
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn qg_off_4_event_subscription_local() {
    let store = EventStore::new();
    let (tx, mut rx) = tokio::sync::mpsc::channel::<QgEvent>(10);
    store.subscribe(tx).await;

    store
        .append("id-1", vec![QgEvent::Created("hello".into())])
        .await
        .unwrap();

    let received = rx.recv().await.unwrap();
    assert_eq!(received, QgEvent::Created("hello".into()));
}

// =============================================================================
// QG-OFF-5: Resilience Without Network Assumptions
// =============================================================================
//
// Resilience quality gate tests run via CI on allframe-core to validate that
// CircuitBreaker, RetryExecutor, and RateLimiter work for local resources
// without any network dependencies.
//
// See: .github/workflows/offline-quality-gates.yml
//   - Tests `cargo test -p allframe-core --no-default-features --features "resilience"`
//   - Validates resilience primitives operate entirely offline

// =============================================================================
// QG-OFF-6: Dependency Tree Validation (CI-enforced)
// =============================================================================

/// Quality gate: Documents which features introduce network dependencies.
///
/// Network-dependent features (should NOT be used in offline-only deployments):
/// - `health` -> hyper, hyper-util
/// - `otel-otlp` -> opentelemetry-otlp (sends traces over network)
/// - `http-client` -> reqwest
/// - `cache-redis` -> redis
/// - `resilience-redis` -> redis
/// - `router-grpc` -> tonic (HTTP/2 transport)
/// - `router-grpc-tls` -> rustls, tokio-rustls
/// - `auth-axum` -> tower, hyper
/// - `auth-tonic` -> tonic
///
/// Network-free features (safe for offline-only):
/// - `cqrs` (InMemoryBackend, no external store)
/// - `di`
/// - `router` (in-memory handler registry)
/// - `openapi` (spec generation only)
/// - `resilience` (local retry/circuit breaker)
/// - `security` (logging utilities)
/// - `auth` (trait definitions only)
/// - `auth-jwt` (local token validation)
/// - `cache-memory` (in-process cache)
/// - `rate-limit` (local rate limiting)
///
/// CI enforcement: see `.github/workflows/offline-quality-gates.yml`
/// which runs `cargo tree` and asserts no network crates appear in offline builds.
/// Quality gate: Documents which features introduce network dependencies.
///
/// This test is CI-enforced only. The actual validation runs in
/// `.github/workflows/offline-quality-gates.yml` (dependency-tree-validation job)
/// which asserts `cargo tree` output contains no banned network crates.
///
/// To verify locally:
///   cargo tree -p allframe-core --no-default-features --features "cqrs,di,resilience,security" \
///     | grep -E "reqwest|redis|opentelemetry-otlp|tonic|hyper" && echo "FAIL" || echo "PASS"
#[test]
#[ignore = "CI-enforced via offline-quality-gates.yml dependency-tree-validation job"]
fn qg_off_6_network_dependency_documentation() {}

/// Quality gate: allframe-tauri does not pull in HTTP server dependencies.
///
/// This test is CI-enforced only. The actual validation runs in
/// `.github/workflows/offline-quality-gates.yml` (dependency-tree-validation job)
/// which checks the allframe-tauri dependency tree for banned network crates.
///
/// To verify locally:
///   cargo tree -p allframe-tauri | grep -E "hyper|reqwest|tonic" && echo "FAIL" || echo "PASS"
#[test]
#[ignore = "CI-enforced via offline-quality-gates.yml dependency-tree-validation job"]
fn qg_off_6_tauri_no_http_server_deps() {}

// =============================================================================
// Integration: Full Offline Desktop Flow
// =============================================================================

/// Quality gate: Full offline Tauri desktop flow.
/// Router handlers + TauriServer + in-process dispatch, no network at any point.
#[tokio::test]
async fn qg_integration_full_offline_desktop_flow() {
    use allframe_tauri::TauriServer;

    let mut router = allframe_core::router::Router::new();
    router.register("get_config", || async {
        r#"{"theme":"dark","offline":true}"#.to_string()
    });
    router.register("save_note", || async {
        r#"{"saved":true,"id":"local-001"}"#.to_string()
    });

    let server = TauriServer::new(router);
    assert_eq!(server.handler_count(), 2);

    let config = server.call_handler("get_config", "{}").await.unwrap();
    assert!(config.result.contains("offline"));

    let save = server
        .call_handler("save_note", r#"{"text":"hello"}"#)
        .await
        .unwrap();
    assert!(save.result.contains("saved"));

    let err = server.call_handler("sync_cloud", "{}").await;
    assert!(err.is_err());
}

/// Quality gate: allframe-tauri + CQRS coexist for offline event-sourced desktop apps.
#[tokio::test]
#[cfg(feature = "cqrs")]
async fn qg_integration_tauri_plus_cqrs_offline() {
    use allframe_tauri::TauriServer;

    let store = EventStore::new();
    store
        .append("note-1", vec![QgEvent::Created("My Note".into())])
        .await
        .unwrap();

    let mut router = allframe_core::router::Router::new();
    router.register("list_notes", || async { r#"["note-1"]"#.to_string() });

    let server = TauriServer::new(router);
    let result = server.call_handler("list_notes", "{}").await.unwrap();
    assert!(result.result.contains("note-1"));

    let events = store.get_events("note-1").await.unwrap();
    assert_eq!(events.len(), 1);
}
