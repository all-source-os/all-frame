//! Application Layer Resilience Orchestration
//!
//! This module provides the orchestration layer that bridges domain-level
//! resilience contracts with infrastructure-level implementations. It maintains
//! Clean Architecture by ensuring domain logic remains pure while
//! infrastructure concerns are handled at the appropriate layer.
//!
//! The application layer is responsible for:
//! - Translating domain resilience policies to infrastructure implementations
//! - Managing resilience state and coordination
//! - Providing observability and monitoring
//! - Ensuring proper error handling across layers

#[cfg(feature = "resilience")]
use std::collections::HashMap;
#[cfg(feature = "resilience")]
use std::sync::Arc;
#[cfg(feature = "resilience")]
use std::time::Duration;

use crate::domain::resilience::{ResilienceDomainError, ResiliencePolicy, ResilientOperation};
#[cfg(feature = "resilience")]
use crate::domain::resilience::BackoffStrategy;
#[cfg(feature = "resilience")]
use crate::resilience::{
    CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError, RateLimiter, RetryConfig,
};
#[cfg(feature = "resilience")]
use dashmap::DashMap;

/// Core trait for resilience orchestration.
/// This trait defines the contract between application layer and
/// infrastructure.
#[async_trait::async_trait]
pub trait ResilienceOrchestrator: Send + Sync {
    /// Execute an operation with the specified resilience policy
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        operation: F,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        E: Into<ResilienceOrchestrationError> + Send;

    /// Execute a resilient operation (domain entity implementing
    /// ResilientOperation)
    async fn execute_operation<T, E, Op>(
        &self,
        operation: Op,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        Op: ResilientOperation<T, E> + Send + Sync,
        E: Into<ResilienceOrchestrationError> + Send,
    {
        let policy = operation.resilience_policy();
        self.execute_with_policy(policy, || operation.execute())
            .await
    }

    /// Get a named circuit breaker for manual control
    fn get_circuit_breaker(&self, name: &str) -> Option<&CircuitBreaker>;

    /// Get a named rate limiter for manual control
    fn get_rate_limiter(&self, name: &str) -> Option<&RateLimiter>;

    /// Get resilience metrics for monitoring
    fn metrics(&self) -> ResilienceMetrics;
}

/// Errors that can occur during resilience orchestration
#[derive(thiserror::Error, Debug, PartialEq)]
pub enum ResilienceOrchestrationError {
    #[error("Domain error: {0}")]
    Domain(#[from] ResilienceDomainError),

    #[error("Infrastructure error: {0}")]
    Infrastructure(String),

    #[error("Configuration error: {0}")]
    Configuration(String),

    #[error("Operation cancelled")]
    Cancelled,
}

/// Stub types for when resilience features are not available
#[cfg(not(feature = "resilience"))]
pub struct CircuitBreaker;

#[cfg(not(feature = "resilience"))]
pub struct RateLimiter;

/// Resilience metrics for monitoring and observability
#[derive(Clone, Debug, Default)]
pub struct ResilienceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub retry_attempts: u64,
    pub circuit_breaker_trips: u64,
    pub rate_limit_hits: u64,
    pub timeout_count: u64,
}

/// Default implementation of ResilienceOrchestrator using infrastructure layer
#[cfg(feature = "resilience")]
pub struct DefaultResilienceOrchestrator {
    circuit_breakers: HashMap<String, CircuitBreaker>,
    rate_limiters: HashMap<String, RateLimiter>,
    dynamic_circuit_breakers: DashMap<String, Arc<CircuitBreaker>>,
    dynamic_rate_limiters: DashMap<String, Arc<RateLimiter>>,
    metrics: parking_lot::Mutex<ResilienceMetrics>,
}

#[cfg(feature = "resilience")]
impl DefaultResilienceOrchestrator {
    /// Create a new orchestrator with default infrastructure components
    pub fn new() -> Self {
        Self {
            circuit_breakers: HashMap::new(),
            rate_limiters: HashMap::new(),
            dynamic_circuit_breakers: DashMap::new(),
            dynamic_rate_limiters: DashMap::new(),
            metrics: parking_lot::Mutex::new(ResilienceMetrics::default()),
        }
    }

    /// Create an orchestrator with custom infrastructure components
    pub fn with_components(
        circuit_breakers: HashMap<String, CircuitBreaker>,
        rate_limiters: HashMap<String, RateLimiter>,
    ) -> Self {
        Self {
            circuit_breakers,
            rate_limiters,
            dynamic_circuit_breakers: DashMap::new(),
            dynamic_rate_limiters: DashMap::new(),
            metrics: parking_lot::Mutex::new(ResilienceMetrics::default()),
        }
    }

    /// Register a named circuit breaker
    pub fn register_circuit_breaker(&mut self, name: String, circuit_breaker: CircuitBreaker) {
        self.circuit_breakers.insert(name, circuit_breaker);
    }

    /// Register a named rate limiter
    pub fn register_rate_limiter(&mut self, name: String, rate_limiter: RateLimiter) {
        self.rate_limiters.insert(name, rate_limiter);
    }

    /// Update metrics for a successful operation
    fn record_success(&self) {
        let mut metrics = self.metrics.lock();
        metrics.total_operations += 1;
        metrics.successful_operations += 1;
    }

    /// Update metrics for a failed operation
    fn record_failure(&self, error: &ResilienceOrchestrationError) {
        let mut metrics = self.metrics.lock();
        metrics.total_operations += 1;
        metrics.failed_operations += 1;

        match error {
            ResilienceOrchestrationError::Domain(ResilienceDomainError::RetryExhausted {
                ..
            }) => {
                // retry_attempts already tracked by record_retry() calls
            }
            ResilienceOrchestrationError::Domain(ResilienceDomainError::CircuitOpen) => {
                metrics.circuit_breaker_trips += 1;
            }
            ResilienceOrchestrationError::Domain(ResilienceDomainError::RateLimited { .. }) => {
                metrics.rate_limit_hits += 1;
            }
            ResilienceOrchestrationError::Domain(ResilienceDomainError::Timeout { .. }) => {
                metrics.timeout_count += 1;
            }
            _ => {}
        }
    }

    /// Record a retry attempt in metrics
    fn record_retry(&self) {
        let mut metrics = self.metrics.lock();
        metrics.retry_attempts += 1;
    }

    /// Get or create a persistent circuit breaker for a policy
    fn get_or_create_circuit_breaker(
        &self,
        failure_threshold: u32,
        recovery_timeout: Duration,
        success_threshold: u32,
    ) -> Arc<CircuitBreaker> {
        let key = format!(
            "cb_{}_{}_{}", failure_threshold, recovery_timeout.as_millis(), success_threshold
        );
        self.dynamic_circuit_breakers
            .entry(key)
            .or_insert_with(|| {
                let config = CircuitBreakerConfig::new(failure_threshold)
                    .with_timeout(recovery_timeout)
                    .with_success_threshold(success_threshold);
                Arc::new(CircuitBreaker::new("policy", config))
            })
            .clone()
    }

    /// Get or create a persistent rate limiter for a policy
    fn get_or_create_rate_limiter(&self, rps: u32, burst: u32) -> Arc<RateLimiter> {
        let key = format!("rl_{}_{}", rps, burst);
        self.dynamic_rate_limiters
            .entry(key)
            .or_insert_with(|| Arc::new(RateLimiter::new(rps, burst)))
            .clone()
    }

    /// Build a RetryConfig from a domain BackoffStrategy.
    /// Domain `max_attempts` = total attempts (1 = no retries).
    /// Infrastructure `max_retries` = retries after initial attempt.
    fn build_retry_config(max_attempts: u32, backoff: &BackoffStrategy) -> RetryConfig {
        let max_retries = max_attempts.saturating_sub(1);
        match backoff {
            BackoffStrategy::Fixed { delay } => RetryConfig::new(max_retries)
                .with_initial_interval(*delay)
                .with_multiplier(1.0)
                .with_randomization_factor(0.0),
            BackoffStrategy::Exponential {
                initial_delay,
                multiplier,
                max_delay,
                jitter,
            } => {
                let mut config = RetryConfig::new(max_retries)
                    .with_initial_interval(*initial_delay)
                    .with_multiplier(*multiplier);

                if let Some(max) = max_delay {
                    config = config.with_max_interval(*max);
                }

                if *jitter {
                    config = config.with_randomization_factor(0.5);
                } else {
                    config = config.with_randomization_factor(0.0);
                }

                config
            }
            BackoffStrategy::Linear {
                initial_delay,
                increment: _,
                max_delay,
            } => {
                let mut config = RetryConfig::new(max_retries)
                    .with_initial_interval(*initial_delay)
                    .with_multiplier(1.0);

                if let Some(max) = max_delay {
                    config = config.with_max_interval(*max);
                }

                config
            }
        }
    }
}

#[cfg(feature = "resilience")]
#[async_trait::async_trait]
impl ResilienceOrchestrator for DefaultResilienceOrchestrator {
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        mut operation: F,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        E: Into<ResilienceOrchestrationError> + Send,
    {
        match policy {
            ResiliencePolicy::None => {
                let result = operation().await;
                match result {
                    Ok(value) => {
                        self.record_success();
                        Ok(value)
                    }
                    Err(error) => {
                        let orch_error = error.into();
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                }
            }

            ResiliencePolicy::Retry {
                max_attempts,
                backoff,
            } => {
                let retry_config = Self::build_retry_config(max_attempts, &backoff);

                // Inline retry loop — RetryExecutor requires E: std::error::Error
                // which is more restrictive than E: Into<ResilienceOrchestrationError>
                let mut attempts = 0u32;
                loop {
                    attempts += 1;
                    match operation().await {
                        Ok(value) => {
                            self.record_success();
                            return Ok(value);
                        }
                        Err(error) => {
                            let msg = format!("{}", error.into());

                            if attempts > retry_config.max_retries {
                                let final_error = ResilienceOrchestrationError::Domain(
                                    ResilienceDomainError::RetryExhausted {
                                        attempts,
                                        last_error: msg,
                                    },
                                );
                                self.record_failure(&final_error);
                                return Err(final_error);
                            }

                            self.record_retry();
                            // error is dropped here, before the await
                        }
                    }
                    let interval = retry_config.calculate_interval(attempts - 1);
                    tokio::time::sleep(interval).await;
                }
            }

            ResiliencePolicy::CircuitBreaker {
                failure_threshold,
                recovery_timeout,
                success_threshold,
            } => {
                let cb = self.get_or_create_circuit_breaker(
                    failure_threshold,
                    recovery_timeout,
                    success_threshold,
                );

                match cb.call(operation).await {
                    Ok(value) => {
                        self.record_success();
                        Ok(value)
                    }
                    Err(CircuitBreakerError::CircuitOpen(_)) => {
                        let orch_error = ResilienceOrchestrationError::Domain(
                            ResilienceDomainError::CircuitOpen,
                        );
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                    Err(CircuitBreakerError::Inner(error)) => {
                        let orch_error = error.into();
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                }
            }

            ResiliencePolicy::RateLimit {
                requests_per_second,
                burst_capacity,
            } => {
                let limiter = self.get_or_create_rate_limiter(requests_per_second, burst_capacity);

                if limiter.check().is_ok() {
                    let result = operation().await;
                    match result {
                        Ok(value) => {
                            self.record_success();
                            Ok(value)
                        }
                        Err(error) => {
                            let orch_error = error.into();
                            self.record_failure(&orch_error);
                            Err(orch_error)
                        }
                    }
                } else {
                    let orch_error =
                        ResilienceOrchestrationError::Domain(ResilienceDomainError::RateLimited {
                            retry_after: None,
                        });
                    self.record_failure(&orch_error);
                    Err(orch_error)
                }
            }

            ResiliencePolicy::Timeout { duration } => {
                let result = tokio::time::timeout(duration, operation()).await;
                match result {
                    Ok(Ok(value)) => {
                        self.record_success();
                        Ok(value)
                    }
                    Ok(Err(error)) => {
                        let orch_error = error.into();
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                    Err(_elapsed) => {
                        let orch_error = ResilienceOrchestrationError::Domain(
                            ResilienceDomainError::Timeout { duration },
                        );
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                }
            }

            ResiliencePolicy::Combined { policies } => {
                if policies.is_empty() {
                    return self
                        .execute_with_policy(ResiliencePolicy::None, operation)
                        .await;
                }

                // Separate guard policies (checked upfront) from execution policies.
                // Guards: RateLimit, CircuitBreaker — checked before the operation runs.
                // Execution: Retry, Timeout — wraps the actual operation call.
                let mut execution_policy = None;

                for policy in policies {
                    match policy {
                        ResiliencePolicy::RateLimit {
                            requests_per_second,
                            burst_capacity,
                        } => {
                            let limiter = self.get_or_create_rate_limiter(
                                requests_per_second,
                                burst_capacity,
                            );
                            if limiter.check().is_err() {
                                let e = ResilienceOrchestrationError::Domain(
                                    ResilienceDomainError::RateLimited { retry_after: None },
                                );
                                self.record_failure(&e);
                                return Err(e);
                            }
                        }
                        ResiliencePolicy::CircuitBreaker {
                            failure_threshold,
                            recovery_timeout,
                            success_threshold,
                        } => {
                            let cb = self.get_or_create_circuit_breaker(
                                failure_threshold,
                                recovery_timeout,
                                success_threshold,
                            );
                            if cb.check().is_err() {
                                let e = ResilienceOrchestrationError::Domain(
                                    ResilienceDomainError::CircuitOpen,
                                );
                                self.record_failure(&e);
                                return Err(e);
                            }
                        }
                        p @ (ResiliencePolicy::Retry { .. }
                        | ResiliencePolicy::Timeout { .. }) => {
                            execution_policy = Some(p);
                        }
                        ResiliencePolicy::None => {}
                        ResiliencePolicy::Combined { .. } => {
                            return Err(ResilienceOrchestrationError::Configuration(
                                "Nested Combined policies are not supported".to_string(),
                            ));
                        }
                    }
                }

                // Execute with the found execution policy (or None for pass-through)
                self.execute_with_policy(
                    execution_policy.unwrap_or(ResiliencePolicy::None),
                    operation,
                )
                .await
            }
        }
    }

    fn get_circuit_breaker(&self, name: &str) -> Option<&CircuitBreaker> {
        self.circuit_breakers.get(name)
    }

    fn get_rate_limiter(&self, name: &str) -> Option<&RateLimiter> {
        self.rate_limiters.get(name)
    }

    fn metrics(&self) -> ResilienceMetrics {
        self.metrics.lock().clone()
    }
}

#[cfg(feature = "resilience")]
impl Default for DefaultResilienceOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(all(test, feature = "resilience"))]
mod tests {
    use std::sync::atomic::{AtomicU32, Ordering};
    use std::sync::Arc;
    use std::time::Duration;

    use super::*;
    use crate::domain::resilience::policies;

    #[tokio::test]
    async fn test_no_resilience_policy() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        let result = orchestrator
            .execute_with_policy(ResiliencePolicy::None, || async {
                Ok::<_, ResilienceOrchestrationError>(42)
            })
            .await;

        assert_eq!(result, Ok(42));
        let metrics = orchestrator.metrics();
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);
    }

    #[tokio::test]
    async fn test_retry_policy_success() {
        let orchestrator = DefaultResilienceOrchestrator::new();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let result = orchestrator
            .execute_with_policy(policies::retry(3), move || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    if count < 2 {
                        Err(ResilienceOrchestrationError::Domain(
                            ResilienceDomainError::Infrastructure {
                                message: "Temporary failure".to_string(),
                            },
                        ))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_circuit_breaker_policy() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        let result1 = orchestrator
            .execute_with_policy(
                ResiliencePolicy::CircuitBreaker {
                    failure_threshold: 2,
                    recovery_timeout: Duration::from_secs(1),
                    success_threshold: 1,
                },
                || async { Ok::<_, ResilienceOrchestrationError>(42) },
            )
            .await;
        assert_eq!(result1, Ok(42));
    }

    #[tokio::test]
    async fn test_circuit_breaker_trips_after_failures() {
        let orchestrator = DefaultResilienceOrchestrator::new();
        let policy = ResiliencePolicy::CircuitBreaker {
            failure_threshold: 2,
            recovery_timeout: Duration::from_secs(60),
            success_threshold: 1,
        };

        // Two failures should trip the circuit breaker
        for _ in 0..2 {
            let _ = orchestrator
                .execute_with_policy(policy.clone(), || async {
                    Err::<i32, _>(ResilienceOrchestrationError::Infrastructure(
                        "fail".to_string(),
                    ))
                })
                .await;
        }

        // Third call should fail with CircuitOpen (not even calling the operation)
        let call_count = Arc::new(AtomicU32::new(0));
        let call_count_clone = call_count.clone();
        let result = orchestrator
            .execute_with_policy(policy.clone(), move || {
                let cc = call_count_clone.clone();
                async move {
                    cc.fetch_add(1, Ordering::SeqCst);
                    Ok::<_, ResilienceOrchestrationError>(42)
                }
            })
            .await;

        assert!(matches!(
            result,
            Err(ResilienceOrchestrationError::Domain(
                ResilienceDomainError::CircuitOpen
            ))
        ));
        // Operation should not have been called
        assert_eq!(call_count.load(Ordering::SeqCst), 0);

        let metrics = orchestrator.metrics();
        assert!(metrics.circuit_breaker_trips > 0);
    }

    #[tokio::test]
    async fn test_circuit_breaker_inner_error_preserved() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        let result = orchestrator
            .execute_with_policy(
                ResiliencePolicy::CircuitBreaker {
                    failure_threshold: 5,
                    recovery_timeout: Duration::from_secs(60),
                    success_threshold: 1,
                },
                || async {
                    Err::<i32, _>(ResilienceOrchestrationError::Infrastructure(
                        "db connection failed".to_string(),
                    ))
                },
            )
            .await;

        // Should preserve the inner error, not report as CircuitOpen
        assert!(matches!(
            result,
            Err(ResilienceOrchestrationError::Infrastructure(ref msg)) if msg == "db connection failed"
        ));
    }

    #[tokio::test]
    async fn test_rate_limit_policy() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        // First request should succeed
        let result1 = orchestrator
            .execute_with_policy(
                ResiliencePolicy::RateLimit {
                    requests_per_second: 1,
                    burst_capacity: 1,
                },
                || async { Ok::<_, ResilienceOrchestrationError>(42) },
            )
            .await;
        assert_eq!(result1, Ok(42));
    }

    #[tokio::test]
    async fn test_rate_limit_persists_across_calls() {
        let orchestrator = DefaultResilienceOrchestrator::new();
        let policy = ResiliencePolicy::RateLimit {
            requests_per_second: 1,
            burst_capacity: 1,
        };

        // First call uses the burst token
        let result1 = orchestrator
            .execute_with_policy(policy.clone(), || async {
                Ok::<_, ResilienceOrchestrationError>(1)
            })
            .await;
        assert!(result1.is_ok());

        // Second call should be rate-limited (same limiter instance)
        let result2 = orchestrator
            .execute_with_policy(policy, || async {
                Ok::<_, ResilienceOrchestrationError>(2)
            })
            .await;
        assert!(matches!(
            result2,
            Err(ResilienceOrchestrationError::Domain(
                ResilienceDomainError::RateLimited { .. }
            ))
        ));

        let metrics = orchestrator.metrics();
        assert!(metrics.rate_limit_hits > 0);
    }

    #[tokio::test]
    async fn test_combined_rate_limit_and_retry() {
        let orchestrator = DefaultResilienceOrchestrator::new();
        let attempts = Arc::new(AtomicU32::new(0));
        let attempts_clone = attempts.clone();

        let policy = ResiliencePolicy::Combined {
            policies: vec![
                ResiliencePolicy::RateLimit {
                    requests_per_second: 100,
                    burst_capacity: 10,
                },
                policies::retry(3),
            ],
        };

        let result = orchestrator
            .execute_with_policy(policy, move || {
                let attempts = attempts_clone.clone();
                async move {
                    let count = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                    if count < 2 {
                        Err(ResilienceOrchestrationError::Infrastructure(
                            "temporary".to_string(),
                        ))
                    } else {
                        Ok(42)
                    }
                }
            })
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts.load(Ordering::SeqCst), 2);
    }

    #[tokio::test]
    async fn test_combined_empty_policies() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        let result = orchestrator
            .execute_with_policy(
                ResiliencePolicy::Combined {
                    policies: vec![],
                },
                || async { Ok::<_, ResilienceOrchestrationError>(42) },
            )
            .await;

        assert_eq!(result, Ok(42));
    }

    #[test]
    fn test_metrics_tracking() {
        let orchestrator = DefaultResilienceOrchestrator::new();
        let metrics = orchestrator.metrics();
        assert_eq!(metrics.total_operations, 0);
        assert_eq!(metrics.successful_operations, 0);
        assert_eq!(metrics.failed_operations, 0);
    }
}

/// Stub implementation when resilience features are not available
#[cfg(not(feature = "resilience"))]
#[derive(Default)]
pub struct DefaultResilienceOrchestrator;

#[cfg(not(feature = "resilience"))]
impl DefaultResilienceOrchestrator {
    pub fn new() -> Self {
        Self
    }
}

#[cfg(not(feature = "resilience"))]
#[async_trait::async_trait]
impl ResilienceOrchestrator for DefaultResilienceOrchestrator {
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        _policy: ResiliencePolicy,
        mut operation: F,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        F: FnMut() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        E: Into<ResilienceOrchestrationError> + Send,
    {
        let result = operation().await;
        match result {
            Ok(value) => Ok(value),
            Err(error) => Err(error.into()),
        }
    }

    fn get_circuit_breaker(&self, _name: &str) -> Option<&CircuitBreaker> {
        None
    }

    fn get_rate_limiter(&self, _name: &str) -> Option<&RateLimiter> {
        None
    }

    fn metrics(&self) -> ResilienceMetrics {
        ResilienceMetrics::default()
    }
}
