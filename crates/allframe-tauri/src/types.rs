//! Request and response types for Tauri IPC commands

use serde::{Deserialize, Serialize};

/// Whether a handler is request/response or streaming
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum HandlerKind {
    /// Standard request/response handler
    RequestResponse,
    /// Streaming handler that sends incremental updates
    Streaming,
}

/// Metadata about a registered handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerInfo {
    /// Handler name (used to call it)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// Whether this handler is request/response or streaming
    pub kind: HandlerKind,
}

/// Response from calling a handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResponse {
    /// The handler's return value
    pub result: String,
}

/// Response from starting a streaming handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StreamStartResponse {
    /// Unique stream ID for this invocation
    pub stream_id: String,
}
