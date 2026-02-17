//! AllSource Core event store backend
//!
//! This backend integrates with AllSource Core to provide production-grade
//! event sourcing with persistence, performance, and advanced features.

#[cfg(feature = "cqrs-allsource")]
use std::sync::Arc;

#[cfg(feature = "cqrs-allsource")]
use async_trait::async_trait;

#[cfg(feature = "cqrs-allsource")]
use super::backend::{BackendStats, EventStoreBackend};
#[cfg(feature = "cqrs-allsource")]
use super::Event;

#[cfg(feature = "cqrs-allsource")]
/// AllSource Core backend for production event sourcing
#[derive(Clone)]
pub struct AllSourceBackend<E: Event> {
    store: Arc<allsource_core::EventStore>,
    _phantom: std::marker::PhantomData<E>,
}

#[cfg(feature = "cqrs-allsource")]
impl<E: Event> AllSourceBackend<E> {
    /// Create a new AllSource backend with default configuration
    pub fn new() -> Result<Self, String> {
        let store = allsource_core::EventStore::default();
        Ok(Self {
            store: Arc::new(store),
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create a new AllSource backend with custom configuration
    pub fn with_config(config: AllSourceConfig) -> Result<Self, String> {
        let mut store_config = allsource_core::store::EventStoreConfig::default();

        if config.enable_persistence {
            store_config = allsource_core::store::EventStoreConfig::with_persistence(
                config
                    .persistence_path
                    .unwrap_or_else(|| "./allsource_data".to_string()),
            );
        }

        let store = allsource_core::EventStore::with_config(store_config);

        Ok(Self {
            store: Arc::new(store),
            _phantom: std::marker::PhantomData,
        })
    }

    /// Create a production-ready configuration (persistence + WAL)
    pub fn production(data_path: &str) -> Result<Self, String> {
        Self::with_config(AllSourceConfig {
            enable_persistence: true,
            enable_wal: true,
            persistence_path: Some(data_path.to_string()),
            wal_path: Some(format!("{}/wal", data_path)),
        })
    }

    /// Access the ExactlyOnceRegistry for idempotency/dedup of event processing
    pub fn exactly_once(&self) -> Arc<allsource_core::ExactlyOnceRegistry> {
        self.store.exactly_once()
    }

    /// Access the SchemaRegistry for JSON Schema validation of events
    pub fn schema_registry(&self) -> Arc<allsource_core::SchemaRegistry> {
        self.store.schema_registry()
    }

    /// Access the PipelineManager for stream processing pipelines
    pub fn pipeline_manager(&self) -> Arc<allsource_core::PipelineManager> {
        self.store.pipeline_manager()
    }
}

#[cfg(feature = "cqrs-allsource")]
impl<E: Event> Default for AllSourceBackend<E> {
    fn default() -> Self {
        Self::new().expect("Failed to create default AllSource backend")
    }
}

#[cfg(feature = "cqrs-allsource")]
/// Configuration for AllSource backend
pub struct AllSourceConfig {
    /// Enable Parquet persistence
    pub enable_persistence: bool,
    /// Enable Write-Ahead Log
    pub enable_wal: bool,
    /// Persistence path (default: "./allsource_data")
    pub persistence_path: Option<String>,
    /// WAL path (default: "./allsource_wal")
    pub wal_path: Option<String>,
}

#[cfg(feature = "cqrs-allsource")]
impl Default for AllSourceConfig {
    fn default() -> Self {
        Self {
            enable_persistence: false,
            enable_wal: false,
            persistence_path: None,
            wal_path: None,
        }
    }
}

#[cfg(feature = "cqrs-allsource")]
#[async_trait]
impl<E: Event> EventStoreBackend<E> for AllSourceBackend<E> {
    async fn append(&self, aggregate_id: &str, events: Vec<E>) -> Result<(), String> {
        // Convert AllFrame events to AllSource events using the new 0.7.0 API
        for event in events {
            let payload = serde_json::to_value(&event)
                .map_err(|e| format!("Failed to serialize event: {}", e))?;

            // Use from_strings which validates and creates proper value objects
            let allsource_event = allsource_core::Event::from_strings(
                format!("allframe.{}", E::event_type_name()),
                aggregate_id.to_string(),
                "default".to_string(),
                payload,
                None,
            )
            .map_err(|e| format!("Failed to create event: {:?}", e))?;

            self.store
                .ingest(allsource_event)
                .map_err(|e| format!("Failed to ingest event: {:?}", e))?;
        }

        Ok(())
    }

    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String> {
        let request = allsource_core::QueryEventsRequest {
            entity_id: Some(aggregate_id.to_string()),
            event_type: None,
            tenant_id: None,
            as_of: None,
            since: None,
            until: None,
            limit: None,
        };

        let allsource_events = self
            .store
            .query(request)
            .map_err(|e| format!("Failed to query events: {:?}", e))?;

        // Convert AllSource events back to AllFrame events
        let mut events = Vec::new();
        for allsource_event in allsource_events {
            let event: E = serde_json::from_value(allsource_event.payload.clone())
                .map_err(|e| format!("Failed to deserialize event: {}", e))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_all_events(&self) -> Result<Vec<E>, String> {
        let request = allsource_core::QueryEventsRequest {
            entity_id: None,
            event_type: None,
            tenant_id: None,
            as_of: None,
            since: None,
            until: None,
            limit: None,
        };

        let allsource_events = self
            .store
            .query(request)
            .map_err(|e| format!("Failed to query all events: {:?}", e))?;

        let mut events = Vec::new();
        for allsource_event in allsource_events {
            let event: E = serde_json::from_value(allsource_event.payload.clone())
                .map_err(|e| format!("Failed to deserialize event: {}", e))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_after(&self, aggregate_id: &str, version: u64) -> Result<Vec<E>, String> {
        let request = allsource_core::QueryEventsRequest {
            entity_id: Some(aggregate_id.to_string()),
            event_type: None,
            tenant_id: None,
            as_of: None,
            since: None,
            until: None,
            limit: None,
        };

        let allsource_events = self
            .store
            .query(request)
            .map_err(|e| format!("Failed to query events: {:?}", e))?;

        // Skip the first `version` events and only deserialize the rest.
        // AllSource's QueryEventsRequest does not support version-based offset,
        // so we skip at the allsource Event level to avoid deserializing events
        // we will discard.
        let mut events = Vec::new();
        for allsource_event in allsource_events.into_iter().skip(version as usize) {
            let event: E = serde_json::from_value(allsource_event.payload.clone())
                .map_err(|e| format!("Failed to deserialize event: {}", e))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        snapshot_data: Vec<u8>,
        version: u64,
    ) -> Result<(), String> {
        let state: serde_json::Value = serde_json::from_slice(&snapshot_data)
            .map_err(|e| format!("Failed to deserialize snapshot data: {}", e))?;

        self.store
            .snapshot_manager()
            .create_snapshot(
                aggregate_id.to_string(),
                state,
                chrono::Utc::now(),
                version as usize,
                allsource_core::infrastructure::persistence::SnapshotType::Manual,
            )
            .map_err(|e| format!("Failed to create snapshot: {:?}", e))?;

        Ok(())
    }

    async fn get_latest_snapshot(&self, aggregate_id: &str) -> Result<(Vec<u8>, u64), String> {
        let snapshot = self
            .store
            .snapshot_manager()
            .get_latest_snapshot(aggregate_id)
            .ok_or_else(|| format!("No snapshot found for aggregate: {}", aggregate_id))?;

        let snapshot_bytes = serde_json::to_vec(&snapshot.state)
            .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;

        Ok((snapshot_bytes, snapshot.event_count as u64))
    }

    async fn flush(&self) -> Result<(), String> {
        self.store
            .flush_storage()
            .map_err(|e| format!("Failed to flush storage: {:?}", e))?;
        Ok(())
    }

    async fn stats(&self) -> BackendStats {
        let allsource_stats = self.store.stats();

        let mut backend_specific = std::collections::HashMap::new();
        backend_specific.insert("backend_type".to_string(), "allsource-core".to_string());
        backend_specific.insert(
            "total_ingested".to_string(),
            allsource_stats.total_ingested.to_string(),
        );
        backend_specific.insert(
            "total_entities".to_string(),
            allsource_stats.total_entities.to_string(),
        );
        backend_specific.insert(
            "total_event_types".to_string(),
            allsource_stats.total_event_types.to_string(),
        );

        let snapshot_stats = self.store.snapshot_manager().stats();

        BackendStats {
            total_events: allsource_stats.total_ingested,
            total_aggregates: allsource_stats.total_entities as u64,
            total_snapshots: snapshot_stats.total_snapshots as u64,
            backend_specific,
        }
    }
}

// Placeholder types when the feature is not enabled
#[cfg(not(feature = "cqrs-allsource"))]
/// Placeholder - requires cqrs-allsource feature
pub struct AllSourceBackend<E> {
    _phantom: std::marker::PhantomData<E>,
}

#[cfg(not(feature = "cqrs-allsource"))]
/// Placeholder - requires cqrs-allsource feature
pub struct AllSourceConfig;
