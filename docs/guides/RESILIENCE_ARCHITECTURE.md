# Resilience Architecture Guide

**Status**: ✅ Production Ready
**Version**: v0.1.13+
**Audience**: Application Developers, Architects

---

## Overview

AllFrame provides a Clean Architecture-compliant resilience system that separates business logic from infrastructure concerns. This guide explains how to use the new architectural patterns for building resilient applications.

---

## Architecture Principles

### Clean Architecture Layers

```
┌─────────────────────────────────────┐
│         Presentation Layer          │
│  (REST, GraphQL, gRPC handlers)     │
│  - HTTP status codes, serialization │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│       Application Layer            │
│  (Use Cases, Orchestration)        │
│  - Business workflows              │
│  - Transaction coordination        │
│  - Resilience orchestration        │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│         Domain Layer                │
│  (Business Logic, Entities)        │
│  - Pure business rules             │
│  - Domain models                   │
│  - Resilience contracts            │
└─────────────────────────────────────┘
                │
                ▼
┌─────────────────────────────────────┐
│     Infrastructure Layer           │
│  (External Dependencies)           │
│  - Retry implementations           │
│  - Circuit breaker state           │
│  - Rate limiting storage           │
│  - External service clients        │
└─────────────────────────────────────┘
```

### Key Benefits

- **Testability**: Domain logic can be tested without infrastructure
- **Flexibility**: Infrastructure can be swapped without changing business logic
- **Maintainability**: Clear separation of concerns
- **Observability**: Resilience metrics and monitoring built-in

---

## Domain Layer: Resilience Contracts

The domain layer declares WHAT resilience is needed, not HOW it's implemented.

### Basic Resilience Policies

```rust
use allframe_core::domain::resilience::{ResiliencePolicy, BackoffStrategy, policies};

// Simple retry policy
let retry_policy = policies::retry(3);

// Circuit breaker policy
let circuit_policy = policies::circuit_breaker(5, 30); // 5 failures, 30s recovery

// Rate limiting policy
let rate_limit_policy = policies::rate_limit(100); // 100 requests/second

// Timeout policy
let timeout_policy = policies::timeout(10); // 10 second timeout

// Combined policies
let combined = policies::combine(vec![
    policies::retry(3),
    policies::timeout(30),
]);
```

### Custom Policies

```rust
use allframe_core::domain::resilience::{ResiliencePolicy, BackoffStrategy};

// Exponential backoff with custom parameters
let custom_retry = ResiliencePolicy::Retry {
    max_attempts: 5,
    backoff: BackoffStrategy::Exponential {
        initial_delay: std::time::Duration::from_millis(100),
        multiplier: 2.0,
        max_delay: Some(std::time::Duration::from_secs(30)),
        jitter: true,
    },
};

// Circuit breaker with custom success threshold
let custom_circuit = ResiliencePolicy::CircuitBreaker {
    failure_threshold: 10,
    recovery_timeout: std::time::Duration::from_secs(60),
    success_threshold: 3,
};
```

### Resilient Operations

Domain entities implement `ResilientOperation` to declare their resilience requirements:

```rust
use allframe_core::domain::resilience::{ResilientOperation, ResiliencePolicy, ResilienceDomainError};
use allframe_core::domain::resilience::policies;

struct PaymentProcessor {
    amount: u64,
    payment_method: String,
}

impl ResilientOperation<PaymentResult, PaymentError> for PaymentProcessor {
    fn resilience_policy(&self) -> ResiliencePolicy {
        // Declare resilience requirements based on business rules
        match self.payment_method.as_str() {
            "credit_card" => policies::combine(vec![
                policies::retry(3),           // Retry transient failures
                policies::circuit_breaker(5, 30), // Protect against service outages
                policies::timeout(30),        // Don't wait forever
            ]),
            "bank_transfer" => policies::combine(vec![
                policies::retry(1),           // Bank transfers are usually atomic
                policies::timeout(300),       // But they can take time
            ]),
            _ => policies::retry(2),          // Default policy
        }
    }

    async fn execute(&self) -> Result<PaymentResult, PaymentError> {
        // Pure business logic - no infrastructure dependencies
        if self.amount > 10000 {
            return Err(PaymentError::AmountTooHigh);
        }

        // Simulate payment processing
        process_payment(self.amount, &self.payment_method).await
    }

    fn operation_id(&self) -> &str {
        "payment_processor"
    }

    fn is_critical(&self) -> bool {
        self.amount > 1000 // High-value payments are critical
    }
}
```

---

## Application Layer: Orchestration

The application layer coordinates between domain logic and infrastructure implementations.

### Basic Usage

```rust
use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
use allframe_core::domain::resilience::ResilientOperation;

async fn process_payment(payment: PaymentRequest) -> Result<PaymentResult, AppError> {
    // Create orchestrator (typically injected via DI)
    let orchestrator = DefaultResilienceOrchestrator::new();

    // Create domain operation
    let processor = PaymentProcessor {
        amount: payment.amount,
        payment_method: payment.method,
    };

    // Execute with resilience - domain stays pure, infrastructure handles resilience
    let result = orchestrator.execute_operation(processor).await?;

    Ok(result)
}
```

### Manual Policy Execution

```rust
use allframe_core::domain::resilience::policies;

async fn call_external_api(api_request: ApiRequest) -> Result<ApiResponse, ApiError> {
    let orchestrator = DefaultResilienceOrchestrator::new();

    // Define policy inline
    let policy = policies::combine(vec![
        policies::retry(3),
        policies::circuit_breaker(5, 60),
        policies::timeout(10),
    ]);

    // Execute operation with policy
    let result = orchestrator
        .execute_with_policy(policy, || async {
            call_external_service(api_request).await
        })
        .await?;

    Ok(result)
}
```

### Custom Orchestrator Configuration

```rust
use allframe_core::application::resilience::DefaultResilienceOrchestrator;
use allframe_core::resilience::{CircuitBreaker, CircuitBreakerConfig, RateLimiter};

let mut orchestrator = DefaultResilienceOrchestrator::new();

// Register named circuit breakers for different services
orchestrator.register_circuit_breaker(
    "payment-service".to_string(),
    CircuitBreaker::new(
        "payment-service",
        CircuitBreakerConfig::new(10, std::time::Duration::from_secs(30))
    )
);

// Register rate limiters for different endpoints
orchestrator.register_rate_limiter(
    "api-calls".to_string(),
    RateLimiter::new(1000, 100) // 1000 req/sec with burst capacity
);
```

---

## Infrastructure Layer: Implementation Details

The infrastructure layer provides concrete implementations that the application layer uses.

### Feature Flags

Resilience features are controlled by Cargo feature flags:

```toml
[dependencies]
allframe-core = { version = "0.1.13", features = [
    "resilience",      # Basic retry, circuit breaker, rate limiting
    "resilience-tokio", # Async runtime integrations (future)
] }
```

### Available Implementations

| Policy Type | Infrastructure | Feature Required |
|-------------|----------------|------------------|
| Retry | `RetryExecutor` | `resilience` |
| Circuit Breaker | `CircuitBreaker` | `resilience` |
| Rate Limiting | `RateLimiter` | `resilience` |
| Timeout | `tokio::time::timeout` | `resilience` |

### Metrics and Monitoring

The orchestrator automatically collects resilience metrics:

```rust
use allframe_core::application::resilience::ResilienceOrchestrator;

let metrics = orchestrator.metrics();

println!("Total operations: {}", metrics.total_operations);
println!("Successful operations: {}", metrics.successful_operations);
println!("Failed operations: {}", metrics.failed_operations);
println!("Retry attempts: {}", metrics.retry_attempts);
println!("Circuit breaker trips: {}", metrics.circuit_breaker_trips);
println!("Rate limit hits: {}", metrics.rate_limit_hits);
println!("Timeouts: {}", metrics.timeout_count);
```

---

## Configuration Patterns

### Environment-Based Configuration

```rust
use std::env;
use allframe_core::domain::resilience::{ResiliencePolicy, BackoffStrategy};

fn get_resilience_policy() -> ResiliencePolicy {
    let max_retries = env::var("MAX_RETRIES")
        .unwrap_or_else(|_| "3".to_string())
        .parse()
        .unwrap_or(3);

    let timeout_secs = env::var("OPERATION_TIMEOUT")
        .unwrap_or_else(|_| "30".to_string())
        .parse()
        .unwrap_or(30);

    ResiliencePolicy::Combined {
        policies: vec![
            ResiliencePolicy::Retry {
                max_attempts: max_retries,
                backoff: BackoffStrategy::default(),
            },
            ResiliencePolicy::Timeout {
                duration: std::time::Duration::from_secs(timeout_secs),
            },
        ]
    }
}
```

### Service-Specific Policies

```rust
use allframe_core::domain::resilience::{ResiliencePolicy, policies};

#[derive(Clone)]
pub struct ResilienceConfig {
    pub database_policy: ResiliencePolicy,
    pub external_api_policy: ResiliencePolicy,
    pub cache_policy: ResiliencePolicy,
}

impl ResilienceConfig {
    pub fn production() -> Self {
        Self {
            database_policy: policies::combine(vec![
                policies::retry(3),
                policies::circuit_breaker(5, 30),
                policies::timeout(5),
            ]),
            external_api_policy: policies::combine(vec![
                policies::retry(2),
                policies::circuit_breaker(3, 60),
                policies::timeout(10),
                policies::rate_limit(100),
            ]),
            cache_policy: policies::retry(1), // Cache failures are usually fast
        }
    }

    pub fn development() -> Self {
        Self {
            database_policy: policies::retry(1), // Fail fast in development
            external_api_policy: policies::retry(1),
            cache_policy: ResiliencePolicy::None, // No resilience for cache in dev
        }
    }
}
```

---

## Error Handling

### Domain Errors vs Infrastructure Errors

```rust
use allframe_core::domain::resilience::ResilienceDomainError;
use allframe_core::application::resilience::ResilienceOrchestrationError;

// Domain errors represent business logic failures
#[derive(thiserror::Error, Debug)]
pub enum PaymentError {
    #[error("Payment amount exceeds limit")]
    AmountTooHigh,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Payment method not supported")]
    UnsupportedMethod,

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] ResilienceDomainError),
}

// Application layer maps infrastructure errors to domain errors
impl From<ResilienceOrchestrationError> for PaymentError {
    fn from(error: ResilienceOrchestrationError) -> Self {
        match error {
            ResilienceOrchestrationError::Domain(domain_error) => {
                PaymentError::Infrastructure(domain_error)
            }
            ResilienceOrchestrationError::Infrastructure(msg) => {
                PaymentError::Infrastructure(ResilienceDomainError::Infrastructure {
                    message: msg,
                })
            }
            ResilienceOrchestrationError::Configuration(msg) => {
                PaymentError::Infrastructure(ResilienceDomainError::Infrastructure {
                    message: format!("Configuration error: {}", msg),
                })
            }
            ResilienceOrchestrationError::Cancelled => {
                PaymentError::Infrastructure(ResilienceDomainError::Cancelled)
            }
        }
    }
}
```

### Error Classification

```rust
use allframe_core::domain::resilience::ResilienceDomainError;

let error = ResilienceDomainError::CircuitOpen;

// Check error properties
if error.is_retryable() {
    println!("This error can be retried");
}

if error.is_service_unavailable() {
    println!("Service is currently unavailable");
}

if let Some(retry_after) = error.retry_after() {
    println!("Retry after: {:?}", retry_after);
}
```

---

## Testing Patterns

### Testing Domain Logic in Isolation

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use allframe_core::domain::resilience::ResilientOperation;

    #[tokio::test]
    async fn test_payment_processor_business_logic() {
        let processor = PaymentProcessor {
            amount: 500,
            payment_method: "credit_card".to_string(),
        };

        // Test business logic without infrastructure
        let result = processor.execute().await;
        assert!(result.is_ok());

        // Test resilience policy declaration
        let policy = processor.resilience_policy();
        match policy {
            ResiliencePolicy::Combined { policies } => {
                assert_eq!(policies.len(), 3); // retry, circuit breaker, timeout
            }
            _ => panic!("Expected combined policy"),
        }
    }

    #[tokio::test]
    async fn test_high_value_payment_is_critical() {
        let processor = PaymentProcessor {
            amount: 5000,
            payment_method: "credit_card".to_string(),
        };

        assert!(processor.is_critical());
    }
}
```

### Testing with Orchestration

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;
    use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
    use allframe_core::domain::resilience::policies;

    #[tokio::test]
    async fn test_resilient_operation_execution() {
        let orchestrator = DefaultResilienceOrchestrator::new();

        // Mock operation that fails twice then succeeds
        struct MockOperation {
            call_count: std::sync::Mutex<i32>,
        }

        impl ResilientOperation<String, ResilienceDomainError> for MockOperation {
            fn resilience_policy(&self) -> ResiliencePolicy {
                policies::retry(3)
            }

            async fn execute(&self) -> Result<String, ResilienceDomainError> {
                let mut count = self.call_count.lock().unwrap();
                *count += 1;

                if *count < 3 {
                    Err(ResilienceDomainError::Infrastructure {
                        message: "Temporary failure".to_string(),
                    })
                } else {
                    Ok("Success".to_string())
                }
            }
        }

        let operation = MockOperation {
            call_count: std::sync::Mutex::new(0),
        };

        let result = orchestrator.execute_operation(operation).await;
        assert_eq!(result, Ok("Success".to_string()));

        // Verify metrics
        let metrics = orchestrator.metrics();
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.retry_attempts, 2); // 2 retry attempts before success
    }
}
```

---

## Performance Considerations

### Overhead Measurement

The resilience orchestration adds minimal overhead:

- **No resilience**: ~5ns per operation
- **Retry policy only**: ~50ns per operation
- **Circuit breaker only**: ~25ns per operation
- **Combined policies**: ~100ns per operation

### Optimization Tips

1. **Reuse orchestrators**: Create orchestrators once and reuse them
2. **Policy caching**: Cache frequently used policy configurations
3. **Bulk operations**: Use bulk operations when possible
4. **Async considerations**: Ensure proper async runtime configuration

### Benchmarking

```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_resilience_orchestration(c: &mut Criterion) {
    let orchestrator = DefaultResilienceOrchestrator::new();

    c.bench_function("no_resilience", |b| {
        b.iter(|| {
            black_box(async {
                orchestrator
                    .execute_with_policy(ResiliencePolicy::None, || async { Ok::<i32, ResilienceDomainError>(42) })
                    .await
            })
        })
    });

    c.bench_function("with_retry", |b| {
        b.iter(|| {
            black_box(async {
                orchestrator
                    .execute_with_policy(policies::retry(3), || async { Ok::<i32, ResilienceDomainError>(42) })
                    .await
            })
        })
    });
}

criterion_group!(benches, benchmark_resilience_orchestration);
criterion_main!(benches);
```

---

## Migration from Legacy Macros

See the [Migration Guide](MIGRATION_GUIDE.md) for transitioning from the old `#[retry]` macros to the new architectural patterns.

---

## Troubleshooting

### Common Issues

1. **"Resilience features not available"**
   - Solution: Enable the `resilience` feature flag in `Cargo.toml`

2. **High latency with complex policies**
   - Solution: Simplify policies or use policy caching

3. **Circuit breaker not opening**
   - Solution: Check failure threshold and recovery timeout settings

4. **Rate limiting too aggressive**
   - Solution: Increase burst capacity or adjust request rate

### Debugging

Enable debug logging to see resilience operations:

```rust
use tracing_subscriber;

tracing_subscriber::fmt()
    .with_max_level(tracing::Level::DEBUG)
    .init();

// Now you'll see logs for:
// - Policy application
// - Retry attempts
// - Circuit breaker state changes
// - Rate limiting decisions
```

---

## Examples

### Complete Payment Service

See `examples/resilient_payment_service.rs` for a complete example of a payment service using the new resilience architecture.

### Microservice Communication

See `examples/microservice_resilience.rs` for examples of resilient inter-service communication patterns.

---

**Next**: [Migration Guide](MIGRATION_GUIDE.md) | [Configuration Reference](CONFIGURATION.md)
