# allframe-tauri

**Tauri 2.x plugin for AllFrame - offline-first desktop apps**

[![Crates.io](https://img.shields.io/crates/v/allframe-tauri.svg)](https://crates.io/crates/allframe-tauri)
[![Documentation](https://docs.rs/allframe-tauri/badge.svg)](https://docs.rs/allframe-tauri)
[![License](https://img.shields.io/badge/license-MIT%20OR%20Apache--2.0-blue.svg)](../../LICENSE-MIT)

Expose AllFrame Router handlers as Tauri IPC commands for desktop applications. No HTTP server needed -- handlers are dispatched in-process via Tauri's IPC bridge.

## Why

LLM-powered desktop apps (local assistants, knowledge bases, IDE extensions) need to call handlers without opening a network port. `allframe-tauri` lets you:

- Dispatch AllFrame handlers via Tauri IPC (no HTTP, no WebSocket)
- Run fully offline with `cqrs-sqlite` for local event sourcing
- Expose the same handlers to both frontend JavaScript and local LLMs (via embedded MCP)

## Quick Start

```toml
[dependencies]
allframe-tauri = "0.1.21"
allframe-core = { version = "0.1.21", features = ["router"] }
tauri = { version = "2", features = ["wry"] }
```

### 1. Rust (Tauri App)

```rust
use allframe_core::router::Router;

fn main() {
    let mut router = Router::new();
    router.register("get_user", || async {
        r#"{"id":1,"name":"Alice"}"#.to_string()
    });

    tauri::Builder::default()
        .plugin(allframe_tauri::init(router))
        .run(tauri::generate_context!())
        .unwrap();
}
```

### Async Boot Lifecycle

Apps that need async initialization (event stores, projections, command buses) before the UI renders can use `builder()` with `on_boot()`:

```rust
use allframe_core::router::Router;
use allframe_tauri::{builder, BootError};

fn main() {
    let mut router = Router::new();
    // Handlers can be registered before state is injected
    router.register_with_state_only::<MyDb, _, _>("query", |db| async move {
        db.query_all().await
    });

    tauri::Builder::default()
        .plugin(
            builder(router)
                .on_boot(2, |ctx| async move {
                    let db = MyDb::open(&ctx.data_dir()?).await
                        .map_err(|e| BootError::Failed(e.to_string()))?;
                    ctx.inject_state(db);
                    ctx.emit_progress("Database ready");

                    // ... more init steps
                    ctx.emit_progress("Projections rebuilt");
                    Ok(())
                })
                .build(),
        )
        .run(tauri::generate_context!())
        .unwrap();
}
```

This handles the "no Tokio reactor on macOS main thread" problem internally. Progress events are emitted as `allframe-tauri:boot-progress` for frontend splash screens. See the [`boot_lifecycle` example](examples/boot_lifecycle.rs) for a runnable demo.

### 2. Permissions (Required for Tauri 2)

Add `"allframe-tauri:default"` to your app's capabilities. Create `src-tauri/capabilities/default.json`:

```json
{
  "$schema": "../gen/schemas/desktop-schema.json",
  "identifier": "default",
  "windows": ["main"],
  "permissions": [
    "core:default",
    "allframe-tauri:default"
  ]
}
```

This grants access to all AllFrame IPC commands (`allframe_list`, `allframe_call`, `allframe_stream`, `allframe_stream_cancel`). For fine-grained control, grant individual permissions instead:

```json
"permissions": [
  "allframe-tauri:allow-allframe-list",
  "allframe-tauri:allow-allframe-call"
]
```

### 3. Frontend (TypeScript)

```typescript
import { invoke } from "@tauri-apps/api/core";

// List available handlers
const handlers = await invoke("plugin:allframe-tauri|allframe_list");

// Call a handler
const result = await invoke("plugin:allframe-tauri|allframe_call", {
    handler: "get_user",
    args: { id: 42 }
});
```

## In-Process Dispatch

`TauriServer` also supports direct in-process calls without the Tauri runtime. Useful for local LLM integration or testing:

```rust
use allframe_core::router::Router;
use allframe_tauri::TauriServer;

let mut router = Router::new();
router.register("search", || async { "results".to_string() });

let server = TauriServer::new(router);
let result = server.call_handler("search", "{}").await.unwrap();
assert_eq!(result.result, "results");
```

## API

| Type | Description |
|------|-------------|
| `TauriServer` | In-process handler dispatcher (no Tauri runtime needed) |
| `init(router)` | Creates a Tauri plugin from an AllFrame Router |
| `builder(router)` | Creates a `BootBuilder` for configuring async boot lifecycle |
| `BootBuilder` | Builder with `.on_boot(steps, closure)` for async initialization |
| `BootContext` | Boot closure context: `inject_state()`, `emit_progress()`, `data_dir()` |
| `BootError` | Boot error type (`Failed`, `DataDir`, `Runtime`) |
| `BootProgress` | Progress event payload (`{ step, total, label }`) |
| `CallResponse` | Handler result wrapper (`{ result: String }`) |
| `HandlerInfo` | Handler metadata (`{ name, description, kind }`) |
| `HandlerKind` | `RequestResponse` or `Streaming` |
| `StreamReceiver` | Receiver for streaming handler items (auto-cancels on drop) |
| `StreamStartResponse` | Streaming init response (`{ stream_id: String }`) |
| `TauriServerError` | Error type (`HandlerNotFound`, `NotStreamingHandler`, `ExecutionFailed`) |

## Features

| Feature | Description | Default |
|---------|-------------|---------|
| `tracing` | Structured logging via `tracing` crate | No |

## Offline-First Architecture

Combine with AllFrame's offline features for a fully self-contained desktop app:

```toml
[dependencies]
allframe-core = { version = "0.1.21", features = ["offline"] }
allframe-tauri = "0.1.21"
```

This gives you:
- SQLite event store (WAL mode, zero network deps)
- Compile-time DI with lazy initialization
- Security utilities for credential handling
- CQRS + Event Sourcing -- all running locally

## License

Licensed under either of:

- Apache License, Version 2.0 ([LICENSE-APACHE](../../LICENSE-APACHE))
- MIT license ([LICENSE-MIT](../../LICENSE-MIT))

at your option.

## Resources

- **Documentation**: https://docs.rs/allframe-tauri
- **Repository**: https://github.com/all-source-os/all-frame
- **Core Framework**: https://crates.io/crates/allframe-core
- **MCP Server**: https://crates.io/crates/allframe-mcp
