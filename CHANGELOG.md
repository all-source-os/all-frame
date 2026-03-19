# Changelog

All notable changes to AllFrame will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

---

## [0.1.24] - 2026-03-19

### Fixed
- **Resilience macro tests** — Updated `#[retry]` and `#[rate_limited]` proc-macro test assertions to match Clean Architecture output format, fixing 2 pre-existing test failures in `allframe-macros`.

### Documentation
- **Release skill** — Added `allframe-mcp` to publish order and test set, clarified handling of pre-existing test failures.

---

## [0.1.23] - 2026-03-19

### Added
- **`register_handlers!` state variants** ([#56](https://github.com/all-source-os/all-frame/issues/56)) — Four new prefixes for bulk-registering handlers that receive injected state: `state`, `state args`, `state streaming`, and `state streaming args`. These expand to `register_with_state_only`, `register_with_state`, `register_streaming_with_state_only`, and `register_streaming_with_state` respectively.
- **`#[allframe_handler]` sync support** ([#56](https://github.com/all-source-os/all-frame/issues/56)) — The attribute macro now accepts sync (non-async) functions. Only `#[allframe_handler(streaming)]` still requires async, since `StreamSender` needs `.await`.

### Fixed
- **Clippy lints for Rust 1.94** — Resolved `manual_pattern_char_comparison`, `get_first`, `redundant_closure`, and `needless_borrow` warnings across `allframe-forge`, `allframe-mcp`, and `allframe-macros`.
- **`offline_desktop` example** — Gated behind required features to prevent build failures when features are not enabled.

### Changed
- **CI** — Use container image for Linux jobs, consolidate steps, fix rustup PATH and dependency tree false positives.

---

## [0.1.22] - 2026-03-19

### Fixed
- **Tauri 2 plugin name mismatch** ([#56](https://github.com/all-source-os/all-frame/issues/56)) — `PluginBuilder::new("allframe")` did not match the `allframe-tauri` identifier that `tauri_plugin::Builder` in `build.rs` derives from the crate name. Tauri 2's ACL resolved permissions under `allframe-tauri:allow-*` but runtime checked against `allframe`, causing every IPC call to fail with `"allframe.allframe_call not allowed. Plugin not found"`. Fixed by aligning the plugin name to `allframe-tauri` via a `PLUGIN_NAME` constant.

### Added
- **`PLUGIN_NAME` constant** — Public `allframe_tauri::PLUGIN_NAME` (`"allframe-tauri"`) ensures the plugin identifier stays in sync with the crate-derived ACL name.
- **`#[allframe_handler]` attribute macro** — Marks functions as AllFrame router handlers, suppressing false `dead_code` warnings the compiler emits because it cannot trace usage through `router.register("name", handler_fn)` closure chains. Also validates: function must be `async`; `streaming` handlers must have a `StreamSender` parameter; non-streaming handlers must not.
- **`register_handlers!` macro** — Bulk-registers handler functions where each is referenced by path, making usage visible to `rustc`'s dead-code analysis. Supports `args`, `streaming`, and `streaming args` prefixes.

### Changed
- **Event name prefix** — Streaming events changed from `allframe:stream:*` to `allframe-tauri:stream:*` and boot progress from `allframe:boot-progress` to `allframe-tauri:boot-progress` to match the corrected plugin name.
- **Frontend invoke pattern** — `plugin:allframe|*` → `plugin:allframe-tauri|*` in generated TypeScript client, examples, and documentation.
- **Capability identifier** — `allframe:default` → `allframe-tauri:default` in permission grants.

### Migration
- **Capabilities:** Change `"allframe:default"` to `"allframe-tauri:default"` in `src-tauri/capabilities/*.json`
- **Frontend:** Change `invoke("plugin:allframe|allframe_call", ...)` to `invoke("plugin:allframe-tauri|allframe_call", ...)`
- **Event listeners:** Change `listen("allframe:boot-progress", ...)` to `listen("allframe-tauri:boot-progress", ...)`
- **Stream events:** Change `allframe:stream:` prefix to `allframe-tauri:stream:`

---

## [0.1.21] - 2026-03-18

### Added
- **Async Boot Lifecycle** ([#42](https://github.com/all-source-os/all-frame/issues/42)) — New `BootBuilder` and `BootContext` for running async initialization inside Tauri 2's synchronous `setup()` closure. Handles the "no Tokio reactor on macOS main thread" problem once, correctly, for all apps.
  - **`allframe_tauri::builder(router)`** — Entry point returning a `BootBuilder` for configuring async boot
  - **`.on_boot(steps, closure)`** — Sets an async boot closure that receives a `BootContext` for state injection and progress reporting
  - **`BootContext::inject_state(state)`** — Injects state into the Router's shared map during boot; handlers resolve it lazily at call time
  - **`BootContext::emit_progress(label)`** — Emits `allframe:boot-progress` Tauri events (`{ step, total, label }`) for frontend splash screens
  - **`BootContext::data_dir()`** — Convenience for resolving the app data directory
  - **`BootProgress`** — Serializable progress event payload
  - **`BootError`** — Serializable error type (`Failed`, `DataDir`, `Runtime`)
  - Internally creates an ephemeral `tokio::runtime::Builder::new_current_thread()` runtime for `block_on` during setup
  - Fully backward compatible: `init(router)` and `init_with_state(router, state)` unchanged
  - **`boot_lifecycle` example** — Runnable example demonstrating the full boot-then-dispatch pattern without a Tauri runtime
- **`Router::shared_states()`** — Public accessor for the shared state map (`SharedStateMap`), enabling `BootContext` to inject state into the same map handlers read from
- **Deferred state resolution** ([#51](https://github.com/all-source-os/all-frame/issues/51)) — Handler state is now resolved at call time, not registration time. Handlers can be registered before their state type is injected (e.g., via `inject_state` during boot). `SharedStateMap` (`Arc<RwLock<HashMap<TypeId, Arc<dyn Any>>>>`) replaces the previous `Arc<dyn Any>` per-handler pattern.
- **CQRS macro trait implementations** — `#[command]`, `#[query]`, and `#[event]` macros now generate real `Command`, `Query`, `EventTypeName`, and `Event` trait implementations instead of placeholder comments.
- **QueryBus + Query/QueryHandler traits** — New `query_bus` module in `allframe-core` mirrors `CommandBus` for the read side of CQRS. Includes `QueryBus`, `Query`, `QueryHandler`, `QueryError`, and `QueryResult`.
- **Architecture layer enforcement** — `#[domain]`, `#[repository]`, `#[use_case]`, and `#[handler]` macros now detect and reject invalid cross-layer field dependencies at compile time using type-name heuristics.
- **`#[derive(Obfuscate)]` enum support** — The Obfuscate derive macro now supports enums with named fields, unnamed fields, and unit variants. Per-variant `#[sensitive]` field obfuscation.
- **Saga macro improvements** — `#[saga_step]` generates metadata constants (`STEP_NAME`, `STEP_TIMEOUT`, `STEP_REQUIRES_COMPENSATION`) instead of `todo!()` trait methods. `#[saga]` strips `#[inject]`/`#[saga_data]` attributes from output. `#[saga_workflow]` generates `Default::default()` step constructors.
- **Forge integration tests** — 12 new integration tests covering all 9 archetypes, missing config error handling, and project name validation.

### Fixed
- **Gateway template edition** — Fixed `edition = "2024"` (non-existent) to `edition = "2021"` in forge gateway template. Generated gateway projects now compile.
- **MSRV consistency** — Aligned `rust-version` across all 9 forge archetype templates to `"1.89"` (matching workspace MSRV). Basic template was missing `rust-version` entirely.
- **Forge error handling** — Replaced `.expect()` panics in `scaffolding.rs` with proper `anyhow` error returns for missing gateway/BFF config.

### Removed
- **Unused Tera dependency** — Removed `tera` from `allframe-forge` Cargo.toml (was listed but never imported).
- **Unused CLI parameters** — Removed `--brokers` and `--all-features` from `allframe ignite` (were accepted but silently ignored).
- **Unimplemented `forge` subcommand** — Removed `allframe forge` CLI command that always returned an error.

### Documentation
- **Resilience migration guide** — Added detailed migration examples to `#[retry]`, `#[circuit_breaker]`, and `#[rate_limited]` deprecation notices showing the Clean Architecture replacement.
- **DI heuristic documentation** — Documented the type-name heuristic used for automatic dependency detection in `#[di_container]`, including when to use explicit `#[depends(...)]`.
- **Fixed macro README** — Changed `#[api]` to `#[api_handler]` in allframe-macros README.

---

## [0.1.20] - 2026-03-18

### Fixed
- **Tauri 2 plugin permissions** ([#53](https://github.com/all-source-os/all-frame/issues/53)) — `allframe-tauri` now includes a `permissions/` directory with auto-generated command permissions and a `default.toml` that grants all AllFrame IPC commands. Previously, `invoke("plugin:allframe|allframe_call")` failed with "Plugin not found" because Tauri 2's security model requires explicit permission grants for all plugin commands.

### Added
- **`build.rs`** in `allframe-tauri` — Uses `tauri_plugin::Builder` to auto-generate permission TOML files for all 4 IPC commands (`allframe_list`, `allframe_call`, `allframe_stream`, `allframe_stream_cancel`).
- **`permissions/default.toml`** — Default permission set granting all AllFrame commands. Consuming apps get full access by adding `"allframe:default"` to their capabilities.

### Changed
- **MSRV bumped to 1.89** ([#54](https://github.com/all-source-os/all-frame/issues/54)) — Transitive dependencies (`async-graphql` 7.2.1, `darling` 0.23, `time` 0.3.47) now require rustc 1.88–1.89. Compatibility matrix CI updated.
- **Multi-state Router** ([#51](https://github.com/all-source-os/all-frame/issues/51)) — `Router.state: Option<Arc<dyn Any>>` replaced with `Router.states: HashMap<TypeId, Arc<dyn Any>>`. `with_state()` can now be called multiple times with different types. New `inject_state(&mut self, state)` for late injection.
- **AppHandle auto-injection** ([#51](https://github.com/all-source-os/all-frame/issues/51)) — `allframe_tauri::init()` automatically injects the Tauri `AppHandle<R>` as state during plugin setup. Handlers can access it via `State<Arc<AppHandle<R>>>`.
- `allframe-tauri` now has `links = "allframe-tauri"` in Cargo.toml (required by `tauri-plugin` build system).
- Added `tauri-plugin` as a build dependency.

---

## [0.1.19] - 2026-03-18

### Added
- **Streaming Handler Support** ([#52](https://github.com/all-source-os/all-frame/issues/52)) - Handlers can now send incremental updates during execution, enabling LLM token streaming, multi-step workflow progress, and real-time agent loops.
  - **`StreamSender`** - Bounded channel sender for streaming handlers with typed `send()` via `IntoStreamItem` trait
  - **`StreamReceiver`** - Receiver with `Drop`-based auto-cancellation of `CancellationToken`
  - **`StreamHandler` trait** - Parallel to `Handler` for streaming dispatch, with 4 handler struct variants (Fn, WithArgs, WithState, WithStateOnly)
  - **`CancellationToken` integration** - Automatic cancellation when receiver is dropped, plus explicit `cancel()` and `token.cancelled()` future for `tokio::select!`
  - **Router streaming registration** - `register_streaming`, `register_streaming_with_args`, `register_streaming_with_state`, `register_streaming_with_state_only`
  - **`register_stream*` adapters** - Bridge `impl Stream<Item = T>` to the channel-based `StreamSender` via `futures_core::Stream`
  - **`spawn_streaming_handler`** - Spawns streaming handler as a tokio task with `'static` lifetime for use in `tokio::spawn`
  - **`HandlerKind` enum** - `allframe_list` now reports whether each handler is `request_response` or `streaming`
  - **TypeScript streaming codegen** - Generated `callStreamHandler` helper with Tauri event listeners, `StreamObserver<T, F>` interface, `StreamSubscription` with `unsubscribe()`, auto-cleanup on complete/error
  - **RxJS adapter** - Generated `toObservable<T>()` function with lazy `import("rxjs")` (zero hard dependency)
  - **`#[tauri_compat(streaming)]`** - Macro variant for streaming handlers: generates args struct excluding `StreamSender` params, rewrites function signature
  - **Tauri IPC bridge** - `allframe_stream` command (spawns stream, returns `stream_id`, emits events) and `allframe_stream_cancel` command (aborts active stream via `AbortHandle`)
  - **`ActiveStreams` managed state** - Tracks active stream tasks for cancellation with automatic cleanup on completion
  - 30 integration tests across 4 test files (`10_streaming_core/router/tauri/codegen.rs`)

### Changed
- `StreamSender::channel()` now returns `(StreamSender, StreamReceiver)` instead of raw `mpsc::Receiver<String>`. The `StreamReceiver` auto-cancels the `CancellationToken` on drop.
- `describe_handler` and `describe_streaming_handler` now use `assert!` instead of `debug_assert!` — panics in release builds if handler doesn't exist.
- `HandlerInfo` gains `kind: HandlerKind` field (serializes as `"request_response"` or `"streaming"`).
- `TauriServer` re-exports core `StreamReceiver` instead of defining its own wrapper.

### Dependencies
- Added `tokio-util` (CancellationToken), `futures-core` (Stream trait), `uuid` (stream IDs) as dependencies
- Added `tokio-stream` as dev-dependency for stream adapter tests

---

## [0.1.18] - 2026-03-17

### Added
- **`#[tauri_compat]` Macro** - Attribute macro that transforms Tauri-style functions (individual params) into AllFrame handlers (single args struct). Generates `{FnName}Args` with `#[derive(Deserialize)]`, handles `Option<T>` with `#[serde(default)]`, and separates `State<...>` extractors automatically.
- **Typed Return Values** - Handlers can now return `impl Serialize` instead of `String`. New `register_typed*` methods auto-serialize return values to JSON. Eliminates manual `serde_json::to_string` at both ends.
- **`IntoHandlerResult` Trait** - Axum-style extensible return type system. Implementations for `String` (passthrough), `Json<T>` (auto-serialize), and `Result<T, E>` (serialize Ok, stringify Err). Collapses 10 handler structs into 4.
- **TypeScript Client Codegen** - `router.generate_ts_client()` produces typed async TS functions from handler metadata. Generated code unwraps `CallResponse` and `JSON.parse`s automatically. Deterministic, idempotent output.
- **`register_result_with_state`** and **`register_result_with_state_only`** - Previously missing Result+State handler variants, now free via `IntoHandlerResult`.

### Changed
- Handler architecture refactored: 10 structs collapsed to 4 generic structs (`HandlerFn`, `HandlerWithArgs`, `HandlerWithState`, `HandlerWithStateOnly`) parameterized over `R: IntoHandlerResult`. Net 200 lines removed.
- Removed `register_tauri_compat` (was a useless alias for `register_with_args`).

### Fixed
- **TS codegen `to_camel_case` bug** - `"GET_USER"` now correctly produces `"getUser"` (was `"gETUSER"`).
- **TS codegen CallResponse unwrapping** - Generated TS functions now unwrap `{ result: string }` and parse JSON instead of returning the raw `CallResponse` wrapper.
- **`describe_handler` validation** - `debug_assert!` verifies the handler exists when attaching type metadata.
- **Generated struct visibility** - `#[tauri_compat]` now matches field visibility to function visibility (was always `pub`).
- **Generated struct `#[allow(dead_code)]`** - Suppresses warnings when the generated args struct isn't directly referenced.

---

## [0.1.17] - 2026-03-01

### Added
- **Typed Handler Args** - `register_with_args()` deserializes JSON args into `T: DeserializeOwned` before calling the handler
- **State Injection** - `Router::with_state()` + `register_with_state()` / `register_with_state_only()` inject `State<Arc<S>>` into handlers
- **Args Forwarding** - `Router::call_handler()` now forwards the request string to handlers instead of ignoring it
- **Tauri DI Convenience** - `allframe_tauri::init_with_state(router, state)` for one-line Tauri plugin setup with DI
- **`State` in Prelude** - `use allframe_core::prelude::State` for ergonomic imports
- **MCP Typed Args** - MCP tools automatically gain typed arg support (zero code changes needed)

### Fixed
- State type mismatches return `Err` instead of panicking in the request path
- Prometheus metrics collector now uses real metric operations instead of no-op stubs

---

## [0.1.16] - 2026-02-28

### Added
- **Offline Desktop Example** (`examples/offline_desktop.rs`) - Standalone integration example combining SQLite event store, TauriServer, embedded MCP, and OfflineCircuitBreaker
- **Tauri Desktop Scaffold** (`examples/tauri-desktop/`) - Minimal Tauri 2.x app with IPC frontend for `cargo tauri dev`

### Fixed
- README roadmap section cleaned up (removed duplicated completed items)
- gRPC docs and allframe forge status updated to reflect actual implementation state

---

## [0.1.15] - 2026-02-28

### Added
- **Offline-First Support** (UC-036) - Complete offline-first architecture for desktop and embedded deployments
  - **SQLite Event Store Backend** (`cqrs-sqlite` feature) - WAL-mode SQLite backend for CQRS event sourcing
    - `SqliteEventStoreBackend<E>` implementing `EventStoreBackend<E>` trait
    - Atomic appends via transactions, WAL checkpointing via `flush()`
    - Zero network dependencies - fully embedded via `rusqlite` with bundled SQLite
    - 7 tests covering persistence, WAL mode, stats, and flush
  - **Offline Resilience Patterns** (`resilience` feature) - Fault tolerance for intermittent connectivity
    - `ConnectivityProbe` trait with `ConnectivityStatus` (Online/Offline/Degraded)
    - `OfflineCircuitBreaker` - queues operations when offline, drains on reconnect
    - `StoreAndForward` - persists operations for later replay with `ReplayReport`
    - `CallResult<T, E>` enum distinguishing executed vs queued operations
    - 5 tests for connectivity detection, queuing, and replay
  - **Projection Sync Engine** (`cqrs` feature) - Bidirectional event sync with conflict resolution
    - `SyncEngine` with pluggable `ConflictResolver` trait
    - Built-in resolvers: `LastWriteWins`, `AppendOnly`, `Manual` (user callback)
    - `SyncCursor` tracking for idempotent sync operations
    - 3 tests for bidirectional sync, idempotency, and conflict resolution
  - **Lazy DI Initialization** (`di` feature) - Deferred dependency initialization
    - `LazyProvider<T>` using `tokio::sync::OnceCell` for thread-safe single-init
    - `LazyContainer` with concurrent warm-up via `tokio::spawn`
    - 3 tests for lazy init, concurrent safety, and bulk warm-up
  - **Saga Compensation Primitives** (`cqrs` feature) - Local rollback for offline sagas
    - `FileSnapshot` with `capture()` and `restore()` for file-level rollback
    - `WriteFileStep` implementing `SagaStep<E>` with automatic snapshot management
    - `CompensationStrategy::LocalRollback` for saga definitions
    - `SqliteSavepoint` for database-level rollback (requires `cqrs-sqlite`)
    - 4 tests for file snapshots, compensation cleanup, and savepoint creation
  - **Embedded MCP Server** - Local-only MCP without network binding
    - `McpServer::new()` creates a server with no router (local-only mode)
    - `register_tool()` and `call_tool_local()` for in-process tool dispatch
    - `is_listening()` to check network binding status
    - 3 tests for local registration, shared registry, and no-network verification
  - **Feature Flags** - Zero-bloat offline compilation
    - `cqrs-sqlite` - SQLite event store (implies `cqrs` + `rusqlite`)
    - `offline` - Full offline bundle (implies `cqrs` + `cqrs-sqlite` + `di` + `security`)
    - Verified: zero network dependencies (`reqwest`, `redis`, `tonic`, `hyper`) in offline builds
    - 2 tests for feature flag verification

- **New Crate: `allframe-tauri`** - Tauri 2.x plugin for offline-first desktop apps
  - IPC handler dispatch for Tauri commands
  - `TauriServer` for in-process handler execution
  - Serializable request/response types for Tauri IPC
  - Full integration with AllFrame's router and CQRS systems

- **Offline Quality Gates** - CI enforcement for offline guarantees
  - New `offline-quality-gates.yml` GitHub Actions workflow
  - Dependency tree validation (bans network crates in offline builds)
  - 19 quality gate tests (+ 2 CI-enforced)
  - Binary size check (<5MB for offline builds)

### Changed
- **`McpServer::new(router)`** renamed to **`McpServer::with_router(router)`**
  - `McpServer::new()` now creates a local-only server with no router
  - `list_tools()` changed from async to sync
- **Resilience orchestrator** updated with `FnMut` support and `DashMap`-based state management
- **Total test count** now 500+ (was 455+)

### Dependencies
- Added `rusqlite` 0.31 (bundled, optional via `cqrs-sqlite` feature)

---

## [0.1.12] - 2025-12-15

### Added
- **MCP Server Debuggability** - Production-ready stdio transport with comprehensive debugging
  - `StdioTransport` - New production-ready stdio transport with graceful shutdown
  - `StdioConfig` - Builder-style configuration for MCP servers
  - Built-in `allframe/debug` tool - Claude can call to get server diagnostics (uptime, request count, tool count, PID)
  - Request/response tracing for debugging MCP protocol issues
  - Graceful shutdown handling (SIGTERM/SIGINT on Unix, Ctrl+C on Windows)
  - Notification handling (`initialized`, `notifications/cancelled`)
  - Optional `tracing` feature for structured logging with tracing-subscriber

- **Environment Variables for MCP Debugging**
  - `ALLFRAME_MCP_DEBUG=1` - Enable debug output to stderr
  - `ALLFRAME_MCP_LOG_FILE=/path/to/file` - Write logs to file instead of stderr
  - `RUST_LOG=debug` - Log level control (when using `tracing` feature)

- **Claude Code Setup Guide** - Comprehensive documentation for Claude Code CLI
  - Step-by-step `.mcp.json` configuration
  - `.claude/settings.local.json` setup with `enabledMcpjsonServers`
  - Troubleshooting guide for connection issues
  - Manual testing commands for debugging

### Deprecated
- **`cqrs-postgres` feature flag** - AllSource v0.10.3 deprecated the upstream `postgres` feature
  in favor of WAL-backed event-sourced repositories. Use `cqrs-allsource` instead.
  This feature flag will be removed in a future release.

### Fixed
- **AllSource Core Re-enabled** - `cqrs-allsource` features now work
  - Upstream fix in allsource-core 0.7.2 resolved `http ^1.2.0` dependency conflict
  - `cqrs-allsource`, `cqrs-postgres`, `cqrs-rocksdb` features are functional again
  - Updated to allsource-core 0.7.2

### Documentation
- Added "Quick Start with Claude Code (CLI)" section to MCP README
- Added debugging section with environment variables table
- Added `StdioTransport` and `StdioConfig` to API Overview
- Updated `ALLSOURCE_CORE_ISSUES.md` to mark issue as resolved
- Added social media content for MCP launch (`docs/social/`)

---

## [0.1.9] - 2025-12-13

### Added
- **Protocol-Agnostic Routing Complete** - Full multi-protocol support
  - REST adapter with path parameters and HTTP methods
  - GraphQL adapter with queries, mutations, and schema generation
  - gRPC adapter with all streaming modes and proto generation
  - Single handler exposed via multiple protocols
  - Automatic schema generation (OpenAPI, GraphQL SDL, .proto)
  - Protocol-specific error handling
  - 78 tests across 5 routing phases

- **MCP Server (Zero-Bloat)** - Separate crate for LLM tool integration
  - Auto-discovery: Handlers automatically become MCP tools
  - JSON Schema generation and validation
  - Type coercion (string → number, boolean)
  - Tool listing and invocation
  - Claude Desktop integration ready
  - 37 tests in allframe-mcp crate

- **`KeyedCircuitBreaker<K>`** - Generic keyed circuit breaker for per-resource isolation
  - Independent circuit breaker state per key (e.g., per exchange, per endpoint)
  - Failures in one resource don't affect others
  - Same API as `KeyedRateLimiter<K>` for consistency
  - Full statistics and reset capabilities per key

- **`resilience-redis` Feature** - Redis-backed distributed rate limiting
  - `RedisRateLimiter` with sliding window algorithm
  - `KeyedRedisRateLimiter` with per-key configuration
  - Atomic operations via Lua scripts
  - Works across multiple instances for distributed deployments
  - Auto-cleanup of expired entries

- **Layered Authentication** - Zero-bloat, feature-gated auth infrastructure
  - `auth`: Core traits only (`Authenticator`, `AuthContext`, `AuthError`) - zero deps
  - `auth-jwt`: JWT validation with HS256/RS256/EdDSA support via `jsonwebtoken`
  - `auth-axum`: Axum extractors (`AuthenticatedUser<C>`) and middleware (`AuthLayer`)
  - `auth-tonic`: gRPC interceptors (`AuthInterceptor`) for tonic services
  - Protocol-agnostic design - same claims work across REST/GraphQL/gRPC
  - Environment-based configuration (`JwtConfig::from_env()`)

### Changed
- Updated workspace version to 0.1.9
- Total test count now 455+ (was 361+)
- **Upgraded `thiserror` from 1.0 to 2.0** - Resolves version conflicts with downstream crates

---

## [0.1.8] - 2025-12-09

### Added
- **Graceful Shutdown Utilities** - Production-ready shutdown handling
  - `ShutdownAwareTaskSpawner` for named tasks with automatic cancellation
  - `spawn()` - Spawn tasks that respond to shutdown signals
  - `spawn_background()` - Background tasks (non-blocking)
  - `spawn_with_result()` - Tasks that return values
  - `GracefulShutdownExt` trait for cleanup orchestration
  - `ShutdownExt` trait for making any future cancellable
  - 17 tests for shutdown utilities

### Documentation
- Added `graceful_shutdown.rs` example
- Added `shutdown_patterns.rs` with 5 common patterns

---

## [0.1.7] - 2025-12-08

### Added
- **Comprehensive Resilience Module** (`resilience` feature) - ~1,000+ lines replacing kraken-gateway code
  - `RetryExecutor` - Async retry with exponential backoff and jitter
  - `RetryConfig` - Configurable retry behavior (max_retries, intervals, randomization)
  - `RetryPolicy` trait - Custom retry decision logic
  - `RetryBudget` - System-wide retry token management to prevent retry storms
  - `AdaptiveRetry` - Adjusts retry behavior based on success/failure rates
  - `RateLimiter` - Token bucket rate limiting with burst support
  - `AdaptiveRateLimiter` - Backs off when receiving external 429 responses
  - `KeyedRateLimiter<K>` - Per-key rate limiting (per-endpoint, per-user)
  - `CircuitBreaker` - Fail-fast pattern with Closed/Open/HalfOpen states
  - `CircuitBreakerConfig` - Configurable thresholds and timeouts
  - `CircuitBreakerManager` - Manages multiple circuit breakers by name

- **Security Module** (`security` feature) - Safe logging utilities
  - `obfuscate_url()` - Strips credentials, path, and query from URLs
  - `obfuscate_redis_url()` - Preserves host/port, hides auth
  - `obfuscate_api_key()` - Shows prefix/suffix only (e.g., "sk_l***mnop")
  - `obfuscate_header()` - Smart header obfuscation (Authorization, Cookie, etc.)
  - `Obfuscate` trait - Custom obfuscation for user types
  - `Sensitive<T>` wrapper - Debug/Display always shows "***"
  - `#[derive(Obfuscate)]` macro - Auto-generate Obfuscate impl with `#[sensitive]` fields

- **New Procedural Macros** (allframe-macros)
  - `#[derive(Obfuscate)]` - Auto-generate safe logging with `#[sensitive]` field attribute
  - `#[retry(max_retries = 3)]` - Wrap async functions with exponential backoff
  - `#[circuit_breaker(failure_threshold = 5)]` - Fail-fast pattern for functions
  - `#[rate_limited(rps = 100, burst = 10)]` - Token bucket rate limiting

- **New Feature Flags for Reduced Dependencies** - Modular feature flags
  - `router-grpc-tls` - TLS/mTLS support for gRPC (tonic/tls-ring, rustls-pemfile, tokio-rustls)
  - `http-client` - Re-exports reqwest for HTTP client functionality
  - `otel-otlp` - Full OpenTelemetry stack with OTLP exporter
  - `metrics` - Prometheus metrics support
  - `cache-memory` - In-memory caching with moka and dashmap
  - `cache-redis` - Redis client for distributed caching
  - `rate-limit` - Basic governor re-export
  - `resilience` - Full resilience module (retry, circuit breaker, rate limiting)
  - `security` - URL/credential obfuscation utilities
  - `utils` - Common utilities bundle (chrono, url, parking_lot, rand)

### Changed
- **`health` feature now optional** - `hyper` and `hyper-util` gated behind `health` feature (still in default)
- **CQRS feature flags** - Added `cqrs-allsource`, `cqrs-postgres`, `cqrs-rocksdb` for AllSource backends
- **Legacy alias** - `grpc-tls` is now deprecated, use `router-grpc-tls` instead

### Documentation
- Updated FEATURE_FLAGS.md with comprehensive documentation for all new features

---

## [0.1.6] - 2025-12-06

### Changed
- **allsource-core upgraded to 0.7.0** - Updated CQRS backend to use latest allsource-core API
  - Migrated to new Event API with `Event::from_strings()` for validated event creation
  - Updated field names (`data` → `payload`)
  - Converted async methods to sync where appropriate
  - Updated `StoreStats` fields (total_entities, total_event_types)

### Fixed
- **Event trait bounds** - Added `Serialize + DeserializeOwned` bounds for proper event serialization

---

## [0.1.5] - 2025-12-06

### Added
- **Zero-warning templates** - `allframe ignite` now generates projects that compile with zero warnings
- **Working Clean Architecture example** - Generated projects include functional Greeter example
  - `Greeter` trait in domain layer
  - `GreetingService` in application layer
  - `ConsoleGreeter` in infrastructure layer
- **Unit tests in templates** - Generated projects include passing tests demonstrating mocking
- **New integration test** - `ignite_creates_project_with_zero_warnings` verifies builds with `-D warnings`

### Fixed
- **Clippy warnings** - Fixed derive for Default, collapsible str::replace calls
- **Template unused imports** - Removed unused `pub use` re-exports from module files

---

## [0.1.4] - 2025-12-06

### Added
- **CLI binary in root crate** - `cargo install allframe` now works correctly
  - Added `allframe-forge` as dependency to root crate
  - Created `src/bin/allframe.rs` binary wrapper
  - Exposed `run()` function from allframe-forge library

### Fixed
- **Graceful shutdown utilities** - Added `GracefulShutdown` and `ShutdownSignal` types
- **`#[derive(GrpcError)]` macro** - Automatic tonic::Status conversion
- **Enhanced `#[traced]` macro** - Configuration options (name, skip, ret, err, level)

---

## [0.1.3] - 2025-12-05

### Added
- Initial crates.io publishing
- allframe-forge CLI for project scaffolding
- allframe-mcp for MCP server integration

---

## [Unreleased]

### Added - Scalar Integration (2025-12-01)

#### Beautiful API Documentation
- **Scalar UI Integration** - Modern OpenAPI 3.1 documentation interface
  - <50KB bundle size (10x smaller than Swagger UI!)
  - Dark mode by default with custom theming support
  - Interactive "Try It" functionality for all endpoints
  - Mobile-friendly responsive design

#### Core Features
- `OpenApiServer` struct for server configuration
- `OpenApiGenerator::with_server()` method for adding server endpoints
- `ScalarConfig` for comprehensive UI customization
  - CDN version pinning for stability
  - SRI (Subresource Integrity) hash support for security
  - Fallback CDN configuration for resilience
  - CORS proxy support for browser compatibility
  - Custom CSS and theming
- `scalar_html()` function for generating documentation HTML

#### Examples
- New `scalar_docs.rs` example (175 lines)
  - Demonstrates all Scalar features
  - Shows 6 REST endpoints with full documentation
  - Includes 3 server configurations
  - Framework integration guide for Axum/Actix/Rocket
- Updated `default_features.rs` to showcase server configuration
- Updated `all_features.rs` to demonstrate complete Scalar integration

#### Documentation
- Comprehensive Scalar Documentation Guide (`docs/guides/SCALAR_DOCUMENTATION.md`) - 500+ lines
  - Quick start guide (4 steps)
  - Complete configuration reference
  - Framework integration examples
  - Advanced usage patterns
  - Troubleshooting section
  - Best practices
- Scalar Integration Complete report (`docs/phases/SCALAR_INTEGRATION_COMPLETE.md`)
- Updated Examples documentation (`docs/phases/EXAMPLES_UPDATED.md`)

#### Testing
- 42 tests for Scalar and OpenAPI features
- 25 Scalar-specific tests covering:
  - Configuration defaults and builder pattern
  - JSON generation
  - HTML generation
  - CDN features
  - SRI hashes
  - Fallback handling
  - Proxy configuration
  - Integration scenarios
- 17 OpenAPI tests covering:
  - Basic generation
  - Route handling
  - Server configuration
  - Schema generation
  - JSON validation

### Added - Binary Size Monitoring (2025-12-01)

#### Automated Monitoring
- **GitHub Actions CI/CD Workflow** for binary size tracking
  - Automated builds for 3 configurations
  - Hard limit enforcement (fails on exceeding targets)
  - Detailed size reporting in CI

#### Local Development Tools
- `scripts/check_size.sh` for local size verification
- `cargo-make` task integration (`cargo make check-size`)
- `cargo-bloat` analysis for size optimization

#### Results
- **All binaries under 2MB** (target was 2-8MB)
  - Minimal config: 1.89MB (target: <2MB) ✅
  - Default features: 1.89MB (target: <5MB) ✅
  - All features: 1.89MB (target: <8MB) ✅
- Exceeded all targets with significant headroom
- Zero-cost abstractions working perfectly

#### Documentation
- Binary Size Monitoring Complete report (`docs/phases/BINARY_SIZE_MONITORING_COMPLETE.md`)

### Updated

#### README.md
- Added Scalar Integration announcement in "What is AllFrame?" section
- Updated "Current Status" to reflect Scalar completion (133 tests passing)
- Added comprehensive Scalar code example in "Core Features"
- Added "Scalar API Documentation" section to documentation index
- Updated roadmap to show Track A (Scalar) and Track B (Binary Size) as complete
- Removed generic Swagger UI references in favor of Scalar

#### Examples
- `default_features.rs`: Now demonstrates server configuration and output
- `all_features.rs`: Now showcases complete Scalar integration with all features

---

## Previous Work (Before Changelog)

### Completed - CQRS Infrastructure (Nov 2025)
- **Phases 1-5** complete with 85% average boilerplate reduction
- CommandBus (90% reduction)
- ProjectionRegistry (90% reduction)
- Event Versioning with auto-upcasting (95% reduction)
- Saga Orchestration with automatic compensation (75% reduction)
- AllSource backend integration

### Completed - Core Features (2024-2025)
- **v0.3**: OpenTelemetry tracing support
- **v0.2**: Compile-time DI + Auto OpenAPI 3.1
- **v0.1**: `allframe ignite` CLI + project scaffolding
- **v0.0**: Repository setup, documentation migration

---

## Statistics

### Test Coverage
- **Total Tests**: 500+ passing
- **CQRS Tests**: 72
- **Router/Protocol Tests**: 78
- **Scalar/OpenAPI Tests**: 42
- **MCP Tests**: 41
- **Resilience/Security Tests**: 55
- **Offline-First Tests**: 28
- **Offline Quality Gates**: 19
- **Graceful Shutdown Tests**: 17
- **Other Tests**: 148+
- **Coverage**: 100% (TDD-enforced)

### Binary Sizes (Release builds)
- Minimal (no features): 1.89MB
- Default features: 1.89MB
- All features: 1.89MB

### Documentation
- **Total Documentation**: 4,000+ lines
- Scalar guides: 675+ lines
- GraphQL guides: 600+ lines
- CQRS guides: 1,000+ lines
- Routing guides: 500+ lines
- Project documentation: 1,200+ lines

---

## Links

- **Repository**: https://github.com/all-source-os/all-frame
- **Documentation**: https://github.com/all-source-os/all-frame/tree/main/docs
- **Twitter Thread**: See `docs/announcements/TWITTER_THREAD_2025_12_01.md`

---

**AllFrame. One frame. Infinite transformations.**
