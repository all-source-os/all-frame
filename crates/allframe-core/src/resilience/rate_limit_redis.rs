//! Redis-backed rate limiting for distributed deployments.
//!
//! Provides sliding window rate limiting using Redis as a backend,
//! allowing rate limits to be shared across multiple instances.
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::resilience::RedisRateLimiter;
//!
//! // Connect to Redis
//! let limiter = RedisRateLimiter::new("redis://localhost:6379", 100, 60).await?;
//!
//! // Check rate limit for a key
//! if limiter.check("user:123").await.is_ok() {
//!     // Process request
//! }
//! ```

use std::time::Duration;

use redis::{aio::ConnectionManager, AsyncCommands, Client};

use super::RateLimitError;

/// Configuration for Redis rate limiter.
#[derive(Debug, Clone)]
pub struct RedisRateLimiterConfig {
    /// Maximum requests allowed in the window.
    pub max_requests: u32,
    /// Time window in seconds.
    pub window_seconds: u64,
    /// Key prefix for Redis keys.
    pub key_prefix: String,
}

impl Default for RedisRateLimiterConfig {
    fn default() -> Self {
        Self {
            max_requests: 100,
            window_seconds: 60,
            key_prefix: "ratelimit".to_string(),
        }
    }
}

impl RedisRateLimiterConfig {
    /// Create a new config with specified limits.
    pub fn new(max_requests: u32, window_seconds: u64) -> Self {
        Self {
            max_requests,
            window_seconds,
            key_prefix: "ratelimit".to_string(),
        }
    }

    /// Set a custom key prefix.
    pub fn with_prefix(mut self, prefix: impl Into<String>) -> Self {
        self.key_prefix = prefix.into();
        self
    }
}

/// Error type for Redis rate limiter operations.
#[derive(Debug)]
pub enum RedisRateLimiterError {
    /// Redis connection error.
    Connection(String),
    /// Redis operation error.
    Redis(String),
}

impl std::fmt::Display for RedisRateLimiterError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RedisRateLimiterError::Connection(msg) => write!(f, "Redis connection error: {}", msg),
            RedisRateLimiterError::Redis(msg) => write!(f, "Redis error: {}", msg),
        }
    }
}

impl std::error::Error for RedisRateLimiterError {}

impl From<redis::RedisError> for RedisRateLimiterError {
    fn from(err: redis::RedisError) -> Self {
        RedisRateLimiterError::Redis(err.to_string())
    }
}

/// Redis-backed sliding window rate limiter.
///
/// Uses Redis sorted sets to implement a sliding window rate limiter
/// that works across distributed deployments.
///
/// ## Algorithm
///
/// Uses the sliding window log algorithm:
/// 1. Remove timestamps older than the window
/// 2. Count remaining timestamps
/// 3. If under limit, add current timestamp
/// 4. Return allow/deny
///
/// ## Features
///
/// - **Distributed**: Works across multiple instances
/// - **Sliding window**: More accurate than fixed windows
/// - **Auto-cleanup**: Old entries are automatically removed
pub struct RedisRateLimiter {
    conn: ConnectionManager,
    config: RedisRateLimiterConfig,
}

impl RedisRateLimiter {
    /// Create a new Redis rate limiter.
    ///
    /// # Arguments
    /// * `redis_url` - Redis connection URL (e.g., "redis://localhost:6379")
    /// * `max_requests` - Maximum requests per window
    /// * `window_seconds` - Window duration in seconds
    pub async fn new(
        redis_url: &str,
        max_requests: u32,
        window_seconds: u64,
    ) -> Result<Self, RedisRateLimiterError> {
        Self::with_config(
            redis_url,
            RedisRateLimiterConfig::new(max_requests, window_seconds),
        )
        .await
    }

    /// Create a new Redis rate limiter with custom configuration.
    pub async fn with_config(
        redis_url: &str,
        config: RedisRateLimiterConfig,
    ) -> Result<Self, RedisRateLimiterError> {
        let client = Client::open(redis_url)
            .map_err(|e| RedisRateLimiterError::Connection(e.to_string()))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| RedisRateLimiterError::Connection(e.to_string()))?;

        Ok(Self { conn, config })
    }

    /// Create from an existing Redis connection manager.
    pub fn from_connection(conn: ConnectionManager, config: RedisRateLimiterConfig) -> Self {
        Self { conn, config }
    }

    /// Check if a request for the given key is allowed.
    ///
    /// Returns `Ok(remaining)` with the number of remaining requests if allowed,
    /// or `Err(RateLimitError)` if rate limited.
    pub async fn check(&self, key: &str) -> Result<u32, RateLimitError> {
        let redis_key = format!("{}:{}", self.config.key_prefix, key);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        let window_start = now - (self.config.window_seconds as f64 * 1000.0);

        let mut conn = self.conn.clone();

        // Lua script for atomic rate limiting
        // This ensures the check-and-increment is atomic
        let script = redis::Script::new(
            r#"
            local key = KEYS[1]
            local now = tonumber(ARGV[1])
            local window_start = tonumber(ARGV[2])
            local max_requests = tonumber(ARGV[3])
            local window_ms = tonumber(ARGV[4])

            -- Remove old entries
            redis.call('ZREMRANGEBYSCORE', key, '-inf', window_start)

            -- Count current entries
            local count = redis.call('ZCARD', key)

            if count < max_requests then
                -- Add new entry
                redis.call('ZADD', key, now, now)
                -- Set expiry
                redis.call('PEXPIRE', key, window_ms)
                return max_requests - count - 1
            else
                -- Get oldest entry to calculate retry time
                local oldest = redis.call('ZRANGE', key, 0, 0, 'WITHSCORES')
                if #oldest > 0 then
                    return -(oldest[2] + window_ms - now)
                end
                return -1
            end
            "#,
        );

        let result: i64 = script
            .key(&redis_key)
            .arg(now)
            .arg(window_start)
            .arg(self.config.max_requests)
            .arg(self.config.window_seconds * 1000)
            .invoke_async(&mut conn)
            .await
            .map_err(|_| RateLimitError {
                retry_after: Duration::from_secs(1),
            })?;

        if result >= 0 {
            Ok(result as u32)
        } else {
            let retry_ms = (-result) as u64;
            Err(RateLimitError {
                retry_after: Duration::from_millis(retry_ms.max(1)),
            })
        }
    }

    /// Get the current count for a key without incrementing.
    pub async fn get_count(&self, key: &str) -> Result<u32, RedisRateLimiterError> {
        let redis_key = format!("{}:{}", self.config.key_prefix, key);
        let now = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis() as f64;

        let window_start = now - (self.config.window_seconds as f64 * 1000.0);

        let mut conn = self.conn.clone();

        // Remove old entries first
        let _: () = conn
            .zrembyscore(&redis_key, "-inf", window_start)
            .await?;

        // Count current entries
        let count: u32 = conn.zcard(&redis_key).await?;

        Ok(count)
    }

    /// Get remaining requests for a key.
    pub async fn get_remaining(&self, key: &str) -> Result<u32, RedisRateLimiterError> {
        let count = self.get_count(key).await?;
        Ok(self.config.max_requests.saturating_sub(count))
    }

    /// Reset the rate limit for a key.
    pub async fn reset(&self, key: &str) -> Result<(), RedisRateLimiterError> {
        let redis_key = format!("{}:{}", self.config.key_prefix, key);
        let mut conn = self.conn.clone();
        let _: () = conn.del(&redis_key).await?;
        Ok(())
    }

    /// Get the configuration.
    pub fn config(&self) -> &RedisRateLimiterConfig {
        &self.config
    }
}

/// Keyed Redis rate limiter with per-key configuration.
///
/// Allows different rate limits for different keys (e.g., different
/// limits for different API tiers).
pub struct KeyedRedisRateLimiter {
    conn: ConnectionManager,
    default_config: RedisRateLimiterConfig,
    /// Custom configs per key pattern
    custom_configs: std::collections::HashMap<String, RedisRateLimiterConfig>,
}

impl KeyedRedisRateLimiter {
    /// Create a new keyed Redis rate limiter.
    pub async fn new(
        redis_url: &str,
        default_config: RedisRateLimiterConfig,
    ) -> Result<Self, RedisRateLimiterError> {
        let client = Client::open(redis_url)
            .map_err(|e| RedisRateLimiterError::Connection(e.to_string()))?;

        let conn = ConnectionManager::new(client)
            .await
            .map_err(|e| RedisRateLimiterError::Connection(e.to_string()))?;

        Ok(Self {
            conn,
            default_config,
            custom_configs: std::collections::HashMap::new(),
        })
    }

    /// Set a custom configuration for a specific key.
    pub fn set_config(&mut self, key: impl Into<String>, config: RedisRateLimiterConfig) {
        self.custom_configs.insert(key.into(), config);
    }

    /// Remove custom configuration for a key.
    pub fn remove_config(&mut self, key: &str) {
        self.custom_configs.remove(key);
    }

    /// Check if a request for the given key is allowed.
    pub async fn check(&self, key: &str) -> Result<u32, RateLimitError> {
        let config = self.custom_configs.get(key).unwrap_or(&self.default_config);
        let limiter = RedisRateLimiter::from_connection(self.conn.clone(), config.clone());
        limiter.check(key).await
    }

    /// Get remaining requests for a key.
    pub async fn get_remaining(&self, key: &str) -> Result<u32, RedisRateLimiterError> {
        let config = self.custom_configs.get(key).unwrap_or(&self.default_config);
        let limiter = RedisRateLimiter::from_connection(self.conn.clone(), config.clone());
        limiter.get_remaining(key).await
    }

    /// Reset rate limit for a key.
    pub async fn reset(&self, key: &str) -> Result<(), RedisRateLimiterError> {
        let config = self.custom_configs.get(key).unwrap_or(&self.default_config);
        let limiter = RedisRateLimiter::from_connection(self.conn.clone(), config.clone());
        limiter.reset(key).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = RedisRateLimiterConfig::default();
        assert_eq!(config.max_requests, 100);
        assert_eq!(config.window_seconds, 60);
        assert_eq!(config.key_prefix, "ratelimit");
    }

    #[test]
    fn test_config_builder() {
        let config = RedisRateLimiterConfig::new(50, 30).with_prefix("myapp");

        assert_eq!(config.max_requests, 50);
        assert_eq!(config.window_seconds, 30);
        assert_eq!(config.key_prefix, "myapp");
    }

    #[test]
    fn test_error_display() {
        let err = RedisRateLimiterError::Connection("timeout".to_string());
        assert!(err.to_string().contains("timeout"));

        let err = RedisRateLimiterError::Redis("command failed".to_string());
        assert!(err.to_string().contains("command failed"));
    }

    // Integration tests require a running Redis instance
    // Run with: cargo test --features resilience-redis -- --ignored

    #[tokio::test]
    #[ignore = "requires Redis"]
    async fn test_redis_rate_limiter_basic() {
        let limiter = RedisRateLimiter::new("redis://localhost:6379", 5, 10)
            .await
            .expect("Failed to connect to Redis");

        // Reset any previous state
        limiter.reset("test:basic").await.ok();

        // Should allow 5 requests
        for i in 0..5 {
            let result = limiter.check("test:basic").await;
            assert!(result.is_ok(), "Request {} should be allowed", i);
        }

        // 6th request should be denied
        let result = limiter.check("test:basic").await;
        assert!(result.is_err(), "6th request should be denied");
    }

    #[tokio::test]
    #[ignore = "requires Redis"]
    async fn test_redis_rate_limiter_remaining() {
        let limiter = RedisRateLimiter::new("redis://localhost:6379", 10, 60)
            .await
            .expect("Failed to connect to Redis");

        limiter.reset("test:remaining").await.ok();

        // Initially should have 10 remaining
        let remaining = limiter.get_remaining("test:remaining").await.unwrap();
        assert_eq!(remaining, 10);

        // After 3 requests, should have 7 remaining
        for _ in 0..3 {
            limiter.check("test:remaining").await.ok();
        }

        let remaining = limiter.get_remaining("test:remaining").await.unwrap();
        assert_eq!(remaining, 7);
    }

    #[tokio::test]
    #[ignore = "requires Redis"]
    async fn test_redis_rate_limiter_reset() {
        let limiter = RedisRateLimiter::new("redis://localhost:6379", 2, 60)
            .await
            .expect("Failed to connect to Redis");

        limiter.reset("test:reset").await.ok();

        // Use up the limit
        limiter.check("test:reset").await.ok();
        limiter.check("test:reset").await.ok();

        // Should be denied
        assert!(limiter.check("test:reset").await.is_err());

        // Reset
        limiter.reset("test:reset").await.unwrap();

        // Should work again
        assert!(limiter.check("test:reset").await.is_ok());
    }
}
