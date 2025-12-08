//! Resilience patterns for building fault-tolerant applications.
//!
//! This module provides comprehensive resilience primitives including:
//! - **Retry**: Exponential backoff with jitter, retry budgets, and adaptive
//!   retry
//! - **Rate Limiting**: Token bucket rate limiting with adaptive and keyed
//!   variants
//! - **Circuit Breaker**: Fail-fast pattern with configurable thresholds
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::resilience::{
//!     RetryExecutor, RetryConfig,
//!     RateLimiter,
//!     CircuitBreaker, CircuitBreakerConfig,
//! };
//!
//! // Retry with exponential backoff
//! let retry = RetryExecutor::new(RetryConfig::default());
//! let result = retry.execute("fetch_data", || async {
//!     // Your fallible operation
//!     Ok::<_, std::io::Error>("success")
//! }).await;
//!
//! // Rate limiting
//! let limiter = RateLimiter::new(100, 10); // 100 RPS, burst of 10
//! if limiter.check().is_ok() {
//!     // Proceed with request
//! }
//!
//! // Circuit breaker
//! let cb = CircuitBreaker::new("external_api", CircuitBreakerConfig::default());
//! let result = cb.call(|| async {
//!     // Your external call
//!     Ok::<_, std::io::Error>("response")
//! }).await;
//! ```

mod circuit_breaker;
mod rate_limit;
mod retry;

pub use circuit_breaker::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerManager, CircuitBreakerStats,
    CircuitOpenError, CircuitState,
};
pub use rate_limit::{
    AdaptiveRateLimiter, KeyedRateLimiter, RateLimitError, RateLimiter, RateLimiterStatus,
};
pub use retry::{AdaptiveRetry, RetryBudget, RetryConfig, RetryError, RetryExecutor, RetryPolicy};
