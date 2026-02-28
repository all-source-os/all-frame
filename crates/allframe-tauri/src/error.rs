//! Tauri-compatible error types
//!
//! Tauri commands require errors to implement `Serialize`,
//! so we use a dedicated error type instead of `anyhow::Error`.

use serde::Serialize;

/// Errors returned by AllFrame Tauri commands
#[derive(Debug, thiserror::Error, Serialize)]
pub enum TauriServerError {
    /// The requested handler was not found in the router
    #[error("Handler not found: {0}")]
    HandlerNotFound(String),

    /// Handler execution failed
    #[error("Execution failed: {0}")]
    ExecutionFailed(String),
}
