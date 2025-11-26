//! CQRS + Event Sourcing implementation
//!
//! This module provides the core CQRS (Command Query Responsibility Segregation)
//! and Event Sourcing infrastructure for AllFrame.

use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::{mpsc, RwLock};

// Re-export macros
#[cfg(feature = "di")]
pub use allframe_macros::{command, command_handler, event, query, query_handler};

/// Trait for Events - immutable facts that represent state changes
pub trait Event: Clone + Send + Sync + 'static {}

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
#[derive(Clone)]
pub struct EventStore<E: Event> {
    events: Arc<RwLock<HashMap<String, Vec<E>>>>,
    subscribers: Arc<RwLock<Vec<mpsc::Sender<E>>>>,
}

impl<E: Event> EventStore<E> {
    /// Create a new event store
    pub fn new() -> Self {
        Self {
            events: Arc::new(RwLock::new(HashMap::new())),
            subscribers: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Append events to an aggregate's event stream
    pub async fn append(&self, aggregate_id: &str, events: Vec<E>) -> Result<(), String> {
        let mut store = self.events.write().await;
        let stream = store.entry(aggregate_id.to_string()).or_insert_with(Vec::new);

        // Notify subscribers
        let subscribers = self.subscribers.read().await;
        for event in &events {
            for subscriber in subscribers.iter() {
                let _ = subscriber.send(event.clone()).await;
            }
        }

        stream.extend(events);
        Ok(())
    }

    /// Get all events for an aggregate
    pub async fn get_events(&self, aggregate_id: &str) -> Result<Vec<E>, String> {
        let store = self.events.read().await;
        Ok(store.get(aggregate_id).cloned().unwrap_or_default())
    }

    /// Get all events from all aggregates (for projection rebuild)
    pub async fn get_all_events(&self) -> Result<Vec<E>, String> {
        let store = self.events.read().await;
        let mut all_events = Vec::new();
        for events in store.values() {
            all_events.extend(events.clone());
        }
        Ok(all_events)
    }

    /// Get events after a specific version (for snapshot optimization)
    pub async fn get_events_after(&self, aggregate_id: &str, version: u64) -> Result<Vec<E>, String> {
        let events = self.get_events(aggregate_id).await?;
        Ok(events.into_iter().skip(version as usize).collect())
    }

    /// Subscribe to event stream
    pub async fn subscribe(&self, tx: mpsc::Sender<E>) {
        let mut subscribers = self.subscribers.write().await;
        subscribers.push(tx);
    }

    /// Save a snapshot (placeholder for future implementation)
    #[allow(dead_code)]
    pub async fn save_snapshot<A: Aggregate<Event = E>>(
        &self,
        _aggregate_id: &str,
        _snapshot: Snapshot<A>,
    ) -> Result<(), String> {
        // Placeholder - will be implemented when needed
        Ok(())
    }

    /// Get latest snapshot (placeholder for future implementation)
    #[allow(dead_code)]
    pub async fn get_latest_snapshot<A: Aggregate<Event = E>>(
        &self,
        _aggregate_id: &str,
    ) -> Result<Snapshot<A>, String> {
        // Placeholder - will be implemented when needed
        Err("Snapshots not implemented yet".to_string())
    }
}

impl<E: Event> Default for EventStore<E> {
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

/// Command Bus for dispatching commands
pub struct CommandBus {
    handlers_count: usize,
}

impl CommandBus {
    /// Create a new command bus
    pub fn new() -> Self {
        Self { handlers_count: 0 }
    }

    /// Register a command handler
    pub fn register<F>(mut self, _handler: F) -> Self
    where
        F: Fn() + Send + Sync + 'static,
    {
        self.handlers_count += 1;
        self
    }

    /// Get number of registered handlers
    pub fn handlers_count(&self) -> usize {
        self.handlers_count
    }

    /// Dispatch a command (placeholder)
    #[allow(dead_code)]
    pub async fn dispatch<C>(&self, _cmd: C) -> Result<(), String> {
        // Placeholder - will be implemented when needed
        Ok(())
    }

    /// Get all events (placeholder for testing)
    #[allow(dead_code)]
    pub async fn get_all_events<E: Event>(&self) -> Result<Vec<E>, String> {
        // Placeholder - will be implemented when needed
        Ok(Vec::new())
    }
}

impl Default for CommandBus {
    fn default() -> Self {
        Self::new()
    }
}

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

/// Saga step for multi-aggregate transactions
#[allow(dead_code)]
pub enum SagaStep {
    /// Debit money from an account
    DebitAccount {
        /// Account ID to debit from
        account_id: String,
        /// Amount to debit
        amount: f64
    },
    /// Credit money to an account
    CreditAccount {
        /// Account ID to credit to
        account_id: String,
        /// Amount to credit
        amount: f64
    },
}

/// Saga trait for coordinating multi-aggregate transactions
#[async_trait::async_trait]
pub trait Saga: Send + Sync {
    /// Execute the saga steps
    async fn execute(&self) -> Result<(), String>;

    /// Compensate for failed steps
    async fn compensate(&self, failed_step: usize) -> Result<(), String>;

    /// Execute a saga step (helper method)
    #[allow(dead_code)]
    async fn execute_step(&self, _step: SagaStep) -> Result<(), String> {
        // Placeholder - will be implemented when needed
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone, Debug, PartialEq)]
    struct TestEvent {
        id: String,
    }

    impl Event for TestEvent {}

    #[tokio::test]
    async fn test_event_store_append_and_retrieve() {
        let store = EventStore::new();

        let events = vec![TestEvent { id: "1".to_string() }];
        store.append("test-aggregate", events).await.unwrap();

        let retrieved = store.get_events("test-aggregate").await.unwrap();
        assert_eq!(retrieved.len(), 1);
        assert_eq!(retrieved[0].id, "1");
    }

    #[tokio::test]
    async fn test_event_store_multiple_aggregates() {
        let store = EventStore::new();

        store.append("agg1", vec![TestEvent { id: "1".to_string() }]).await.unwrap();
        store.append("agg2", vec![TestEvent { id: "2".to_string() }]).await.unwrap();

        let agg1_events = store.get_events("agg1").await.unwrap();
        let agg2_events = store.get_events("agg2").await.unwrap();

        assert_eq!(agg1_events.len(), 1);
        assert_eq!(agg2_events.len(), 1);
        assert_eq!(agg1_events[0].id, "1");
        assert_eq!(agg2_events[0].id, "2");
    }

    #[tokio::test]
    async fn test_command_bus_handlers() {
        let bus = CommandBus::new()
            .register(|| {})
            .register(|| {});

        assert_eq!(bus.handlers_count(), 2);
    }
}
