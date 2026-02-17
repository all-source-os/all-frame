//! Domain Layer Resilience Contracts
//!
//! This module defines resilience contracts that the domain layer can declare
//! without depending on infrastructure implementations. These contracts allow
//! domain logic to specify resilience requirements while maintaining Clean
//! Architecture principles.
//!
//! The domain layer defines WHAT resilience is needed, not HOW it's
//! implemented.

use std::time::Duration;

/// Resilience policies that domain entities can declare.
/// These represent business requirements for reliability, not implementation
/// details.
#[derive(Clone, Debug, PartialEq)]
pub enum ResiliencePolicy {
    /// No resilience - execute once
    None,

    /// Retry on failure with backoff strategy
    Retry {
        max_attempts: u32,
        backoff: BackoffStrategy,
    },

    /// Circuit breaker pattern for fault tolerance
    CircuitBreaker {
        failure_threshold: u32,
        recovery_timeout: Duration,
        success_threshold: u32,
    },

    /// Rate limiting to prevent resource exhaustion
    RateLimit {
        requests_per_second: u32,
        burst_capacity: u32,
    },

    /// Timeout protection
    Timeout { duration: Duration },

    /// Combination of multiple policies
    Combined { policies: Vec<ResiliencePolicy> },
}

/// Backoff strategies for retry operations
#[derive(Clone, Debug, PartialEq)]
pub enum BackoffStrategy {
    /// Fixed delay between attempts
    Fixed { delay: Duration },

    /// Exponential backoff with optional jitter
    Exponential {
        initial_delay: Duration,
        multiplier: f64,
        max_delay: Option<Duration>,
        jitter: bool,
    },

    /// Linear backoff
    Linear {
        initial_delay: Duration,
        increment: Duration,
        max_delay: Option<Duration>,
    },
}

impl Default for BackoffStrategy {
    fn default() -> Self {
        Self::Exponential {
            initial_delay: Duration::from_millis(100),
            multiplier: 2.0,
            max_delay: Some(Duration::from_secs(30)),
            jitter: true,
        }
    }
}

/// Domain-level resilience errors.
/// These represent business-level error conditions that can be mapped
/// to infrastructure-level errors by the application layer.
#[derive(thiserror::Error, Debug, Clone, PartialEq)]
pub enum ResilienceDomainError {
    #[error("Operation timed out after {duration:?}")]
    Timeout { duration: Duration },

    #[error("Operation failed after {attempts} attempts")]
    RetryExhausted { attempts: u32, last_error: String },

    #[error("Circuit breaker is open - service unavailable")]
    CircuitOpen,

    #[error("Rate limit exceeded - too many requests")]
    RateLimited { retry_after: Option<Duration> },

    #[error("Operation cancelled")]
    Cancelled,

    #[error("Infrastructure error: {message}")]
    Infrastructure { message: String },
}

impl ResilienceDomainError {
    /// Check if this error represents a temporary failure that might be retried
    pub fn is_retryable(&self) -> bool {
        match self {
            Self::Timeout { .. } => true,
            Self::RetryExhausted { .. } => false, // Already exhausted retries
            Self::CircuitOpen => false,           // Circuit breaker protects from further calls
            Self::RateLimited { .. } => true,     // Can retry after backoff
            Self::Cancelled => false,             // Operation was intentionally cancelled
            Self::Infrastructure { .. } => true,  // Infrastructure issues might be transient
        }
    }

    /// Check if this error indicates the service is unavailable
    pub fn is_service_unavailable(&self) -> bool {
        matches!(self, Self::CircuitOpen)
    }

    /// Get suggested retry delay if applicable
    pub fn retry_after(&self) -> Option<Duration> {
        match self {
            Self::RateLimited { retry_after } => *retry_after,
            _ => None,
        }
    }
}

/// Trait for domain operations that declare resilience requirements.
/// Domain entities implement this to specify how they should be executed
/// reliably.
#[async_trait::async_trait]
pub trait ResilientOperation<T, E> {
    /// Declare the resilience policy required for this operation
    fn resilience_policy(&self) -> ResiliencePolicy;

    /// Execute the core business logic
    async fn execute(&self) -> Result<T, E>;

    /// Get a unique identifier for this operation (for circuit breakers,
    /// metrics, etc.)
    fn operation_id(&self) -> &str {
        std::any::type_name::<Self>()
    }

    /// Check if this operation is critical (affects circuit breaker behavior)
    fn is_critical(&self) -> bool {
        true
    }
}

/// Trait for domain services that need resilience.
/// Services implement this to declare their resilience requirements at the
/// service level.
#[async_trait::async_trait]
pub trait ResilientService {
    /// Get the default resilience policy for this service
    fn default_resilience_policy(&self) -> ResiliencePolicy {
        ResiliencePolicy::None
    }

    /// Get service-specific resilience policies for different operations
    fn operation_policies(&self) -> std::collections::HashMap<String, ResiliencePolicy> {
        std::collections::HashMap::new()
    }

    /// Get the service identifier for monitoring and circuit breakers
    fn service_id(&self) -> &str {
        std::any::type_name::<Self>()
    }
}

/// Configuration for resilience behavior.
/// Domain layer can provide hints about expected behavior without knowing
/// implementation.
#[derive(Clone, Debug)]
pub struct ResilienceConfig {
    /// Whether to enable resilience globally
    pub enabled: bool,

    /// Default policies for different operation types
    pub default_policies: std::collections::HashMap<String, ResiliencePolicy>,

    /// Service-specific overrides
    pub service_overrides: std::collections::HashMap<String, ResiliencePolicy>,
}

impl Default for ResilienceConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            default_policies: std::collections::HashMap::new(),
            service_overrides: std::collections::HashMap::new(),
        }
    }
}

/// Helper for creating common resilience policies
pub mod policies {
    use std::time::Duration;

    use super::*;

    /// Create a simple retry policy
    pub fn retry(max_attempts: u32) -> ResiliencePolicy {
        ResiliencePolicy::Retry {
            max_attempts,
            backoff: BackoffStrategy::default(),
        }
    }

    /// Create a circuit breaker policy
    pub fn circuit_breaker(failure_threshold: u32, recovery_timeout_secs: u64) -> ResiliencePolicy {
        ResiliencePolicy::CircuitBreaker {
            failure_threshold,
            recovery_timeout: Duration::from_secs(recovery_timeout_secs),
            success_threshold: 3,
        }
    }

    /// Create a rate limiting policy
    pub fn rate_limit(requests_per_second: u32) -> ResiliencePolicy {
        ResiliencePolicy::RateLimit {
            requests_per_second,
            burst_capacity: requests_per_second / 4, // Allow burst of 25%
        }
    }

    /// Create a timeout policy
    pub fn timeout(seconds: u64) -> ResiliencePolicy {
        ResiliencePolicy::Timeout {
            duration: Duration::from_secs(seconds),
        }
    }

    /// Combine multiple policies
    pub fn combine(policies: Vec<ResiliencePolicy>) -> ResiliencePolicy {
        ResiliencePolicy::Combined { policies }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_resilience_policy_creation() {
        let retry_policy = policies::retry(3);
        assert_eq!(
            retry_policy,
            ResiliencePolicy::Retry {
                max_attempts: 3,
                backoff: BackoffStrategy::default(),
            }
        );

        let circuit_policy = policies::circuit_breaker(5, 30);
        match circuit_policy {
            ResiliencePolicy::CircuitBreaker {
                failure_threshold,
                recovery_timeout,
                ..
            } => {
                assert_eq!(failure_threshold, 5);
                assert_eq!(recovery_timeout, Duration::from_secs(30));
            }
            _ => panic!("Expected CircuitBreaker policy"),
        }
    }

    #[test]
    fn test_domain_error_retryable() {
        assert!(ResilienceDomainError::Timeout {
            duration: Duration::from_secs(1)
        }
        .is_retryable());
        assert!(ResilienceDomainError::RateLimited { retry_after: None }.is_retryable());
        assert!(!ResilienceDomainError::CircuitOpen.is_retryable());
        assert!(!ResilienceDomainError::Cancelled.is_retryable());
    }

    #[test]
    fn test_backoff_strategy_default() {
        let strategy = BackoffStrategy::default();
        match strategy {
            BackoffStrategy::Exponential {
                initial_delay,
                multiplier,
                max_delay,
                jitter,
            } => {
                assert_eq!(initial_delay, Duration::from_millis(100));
                assert_eq!(multiplier, 2.0);
                assert_eq!(max_delay, Some(Duration::from_secs(30)));
                assert!(jitter);
            }
            _ => panic!("Expected Exponential backoff"),
        }
    }

    #[test]
    fn test_combined_policies() {
        let retry = policies::retry(3);
        let timeout = policies::timeout(10);
        let combined = policies::combine(vec![retry.clone(), timeout]);

        match combined {
            ResiliencePolicy::Combined { policies } => {
                assert_eq!(policies.len(), 2);
                assert_eq!(policies[0], retry);
            }
            _ => panic!("Expected Combined policy"),
        }
    }
}
