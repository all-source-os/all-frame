# UC-036: Offline-First and Offline-Only Optimizations

**GitHub Issue:** [#36](https://github.com/all-source-os/all-frame/issues/36)
**Status:** Specification
**Date:** 2026-02-28

## Context

AllFrame is being evaluated for **Tauri 2 desktop applications** (macOS) where the transport
layer is IPC, not HTTP. These deployments use AllFrame's `cqrs`, `di`, `resilience`, and
`security` modules with `default-features = false` — no router, no HTTP server.

The target use case is **offline-first** (works without internet, syncs when available) and
in some deployments **offline-only** (air-gapped environments, local LLMs via Ollama).

## Actors

- **Desktop User** — end user of a Tauri 2 desktop application
- **Developer** — Rust developer building an offline-first application with AllFrame
- **Local LLM** — an Ollama instance running on the same machine (no network)
- **Remote Service** — cloud API available intermittently (sync target)

---

## UC-036.1: Offline Event Store Backend (SQLite/redb)

### Summary
As a developer, I want an `EventStoreBackend` implementation backed by SQLite (or redb)
so that I can run event sourcing on desktop/mobile without a network database.

### Preconditions
- Feature flag `cqrs-sqlite` (or `cqrs-redb`) is enabled
- A writable filesystem path is available for the database file

### Flow
1. Developer creates a `SqliteEventStore` with a file path
2. `SqliteEventStore` opens/creates the database with WAL mode enabled
3. Developer wraps it in `EventStore::with_backend(sqlite_backend)`
4. Events are appended atomically with built-in ordering guarantees
5. Events are retrievable by aggregate ID and by version
6. Snapshots are persisted in the same database
7. `flush()` ensures WAL checkpoint for durability

### Acceptance Criteria
- `SqliteEventStoreBackend` implements `EventStoreBackend<E>` for any `E: Event`
- WAL mode is enabled by default for concurrent read/write from multiple windows
- Atomic event append — partial writes never corrupt the store
- `get_events_after(id, version)` returns only events after the given version
- `save_snapshot` / `get_latest_snapshot` persist to SQLite
- `stats()` returns accurate `BackendStats`
- No network dependencies in the dependency tree when using this backend
- Binary size impact < 500KB (SQLite is statically linked)

### Expected API
```rust
let backend = SqliteEventStoreBackend::new("path/to/events.db").await?;
let store = EventStore::with_backend(backend);
store.append("aggregate-1", vec![MyEvent::Created { id: "1".into() }]).await?;
```

---

## UC-036.2: Offline-Aware Resilience Patterns

### Summary
As a developer, I want resilience patterns that understand network unavailability
so that my desktop app queues operations for replay instead of failing after threshold.

### Preconditions
- Feature flag `resilience` is enabled
- A `ConnectivityProbe` implementation is provided (or default is used)

### Flow

#### UC-036.2a: ConnectivityProbe
1. Developer implements the `ConnectivityProbe` trait or uses the default
2. Probe reports `Online`, `Offline`, or `Degraded` status
3. Resilience decorators query the probe before executing operations

#### UC-036.2b: OfflineCircuitBreaker
1. Developer creates an `OfflineCircuitBreaker` with a `ConnectivityProbe`
2. When probe reports `Offline`, operations are queued (not failed)
3. When probe reports `Online`, queued operations drain automatically
4. Standard circuit breaker behavior applies when online

#### UC-036.2c: StoreAndForward
1. Developer wraps an async operation with `StoreAndForward`
2. When execution fails due to network, the operation is persisted locally
3. A background task monitors connectivity and retries persisted operations
4. Successfully replayed operations are removed from the queue
5. Failed replays remain queued with exponential backoff

### Acceptance Criteria
- `ConnectivityProbe` trait is injectable into all resilience decorators
- `OfflineCircuitBreaker` queues operations when offline, drains when online
- `StoreAndForward` persists operations to local storage (SQLite or in-memory)
- Queue ordering is preserved (FIFO)
- Replay failures don't block other queued operations
- Metrics track: queued count, replayed count, failed replay count

### Expected API
```rust
#[async_trait]
trait ConnectivityProbe: Send + Sync {
    async fn check(&self) -> ConnectivityStatus;
}

enum ConnectivityStatus {
    Online,
    Offline,
    Degraded { reason: String },
}

let probe = DefaultConnectivityProbe::new();
let cb = OfflineCircuitBreaker::new("sync-service", probe);
let saf = StoreAndForward::new(sqlite_queue, probe);
```

---

## UC-036.3: Local-First Projection Sync

### Summary
As a developer, I want projections to materialize locally and sync bidirectionally
when online so that the app works offline and merges state on reconnect.

### Preconditions
- A local event store is configured
- A `ConflictResolver` strategy is selected

### Flow
1. Events are appended to the local event store (offline-capable)
2. Projections rebuild from the local event store
3. When online, local events sync to a remote event store
4. Remote events sync down to the local store
5. Conflicts are resolved using the configured strategy
6. Projections rebuild from the merged event stream

### Acceptance Criteria
- `SyncEngine` trait abstracts bidirectional event sync
- `ConflictResolver` trait is pluggable with built-in strategies:
  - `LastWriteWins` — by timestamp
  - `AppendOnly` — no conflicts (all events are additive)
  - `Manual` — callback for custom resolution
- Sync is idempotent — replaying the same sync produces the same result
- Projection rebuild after sync is consistent
- Sync progress is trackable (events synced, pending, conflicts)

### Expected API
```rust
#[async_trait]
trait SyncEngine: Send + Sync {
    async fn push_events(&self, events: Vec<SyncableEvent>) -> Result<SyncResult, SyncError>;
    async fn pull_events(&self, since: SyncCursor) -> Result<Vec<SyncableEvent>, SyncError>;
    async fn sync(&self) -> Result<SyncReport, SyncError>;
}

#[async_trait]
trait ConflictResolver: Send + Sync {
    async fn resolve(&self, local: &[Event], remote: &[Event]) -> Vec<Event>;
}

let sync = SyncEngine::new(local_store, remote_store, LastWriteWins);
let report = sync.sync().await?;
```

---

## UC-036.4: Feature Flag — `offline` or `embedded`

### Summary
As a developer, I want a single feature flag that configures AllFrame for offline/embedded
use, excluding all network-dependent defaults.

### Preconditions
- None

### Flow
1. Developer adds `allframe-core = { features = ["offline"] }` to Cargo.toml
2. This implies: `cqrs-sqlite`, `di`, `resilience` (local variant), `security`
3. This excludes: HTTP client, Redis cache, OTLP exporter, TLS deps
4. The entire dependency tree compiles without `openssl`/`rustls`

### Acceptance Criteria
- `offline` feature flag exists in `allframe-core/Cargo.toml`
- Enabling `offline` does NOT pull in: `reqwest`, `redis`, `opentelemetry-otlp`,
  `tonic`, `hyper`, `rustls`, `openssl`
- `offline` implies: `cqrs` + SQLite backend, `di`, `security`
- Binary produced with `offline` feature has no network syscalls at startup
- `cargo tree` with `offline` shows no TLS or HTTP crates

### Expected Cargo.toml
```toml
[features]
offline = ["cqrs", "cqrs-sqlite", "di", "security"]
```

---

## UC-036.5: Embedded MCP Server Without Network

### Summary
As a developer, I want an in-process MCP channel where client and server live in the
same binary so that local LLMs can call tools without opening a network port.

### Preconditions
- AllFrame MCP module is available
- A local LLM (e.g., Ollama) is running in-process or via local socket

### Flow
1. Developer creates an `McpServer` with registered tools
2. For local LLM: `call_tool_local(name, args)` dispatches directly (no serialization)
3. For external LLM: `serve_stdio()` or `serve_sse()` opens a transport
4. Both paths use the same tool registry

### Acceptance Criteria
- `McpServer::call_tool_local()` dispatches without serialization overhead
- `McpServer::serve_stdio()` provides stdio transport for external tools
- Tool registry is shared between local and network paths
- No network port is opened when using local-only mode
- Round-trip latency for `call_tool_local` < 1ms for simple tools

### Expected API
```rust
let mcp = McpServer::new(router);

// In-process (zero-copy for local LLM)
let result = mcp.call_tool_local("run_skill", args).await?;

// External (stdio transport for Claude, etc.)
mcp.serve_stdio().await?;
```

---

## UC-036.6: DI Container — Lazy Initialization for Desktop Cold Start

### Summary
As a developer, I want lazy initialization in the DI container so that heavy services
initialize in the background while the UI renders immediately.

### Preconditions
- Feature flag `di` is enabled
- `#[di_container]` macro is used

### Flow
1. Developer marks heavy dependencies with `#[lazy]`
2. Container construction skips lazy bindings (fast startup)
3. Lazy bindings initialize on first `.get()` call
4. Developer can call `container.warm_up()` to eagerly initialize all lazy bindings
5. `warm_up()` runs all lazy initializations concurrently

### Acceptance Criteria
- `#[lazy]` attribute on DI bindings defers initialization
- Container construction with lazy bindings < 1ms (no I/O)
- First `.get()` on a lazy binding triggers initialization
- `warm_up()` initializes all lazy bindings concurrently
- Thread-safe: concurrent `.get()` calls don't cause double initialization
- Initialization errors propagate correctly on `.get()`

### Expected API
```rust
#[di_container]
struct AppContainer {
    #[lazy]
    embedding_engine: Arc<EmbeddingEngine>,

    #[eager]  // Default
    config: Arc<AppConfig>,
}

let container = AppContainer::build().await?;  // Only config initialized
// ... show UI ...
container.warm_up().await?;  // Now initialize embedding_engine
```

---

## UC-036.7: Saga Compensation with Local Rollback

### Summary
As a developer, I want saga compensation steps that support local file/DB rollback
so that desktop workflows can undo file writes and SQLite changes on failure.

### Preconditions
- Feature flag `cqrs` is enabled
- Saga is operating on local resources (files, SQLite)

### Flow
1. Developer defines a saga with steps that write to local files or SQLite
2. Before each step executes, a compensation checkpoint is created:
   - Files: original content is snapshotted
   - SQLite: a savepoint is created
3. If a later step fails, compensation runs in reverse:
   - Files: restored from snapshot
   - SQLite: rolled back to savepoint
4. Compensation checkpoints are cleaned up after saga completion

### Acceptance Criteria
- `FileSnapshot` compensation primitive: captures and restores file contents
- `SqliteSavepoint` compensation primitive: creates and rolls back savepoints
- Compensation runs in reverse step order (same as existing saga behavior)
- Partial compensation failures are reported but don't prevent other compensations
- Cleanup removes snapshots/savepoints after successful saga completion
- No resource leaks on saga success or failure

### Expected API
```rust
let saga = SagaDefinition::new("process_document")
    .add_step(WriteFileStep { path, content })       // compensate: restore original
    .add_step(UpdateDbStep { query })                 // compensate: ROLLBACK TO SAVEPOINT
    .add_step(CallLlmStep { prompt })                 // compensate: no-op (idempotent)
    .with_compensation(CompensationStrategy::LocalRollback);
```

---

## Priority Order

Based on implementation complexity and value delivery:

1. **UC-036.4** — Feature flag (foundation, enables everything else)
2. **UC-036.1** — SQLite event store (core offline capability)
3. **UC-036.2** — Offline resilience (essential for offline-first UX)
4. **UC-036.6** — Lazy DI (desktop cold start performance)
5. **UC-036.7** — Local saga compensation (desktop workflow safety)
6. **UC-036.3** — Projection sync (hardest problem, highest value)
7. **UC-036.5** — Embedded MCP (local LLM integration)

## Cross-Cutting Concerns

- **No OpenSSL/rustls dependency** when only offline features are used
- **Binary size budget**: offline-only build should add < 2MB over minimal build
- **Cold start target**: < 500ms to interactive UI (lazy DI enables this)
- **Data integrity**: WAL mode, atomic writes, saga compensation = zero data loss
