//! Cache traits and abstractions

use std::{future::Future, pin::Pin, time::Duration};

use serde::{de::DeserializeOwned, Serialize};

/// A cache backend that can store and retrieve values
///
/// This trait defines the core operations for any cache implementation.
/// All operations are async to support both local and remote cache backends.
pub trait Cache: Send + Sync {
    /// Get a value from the cache
    ///
    /// Returns `None` if the key doesn't exist or has expired.
    fn get<T: DeserializeOwned + Send>(
        &self,
        key: &str,
    ) -> Pin<Box<dyn Future<Output = Option<T>> + Send + '_>>;

    /// Set a value in the cache with an optional TTL
    ///
    /// If `ttl` is `None`, the cache's default TTL will be used (if any),
    /// otherwise the entry won't expire.
    fn set<T: Serialize + Send + Sync>(
        &self,
        key: &str,
        value: &T,
        ttl: Option<Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    /// Delete a value from the cache
    ///
    /// Returns `true` if the key existed and was deleted.
    fn delete(&self, key: &str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;

    /// Check if a key exists in the cache
    fn exists(&self, key: &str) -> Pin<Box<dyn Future<Output = bool> + Send + '_>>;

    /// Get multiple values from the cache
    ///
    /// Returns a vector of optional values in the same order as the keys.
    fn get_many<T: DeserializeOwned + Send>(
        &self,
        keys: &[&str],
    ) -> Pin<Box<dyn Future<Output = Vec<Option<T>>> + Send + '_>> {
        let keys: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
        Box::pin(async move {
            let mut results = Vec::with_capacity(keys.len());
            for key in keys {
                results.push(self.get(&key).await);
            }
            results
        })
    }

    /// Set multiple values in the cache
    ///
    /// Default implementation calls `set` for each entry sequentially.
    /// Backends may override this with a more efficient batch operation.
    fn set_many<'a, T: Serialize + Send + Sync + 'a>(
        &'a self,
        entries: Vec<(String, T)>,
        ttl: Option<Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        Box::pin(async move {
            for (key, value) in entries {
                self.set(&key, &value, ttl).await;
            }
        })
    }

    /// Delete multiple keys from the cache
    ///
    /// Returns the number of keys that were deleted.
    fn delete_many(&self, keys: &[&str]) -> Pin<Box<dyn Future<Output = usize> + Send + '_>> {
        let keys: Vec<String> = keys.iter().map(|k| k.to_string()).collect();
        Box::pin(async move {
            let mut count = 0;
            for key in keys {
                if self.delete(&key).await {
                    count += 1;
                }
            }
            count
        })
    }

    /// Clear all entries from the cache
    fn clear(&self) -> Pin<Box<dyn Future<Output = ()> + Send + '_>>;

    /// Get the number of entries in the cache (if supported)
    ///
    /// Returns `None` if the cache doesn't support this operation.
    fn len(&self) -> Pin<Box<dyn Future<Output = Option<usize>> + Send + '_>> {
        Box::pin(async { None })
    }

    /// Check if the cache is empty
    fn is_empty(&self) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        Box::pin(async move { self.len().await.map(|n| n == 0).unwrap_or(true) })
    }
}

/// Extension trait for cache operations with typed keys
pub trait CacheExt: Cache {
    /// Get a value using a typed cache key
    fn get_keyed<K: super::CacheKey, T: DeserializeOwned + Send>(
        &self,
        key: &K,
    ) -> Pin<Box<dyn Future<Output = Option<T>> + Send + '_>> {
        let key = key.cache_key();
        Box::pin(async move { self.get(&key).await })
    }

    /// Set a value using a typed cache key
    fn set_keyed<'a, K: super::CacheKey, T: Serialize + Send + Sync + 'a>(
        &'a self,
        key: &K,
        value: T,
        ttl: Option<Duration>,
    ) -> Pin<Box<dyn Future<Output = ()> + Send + 'a>> {
        let key = key.cache_key();
        Box::pin(async move { self.set(&key, &value, ttl).await })
    }

    /// Delete a value using a typed cache key
    fn delete_keyed<K: super::CacheKey>(
        &self,
        key: &K,
    ) -> Pin<Box<dyn Future<Output = bool> + Send + '_>> {
        let key = key.cache_key();
        Box::pin(async move { self.delete(&key).await })
    }
}

// Blanket implementation for all Cache types
impl<C: Cache + ?Sized> CacheExt for C {}

/// Result type for cache operations that can fail
pub type CacheResult<T> = Result<T, CacheError>;

/// Errors that can occur during cache operations
#[derive(Debug, Clone)]
pub enum CacheError {
    /// Serialization error
    Serialization(String),
    /// Deserialization error
    Deserialization(String),
    /// Connection error
    Connection(String),
    /// Operation timed out
    Timeout,
    /// Other error
    Other(String),
}

impl std::fmt::Display for CacheError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CacheError::Serialization(msg) => write!(f, "Serialization error: {}", msg),
            CacheError::Deserialization(msg) => write!(f, "Deserialization error: {}", msg),
            CacheError::Connection(msg) => write!(f, "Connection error: {}", msg),
            CacheError::Timeout => write!(f, "Operation timed out"),
            CacheError::Other(msg) => write!(f, "Cache error: {}", msg),
        }
    }
}

impl std::error::Error for CacheError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_error_display() {
        let err = CacheError::Connection("refused".into());
        assert!(err.to_string().contains("refused"));
    }
}
