//! Rate limiting primitives with adaptive and keyed variants.
//!
//! Provides token bucket rate limiting for controlling request throughput.

use std::{
    hash::Hash,
    num::NonZeroU32,
    sync::{
        atomic::{AtomicU32, AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant},
};

use dashmap::DashMap;
use governor::{
    clock::{Clock, DefaultClock},
    state::{InMemoryState, NotKeyed},
    Quota, RateLimiter as GovernorRateLimiter,
};
use parking_lot::RwLock;

/// Error returned when rate limit is exceeded.
#[derive(Debug, Clone)]
pub struct RateLimitError {
    /// When the rate limit will reset.
    pub retry_after: Duration,
}

impl std::fmt::Display for RateLimitError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "rate limit exceeded, retry after {:?}", self.retry_after)
    }
}

impl std::error::Error for RateLimitError {}

/// Status information for a rate limiter.
#[derive(Debug, Clone)]
pub struct RateLimiterStatus {
    /// Current requests per second.
    pub current_rps: f64,
    /// Configured maximum RPS.
    pub max_rps: u32,
    /// Burst capacity.
    pub burst_size: u32,
    /// Whether currently rate limited.
    pub is_limited: bool,
    /// Number of requests allowed in the last minute.
    pub requests_last_minute: u64,
    /// Number of requests rejected in the last minute.
    pub rejections_last_minute: u64,
}

/// Token bucket rate limiter.
///
/// Allows a sustained rate of requests with burst capacity.
pub struct RateLimiter {
    limiter: GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>,
    rps: u32,
    burst_size: u32,
    requests: AtomicU64,
    rejections: AtomicU64,
    last_reset: RwLock<Instant>,
}

impl RateLimiter {
    /// Create a new rate limiter.
    ///
    /// # Arguments
    /// * `rps` - Maximum requests per second
    /// * `burst_size` - Additional burst capacity above the sustained rate
    pub fn new(rps: u32, burst_size: u32) -> Self {
        let rps_nz = NonZeroU32::new(rps.max(1)).unwrap();
        let burst_nz = NonZeroU32::new(burst_size.max(1)).unwrap();

        let quota = Quota::per_second(rps_nz).allow_burst(burst_nz);
        let limiter = GovernorRateLimiter::direct(quota);

        Self {
            limiter,
            rps,
            burst_size,
            requests: AtomicU64::new(0),
            rejections: AtomicU64::new(0),
            last_reset: RwLock::new(Instant::now()),
        }
    }

    /// Check if a request is allowed without blocking.
    ///
    /// Returns `Ok(())` if allowed, `Err(RateLimitError)` if rate limited.
    pub fn check(&self) -> Result<(), RateLimitError> {
        self.maybe_reset_counters();

        match self.limiter.check() {
            Ok(_) => {
                self.requests.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(not_until) => {
                self.rejections.fetch_add(1, Ordering::Relaxed);
                Err(RateLimitError {
                    retry_after: not_until.wait_time_from(DefaultClock::default().now()),
                })
            }
        }
    }

    /// Wait until a request is allowed.
    ///
    /// Blocks the current task until the rate limit permits the request.
    pub async fn wait(&self) {
        self.maybe_reset_counters();
        self.limiter.until_ready().await;
        self.requests.fetch_add(1, Ordering::Relaxed);
    }

    /// Get the current status of the rate limiter.
    pub fn get_status(&self) -> RateLimiterStatus {
        self.maybe_reset_counters();

        let requests = self.requests.load(Ordering::Relaxed);
        let rejections = self.rejections.load(Ordering::Relaxed);
        let elapsed = self.last_reset.read().elapsed().as_secs_f64().max(1.0);

        RateLimiterStatus {
            current_rps: requests as f64 / elapsed.min(60.0),
            max_rps: self.rps,
            burst_size: self.burst_size,
            is_limited: self.limiter.check().is_err(),
            requests_last_minute: requests,
            rejections_last_minute: rejections,
        }
    }

    fn maybe_reset_counters(&self) {
        let mut last = self.last_reset.write();
        if last.elapsed() > Duration::from_secs(60) {
            self.requests.store(0, Ordering::Relaxed);
            self.rejections.store(0, Ordering::Relaxed);
            *last = Instant::now();
        }
    }
}

/// Adaptive rate limiter that backs off when receiving external rate limits.
///
/// When external services return 429 responses, this limiter reduces its
/// throughput to avoid hammering the service.
pub struct AdaptiveRateLimiter {
    /// Base rate limiter.
    base_rps: u32,
    burst_size: u32,
    /// Current effective RPS (may be reduced).
    current_rps: AtomicU32,
    /// Underlying rate limiter.
    limiter: RwLock<GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock>>,
    /// Consecutive rate limit responses.
    consecutive_limits: AtomicU32,
    /// Last rate limit time.
    last_limit: RwLock<Option<Instant>>,
    /// Recovery interval.
    recovery_interval: Duration,
    /// Minimum RPS (floor).
    min_rps: u32,
    /// Backoff factor when rate limited.
    backoff_factor: f64,
    /// Statistics.
    requests: AtomicU64,
    rejections: AtomicU64,
    external_limits: AtomicU64,
}

impl AdaptiveRateLimiter {
    /// Create a new adaptive rate limiter.
    ///
    /// # Arguments
    /// * `rps` - Base requests per second
    /// * `burst_size` - Burst capacity
    pub fn new(rps: u32, burst_size: u32) -> Self {
        let limiter = Self::create_limiter(rps, burst_size);

        Self {
            base_rps: rps,
            burst_size,
            current_rps: AtomicU32::new(rps),
            limiter: RwLock::new(limiter),
            consecutive_limits: AtomicU32::new(0),
            last_limit: RwLock::new(None),
            recovery_interval: Duration::from_secs(30),
            min_rps: 1,
            backoff_factor: 0.5,
            requests: AtomicU64::new(0),
            rejections: AtomicU64::new(0),
            external_limits: AtomicU64::new(0),
        }
    }

    /// Set the recovery interval.
    pub fn with_recovery_interval(mut self, interval: Duration) -> Self {
        self.recovery_interval = interval;
        self
    }

    /// Set the minimum RPS floor.
    pub fn with_min_rps(mut self, min_rps: u32) -> Self {
        self.min_rps = min_rps.max(1);
        self
    }

    /// Set the backoff factor (0.0-1.0).
    pub fn with_backoff_factor(mut self, factor: f64) -> Self {
        self.backoff_factor = factor.clamp(0.1, 0.9);
        self
    }

    fn create_limiter(
        rps: u32,
        burst_size: u32,
    ) -> GovernorRateLimiter<NotKeyed, InMemoryState, DefaultClock> {
        let rps_nz = NonZeroU32::new(rps.max(1)).unwrap();
        let burst_nz = NonZeroU32::new(burst_size.max(1)).unwrap();
        let quota = Quota::per_second(rps_nz).allow_burst(burst_nz);
        GovernorRateLimiter::direct(quota)
    }

    /// Record a successful request.
    pub fn record_success(&self) {
        self.consecutive_limits.store(0, Ordering::Relaxed);
        self.maybe_recover();
    }

    /// Record a rate limit response from an external service.
    pub fn record_rate_limit(&self) {
        self.external_limits.fetch_add(1, Ordering::Relaxed);
        let consecutive = self.consecutive_limits.fetch_add(1, Ordering::Relaxed) + 1;
        *self.last_limit.write() = Some(Instant::now());

        // Reduce rate based on consecutive rate limits
        let reduction = self.backoff_factor.powi(consecutive.min(5) as i32);
        let new_rps = ((self.base_rps as f64 * reduction) as u32).max(self.min_rps);

        self.current_rps.store(new_rps, Ordering::Relaxed);
        *self.limiter.write() = Self::create_limiter(new_rps, self.burst_size);
    }

    fn maybe_recover(&self) {
        let last_limit = *self.last_limit.read();
        if let Some(last) = last_limit {
            if last.elapsed() > self.recovery_interval {
                // Gradually recover to base rate
                let current = self.current_rps.load(Ordering::Relaxed);
                if current < self.base_rps {
                    let new_rps = ((current as f64 * 1.5) as u32).min(self.base_rps);
                    self.current_rps.store(new_rps, Ordering::Relaxed);
                    *self.limiter.write() = Self::create_limiter(new_rps, self.burst_size);

                    if new_rps >= self.base_rps {
                        *self.last_limit.write() = None;
                    }
                }
            }
        }
    }

    /// Check if a request is allowed.
    pub fn check(&self) -> Result<(), RateLimitError> {
        self.maybe_recover();

        match self.limiter.read().check() {
            Ok(_) => {
                self.requests.fetch_add(1, Ordering::Relaxed);
                Ok(())
            }
            Err(not_until) => {
                self.rejections.fetch_add(1, Ordering::Relaxed);
                Err(RateLimitError {
                    retry_after: not_until.wait_time_from(DefaultClock::default().now()),
                })
            }
        }
    }

    /// Wait until a request is allowed.
    pub async fn wait(&self) {
        self.maybe_recover();
        loop {
            let check_result = self.limiter.read().check();
            match check_result {
                Ok(_) => {
                    self.requests.fetch_add(1, Ordering::Relaxed);
                    return;
                }
                Err(not_until) => {
                    let wait_time =
                        not_until.wait_time_from(DefaultClock::default().now());
                    tokio::time::sleep(wait_time).await;
                }
            }
        }
    }

    /// Get the current status.
    pub fn get_status(&self) -> RateLimiterStatus {
        RateLimiterStatus {
            current_rps: self.current_rps.load(Ordering::Relaxed) as f64,
            max_rps: self.base_rps,
            burst_size: self.burst_size,
            is_limited: self.limiter.read().check().is_err(),
            requests_last_minute: self.requests.load(Ordering::Relaxed),
            rejections_last_minute: self.rejections.load(Ordering::Relaxed),
        }
    }

    /// Get external rate limit count.
    pub fn external_limit_count(&self) -> u64 {
        self.external_limits.load(Ordering::Relaxed)
    }
}

/// Per-key rate limiter for limiting different resources independently.
///
/// Useful for per-endpoint, per-user, or per-API-key rate limiting.
pub struct KeyedRateLimiter<K: Hash + Eq + Clone + Send + Sync + 'static> {
    limiters: DashMap<K, Arc<RateLimiter>>,
    default_rps: u32,
    default_burst: u32,
}

impl<K: Hash + Eq + Clone + Send + Sync + 'static> KeyedRateLimiter<K> {
    /// Create a new keyed rate limiter with default limits.
    ///
    /// # Arguments
    /// * `default_rps` - Default requests per second for new keys
    /// * `default_burst` - Default burst capacity for new keys
    pub fn new(default_rps: u32, default_burst: u32) -> Self {
        Self {
            limiters: DashMap::new(),
            default_rps,
            default_burst,
        }
    }

    /// Set a specific limit for a key.
    pub fn set_limit(&self, key: K, rps: u32, burst: u32) {
        self.limiters
            .insert(key, Arc::new(RateLimiter::new(rps, burst)));
    }

    /// Remove limit for a key (will use default on next access).
    pub fn remove_limit(&self, key: &K) {
        self.limiters.remove(key);
    }

    /// Check if a request for a key is allowed.
    pub fn check(&self, key: &K) -> Result<(), RateLimitError> {
        let limiter = self.get_or_create(key);
        limiter.check()
    }

    /// Wait until a request for a key is allowed.
    pub async fn wait(&self, key: &K) {
        let limiter = self.get_or_create(key);
        limiter.wait().await
    }

    /// Get status for a specific key.
    pub fn get_status(&self, key: &K) -> Option<RateLimiterStatus> {
        self.limiters.get(key).map(|l| l.get_status())
    }

    /// Get all keys with their status.
    pub fn get_all_status(&self) -> Vec<(K, RateLimiterStatus)> {
        self.limiters
            .iter()
            .map(|entry| (entry.key().clone(), entry.value().get_status()))
            .collect()
    }

    /// Clear all limiters.
    pub fn clear(&self) {
        self.limiters.clear();
    }

    fn get_or_create(&self, key: &K) -> Arc<RateLimiter> {
        self.limiters
            .entry(key.clone())
            .or_insert_with(|| Arc::new(RateLimiter::new(self.default_rps, self.default_burst)))
            .clone()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rate_limiter_basic() {
        let limiter = RateLimiter::new(10, 5);

        // Should allow initial burst
        for _ in 0..5 {
            assert!(limiter.check().is_ok());
        }
    }

    #[test]
    fn test_rate_limiter_status() {
        let limiter = RateLimiter::new(100, 10);
        let status = limiter.get_status();

        assert_eq!(status.max_rps, 100);
        assert_eq!(status.burst_size, 10);
    }

    #[test]
    fn test_adaptive_rate_limiter_backoff() {
        let limiter = AdaptiveRateLimiter::new(100, 10).with_backoff_factor(0.5);

        // Simulate rate limit responses
        limiter.record_rate_limit();
        let status1 = limiter.get_status();
        assert!(status1.current_rps < 100.0);

        limiter.record_rate_limit();
        let status2 = limiter.get_status();
        assert!(status2.current_rps < status1.current_rps);
    }

    #[test]
    fn test_adaptive_rate_limiter_recovery() {
        let limiter = AdaptiveRateLimiter::new(100, 10)
            .with_recovery_interval(Duration::from_millis(1))
            .with_backoff_factor(0.5);

        limiter.record_rate_limit();
        let reduced = limiter.get_status().current_rps;
        assert!(reduced < 100.0);

        // Record success after recovery interval
        std::thread::sleep(Duration::from_millis(10));
        limiter.record_success();
        // Note: Recovery is gradual, may need multiple cycles
    }

    #[test]
    fn test_keyed_rate_limiter() {
        let limiter = KeyedRateLimiter::new(10, 5);

        // Different keys should have independent limits
        for _ in 0..5 {
            assert!(limiter.check(&"key1").is_ok());
            assert!(limiter.check(&"key2").is_ok());
        }
    }

    #[test]
    fn test_keyed_rate_limiter_custom_limits() {
        let limiter = KeyedRateLimiter::new(10, 5);

        // Set custom limit for specific key
        limiter.set_limit("premium", 100, 50);

        let status = limiter.get_status(&"premium").unwrap();
        assert_eq!(status.max_rps, 100);
        assert_eq!(status.burst_size, 50);
    }

    #[test]
    fn test_keyed_rate_limiter_all_status() {
        let limiter = KeyedRateLimiter::new(10, 5);

        limiter.check(&"a").ok();
        limiter.check(&"b").ok();
        limiter.check(&"c").ok();

        let all = limiter.get_all_status();
        assert_eq!(all.len(), 3);
    }

    #[test]
    fn test_rate_limit_error_display() {
        let err = RateLimitError {
            retry_after: Duration::from_secs(5),
        };
        let msg = format!("{}", err);
        assert!(msg.contains("rate limit exceeded"));
        assert!(msg.contains("5"));
    }

    #[tokio::test]
    async fn test_rate_limiter_wait() {
        let limiter = RateLimiter::new(1000, 100);

        let start = Instant::now();
        for _ in 0..10 {
            limiter.wait().await;
        }
        // Should complete quickly with high RPS
        assert!(start.elapsed() < Duration::from_secs(1));
    }
}
