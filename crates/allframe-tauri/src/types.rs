//! Request and response types for Tauri IPC commands

use serde::{Deserialize, Serialize};

/// Metadata about a registered handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerInfo {
    /// Handler name (used to call it)
    pub name: String,
    /// Human-readable description
    pub description: String,
}

/// Response from calling a handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResponse {
    /// The handler's return value
    pub result: String,
}
