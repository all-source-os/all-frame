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
allframe-tauri = "0.1.15"
allframe-core = { version = "0.1.15", features = ["router"] }
tauri = { version = "2", features = ["wry"] }
```

### Rust (Tauri App)

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

### Frontend (TypeScript)

```typescript
import { invoke } from "@tauri-apps/api/core";

// List available handlers
const handlers = await invoke("plugin:allframe|allframe_list");

// Call a handler
const result = await invoke("plugin:allframe|allframe_call", {
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
| `CallResponse` | Handler result wrapper (`{ result: String }`) |
| `HandlerInfo` | Handler metadata (`{ name, description }`) |
| `TauriServerError` | Error type (`HandlerNotFound`, `DispatchError`) |

## Features

| Feature | Description | Default |
|---------|-------------|---------|
| `tracing` | Structured logging via `tracing` crate | No |

## Offline-First Architecture

Combine with AllFrame's offline features for a fully self-contained desktop app:

```toml
[dependencies]
allframe-core = { version = "0.1.15", features = ["offline"] }
allframe-tauri = "0.1.15"
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
