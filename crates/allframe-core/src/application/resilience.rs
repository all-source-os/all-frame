//! Application Layer Resilience Orchestration
//!
//! This module provides the orchestration layer that bridges domain-level resilience
//! contracts with infrastructure-level implementations. It maintains Clean Architecture
//! by ensuring domain logic remains pure while infrastructure concerns are handled
//! at the appropriate layer.
//!
//! The application layer is responsible for:
//! - Translating domain resilience policies to infrastructure implementations
//! - Managing resilience state and coordination
//! - Providing observability and monitoring
//! - Ensuring proper error handling across layers

use crate::domain::resilience::{
    ResiliencePolicy, ResilienceDomainError, ResilientOperation,
};
#[cfg(feature = "resilience")]
use crate::resilience::{
    RetryExecutor, RetryConfig, CircuitBreaker, CircuitBreakerConfig,
    RateLimiter, RateLimiterConfig, RetryError,
};
#[cfg(feature = "resilience")]
use std::collections::HashMap;
#[cfg(feature = "resilience")]
use std::sync::Arc;
#[cfg(feature = "resilience")]
use std::time::Duration;

/// Core trait for resilience orchestration.
/// This trait defines the contract between application layer and infrastructure.
#[async_trait::async_trait]
pub trait ResilienceOrchestrator: Send + Sync {
    /// Execute an operation with the specified resilience policy
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        operation: F,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        E: Into<ResilienceOrchestrationError> + Send;

    /// Execute a resilient operation (domain entity implementing ResilientOperation)
    async fn execute_operation<T, E, Op>(
        &self,
        operation: Op,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        Op: ResilientOperation<T, E> + Send + Sync,
        E: Into<ResilienceOrchestrationError> + Send,
    {
        let policy = operation.resilience_policy();
        self.execute_with_policy(policy, || operation.execute()).await
    }

    /// Get a named circuit breaker for manual control
    fn get_circuit_breaker(&self, name: &str) -> Option<&CircuitBreaker>;

    /// Get a named rate limiter for manual control
    fn get_rate_limiter(&self, name: &str) -> Option<&RateLimiter>;

    /// Get resilience metrics for monitoring
    fn metrics(&self) -> ResilienceMetrics;
}

/// Errors that can occur during resilience orchestration
#[derive(thiserror::Error, Debug)]
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
    retry_executor: Arc<RetryExecutor>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    rate_limiters: HashMap<String, RateLimiter>,
    metrics: std::sync::Mutex<ResilienceMetrics>,
}

#[cfg(feature = "resilience")]
impl DefaultResilienceOrchestrator {
    /// Create a new orchestrator with default infrastructure components
    pub fn new() -> Self {
        Self {
            retry_executor: Arc::new(RetryExecutor::new(RetryConfig::default())),
            circuit_breakers: HashMap::new(),
            rate_limiters: HashMap::new(),
            metrics: std::sync::Mutex::new(ResilienceMetrics::default()),
        }
    }

    /// Create an orchestrator with custom infrastructure components
    pub fn with_components(
        retry_executor: Arc<RetryExecutor>,
        circuit_breakers: HashMap<String, CircuitBreaker>,
        rate_limiters: HashMap<String, RateLimiter>,
    ) -> Self {
        Self {
            retry_executor,
            circuit_breakers,
            rate_limiters,
            metrics: std::sync::Mutex::new(ResilienceMetrics::default()),
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

    /// Get or create a circuit breaker with the given configuration
    fn get_or_create_circuit_breaker(
        &mut self,
        name: &str,
        failure_threshold: u32,
        recovery_timeout: Duration,
    ) -> &CircuitBreaker {
        self.circuit_breakers
            .entry(name.to_string())
            .or_insert_with(|| {
                CircuitBreaker::new(
                    name,
                    CircuitBreakerConfig::new(failure_threshold, recovery_timeout),
                )
            })
    }

    /// Get or create a rate limiter with the given configuration
    fn get_or_create_rate_limiter(
        &mut self,
        name: &str,
        requests_per_second: u32,
        burst_capacity: u32,
    ) -> &RateLimiter {
        self.rate_limiters
            .entry(name.to_string())
            .or_insert_with(|| {
                RateLimiter::new(
                    requests_per_second,
                    burst_capacity,
                )
            })
    }

    /// Update metrics for a successful operation
    fn record_success(&self) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_operations += 1;
        metrics.successful_operations += 1;
    }

    /// Update metrics for a failed operation
    fn record_failure(&self, error: &ResilienceOrchestrationError) {
        let mut metrics = self.metrics.lock().unwrap();
        metrics.total_operations += 1;
        metrics.failed_operations += 1;

        match error {
            ResilienceOrchestrationError::Domain(ResilienceDomainError::RetryExhausted { attempts, .. }) => {
                metrics.retry_attempts += *attempts as u64;
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
}

#[cfg(feature = "resilience")]
#[async_trait::async_trait]
impl ResilienceOrchestrator for DefaultResilienceOrchestrator {
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        operation: F,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        F: FnOnce() -> Fut + Send,
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

            ResiliencePolicy::Retry { max_attempts, backoff } => {
                let retry_config = match backoff {
                    BackoffStrategy::Fixed { delay } => {
                        RetryConfig::new(max_attempts).with_fixed_delay(delay)
                    }
                    BackoffStrategy::Exponential { initial_delay, multiplier, max_delay, jitter } => {
                        let mut config = RetryConfig::new(max_attempts)
                            .with_initial_interval(initial_delay)
                            .with_multiplier(multiplier);

                        if let Some(max) = max_delay {
                            config = config.with_max_interval(max);
                        }

                        if jitter {
                            config = config.with_jitter();
                        }

                        config
                    }
                    BackoffStrategy::Linear { initial_delay, increment, max_delay } => {
                        // For linear backoff, we'll use exponential with multiplier 1.0
                        // This is a simplification - a full implementation would need
                        // a custom backoff strategy in the infrastructure layer
                        let mut config = RetryConfig::new(max_attempts)
                            .with_initial_interval(initial_delay)
                            .with_multiplier(1.0);

                        if let Some(max) = max_delay {
                            config = config.with_max_interval(max);
                        }

                        config
                    }
                };

                let retry_result = self.retry_executor
                    .execute_with_config(retry_config, operation)
                    .await;

                match retry_result {
                    Ok(value) => {
                        self.record_success();
                        Ok(value)
                    }
                    Err(retry_error) => {
                        let orch_error = match retry_error {
                            RetryError::OperationFailed { last_error, attempts } => {
                                ResilienceOrchestrationError::Domain(
                                    ResilienceDomainError::RetryExhausted {
                                        attempts: attempts as u32,
                                        last_error: format!("{:?}", last_error),
                                    }
                                )
                            }
                            RetryError::Timeout => {
                                ResilienceOrchestrationError::Domain(
                                    ResilienceDomainError::Timeout {
                                        duration: Duration::from_secs(30), // Default timeout
                                    }
                                )
                            }
                        };
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                }
            }

            ResiliencePolicy::CircuitBreaker { failure_threshold, recovery_timeout, success_threshold } => {
                let cb = self.get_or_create_circuit_breaker(
                    "default",
                    failure_threshold,
                    recovery_timeout,
                );

                let cb_result = cb.call(operation).await;

                match cb_result {
                    Ok(value) => {
                        self.record_success();
                        Ok(value)
                    }
                    Err(cb_error) => {
                        // Circuit breaker errors are typically wrapped operation errors
                        // For now, we'll treat all circuit breaker failures as domain errors
                        let orch_error = ResilienceOrchestrationError::Domain(
                            ResilienceDomainError::CircuitOpen
                        );
                        self.record_failure(&orch_error);
                        Err(orch_error)
                    }
                }
            }

            ResiliencePolicy::RateLimit { requests_per_second, burst_capacity } => {
                let limiter = self.get_or_create_rate_limiter(
                    "default",
                    requests_per_second,
                    burst_capacity,
                );

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
                    let orch_error = ResilienceOrchestrationError::Domain(
                        ResilienceDomainError::RateLimited { retry_after: None }
                    );
                    self.record_failure(&orch_error);
                    Err(orch_error)
                }
            }

            ResiliencePolicy::Timeout { duration } => {
                // For timeout, we'd need tokio::time::timeout
                // This is a simplified implementation
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

            ResiliencePolicy::Combined { policies } => {
                // For combined policies, we apply them in sequence
                // This is a simplified implementation - a full implementation
                // would need more sophisticated policy composition
                let mut final_result = None;
                let mut final_error = None;

                for policy in policies {
                    let result = self.execute_with_policy(policy, || operation()).await;
                    match result {
                        Ok(value) => {
                            final_result = Some(value);
                            break; // Success, no need to continue
                        }
                        Err(error) => {
                            final_error = Some(error);
                            // Continue with next policy
                        }
                    }
                }

                match final_result {
                    Some(value) => {
                        self.record_success();
                        Ok(value)
                    }
                    None => {
                        let error = final_error.unwrap_or_else(|| {
                            ResilienceOrchestrationError::Configuration(
                                "No policies succeeded".to_string()
                            )
                        });
                        self.record_failure(&error);
                        Err(error)
                    }
                }
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
        self.metrics.lock().unwrap().clone()
    }
}

impl Default for DefaultResilienceOrchestrator {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::resilience::policies;

    #[tokio::test]
    async fn test_no_resilience_policy() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        let result = orchestrator
            .execute_with_policy(ResiliencePolicy::None, || async { Ok::<_, ResilienceOrchestrationError>(42) })
            .await;

        assert_eq!(result, Ok(42));
        let metrics = orchestrator.metrics();
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);
    }

    #[tokio::test]
    async fn test_retry_policy_success() {
        let orchestrator = DefaultResilienceOrchestrator::new();
        let mut attempts = 0;

        let result = orchestrator
            .execute_with_policy(
                policies::retry(3),
                || async {
                    attempts += 1;
                    if attempts < 2 {
                        Err(ResilienceOrchestrationError::Domain(
                            ResilienceDomainError::Infrastructure {
                                message: "Temporary failure".to_string()
                            }
                        ))
                    } else {
                        Ok(42)
                    }
                }
            )
            .await;

        assert_eq!(result, Ok(42));
        assert_eq!(attempts, 2); // Should succeed on second attempt
    }

    #[tokio::test]
    async fn test_circuit_breaker_policy() {
        let mut orchestrator = DefaultResilienceOrchestrator::new();

        // Register a circuit breaker that opens after 2 failures
        orchestrator.register_circuit_breaker(
            "test".to_string(),
            CircuitBreaker::new("test", CircuitBreakerConfig::new(2, Duration::from_secs(1))),
        );

        // This should work
        let result1 = orchestrator
            .execute_with_policy(
                ResiliencePolicy::CircuitBreaker {
                    failure_threshold: 2,
                    recovery_timeout: Duration::from_secs(1),
                    success_threshold: 1,
                },
                || async { Ok::<_, ResilienceOrchestrationError>(42) }
            )
            .await;
        assert_eq!(result1, Ok(42));
    }

    #[tokio::test]
    async fn test_rate_limit_policy() {
        let mut orchestrator = DefaultResilienceOrchestrator::new();

        // Register a rate limiter that allows 1 request per second
        orchestrator.register_rate_limiter(
            "test".to_string(),
            RateLimiter::new(1, 1),
        );

        // First request should succeed
        let result1 = orchestrator
            .execute_with_policy(
                ResiliencePolicy::RateLimit {
                    requests_per_second: 1,
                    burst_capacity: 1,
                },
                || async { Ok::<_, ResilienceOrchestrationError>(42) }
            )
            .await;
        assert_eq!(result1, Ok(42));

        // Second request should be rate limited
        let result2 = orchestrator
            .execute_with_policy(
                ResiliencePolicy::RateLimit {
                    requests_per_second: 1,
                    burst_capacity: 1,
                },
                || async { Ok::<_, ResilienceOrchestrationError>(42) }
            )
            .await;

        match result2 {
            Err(ResilienceOrchestrationError::Domain(ResilienceDomainError::RateLimited { .. })) => {
                // Expected rate limiting
            }
            other => panic!("Expected rate limiting error, got {:?}", other),
        }
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
        operation: F,
    ) -> Result<T, ResilienceOrchestrationError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<T, E>> + Send,
        E: Into<ResilienceOrchestrationError> + Send,
    {
        // Without resilience features, just execute the operation once
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

