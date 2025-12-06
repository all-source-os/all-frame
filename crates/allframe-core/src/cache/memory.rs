//! In-memory cache implementation

use std::{
    collections::HashMap,
    future::Future,
    pin::Pin,
    sync::RwLock,
    time::{Duration, Instant},
};

use serde::{de::DeserializeOwned, Serialize};

use super::{Cache, CacheConfig};

/// An entry in the memory cache
struct CacheEntry {
    /// Serialized value
    data: Vec<u8>,
    /// Expiration time (if any)
    expires_at: Option<Instant>,
}

impl CacheEntry {
    fn new(data: Vec<u8>, ttl: Option<Duration>) -> Self {
        Self {
            data,
            expires_at: ttl.map(|d| Instant::now() + d),
        }
    }

    fn is_expired(&self) -> bool {
        self.expires_at.map(|t| Instant::now() > t).unwrap_or(false)
    }
}

/// In-memory cache with TTL support
///
/// This cache stores values in memory using a `HashMap` with optional
/// time-to-live (TTL) for entries. It's suitable for single-process
/// applications or as a fallback cache.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::cache::{Cache, MemoryCache, CacheConfig};
/// use std::time::Duration;
///
/// let cache = MemoryCache::with_config(
///     CacheConfig::new()
///         .prefix("myapp")
///         .default_ttl(Duration::from_secs(300))
/// );
///
/// cache.set("key", &"value", None).await;
/// let value: Option<String> = cache.get("key").await;
/// ```
pub struct MemoryCache {
    entries: RwLock<HashMap<String, CacheEntry>>,
    config: CacheConfig,
}

impl MemoryCache {
    /// Create a new memory cache with default configuration
    pub fn new() -> Self {
        Self::with_config(CacheConfig::default())
    }

    /// Create a new memory cache with custom configuration
    pub fn with_config(config: CacheConfig) -> Self {
        Self {
            entries: RwLock::new(HashMap::new()),
            config,
        }
    }

    /// Create a builder for the memory cache
    pub fn builder() -> MemoryCacheBuilder {
        MemoryCacheBuilder::new()
    }

    /// Remove expired entries from the cache
    ///
    /// This is called automatically on get operations, but can be called
    /// manually if needed.
    pub fn cleanup_expired(&self) {
        let mut entries = self.entries.write().unwrap();
        entries.retain(|_, entry| !entry.is_expired());
    }

    /// Get the effective TTL for an entry
    fn effective_ttl(&self, ttl: Option<Duration>) -> Option<Duration> {
        ttl.or(self.config.default_ttl)
    }

    /// Build the full key with prefix
    fn full_key(&self, key: &str) -> String {
        self.config.build_key(key)
    }
}

impl Default for MemoryCache {
    fn default() -> Self {
        Self::new()
    }
}

impl Cache for MemoryCache {
    fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Option<T>> + Send + '_>> {
        let key = self.full_key(key);
        Box::pin(async move {
            let entries = self.entries.read().unwrap();
            entries.get(&key).and_then(|entry| {
                if entry.is_expired() {
                    None
                } else {
                    serde_json::from_slice(&entry.data).ok()
                }
            })
        })
    }

    fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        let key = self.full_key(key);
        let ttl = self.effective_ttl(ttl);

        // Serialize outside the lock
        let data = match serde_json::to_vec(value) {
            Ok(d) => d,
            Err(_) => return Box::pin(async {}),
        };

        Box::pin(async move {
            let mut entries = self.entries.write().unwrap();

            // Check max entries limit
            if let Some(max) = self.config.max_entries {
                if entries.len() >= max && !entries.contains_key(&key) {
                    // Remove expired entries first
                    entries.retain(|_, entry| !entry.is_expired());

                    // If still at limit, remove oldest entry (simple LRU approximation)
                    if entries.len() >= max {
                        if let Some(oldest_key) = entries.keys().next().cloned() {
                            entries.remove(&oldest_key);
                        }
                    }
                }
            }

            entries.insert(key, CacheEntry::new(data, ttl));
        })
    }

    fn delete(&self, key: &str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let key = self.full_key(key);
        Box::pin(async move {
            let mut entries = self.entries.write().unwrap();
            entries.remove(&key).is_some()
        })
    }

    fn exists(&self, key: &str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let key = self.full_key(key);
        Box::pin(async move {
            let entries = self.entries.read().unwrap();
            entries
                .get(&key)
                .map(|entry| !entry.is_expired())
                .unwrap_or(false)
        })
    }

    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>> {
        Box::pin(async move {
            let mut entries = self.entries.write().unwrap();
            entries.clear();
        })
    }

    fn len(&self) -> Pin<Box<dyn Future<Output = Option<usize>> + Send + '_>> {
        Box::pin(async move {
            let entries = self.entries.read().unwrap();
            // Count non-expired entries
            let count = entries.values().filter(|e| !e.is_expired()).count();
            Some(count)
        })
    }
}

/// Builder for MemoryCache
pub struct MemoryCacheBuilder {
    config: CacheConfig,
}

impl MemoryCacheBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self {
            config: CacheConfig::default(),
        }
    }

    /// Set the key prefix
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.config.prefix = Some(prefix.into());
        self
    }

    /// Set the default TTL
    pub fn default_ttl(mut self, ttl: Duration) -> Self {
        self.config.default_ttl = Some(ttl);
        self
    }

    /// Set the maximum number of entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.config.max_entries = Some(max);
        self
    }

    /// Build the cache
    pub fn build(self) -> MemoryCache {
        MemoryCache::with_config(self.config)
    }
}

impl Default for MemoryCacheBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_cache_basic() {
        let cache = MemoryCache::new();

        cache.set("key", &"value", None).await;
        let value: Option<String> = cache.get("key").await;
        assert_eq!(value, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_memory_cache_missing_key() {
        let cache = MemoryCache::new();
        let value: Option<String> = cache.get("nonexistent").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_cache_delete() {
        let cache = MemoryCache::new();

        cache.set("key", &"value", None).await;
        assert!(cache.exists("key").await);

        let deleted = cache.delete("key").await;
        assert!(deleted);
        assert!(!cache.exists("key").await);
    }

    #[tokio::test]
    async fn test_memory_cache_delete_nonexistent() {
        let cache = MemoryCache::new();
        let deleted = cache.delete("nonexistent").await;
        assert!(!deleted);
    }

    #[tokio::test]
    async fn test_memory_cache_clear() {
        let cache = MemoryCache::new();

        cache.set("key1", &"value1", None).await;
        cache.set("key2", &"value2", None).await;

        cache.clear().await;

        assert!(!cache.exists("key1").await);
        assert!(!cache.exists("key2").await);
    }

    #[tokio::test]
    async fn test_memory_cache_len() {
        let cache = MemoryCache::new();

        assert_eq!(cache.len().await, Some(0));

        cache.set("key1", &"value1", None).await;
        cache.set("key2", &"value2", None).await;

        assert_eq!(cache.len().await, Some(2));
    }

    #[tokio::test]
    async fn test_memory_cache_ttl_expired() {
        let cache = MemoryCache::new();

        // Set with very short TTL
        cache
            .set("key", &"value", Some(Duration::from_millis(1)))
            .await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        let value: Option<String> = cache.get("key").await;
        assert_eq!(value, None);
    }

    #[tokio::test]
    async fn test_memory_cache_ttl_not_expired() {
        let cache = MemoryCache::new();

        cache
            .set("key", &"value", Some(Duration::from_secs(60)))
            .await;

        let value: Option<String> = cache.get("key").await;
        assert_eq!(value, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_memory_cache_with_prefix() {
        let cache = MemoryCache::builder().prefix("test").build();

        cache.set("key", &"value", None).await;

        // The internal key should have the prefix
        let value: Option<String> = cache.get("key").await;
        assert_eq!(value, Some("value".to_string()));
    }

    #[tokio::test]
    async fn test_memory_cache_max_entries() {
        let cache = MemoryCache::builder().max_entries(2).build();

        cache.set("key1", &"value1", None).await;
        cache.set("key2", &"value2", None).await;
        cache.set("key3", &"value3", None).await;

        // Should have at most 2 entries
        assert!(cache.len().await.unwrap() <= 2);
    }

    #[tokio::test]
    async fn test_memory_cache_complex_type() {
        #[derive(serde::Serialize, serde::Deserialize, PartialEq, Debug)]
        struct User {
            id: u64,
            name: String,
        }

        let cache = MemoryCache::new();
        let user = User {
            id: 1,
            name: "Alice".to_string(),
        };

        cache.set("user:1", &user, None).await;

        let retrieved: Option<User> = cache.get("user:1").await;
        assert_eq!(retrieved, Some(user));
    }

    #[tokio::test]
    async fn test_memory_cache_builder_default_ttl() {
        let cache = MemoryCache::builder()
            .default_ttl(Duration::from_millis(1))
            .build();

        // Set without explicit TTL - should use default
        cache.set("key", &"value", None).await;

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(10)).await;

        let value: Option<String> = cache.get("key").await;
        assert_eq!(value, None);
    }
}
