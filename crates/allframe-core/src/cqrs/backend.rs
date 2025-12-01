//! Event store backend abstraction
//!
//! This module provides a trait-based abstraction for event store backends,
//! allowing AllFrame to support multiple storage implementations including
//! in-memory (for testing/MVP) and AllSource Core (for production).

use async_trait::async_trait;

use super::Event;

/// Backend trait for event storage implementations
#[async_trait]
pub trait EventStoreBackend<E: Event>: Send + Sync {
    /// Append events to an aggregate's event stream
    async fn append(&self, aggregate_id: &str, events: Vec<E>) -> Result<(), String>;

    /// Get all events for a specific aggregate
    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String>;

    /// Get all events from all aggregates (for projection rebuild)
    async fn get_all_events(&self) -> Result<Vec<E>, String>;

    /// Get events after a specific version (for snapshot optimization)
    async fn get_events_after(&self, aggregate_id: &str, version: u64) -> Result<Vec<E>, String>;

    /// Save a snapshot (optional, return Ok(()) if not supported)
    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        snapshot_data: Vec<u8>,
        version: u64,
    ) -> Result<(), String> {
        let _ = (aggregate_id, snapshot_data, version);
        Ok(()) // Default: no-op
    }

    /// Get latest snapshot (optional, return Err if not supported)
    async fn get_latest_snapshot(&self, aggregate_id: &str) -> Result<(Vec<u8>, u64), String> {
        let _ = aggregate_id;
        Err("Snapshots not supported by this backend".to_string())
    }

    /// Flush any pending writes (optional, for write-ahead log or batching)
    async fn flush(&self) -> Result<(), String> {
        Ok(()) // Default: no-op
    }

    /// Get backend statistics
    async fn stats(&self) -> BackendStats {
        BackendStats::default()
    }
}

/// Backend statistics
#[derive(Debug, Clone, Default)]
pub struct BackendStats {
    /// Total number of events stored
    pub total_events: u64,
    /// Total number of aggregates
    pub total_aggregates: u64,
    /// Total number of snapshots
    pub total_snapshots: u64,
    /// Backend-specific stats
    pub backend_specific: std::collections::HashMap<String, String>,
}
