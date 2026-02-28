//! Tauri IPC command handlers
//!
//! The actual `#[tauri::command]` functions live in `plugin.rs` because
//! `tauri::generate_handler![]` requires the command functions to be
//! defined in the same module where the macro is invoked.
//!
//! This module re-exports them for documentation purposes.
//!
//! ## Available Commands
//!
//! ### `plugin:allframe|allframe_list`
//! Returns a list of all registered handlers as `Vec<HandlerInfo>`.
//!
//! ### `plugin:allframe|allframe_call`
//! Calls a handler by name. Parameters:
//! - `handler: String` — the handler name
//! - `args: Value` — JSON arguments passed to the handler
