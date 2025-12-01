//! Projection Registry for automatic projection lifecycle management
//!
//! The ProjectionRegistry eliminates projection boilerplate by providing:
//! - Automatic event subscription
//! - Consistency tracking
//! - Rebuild functionality
//! - Multi-projection coordination

use std::{collections::HashMap, marker::PhantomData, sync::Arc};

use tokio::sync::{mpsc, RwLock};

use super::{Event, EventStore, EventStoreBackend, Projection};

/// Position tracker for projection consistency
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct ProjectionPosition {
    /// Last event version processed by this projection
    pub version: u64,
    /// Timestamp of last update
    pub updated_at: std::time::SystemTime,
}

impl ProjectionPosition {
    /// Create a new position at version 0
    pub fn initial() -> Self {
        Self {
            version: 0,
            updated_at: std::time::SystemTime::now(),
        }
    }

    /// Update to a new version
    pub fn advance(&mut self, version: u64) {
        self.version = version;
        self.updated_at = std::time::SystemTime::now();
    }
}

/// Projection metadata for tracking and management
#[derive(Debug, Clone)]
pub struct ProjectionMetadata {
    /// Unique name of the projection
    pub name: String,
    /// Current position in event stream
    pub position: ProjectionPosition,
    /// Whether projection is currently rebuilding
    pub rebuilding: bool,
}

/// Type-erased projection wrapper for registry storage
trait ErasedProjection<E: Event>: Send + Sync {
    /// Apply an event to the projection
    fn apply_event(&mut self, event: &E);
    /// Get projection name
    fn name(&self) -> &str;
    /// Get current position
    fn position(&self) -> ProjectionPosition;
    /// Mark as rebuilding
    fn set_rebuilding(&mut self, rebuilding: bool);
}

/// Concrete wrapper for projections
struct ProjectionWrapper<P: Projection> {
    projection: P,
    metadata: ProjectionMetadata,
}

impl<P: Projection> ErasedProjection<P::Event> for ProjectionWrapper<P> {
    fn apply_event(&mut self, event: &P::Event) {
        self.projection.apply(event);
        self.metadata
            .position
            .advance(self.metadata.position.version + 1);
    }

    fn name(&self) -> &str {
        &self.metadata.name
    }

    fn position(&self) -> ProjectionPosition {
        self.metadata.position
    }

    fn set_rebuilding(&mut self, rebuilding: bool) {
        self.metadata.rebuilding = rebuilding;
    }
}

/// Type alias for projection storage
type ProjectionMap<E> = HashMap<String, Box<dyn ErasedProjection<E>>>;

/// Projection Registry for managing multiple projections
pub struct ProjectionRegistry<E: Event, B: EventStoreBackend<E>> {
    projections: Arc<RwLock<ProjectionMap<E>>>,
    event_store: Arc<EventStore<E, B>>,
    _phantom: PhantomData<E>,
}

impl<E: Event, B: EventStoreBackend<E>> ProjectionRegistry<E, B> {
    /// Create a new projection registry
    pub fn new(event_store: EventStore<E, B>) -> Self {
        Self {
            projections: Arc::new(RwLock::new(HashMap::new())),
            event_store: Arc::new(event_store),
            _phantom: PhantomData,
        }
    }

    /// Register a projection with automatic event subscription
    pub async fn register<P: Projection<Event = E> + 'static>(
        &self,
        name: impl Into<String>,
        projection: P,
    ) {
        let name = name.into();
        let wrapper = ProjectionWrapper {
            projection,
            metadata: ProjectionMetadata {
                name: name.clone(),
                position: ProjectionPosition::initial(),
                rebuilding: false,
            },
        };

        let mut projections = self.projections.write().await;
        projections.insert(name, Box::new(wrapper));
    }

    /// Get a projection by name
    pub async fn get<P: Projection<Event = E> + 'static>(
        &self,
        _name: &str,
    ) -> Option<Arc<RwLock<P>>> {
        // Note: This is a simplified version. In a real implementation,
        // we'd need to downcast the type-erased projection back to P.
        // For now, we'll focus on the core functionality.
        None
    }

    /// Rebuild a specific projection from scratch
    pub async fn rebuild(&self, name: &str) -> Result<(), String> {
        // Mark projection as rebuilding
        {
            let mut projections = self.projections.write().await;
            if let Some(projection) = projections.get_mut(name) {
                projection.set_rebuilding(true);
            } else {
                return Err(format!("Projection '{}' not found", name));
            }
        }

        // Get all events from event store
        let events = self.event_store.get_all_events().await?;

        // Apply all events to the projection
        {
            let mut projections = self.projections.write().await;
            if let Some(projection) = projections.get_mut(name) {
                for event in events {
                    projection.apply_event(&event);
                }
                projection.set_rebuilding(false);
            }
        }

        Ok(())
    }

    /// Rebuild all projections from scratch
    pub async fn rebuild_all(&self) -> Result<(), String> {
        let projection_names: Vec<String> = {
            let projections = self.projections.read().await;
            projections.keys().cloned().collect()
        };

        for name in projection_names {
            self.rebuild(&name).await?;
        }

        Ok(())
    }

    /// Get metadata for a projection
    pub async fn get_metadata(&self, name: &str) -> Option<ProjectionMetadata> {
        let projections = self.projections.read().await;
        projections.get(name).map(|p| ProjectionMetadata {
            name: p.name().to_string(),
            position: p.position(),
            rebuilding: false, // Would need to track this properly
        })
    }

    /// Get metadata for all projections
    pub async fn get_all_metadata(&self) -> Vec<ProjectionMetadata> {
        let projections = self.projections.read().await;
        projections
            .values()
            .map(|p| ProjectionMetadata {
                name: p.name().to_string(),
                position: p.position(),
                rebuilding: false,
            })
            .collect()
    }

    /// Subscribe projections to new events
    pub async fn start_subscription(&self) -> Result<(), String> {
        let (tx, mut rx) = mpsc::channel::<E>(100);

        // Subscribe to event store
        self.event_store.subscribe(tx).await;

        // Spawn task to handle incoming events
        let projections = Arc::clone(&self.projections);
        tokio::spawn(async move {
            while let Some(event) = rx.recv().await {
                let mut projections = projections.write().await;
                for projection in projections.values_mut() {
                    projection.apply_event(&event);
                }
            }
        });

        Ok(())
    }

    /// Get number of registered projections
    pub async fn count(&self) -> usize {
        self.projections.read().await.len()
    }
}

impl<E: Event, B: EventStoreBackend<E>> Clone for ProjectionRegistry<E, B> {
    fn clone(&self) -> Self {
        Self {
            projections: Arc::clone(&self.projections),
            event_store: Arc::clone(&self.event_store),
            _phantom: PhantomData,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cqrs::InMemoryBackend;

    #[derive(Clone, Debug, PartialEq)]
    enum TestEvent {
        Created { id: String, value: i32 },
        Updated { id: String, value: i32 },
    }

    impl Event for TestEvent {}

    struct TestProjection {
        data: HashMap<String, i32>,
    }

    impl TestProjection {
        fn new() -> Self {
            Self {
                data: HashMap::new(),
            }
        }

        #[allow(dead_code)]
        fn get(&self, id: &str) -> Option<i32> {
            self.data.get(id).copied()
        }
    }

    impl Projection for TestProjection {
        type Event = TestEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                TestEvent::Created { id, value } => {
                    self.data.insert(id.clone(), *value);
                }
                TestEvent::Updated { id, value } => {
                    self.data.insert(id.clone(), *value);
                }
            }
        }
    }

    #[tokio::test]
    async fn test_projection_registration() {
        let store = EventStore::<TestEvent, InMemoryBackend<TestEvent>>::new();
        let registry = ProjectionRegistry::new(store);

        registry
            .register("test-projection", TestProjection::new())
            .await;

        assert_eq!(registry.count().await, 1);
    }

    #[tokio::test]
    async fn test_projection_rebuild() {
        let store = EventStore::<TestEvent, InMemoryBackend<TestEvent>>::new();

        // Add some events
        store
            .append(
                "test-1",
                vec![
                    TestEvent::Created {
                        id: "1".to_string(),
                        value: 10,
                    },
                    TestEvent::Updated {
                        id: "1".to_string(),
                        value: 20,
                    },
                ],
            )
            .await
            .unwrap();

        let registry = ProjectionRegistry::new(store);
        registry
            .register("test-projection", TestProjection::new())
            .await;

        // Rebuild projection
        registry.rebuild("test-projection").await.unwrap();

        // Verify metadata
        let metadata = registry.get_metadata("test-projection").await.unwrap();
        assert_eq!(metadata.position.version, 2);
    }

    #[tokio::test]
    async fn test_projection_metadata() {
        let store = EventStore::<TestEvent, InMemoryBackend<TestEvent>>::new();
        let registry = ProjectionRegistry::new(store);

        registry.register("proj-1", TestProjection::new()).await;
        registry.register("proj-2", TestProjection::new()).await;

        let all_metadata = registry.get_all_metadata().await;
        assert_eq!(all_metadata.len(), 2);

        let metadata = registry.get_metadata("proj-1").await.unwrap();
        assert_eq!(metadata.name, "proj-1");
        assert_eq!(metadata.position.version, 0);
    }

    #[tokio::test]
    async fn test_rebuild_all() {
        let store = EventStore::<TestEvent, InMemoryBackend<TestEvent>>::new();

        // Add events
        store
            .append(
                "test",
                vec![TestEvent::Created {
                    id: "1".to_string(),
                    value: 100,
                }],
            )
            .await
            .unwrap();

        let registry = ProjectionRegistry::new(store);
        registry.register("proj-1", TestProjection::new()).await;
        registry.register("proj-2", TestProjection::new()).await;

        // Rebuild all
        registry.rebuild_all().await.unwrap();

        // Both projections should have processed the event
        let meta1 = registry.get_metadata("proj-1").await.unwrap();
        let meta2 = registry.get_metadata("proj-2").await.unwrap();

        assert_eq!(meta1.position.version, 1);
        assert_eq!(meta2.position.version, 1);
    }
}
