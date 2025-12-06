//! Cache abstraction for AllFrame applications
//!
//! This module provides a unified cache interface with support for multiple
//! backends including in-memory caching and Redis.
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::cache::{Cache, MemoryCache};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let cache = MemoryCache::new();
//!
//!     // Set a value with TTL
//!     cache.set("key", &"value", Some(Duration::from_secs(60))).await;
//!
//!     // Get the value
//!     let value: Option<String> = cache.get("key").await;
//!     assert_eq!(value, Some("value".to_string()));
//! }
//! ```

mod memory;
mod traits;

pub use memory::MemoryCache;
pub use traits::*;

/// Cache key generation trait
///
/// Implement this trait for types that can be used as cache keys.
pub trait CacheKey {
    /// Generate the cache key string
    fn cache_key(&self) -> String;
}

impl CacheKey for String {
    fn cache_key(&self) -> String {
        self.clone()
    }
}

impl CacheKey for &str {
    fn cache_key(&self) -> String {
        (*self).to_string()
    }
}

impl CacheKey for u64 {
    fn cache_key(&self) -> String {
        self.to_string()
    }
}

impl CacheKey for i64 {
    fn cache_key(&self) -> String {
        self.to_string()
    }
}

/// Cache configuration
#[derive(Debug, Clone, Default)]
pub struct CacheConfig {
    /// Key prefix for namespacing
    pub prefix: Option<String>,
    /// Default TTL for entries
    pub default_ttl: Option<std::time::Duration>,
    /// Maximum number of entries (for memory cache)
    pub max_entries: Option<usize>,
}

impl CacheConfig {
    /// Create a new cache configuration
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the key prefix
    pub fn prefix(mut self, prefix: impl Into<String>) -> Self {
        self.prefix = Some(prefix.into());
        self
    }

    /// Set the default TTL
    pub fn default_ttl(mut self, ttl: std::time::Duration) -> Self {
        self.default_ttl = Some(ttl);
        self
    }

    /// Set the maximum number of entries
    pub fn max_entries(mut self, max: usize) -> Self {
        self.max_entries = Some(max);
        self
    }

    /// Build a prefixed key
    pub fn build_key(&self, key: &str) -> String {
        match &self.prefix {
            Some(prefix) => format!("{}:{}", prefix, key),
            None => key.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_key_string() {
        let key = "test".to_string();
        assert_eq!(key.cache_key(), "test");
    }

    #[test]
    fn test_cache_key_str() {
        let key = "test";
        assert_eq!(key.cache_key(), "test");
    }

    #[test]
    fn test_cache_key_u64() {
        let key: u64 = 42;
        assert_eq!(key.cache_key(), "42");
    }

    #[test]
    fn test_cache_config_prefix() {
        let config = CacheConfig::new().prefix("myapp");
        assert_eq!(config.build_key("user:1"), "myapp:user:1");
    }

    #[test]
    fn test_cache_config_no_prefix() {
        let config = CacheConfig::new();
        assert_eq!(config.build_key("user:1"), "user:1");
    }
}
