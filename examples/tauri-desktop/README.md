# AllFrame Tauri Desktop Example

Minimal Tauri 2.x desktop app demonstrating AllFrame's offline-first IPC.

## Prerequisites

Install [Tauri CLI](https://v2.tauri.app/start/prerequisites/):

```bash
cargo install tauri-cli --version "^2"
```

## Run

```bash
cd examples/tauri-desktop
cargo tauri dev
```

## What it shows

- **AllFrame Router** handlers registered as Tauri IPC commands
- **No HTTP server** — all dispatch is in-process via `allframe_tauri::init(router)`
- **Works fully offline** — zero network dependencies
- **Frontend** discovers handlers via `plugin:allframe|allframe_list` and calls them via `plugin:allframe|allframe_call`

## Architecture

```
┌─────────────────────────────────┐
│  Frontend (index.html)          │
│  invoke("plugin:allframe|...")  │
└──────────┬──────────────────────┘
           │ Tauri IPC (no HTTP)
┌──────────▼──────────────────────┐
│  allframe_tauri::init(router)   │
│  TauriServer → Router dispatch  │
└──────────┬──────────────────────┘
           │ In-process
┌──────────▼──────────────────────┐
│  Handler closures               │
│  (list_notes, get_note, ...)    │
└─────────────────────────────────┘
```
