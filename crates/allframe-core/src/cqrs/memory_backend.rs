//! In-memory event store backend
//!
//! This is the default backend for AllFrame CQRS, providing a simple
//! HashMap-based storage suitable for testing, development, and MVPs.

use super::backend::{BackendStats, EventStoreBackend};
use super::Event;
use async_trait::async_trait;
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

/// In-memory event store backend
#[derive(Clone)]
pub struct InMemoryBackend<E: Event> {
    events: Arc<RwLock<HashMap<String, Vec<E>>>>,
    snapshots: Arc<RwLock<HashMap<String, (Vec<u8>, u64)>>>,
}

impl<E: Event> InMemoryBackend<E> {
    /// Create a new in-memory backend
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            snapshots: Arc::new(RwLock::new(HashMap::new())),
        }
    }
}

impl<E: Event> Default for InMemoryBackend<E> {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl<E: Event> EventStoreBackend<E> for InMemoryBackend<E> {
    async fn append(&self, aggregate_id: &str, events: Vec<E>) -> Result<(), String> {
        let mut store = self.events.write().await;
        let stream = store
            .entry(aggregate_id.to_string())
            .or_insert_with(Vec::new);
        stream.extend(events);
        Ok(())
    }

    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String> {
        let store = self.events.read().await;
        Ok(store.get(aggregate_id).cloned().unwrap_or_default())
    }

    async fn get_all_events(&self) -> Result<Vec<E>, String> {
        let store = self.events.read().await;
        let mut all_events = Vec::new();
        for events in store.values() {
            all_events.extend(events.clone());
        }
        Ok(all_events)
    }

    async fn get_events_after(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> Result<Vec<E>, String> {
        let events = self.get_events(aggregate_id).await?;
        Ok(events.into_iter().skip(version as usize).collect())
    }

    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        snapshot_data: Vec<u8>,
        version: u64,
    ) -> Result<(), String> {
        let mut snapshots = self.snapshots.write().await;
        snapshots.insert(aggregate_id.to_string(), (snapshot_data, version));
        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<(Vec<u8>, u64), String> {
        let snapshots = self.snapshots.read().await;
        snapshots
            .get(aggregate_id)
            .cloned()
            .ok_or_else(|| "No snapshot found".to_string())
    }

    async fn stats(&self) -> BackendStats {
        let store = self.events.read().await;
        let snapshots = self.snapshots.read().await;

        let total_events: u64 = store.values().map(|v| v.len() as u64).sum();
        let total_aggregates = store.len() as u64;
        let total_snapshots = snapshots.len() as u64;

        let mut backend_specific = HashMap::new();
        backend_specific.insert("backend_type".to_string(), "in-memory".to_string());

        BackendStats {
            total_events,
            total_aggregates,
            total_snapshots,
            backend_specific,
        }
    }
}
