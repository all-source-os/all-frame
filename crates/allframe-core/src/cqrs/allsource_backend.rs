//! AllSource Core event store backend
//!
//! This backend integrates with AllSource Core to provide production-grade
//! event sourcing with persistence, performance, and advanced features.

#[cfg(feature = "cqrs-allsource")]
use super::backend::{BackendStats, EventStoreBackend};
#[cfg(feature = "cqrs-allsource")]
use super::Event;
#[cfg(feature = "cqrs-allsource")]
use async_trait::async_trait;
#[cfg(feature = "cqrs-allsource")]
use std::collections::HashMap;
#[cfg(feature = "cqrs-allsource")]
use std::sync::Arc;

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
        let mut store_config = allsource_core::EventStoreConfig::default();

        if config.enable_persistence {
            store_config = store_config.with_persistence(
                config
                    .persistence_path
                    .unwrap_or_else(|| "./allsource_data".to_string()),
            );
        }

        if config.enable_wal {
            store_config = store_config.with_wal(
                config
                    .wal_path
                    .unwrap_or_else(|| "./allsource_wal".to_string()),
            );
        }

        let store = allsource_core::EventStore::with_config(store_config)
            .map_err(|e| format!("Failed to initialize AllSource: {:?}", e))?;

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
        // Convert AllFrame events to AllSource events
        for event in events {
            let allsource_event = allsource_core::Event {
                id: uuid::Uuid::new_v4().to_string(),
                entity_id: aggregate_id.to_string(),
                event_type: std::any::type_name::<E>().to_string(),
                data: serde_json::to_value(&event)
                    .map_err(|e| format!("Failed to serialize event: {}", e))?,
                metadata: serde_json::json!({}),
                timestamp: chrono::Utc::now(),
            };

            self.store
                .ingest(allsource_event)
                .await
                .map_err(|e| format!("Failed to ingest event: {:?}", e))?;
        }

        Ok(())
    }

    async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String> {
        let request = allsource_core::QueryEventsRequest {
            entity_id: Some(aggregate_id.to_string()),
            event_type: None,
            start_time: None,
            end_time: None,
            limit: None,
        };

        let allsource_events = self
            .store
            .query(request)
            .await
            .map_err(|e| format!("Failed to query events: {:?}", e))?;

        // Convert AllSource events back to AllFrame events
        let mut events = Vec::new();
        for allsource_event in allsource_events {
            let event: E = serde_json::from_value(allsource_event.data)
                .map_err(|e| format!("Failed to deserialize event: {}", e))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_all_events(&self) -> Result<Vec<E>, String> {
        let request = allsource_core::QueryEventsRequest {
            entity_id: None,
            event_type: None,
            start_time: None,
            end_time: None,
            limit: None,
        };

        let allsource_events = self
            .store
            .query(request)
            .await
            .map_err(|e| format!("Failed to query all events: {:?}", e))?;

        let mut events = Vec::new();
        for allsource_event in allsource_events {
            let event: E = serde_json::from_value(allsource_event.data)
                .map_err(|e| format!("Failed to deserialize event: {}", e))?;
            events.push(event);
        }

        Ok(events)
    }

    async fn get_events_after(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> Result<Vec<E>, String> {
        let all_events = self.get_events(aggregate_id).await?;
        Ok(all_events.into_iter().skip(version as usize).collect())
    }

    async fn save_snapshot(
        &self,
        aggregate_id: &str,
        snapshot_data: Vec<u8>,
        version: u64,
    ) -> Result<(), String> {
        self.store
            .create_snapshot(aggregate_id)
            .await
            .map_err(|e| format!("Failed to create snapshot: {:?}", e))?;

        // Store metadata about the snapshot version
        let snapshot_metadata = serde_json::json!({
            "snapshot_data_len": snapshot_data.len(),
            "version": version,
        });

        let snapshot_event = allsource_core::Event {
            id: uuid::Uuid::new_v4().to_string(),
            entity_id: format!("{}__snapshot", aggregate_id),
            event_type: "SnapshotCreated".to_string(),
            data: snapshot_metadata,
            metadata: serde_json::json!({}),
            timestamp: chrono::Utc::now(),
        };

        self.store
            .ingest(snapshot_event)
            .await
            .map_err(|e| format!("Failed to store snapshot metadata: {:?}", e))?;

        Ok(())
    }

    async fn get_latest_snapshot(
        &self,
        aggregate_id: &str,
    ) -> Result<(Vec<u8>, u64), String> {
        let snapshot_json = self
            .store
            .get_snapshot(aggregate_id)
            .await
            .map_err(|e| format!("Failed to get snapshot: {:?}", e))?;

        // For MVP, we return empty data - full snapshot retrieval will be implemented
        // when AllSource Core exposes snapshot data retrieval
        Ok((Vec::new(), 0))
    }

    async fn flush(&self) -> Result<(), String> {
        self.store
            .flush_storage()
            .await
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
        backend_specific.insert("uptime".to_string(), allsource_stats.uptime.to_string());

        BackendStats {
            total_events: allsource_stats.total_ingested,
            total_aggregates: 0, // AllSource doesn't track aggregate count separately
            total_snapshots: 0,  // Will be implemented when snapshot retrieval is available
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
