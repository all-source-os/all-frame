//! Event Versioning and Upcasting for schema evolution
//!
//! This module provides automatic event versioning and migration, eliminating
//! the need for manual version checking and conversion code.

use std::{any::TypeId, collections::HashMap, marker::PhantomData, sync::Arc};

use tokio::sync::RwLock;

use super::Event;

/// Trait for versioned events
pub trait VersionedEvent: Event {
    /// Get the version number of this event
    fn version(&self) -> u32;

    /// Get the event type name (for serialization)
    fn event_type(&self) -> &'static str;
}

/// Trait for upcasting events from one version to another
pub trait Upcaster<From: Event, To: Event>: Send + Sync {
    /// Convert an event from an older version to a newer version
    fn upcast(&self, from: From) -> To;
}

/// Automatic upcaster using From trait
pub struct AutoUpcaster<From: Event, To: Event> {
    _phantom: PhantomData<(From, To)>,
}

impl<From: Event, To: Event> AutoUpcaster<From, To>
where
    To: std::convert::From<From>,
{
    /// Create a new automatic upcaster
    pub fn new() -> Self {
        Self {
            _phantom: PhantomData,
        }
    }
}

impl<From: Event, To: Event> Default for AutoUpcaster<From, To>
where
    To: std::convert::From<From>,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<From: Event, To: Event> Upcaster<From, To> for AutoUpcaster<From, To>
where
    To: std::convert::From<From>,
{
    fn upcast(&self, from: From) -> To {
        from.into()
    }
}

/// Type-erased upcaster for registry storage
trait ErasedUpcaster<E: Event>: Send + Sync {
    /// Upcast an event, returning the new version
    #[allow(dead_code)]
    fn upcast_erased(&self, event: Box<dyn std::any::Any>) -> Option<E>;
}

/// Wrapper for concrete upcasters
struct UpcasterWrapper<From: Event, To: Event, U: Upcaster<From, To>> {
    #[allow(dead_code)]
    upcaster: Arc<U>,
    _phantom: PhantomData<(From, To)>,
}

impl<From: Event, To: Event, U: Upcaster<From, To>> ErasedUpcaster<To>
    for UpcasterWrapper<From, To, U>
{
    fn upcast_erased(&self, event: Box<dyn std::any::Any>) -> Option<To> {
        match event.downcast::<From>() {
            Ok(from_event) => Some(self.upcaster.upcast(*from_event)),
            Err(_) => None,
        }
    }
}

/// Type alias for upcaster storage
type UpcasterMap<E> = HashMap<(TypeId, TypeId), Box<dyn ErasedUpcaster<E>>>;

/// Migration path from one event version to another
#[derive(Debug, Clone)]
pub struct MigrationPath {
    /// Starting version
    pub from_version: u32,
    /// Target version
    pub to_version: u32,
    /// Event type name
    pub event_type: String,
}

impl MigrationPath {
    /// Create a new migration path
    pub fn new(from: u32, to: u32, event_type: impl Into<String>) -> Self {
        Self {
            from_version: from,
            to_version: to,
            event_type: event_type.into(),
        }
    }
}

/// Version registry for managing event versions and migrations
pub struct VersionRegistry<E: Event> {
    /// Registered upcasters by (from_type, to_type)
    upcasters: Arc<RwLock<UpcasterMap<E>>>,
    /// Migration paths by event type
    migrations: Arc<RwLock<HashMap<String, Vec<MigrationPath>>>>,
    _phantom: PhantomData<E>,
}

impl<E: Event> VersionRegistry<E> {
    /// Create a new version registry
    pub fn new() -> Self {
        Self {
            upcasters: Arc::new(RwLock::new(HashMap::new())),
            migrations: Arc::new(RwLock::new(HashMap::new())),
            _phantom: PhantomData,
        }
    }

    /// Register an upcaster for converting from one event version to another
    pub async fn register_upcaster<F: Event + 'static, U: Upcaster<F, E> + 'static>(
        &self,
        upcaster: U,
    ) {
        let from_type = TypeId::of::<F>();
        let to_type = TypeId::of::<E>();

        let wrapper = UpcasterWrapper {
            upcaster: Arc::new(upcaster),
            _phantom: PhantomData,
        };

        let mut upcasters = self.upcasters.write().await;
        upcasters.insert((from_type, to_type), Box::new(wrapper));
    }

    /// Register a migration path
    pub async fn register_migration(&self, path: MigrationPath) {
        let mut migrations = self.migrations.write().await;
        migrations
            .entry(path.event_type.clone())
            .or_insert_with(Vec::new)
            .push(path);
    }

    /// Get all registered migration paths
    pub async fn get_migrations(&self) -> Vec<MigrationPath> {
        let migrations = self.migrations.read().await;
        migrations.values().flatten().cloned().collect()
    }

    /// Get migrations for a specific event type
    pub async fn get_migrations_for(&self, event_type: &str) -> Vec<MigrationPath> {
        let migrations = self.migrations.read().await;
        migrations.get(event_type).cloned().unwrap_or_default()
    }

    /// Check if an upcaster is registered
    pub async fn has_upcaster<F: Event + 'static, T: Event + 'static>(&self) -> bool {
        let from_type = TypeId::of::<F>();
        let to_type = TypeId::of::<T>();
        let upcasters = self.upcasters.read().await;
        upcasters.contains_key(&(from_type, to_type))
    }

    /// Get number of registered upcasters
    pub async fn upcaster_count(&self) -> usize {
        self.upcasters.read().await.len()
    }

    /// Get number of registered migrations
    pub async fn migration_count(&self) -> usize {
        self.migrations.read().await.values().map(|v| v.len()).sum()
    }
}

impl<E: Event> Default for VersionRegistry<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Event> Clone for VersionRegistry<E> {
    fn clone(&self) -> Self {
        Self {
            upcasters: Arc::clone(&self.upcasters),
            migrations: Arc::clone(&self.migrations),
            _phantom: PhantomData,
        }
    }
}

/// Helper macro for defining versioned events (simplified version)
///
/// In a real implementation, this would be a proc macro that generates
/// the boilerplate automatically. For now, this is a documentation example.
///
/// # Example
/// ```ignore
/// #[versioned_event(version = 2)]
/// #[migration(from = 1, via = "upgrade_v1_to_v2")]
/// struct UserCreated {
///     user_id: String,
///     email: String,
///     #[added(version = 2, default = "Unknown")]
///     name: String,
/// }
/// ```

#[cfg(test)]
mod tests {
    use super::*;
    use crate::cqrs::EventTypeName;

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct UserCreatedV1 {
        user_id: String,
        email: String,
    }

    impl EventTypeName for UserCreatedV1 {}
    impl Event for UserCreatedV1 {}

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    struct UserCreatedV2 {
        user_id: String,
        email: String,
        name: String,
    }

    impl EventTypeName for UserCreatedV2 {}
    impl Event for UserCreatedV2 {}

    impl From<UserCreatedV1> for UserCreatedV2 {
        fn from(v1: UserCreatedV1) -> Self {
            Self {
                user_id: v1.user_id,
                email: v1.email,
                name: "Unknown".to_string(),
            }
        }
    }

    #[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
    enum TestEvent {
        #[allow(dead_code)]
        V1(UserCreatedV1),
        V2(UserCreatedV2),
    }

    impl EventTypeName for TestEvent {}
    impl Event for TestEvent {}

    impl From<UserCreatedV2> for TestEvent {
        fn from(v2: UserCreatedV2) -> Self {
            TestEvent::V2(v2)
        }
    }

    #[tokio::test]
    async fn test_registry_creation() {
        let registry: VersionRegistry<TestEvent> = VersionRegistry::new();
        assert_eq!(registry.upcaster_count().await, 0);
        assert_eq!(registry.migration_count().await, 0);
    }

    #[tokio::test]
    async fn test_upcaster_registration() {
        let registry: VersionRegistry<UserCreatedV2> = VersionRegistry::new();

        // Register automatic upcaster using From trait
        registry
            .register_upcaster(AutoUpcaster::<UserCreatedV1, UserCreatedV2>::new())
            .await;

        assert_eq!(registry.upcaster_count().await, 1);
        assert!(
            registry
                .has_upcaster::<UserCreatedV1, UserCreatedV2>()
                .await
        );
    }

    #[tokio::test]
    async fn test_migration_path_registration() {
        let registry: VersionRegistry<TestEvent> = VersionRegistry::new();

        let path = MigrationPath::new(1, 2, "UserCreated");
        registry.register_migration(path).await;

        assert_eq!(registry.migration_count().await, 1);

        let migrations = registry.get_migrations_for("UserCreated").await;
        assert_eq!(migrations.len(), 1);
        assert_eq!(migrations[0].from_version, 1);
        assert_eq!(migrations[0].to_version, 2);
    }

    #[tokio::test]
    async fn test_multiple_migrations() {
        let registry: VersionRegistry<TestEvent> = VersionRegistry::new();

        // Register migration chain: v1 -> v2 -> v3
        registry
            .register_migration(MigrationPath::new(1, 2, "UserCreated"))
            .await;
        registry
            .register_migration(MigrationPath::new(2, 3, "UserCreated"))
            .await;

        assert_eq!(registry.migration_count().await, 2);

        let migrations = registry.get_migrations_for("UserCreated").await;
        assert_eq!(migrations.len(), 2);
    }

    #[tokio::test]
    async fn test_auto_upcaster() {
        let upcaster = AutoUpcaster::<UserCreatedV1, UserCreatedV2>::new();

        let v1 = UserCreatedV1 {
            user_id: "123".to_string(),
            email: "test@example.com".to_string(),
        };

        let v2 = upcaster.upcast(v1.clone());

        assert_eq!(v2.user_id, v1.user_id);
        assert_eq!(v2.email, v1.email);
        assert_eq!(v2.name, "Unknown");
    }

    #[test]
    fn test_migration_path_creation() {
        let path = MigrationPath::new(1, 2, "UserCreated");

        assert_eq!(path.from_version, 1);
        assert_eq!(path.to_version, 2);
        assert_eq!(path.event_type, "UserCreated");
    }
}
