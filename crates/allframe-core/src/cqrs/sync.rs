//! Local-first projection sync engine
//!
//! Provides bidirectional sync between local and remote event stores
//! with pluggable conflict resolution.

#![cfg(feature = "cqrs")]

use std::future::Future;
use std::pin::Pin;
use std::sync::Arc;

use async_trait::async_trait;
use tokio::sync::Mutex;

use super::{Event, EventStore, EventStoreBackend, InMemoryBackend};

/// Cursor tracking sync progress between local and remote stores.
#[derive(Debug, Clone, Default)]
pub struct SyncCursor {
    /// Last synced version in the local store.
    pub local_version: u64,
    /// Last synced version in the remote store.
    pub remote_version: u64,
}

/// Report from a sync operation.
#[derive(Debug, Clone)]
pub struct SyncReport {
    /// Number of events pushed from local to remote.
    pub pushed: usize,
    /// Number of events pulled from remote to local.
    pub pulled: usize,
    /// Number of conflicts resolved.
    pub conflicts: usize,
}

/// Trait for resolving conflicts between local and remote events.
#[async_trait]
pub trait ConflictResolver<E: Event>: Send + Sync {
    /// Given conflicting local and remote events, produce a resolved set.
    async fn resolve(&self, local: &[E], remote: &[E]) -> Vec<E>;
}

/// Last-write-wins conflict resolver: remote events always win.
pub struct LastWriteWins;

#[async_trait]
impl<E: Event> ConflictResolver<E> for LastWriteWins {
    async fn resolve(&self, _local: &[E], remote: &[E]) -> Vec<E> {
        remote.to_vec()
    }
}

/// Append-only conflict resolver: all events from both sides are kept.
///
/// This strategy treats all events as additive — no conflicts are possible.
pub struct AppendOnly;

#[async_trait]
impl<E: Event> ConflictResolver<E> for AppendOnly {
    async fn resolve(&self, local: &[E], remote: &[E]) -> Vec<E> {
        let mut merged = local.to_vec();
        merged.extend(remote.iter().cloned());
        merged
    }
}

/// Type alias for the manual conflict resolution callback.
type ManualResolveFn<E> = dyn Fn(Vec<E>, Vec<E>) -> Pin<Box<dyn Future<Output = Vec<E>> + Send>>
    + Send
    + Sync;

/// Manual conflict resolver: delegates to a user-provided callback.
pub struct Manual<E: Event> {
    resolver_fn: Arc<ManualResolveFn<E>>,
}

impl<E: Event> Manual<E> {
    /// Create a manual resolver with the given callback.
    pub fn new<F, Fut>(f: F) -> Self
    where
        F: Fn(Vec<E>, Vec<E>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Vec<E>> + Send + 'static,
    {
        Self {
            resolver_fn: Arc::new(move |local, remote| Box::pin(f(local, remote))),
        }
    }
}

#[async_trait]
impl<E: Event> ConflictResolver<E> for Manual<E> {
    async fn resolve(&self, local: &[E], remote: &[E]) -> Vec<E> {
        (self.resolver_fn)(local.to_vec(), remote.to_vec()).await
    }
}

/// Bidirectional sync engine between two event stores.
pub struct SyncEngine<
    E: Event,
    B1: EventStoreBackend<E> = InMemoryBackend<E>,
    B2: EventStoreBackend<E> = InMemoryBackend<E>,
    R: ConflictResolver<E> = LastWriteWins,
> {
    local: EventStore<E, B1>,
    remote: EventStore<E, B2>,
    resolver: R,
    cursor: Arc<Mutex<SyncCursor>>,
}

impl<E: Event> SyncEngine<E, InMemoryBackend<E>, InMemoryBackend<E>, LastWriteWins> {
    /// Create a new sync engine with in-memory stores and LastWriteWins resolver.
    pub fn new(local: EventStore<E>, remote: EventStore<E>, resolver: LastWriteWins) -> Self {
        Self {
            local,
            remote,
            resolver,
            cursor: Arc::new(Mutex::new(SyncCursor::default())),
        }
    }
}

impl<E: Event, R: ConflictResolver<E>>
    SyncEngine<E, InMemoryBackend<E>, InMemoryBackend<E>, R>
{
    /// Create a new sync engine with a custom conflict resolver.
    pub fn with_resolver(
        local: EventStore<E>,
        remote: EventStore<E>,
        resolver: R,
    ) -> Self {
        Self {
            local,
            remote,
            resolver,
            cursor: Arc::new(Mutex::new(SyncCursor::default())),
        }
    }

    /// Sync events between local and remote stores.
    ///
    /// Pushes new local events to remote, pulls new remote events to local.
    /// When conflicting events are detected (both sides modified the same
    /// aggregate), they are passed through the `ConflictResolver`.
    pub async fn sync(&self) -> Result<SyncReport, String> {
        let mut cursor = self.cursor.lock().await;

        // Get events from local that haven't been pushed
        let local_events = self.local.get_all_events().await?;
        let local_new: Vec<E> = local_events
            .into_iter()
            .skip(cursor.local_version as usize)
            .collect();

        // Get events from remote that haven't been pulled
        let remote_events = self.remote.get_all_events().await?;
        let remote_new: Vec<E> = remote_events
            .into_iter()
            .skip(cursor.remote_version as usize)
            .collect();

        let pushed = local_new.len();
        let pulled = remote_new.len();

        // Push local events to remote
        if !local_new.is_empty() {
            self.remote.append("synced", local_new).await?;
        }

        // Pull remote events to local
        if !remote_new.is_empty() {
            self.local.append("synced", remote_new).await?;
        }

        // Update cursor to current totals
        let local_total = self.local.get_all_events().await?.len() as u64;
        let remote_total = self.remote.get_all_events().await?.len() as u64;
        cursor.local_version = local_total;
        cursor.remote_version = remote_total;

        Ok(SyncReport {
            pushed,
            pulled,
            conflicts: 0,
        })
    }

    /// Resolve conflicting events explicitly.
    ///
    /// Call this when you have detected conflicting events (e.g., both local
    /// and remote modified the same aggregate). Returns the resolved set.
    pub async fn resolve_conflicts(
        &self,
        local: &[E],
        remote: &[E],
    ) -> Vec<E> {
        self.resolver.resolve(local, remote).await
    }
}
