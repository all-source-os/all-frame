//! Retry patterns with exponential backoff, jitter, and adaptive behavior.
//!
//! Provides retry mechanisms for transient failures with configurable backoff
//! strategies.

use std::{
    future::Future,
    sync::atomic::{AtomicU32, Ordering},
    time::{Duration, Instant},
};

use parking_lot::RwLock;
use rand::Rng;

/// Configuration for retry behavior.
#[derive(Debug, Clone)]
pub struct RetryConfig {
    /// Maximum number of retry attempts (0 = no retries, just the initial
    /// attempt).
    pub max_retries: u32,
    /// Initial interval between retries.
    pub initial_interval: Duration,
    /// Maximum interval between retries.
    pub max_interval: Duration,
    /// Multiplier for exponential backoff.
    pub multiplier: f64,
    /// Randomization factor for jitter (0.0 = no jitter, 0.5 = +/- 50%).
    pub randomization_factor: f64,
    /// Maximum total elapsed time for all retries. None = no limit.
    pub max_elapsed_time: Option<Duration>,
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_retries: 3,
            initial_interval: Duration::from_millis(500),
            max_interval: Duration::from_secs(30),
            multiplier: 2.0,
            randomization_factor: 0.5,
            max_elapsed_time: Some(Duration::from_secs(60)),
        }
    }
}

impl RetryConfig {
    /// Create a new retry config with specified max retries.
    pub fn new(max_retries: u32) -> Self {
        Self {
            max_retries,
            ..Default::default()
        }
    }

    /// Set the initial interval.
    pub fn with_initial_interval(mut self, interval: Duration) -> Self {
        self.initial_interval = interval;
        self
    }

    /// Set the maximum interval.
    pub fn with_max_interval(mut self, interval: Duration) -> Self {
        self.max_interval = interval;
        self
    }

    /// Set the backoff multiplier.
    pub fn with_multiplier(mut self, multiplier: f64) -> Self {
        self.multiplier = multiplier;
        self
    }

    /// Set the randomization factor for jitter.
    pub fn with_randomization_factor(mut self, factor: f64) -> Self {
        self.randomization_factor = factor.clamp(0.0, 1.0);
        self
    }

    /// Set the maximum elapsed time.
    pub fn with_max_elapsed_time(mut self, time: Option<Duration>) -> Self {
        self.max_elapsed_time = time;
        self
    }

    /// Calculate the next backoff interval with jitter.
    pub fn calculate_interval(&self, attempt: u32) -> Duration {
        let base = self.initial_interval.as_secs_f64() * self.multiplier.powi(attempt as i32);
        let capped = base.min(self.max_interval.as_secs_f64());

        // Apply jitter
        let jitter_range = capped * self.randomization_factor;
        let mut rng = rand::thread_rng();
        let jitter = rng.gen_range(-jitter_range..=jitter_range);
        let final_interval = (capped + jitter).max(0.0);

        Duration::from_secs_f64(final_interval)
    }
}

/// Error returned when all retry attempts fail.
#[derive(Debug)]
pub struct RetryError<E> {
    /// The last error encountered.
    pub last_error: E,
    /// Number of attempts made.
    pub attempts: u32,
    /// Total elapsed time.
    pub elapsed: Duration,
}

impl<E: std::fmt::Display> std::fmt::Display for RetryError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "retry exhausted after {} attempts ({:?}): {}",
            self.attempts, self.elapsed, self.last_error
        )
    }
}

impl<E: std::error::Error + 'static> std::error::Error for RetryError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        Some(&self.last_error)
    }
}

/// Trait for determining if an error should trigger a retry.
pub trait RetryPolicy: Send + Sync {
    /// Returns true if the operation should be retried for this error.
    fn should_retry(&self, error: &dyn std::error::Error) -> bool;
}

/// Default retry policy that retries all errors.
#[derive(Debug, Clone, Default)]
pub struct AlwaysRetry;

impl RetryPolicy for AlwaysRetry {
    fn should_retry(&self, _error: &dyn std::error::Error) -> bool {
        true
    }
}

/// Retry policy that never retries.
#[derive(Debug, Clone, Default)]
#[allow(dead_code)]
pub struct NeverRetry;

impl RetryPolicy for NeverRetry {
    fn should_retry(&self, _error: &dyn std::error::Error) -> bool {
        false
    }
}

/// Executes async operations with exponential backoff and jitter.
pub struct RetryExecutor<P: RetryPolicy = AlwaysRetry> {
    config: RetryConfig,
    policy: P,
}

impl RetryExecutor<AlwaysRetry> {
    /// Create a new retry executor with default policy.
    pub fn new(config: RetryConfig) -> Self {
        Self {
            config,
            policy: AlwaysRetry,
        }
    }
}

impl<P: RetryPolicy> RetryExecutor<P> {
    /// Create a retry executor with a custom policy.
    pub fn with_policy(config: RetryConfig, policy: P) -> Self {
        Self { config, policy }
    }

    /// Execute an async operation with retries.
    ///
    /// The operation will be retried according to the configuration until:
    /// - It succeeds
    /// - Max retries is reached
    /// - Max elapsed time is reached
    /// - The retry policy says not to retry
    pub async fn execute<F, Fut, T, E>(&self, name: &str, mut f: F) -> Result<T, RetryError<E>>
    where
        F: FnMut() -> Fut,
        Fut: Future<Output = Result<T, E>>,
        E: std::error::Error + 'static,
    {
        let start = Instant::now();
        let mut attempts = 0u32;

        loop {
            attempts += 1;

            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    // Check if we should retry
                    if !self.policy.should_retry(&e) {
                        return Err(RetryError {
                            last_error: e,
                            attempts,
                            elapsed: start.elapsed(),
                        });
                    }

                    // Check if we've exceeded max retries
                    if attempts > self.config.max_retries {
                        return Err(RetryError {
                            last_error: e,
                            attempts,
                            elapsed: start.elapsed(),
                        });
                    }

                    // Check if we've exceeded max elapsed time
                    if let Some(max_elapsed) = self.config.max_elapsed_time {
                        if start.elapsed() >= max_elapsed {
                            return Err(RetryError {
                                last_error: e,
                                attempts,
                                elapsed: start.elapsed(),
                            });
                        }
                    }

                    // Calculate and wait for backoff interval
                    let interval = self.config.calculate_interval(attempts - 1);

                    // Log retry attempt (could be made configurable)
                    #[cfg(feature = "otel")]
                    tracing::debug!(
                        operation = name,
                        attempt = attempts,
                        next_retry_in = ?interval,
                        "retrying operation"
                    );
                    let _ = name; // Silence unused warning when otel is disabled

                    tokio::time::sleep(interval).await;
                }
            }
        }
    }

    /// Get the configuration.
    pub fn config(&self) -> &RetryConfig {
        &self.config
    }
}

/// Prevents retry storms by limiting total retry capacity.
///
/// A retry budget tracks available retry tokens and prevents excessive
/// retries when the system is under stress.
pub struct RetryBudget {
    /// Maximum tokens available.
    max_tokens: u32,
    /// Current available tokens.
    tokens: AtomicU32,
    /// Tokens recovered per second.
    recovery_rate: f64,
    /// Last recovery time.
    last_recovery: RwLock<Instant>,
}

impl RetryBudget {
    /// Create a new retry budget.
    ///
    /// # Arguments
    /// * `max_tokens` - Maximum retry tokens available
    /// * `recovery_rate` - Tokens recovered per second
    pub fn new(max_tokens: u32, recovery_rate: f64) -> Self {
        Self {
            max_tokens,
            tokens: AtomicU32::new(max_tokens),
            recovery_rate,
            last_recovery: RwLock::new(Instant::now()),
        }
    }

    /// Try to consume retry tokens.
    ///
    /// Returns true if tokens were available and consumed.
    pub fn try_consume(&self, amount: u32) -> bool {
        self.recover_tokens();

        loop {
            let current = self.tokens.load(Ordering::Acquire);
            if current < amount {
                return false;
            }

            if self
                .tokens
                .compare_exchange(
                    current,
                    current - amount,
                    Ordering::AcqRel,
                    Ordering::Relaxed,
                )
                .is_ok()
            {
                return true;
            }
        }
    }

    /// Get remaining tokens.
    pub fn remaining(&self) -> u32 {
        self.recover_tokens();
        self.tokens.load(Ordering::Acquire)
    }

    /// Recover tokens based on elapsed time.
    fn recover_tokens(&self) {
        let mut last = self.last_recovery.write();
        let elapsed = last.elapsed();

        if elapsed.as_secs_f64() > 0.1 {
            // Only recover every 100ms
            let recovered = (elapsed.as_secs_f64() * self.recovery_rate) as u32;
            if recovered > 0 {
                let current = self.tokens.load(Ordering::Acquire);
                let new_value = (current + recovered).min(self.max_tokens);
                self.tokens.store(new_value, Ordering::Release);
                *last = Instant::now();
            }
        }
    }

    /// Reset the budget to full capacity.
    pub fn reset(&self) {
        self.tokens.store(self.max_tokens, Ordering::Release);
        *self.last_recovery.write() = Instant::now();
    }
}

impl Default for RetryBudget {
    fn default() -> Self {
        Self::new(100, 10.0) // 100 tokens, recover 10/second
    }
}

/// Adapts retry behavior based on recent success/failure rates.
///
/// When failures increase, the retry configuration becomes more conservative
/// (longer delays, fewer retries). When success rate improves, it relaxes.
pub struct AdaptiveRetry {
    base_config: RetryConfig,
    /// Recent outcomes (true = success, false = failure).
    outcomes: RwLock<Vec<(Instant, bool)>>,
    /// Window size for calculating success rate.
    window: Duration,
}

impl AdaptiveRetry {
    /// Create a new adaptive retry with base configuration.
    pub fn new(base_config: RetryConfig) -> Self {
        Self {
            base_config,
            outcomes: RwLock::new(Vec::new()),
            window: Duration::from_secs(60),
        }
    }

    /// Create with a custom window size.
    pub fn with_window(mut self, window: Duration) -> Self {
        self.window = window;
        self
    }

    /// Record an operation outcome.
    pub fn record_outcome(&self, success: bool) {
        let mut outcomes = self.outcomes.write();
        outcomes.push((Instant::now(), success));

        // Prune old entries
        let cutoff = Instant::now() - self.window;
        outcomes.retain(|(time, _)| *time > cutoff);
    }

    /// Get success rate (0.0 - 1.0).
    pub fn success_rate(&self) -> f64 {
        let outcomes = self.outcomes.read();
        if outcomes.is_empty() {
            return 1.0;
        }

        let successes = outcomes.iter().filter(|(_, s)| *s).count();
        successes as f64 / outcomes.len() as f64
    }

    /// Get adjusted configuration based on success rate.
    ///
    /// Lower success rates result in:
    /// - Longer initial intervals
    /// - Fewer max retries
    /// - Higher multiplier
    pub fn get_adjusted_config(&self) -> RetryConfig {
        let success_rate = self.success_rate();

        // Scale factor: 1.0 at 100% success, up to 3.0 at 0% success
        let scale = 1.0 + (2.0 * (1.0 - success_rate));

        let mut config = self.base_config.clone();

        // Increase initial interval
        config.initial_interval =
            Duration::from_secs_f64(self.base_config.initial_interval.as_secs_f64() * scale);

        // Reduce max retries when success rate is low
        if success_rate < 0.5 {
            config.max_retries = (self.base_config.max_retries / 2).max(1);
        }

        // Increase multiplier when success rate is low
        config.multiplier = self.base_config.multiplier * (1.0 + (1.0 - success_rate));

        config
    }

    /// Create a retry executor with the current adjusted config.
    pub fn executor(&self) -> RetryExecutor<AlwaysRetry> {
        RetryExecutor::new(self.get_adjusted_config())
    }
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;

    use super::*;

    #[test]
    fn test_retry_config_default() {
        let config = RetryConfig::default();
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.initial_interval, Duration::from_millis(500));
        assert_eq!(config.multiplier, 2.0);
    }

    #[test]
    fn test_retry_config_builder() {
        let config = RetryConfig::new(5)
            .with_initial_interval(Duration::from_secs(1))
            .with_max_interval(Duration::from_secs(60))
            .with_multiplier(1.5)
            .with_randomization_factor(0.3);

        assert_eq!(config.max_retries, 5);
        assert_eq!(config.initial_interval, Duration::from_secs(1));
        assert_eq!(config.max_interval, Duration::from_secs(60));
        assert_eq!(config.multiplier, 1.5);
        assert_eq!(config.randomization_factor, 0.3);
    }

    #[test]
    fn test_calculate_interval_exponential() {
        let config = RetryConfig::new(5)
            .with_initial_interval(Duration::from_secs(1))
            .with_randomization_factor(0.0); // No jitter for predictable test

        let interval0 = config.calculate_interval(0);
        let interval1 = config.calculate_interval(1);
        let interval2 = config.calculate_interval(2);

        assert_eq!(interval0, Duration::from_secs(1));
        assert_eq!(interval1, Duration::from_secs(2));
        assert_eq!(interval2, Duration::from_secs(4));
    }

    #[test]
    fn test_calculate_interval_capped() {
        let config = RetryConfig::new(10)
            .with_initial_interval(Duration::from_secs(1))
            .with_max_interval(Duration::from_secs(10))
            .with_randomization_factor(0.0);

        let interval5 = config.calculate_interval(5); // Would be 32s without cap
        assert_eq!(interval5, Duration::from_secs(10));
    }

    #[test]
    fn test_retry_budget_consume() {
        let budget = RetryBudget::new(10, 0.0); // No recovery

        assert!(budget.try_consume(5));
        assert_eq!(budget.remaining(), 5);
        assert!(budget.try_consume(5));
        assert_eq!(budget.remaining(), 0);
        assert!(!budget.try_consume(1)); // No more tokens
    }

    #[test]
    fn test_retry_budget_reset() {
        let budget = RetryBudget::new(10, 0.0);
        budget.try_consume(10);
        assert_eq!(budget.remaining(), 0);

        budget.reset();
        assert_eq!(budget.remaining(), 10);
    }

    #[test]
    fn test_adaptive_retry_success_rate() {
        let adaptive = AdaptiveRetry::new(RetryConfig::default());

        // No outcomes = 100% success rate
        assert_eq!(adaptive.success_rate(), 1.0);

        // Record some outcomes
        adaptive.record_outcome(true);
        adaptive.record_outcome(true);
        adaptive.record_outcome(false);
        adaptive.record_outcome(false);

        assert_eq!(adaptive.success_rate(), 0.5);
    }

    #[test]
    fn test_adaptive_retry_config_adjustment() {
        let base = RetryConfig::new(4)
            .with_initial_interval(Duration::from_secs(1))
            .with_multiplier(2.0);

        let adaptive = AdaptiveRetry::new(base);

        // 25% success rate (below 50% threshold)
        adaptive.record_outcome(true);
        adaptive.record_outcome(false);
        adaptive.record_outcome(false);
        adaptive.record_outcome(false);

        let adjusted = adaptive.get_adjusted_config();

        // Should have reduced max retries (success_rate < 0.5)
        assert_eq!(adjusted.max_retries, 2);
        // Should have increased initial interval
        assert!(adjusted.initial_interval > Duration::from_secs(1));
    }

    #[tokio::test]
    async fn test_retry_executor_success() {
        let executor = RetryExecutor::new(RetryConfig::new(3));
        let result = executor
            .execute("test", || async { Ok::<_, std::io::Error>("success") })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "success");
    }

    #[tokio::test]
    async fn test_retry_executor_failure() {
        let config = RetryConfig::new(2)
            .with_initial_interval(Duration::from_millis(10))
            .with_max_elapsed_time(None);

        let executor = RetryExecutor::new(config);
        let result = executor
            .execute("test", || async {
                Err::<(), _>(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "always fails",
                ))
            })
            .await;

        assert!(result.is_err());
        let err = result.unwrap_err();
        assert_eq!(err.attempts, 3); // Initial + 2 retries
    }

    #[tokio::test]
    async fn test_retry_executor_eventual_success() {
        let config = RetryConfig::new(3).with_initial_interval(Duration::from_millis(10));

        let executor = RetryExecutor::new(config);
        let attempt = Arc::new(AtomicU32::new(0));
        let attempt_clone = attempt.clone();

        let result = executor
            .execute("test", || {
                let attempt = attempt_clone.clone();
                async move {
                    let current = attempt.fetch_add(1, Ordering::SeqCst);
                    if current < 2 {
                        Err(std::io::Error::new(std::io::ErrorKind::Other, "not yet"))
                    } else {
                        Ok("success")
                    }
                }
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(attempt.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn test_always_retry_policy() {
        let policy = AlwaysRetry;
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        assert!(policy.should_retry(&error));
    }

    #[test]
    fn test_never_retry_policy() {
        let policy = NeverRetry;
        let error = std::io::Error::new(std::io::ErrorKind::Other, "test");
        assert!(!policy.should_retry(&error));
    }
}
