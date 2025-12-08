//! CQRS + Event Sourcing implementation
//!
//! This module provides the core CQRS (Command Query Responsibility
//! Segregation) and Event Sourcing infrastructure for AllFrame.

use std::sync::Arc;

// Re-export macros
#[cfg(feature = "di")]
pub use allframe_macros::{command, command_handler, event, query, query_handler};
use tokio::sync::{mpsc, RwLock};

// Backend abstraction
mod backend;
mod memory_backend;

#[cfg(feature = "cqrs-allsource")]
mod allsource_backend;

// Command bus infrastructure
mod command_bus;

// Projection registry infrastructure
mod projection_registry;

// Event versioning infrastructure
mod event_versioning;

// Saga orchestration infrastructure
mod saga_orchestrator;

// Re-export backend types
#[cfg(feature = "cqrs-allsource")]
pub use allsource_backend::{AllSourceBackend, AllSourceConfig};
pub use backend::{BackendStats, EventStoreBackend};
// Re-export command bus types
pub use command_bus::{
    Command, CommandBus, CommandError, CommandHandler, CommandResult, ValidationError,
};
// Re-export event versioning types
pub use event_versioning::{
    AutoUpcaster, MigrationPath, Upcaster, VersionRegistry, VersionedEvent,
};
pub use memory_backend::InMemoryBackend;
// Re-export projection registry types
pub use projection_registry::{ProjectionMetadata, ProjectionPosition, ProjectionRegistry};
// Re-export saga orchestration types
pub use saga_orchestrator::{
    SagaDefinition, SagaError, SagaMetadata, SagaOrchestrator, SagaResult, SagaStatus, SagaStep,
};

/// Trait for Events - immutable facts that represent state changes
pub trait Event:
    Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static
{
}

/// Trait for Projections - read models built from events
pub trait Projection: Send + Sync {
    /// The event type this projection handles
    type Event: Event;

    /// Apply an event to update the projection state
    fn apply(&mut self, event: &Self::Event);
}

/// Trait for Aggregates - domain objects rebuilt from events
pub trait Aggregate: Default + Send + Sync {
    /// The event type this aggregate handles
    type Event: Event;

    /// Apply an event to the aggregate
    fn apply_event(&mut self, event: &Self::Event);
}

/// Event Store - append-only log of domain events
///
/// The EventStore uses a pluggable backend architecture:
/// - Default: InMemoryBackend (for testing/MVP)
/// - Production: AllSourceBackend (requires cqrs-allsource feature)
#[derive(Clone)]
pub struct EventStore<E: Event, B: EventStoreBackend<E> = InMemoryBackend<E>> {
    backend: Arc<B>,
    subscribers: Arc<RwLock<Vec<mpsc::Sender<E>>>>,
    _phantom: std::marker::PhantomData<E>,
}

impl<E: Event> EventStore<E, InMemoryBackend<E>> {
    /// Create a new event store with in-memory backend
    pub fn new() -> Self {
        Self::with_backend(InMemoryBackend::new())
    }
}

impl<E: Event, B: EventStoreBackend<E>> EventStore<E, B> {
    /// Create a new event store with a custom backend
    pub fn with_backend(backend: B) -> Self {
        Self {
            backend: Arc::new(backend),
            subscribers: Arc::new(RwLock::new(Vec::new())),
            _phantom: std::marker::PhantomData,
        }
    }

    /// Get a reference to the backend
    pub fn backend(&self) -> &B {
        &self.backend
    }

    /// Append events to an aggregate's event stream
    pub async fn append(&self, aggregate_id: &str, events: Vec<E>) -> Result<(), String> {
        // Notify subscribers before appending
        let subscribers = self.subscribers.read().await;
        for event in &events {
            for subscriber in subscribers.iter() {
                let _ = subscriber.send(event.clone()).await;
            }
        }

        // Delegate to backend
        self.backend.append(aggregate_id, events).await
    }

    /// Get all events for an aggregate
    pub async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String> {
        self.backend.get_events(aggregate_id).await
    }

    /// Get all events from all aggregates (for projection rebuild)
    pub async fn get_all_events(&self) -> Result<Vec<E>, String> {
        self.backend.get_all_events().await
    }

    /// Get events after a specific version (for snapshot optimization)
    pub async fn get_events_after(
        &self,
        aggregate_id: &str,
        version: u64,
    ) -> Result<Vec<E>, String> {
        self.backend.get_events_after(aggregate_id, version).await
    }

    /// Subscribe to event stream
    pub async fn subscribe(&self, tx: mpsc::Sender<E>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
    }

    /// Save a snapshot
    pub async fn save_snapshot<A>(
        &self,
        aggregate_id: &str,
        snapshot: Snapshot<A>,
    ) -> Result<(), String>
    where
        A: Aggregate<Event = E> + serde::Serialize,
    {
        let snapshot_data = serde_json::to_vec(&snapshot.aggregate)
            .map_err(|e| format!("Failed to serialize snapshot: {}", e))?;

        self.backend
            .save_snapshot(aggregate_id, snapshot_data, snapshot.version)
            .await
    }

    /// Get latest snapshot
    pub async fn get_latest_snapshot<A>(&self, aggregate_id: &str) -> Result<Snapshot<A>, String>
    where
        A: Aggregate<Event = E> + serde::de::DeserializeOwned,
    {
        let (snapshot_data, version) = self.backend.get_latest_snapshot(aggregate_id).await?;

        let aggregate: A = serde_json::from_slice(&snapshot_data)
            .map_err(|e| format!("Failed to deserialize snapshot: {}", e))?;

        Ok(Snapshot { aggregate, version })
    }

    /// Flush pending writes to storage (useful with WAL or batching backends)
    pub async fn flush(&self) -> Result<(), String> {
        self.backend.flush().await
    }

    /// Get backend statistics
    pub async fn stats(&self) -> BackendStats {
        self.backend.stats().await
    }
}

impl<E: Event> Default for EventStore<E, InMemoryBackend<E>> {
    fn default() -> Self {
        Self::new()
    }
}

/// Snapshot for aggregate optimization
pub struct Snapshot<A: Aggregate> {
    /// The aggregate state at this version
    pub aggregate: A,
    /// The version number of this snapshot
    pub version: u64,
}

impl<A: Aggregate> Snapshot<A> {
    /// Create a snapshot
    pub fn create(aggregate: A, version: u64) -> Self {
        Self { aggregate, version }
    }

    /// Convert snapshot back to aggregate
    pub fn into_aggregate(self) -> A {
        self.aggregate
    }
}

// Old CommandBus removed - use command_bus::CommandBus<E> instead

/// Query Bus for dispatching queries
pub struct QueryBus;

impl QueryBus {
    /// Create a new query bus
    pub fn new() -> Self {
        Self
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

// Old Saga types removed - use saga_orchestrator module instead
// The new saga system provides:
// - SagaStep trait for defining steps
// - SagaDefinition for building sagas
// - SagaOrchestrator for execution with automatic compensation

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct TestEvent {
        id: String,
    }

    impl Event for TestEvent {}

    #[tokio::test]
    async fn test_event_store_append_and_retrieve() {
        let store = EventStore::new();

        let events = vec![TestEvent {
            id: "1".to_string(),
        }];
        store.append("test-aggregate", events).await.unwrap();

        let retrieved = store.get_events("test-aggregate").await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].id, "1");
    }

    #[tokio::test]
    async fn test_event_store_multiple_aggregates() {
        let store = EventStore::new();

        store
            .append(
                "agg1",
                vec![TestEvent {
                    id: "1".to_string(),
                }],
            )
            .await
            .unwrap();
        store
            .append(
                "agg2",
                vec![TestEvent {
                    id: "2".to_string(),
                }],
            )
            .await
            .unwrap();

        let agg1_events = store.get_events("agg1").await.unwrap();
        let agg2_events = store.get_events("agg2").await.unwrap();

        assert_eq!(agg1_events.len(), 1);
        assert_eq!(agg2_events.len(), 1);
        assert_eq!(agg1_events[0].id, "1");
        assert_eq!(agg2_events[0].id, "2");
    }

    #[tokio::test]
    async fn test_command_bus_handlers() {
        use crate::cqrs::{Command, CommandBus, CommandHandler, CommandResult};

        #[derive(Clone)]
        struct TestCommand1;
        impl Command for TestCommand1 {}

        #[derive(Clone)]
        struct TestCommand2;
        impl Command for TestCommand2 {}

        struct Handler1;
        #[async_trait::async_trait]
        impl CommandHandler<TestCommand1, TestEvent> for Handler1 {
            async fn handle(&self, _cmd: TestCommand1) -> CommandResult<TestEvent> {
                Ok(vec![])
            }
        }

        struct Handler2;
        #[async_trait::async_trait]
        impl CommandHandler<TestCommand2, TestEvent> for Handler2 {
            async fn handle(&self, _cmd: TestCommand2) -> CommandResult<TestEvent> {
                Ok(vec![])
            }
        }

        let bus: CommandBus<TestEvent> = CommandBus::new();
        bus.register(Handler1).await;
        bus.register(Handler2).await;

        assert_eq!(bus.handlers_count().await, 2);
    }
}
