//! CQRS + Event Sourcing implementation
//!
//! This module provides the core CQRS (Command Query Responsibility
//! Segregation) and Event Sourcing infrastructure for AllFrame.


// Declare submodules
pub mod backend;
pub mod command_bus;
pub mod event_versioning;
pub mod projection_registry;
pub mod saga;
pub mod saga_orchestrator;
pub mod allsource_backend;
pub mod memory_backend;

/// Trait for Events - immutable facts that represent state changes
pub trait Event:
    Clone + Send + Sync + serde::Serialize + serde::de::DeserializeOwned + 'static
{
}

/// Trait for Aggregates - domain objects rebuilt from events
pub trait Aggregate: Default + Send + Sync {
    /// The event type this aggregate handles
    type Event: Event;

    /// Apply an event to the aggregate
    fn apply_event(&mut self, event: &Self::Event);
}

/// Trait for Projections - read models built from events
pub trait Projection: Send + Sync {
    /// The event type this projection handles
    type Event: Event;

    /// Apply an event to update the projection state
    fn apply(&mut self, event: &Self::Event);
}

/// Event Store - append-only log of domain events
///
/// The EventStore uses a pluggable backend architecture:
/// - Default: InMemoryBackend (for testing/MVP)
/// - Production: AllSourceBackend (requires cqrs-allsource feature)
#[derive(Clone)]
pub struct EventStore<E: Event, B: EventStoreBackend<E> = InMemoryBackend<E>> {
    backend: std::sync::Arc<B>,
    subscribers: std::sync::Arc<tokio::sync::RwLock<Vec<tokio::sync::mpsc::Sender<E>>>>,
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
            backend: std::sync::Arc::new(backend),
            subscribers: std::sync::Arc::new(tokio::sync::RwLock::new(Vec::new())),
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

    /// Subscribe to event stream
    pub async fn subscribe(&self, tx: tokio::sync::mpsc::Sender<E>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
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

// Re-export all CQRS types for convenience
pub use backend::*;
pub use command_bus::*;
pub use event_versioning::*;
pub use projection_registry::*;
// Resolve SagaError conflict - prefer saga_orchestrator version
pub use saga_orchestrator::{SagaDefinition, SagaMetadata, SagaOrchestrator, SagaResult, SagaStatus, SagaStep as OrchestratorSagaStep};
pub use saga::{CompensationResult, MacroSagaOrchestrator, Saga, SagaStep as MacroSagaStep, SagaContext, SagaError, StepExecutionResult, StepOutput};
pub use allsource_backend::*;
pub use memory_backend::*;

// Re-export AllSource v0.10.3 services behind the cqrs-allsource feature
#[cfg(feature = "cqrs-allsource")]
pub use allsource_core::ExactlyOnceRegistry;
#[cfg(feature = "cqrs-allsource")]
pub use allsource_core::SchemaRegistry;
#[cfg(feature = "cqrs-allsource")]
pub use allsource_core::PipelineManager;
