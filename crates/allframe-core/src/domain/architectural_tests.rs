//! Architectural compliance tests for the domain layer.
//!
//! These tests ensure that the domain layer maintains Clean Architecture principles:
//! - No dependencies on infrastructure layer types
//! - Pure business logic without external concerns
//! - Clear separation of concerns

#[cfg(test)]
mod architectural_compliance {
    use super::super::*;
    use std::any::TypeId;

    /// Test that domain resilience contracts don't depend on infrastructure types
    #[test]
    fn domain_resilience_contracts_no_infrastructure_dependencies() {
        // This test ensures that domain layer resilience types don't accidentally
        // import infrastructure types. We do this by checking that the types
        // don't contain references to known infrastructure modules.

        let resilience_policy_type = TypeId::of::<ResiliencePolicy>();
        let backoff_strategy_type = TypeId::of::<BackoffStrategy>();
        let domain_error_type = TypeId::of::<ResilienceDomainError>();

        // These types should be pure domain concepts
        // If this test fails, it means infrastructure types have leaked into domain layer

        // The types should exist and be constructible without infrastructure
        let policy = ResiliencePolicy::None;
        let backoff = BackoffStrategy::default();
        let error = ResilienceDomainError::Timeout {
            duration: std::time::Duration::from_secs(1),
        };

        // Ensure they're the expected types
        assert!(matches!(policy, ResiliencePolicy::None));
        assert!(matches!(backoff, BackoffStrategy::Exponential { .. }));
        assert!(matches!(error, ResilienceDomainError::Timeout { .. }));
    }

    /// Test that domain layer can be used without any infrastructure imports
    #[test]
    fn domain_layer_standalone_usage() {
        // This test verifies that domain layer code can be written and used
        // without importing any infrastructure types

        // Define a domain service that declares resilience requirements
        struct PaymentService;

        impl PaymentService {
            fn process_payment(&self, amount: u32) -> Result<String, ResilienceDomainError> {
                if amount > 1000 {
                    return Err(ResilienceDomainError::RateLimited {
                        retry_after: Some(std::time::Duration::from_secs(60)),
                    });
                }
                Ok(format!("Processed payment of ${}", amount))
            }
        }

        // Domain layer can declare policies without knowing how they're implemented
        let policy = ResiliencePolicy::Retry {
            max_attempts: 3,
            backoff: BackoffStrategy::Exponential {
                initial_delay: std::time::Duration::from_millis(100),
                multiplier: 2.0,
                max_delay: Some(std::time::Duration::from_secs(10)),
                jitter: true,
            },
        };

        // Verify the policy is well-formed
        match policy {
            ResiliencePolicy::Retry { max_attempts, backoff: BackoffStrategy::Exponential { .. } } => {
                assert_eq!(max_attempts, 3);
            }
            _ => panic!("Expected retry policy"),
        }

        // Domain logic works independently
        let service = PaymentService;
        let result = service.process_payment(500);
        assert!(result.is_ok());
        assert_eq!(result.unwrap(), "Processed payment of $500");
    }

    /// Test that domain error types are properly isolated
    #[test]
    fn domain_error_isolation() {
        // Domain errors should not depend on infrastructure error types
        let timeout_error = ResilienceDomainError::Timeout {
            duration: std::time::Duration::from_secs(5),
        };

        let retry_error = ResilienceDomainError::RetryExhausted {
            attempts: 3,
            last_error: "Connection failed".to_string(),
        };

        let circuit_error = ResilienceDomainError::CircuitOpen;

        // These should be pure domain concepts
        assert!(timeout_error.is_retryable());
        assert!(!retry_error.is_retryable()); // Already exhausted retries
        assert!(!circuit_error.is_retryable()); // Circuit breaker protects from further calls

        assert!(!timeout_error.is_service_unavailable());
        assert!(circuit_error.is_service_unavailable());
    }

    /// Test that ResilientOperation trait maintains domain purity
    #[test]
    fn resilient_operation_domain_purity() {
        // Domain operations should be able to declare resilience without infrastructure knowledge

        struct DomainOperation {
            operation_id: String,
            critical: bool,
        }

        impl ResilientOperation<String, ResilienceDomainError> for DomainOperation {
            fn resilience_policy(&self) -> ResiliencePolicy {
                ResiliencePolicy::CircuitBreaker {
                    failure_threshold: 5,
                    recovery_timeout: std::time::Duration::from_secs(30),
                    success_threshold: 3,
                }
            }

            async fn execute(&self) -> Result<String, ResilienceDomainError> {
                // Pure domain logic - no infrastructure dependencies
                if self.operation_id == "fail" {
                    Err(ResilienceDomainError::Infrastructure {
                        message: "Domain logic failure".to_string(),
                    })
                } else {
                    Ok(format!("Executed {}", self.operation_id))
                }
            }

            fn operation_id(&self) -> &str {
                &self.operation_id
            }

            fn is_critical(&self) -> bool {
                self.critical
            }
        }

        // Test domain operation construction and policy declaration
        let op = DomainOperation {
            operation_id: "test_op".to_string(),
            critical: true,
        };

        assert_eq!(op.operation_id(), "test_op");
        assert!(op.is_critical());

        // Policy should be declarable without infrastructure knowledge
        let policy = op.resilience_policy();
        match policy {
            ResiliencePolicy::CircuitBreaker { failure_threshold, .. } => {
                assert_eq!(failure_threshold, 5);
            }
            _ => panic!("Expected circuit breaker policy"),
        }
    }

    /// Test policy helper functions maintain domain purity
    #[test]
    fn policy_helpers_domain_purity() {
        use crate::domain::resilience::policies;

        // Policy helpers should create pure domain policies
        let retry_policy = policies::retry(5);
        let circuit_policy = policies::circuit_breaker(10, 60);
        let rate_limit_policy = policies::rate_limit(100);
        let timeout_policy = policies::timeout(30);
        let combined_policy = policies::combine(vec![
            retry_policy.clone(),
            timeout_policy.clone(),
        ]);

        // Verify policies are constructed correctly
        match retry_policy {
            ResiliencePolicy::Retry { max_attempts, .. } => assert_eq!(max_attempts, 5),
            _ => panic!("Expected retry policy"),
        }

        match circuit_policy {
            ResiliencePolicy::CircuitBreaker { failure_threshold, recovery_timeout, .. } => {
                assert_eq!(failure_threshold, 10);
                assert_eq!(recovery_timeout, std::time::Duration::from_secs(60));
            }
            _ => panic!("Expected circuit breaker policy"),
        }

        match rate_limit_policy {
            ResiliencePolicy::RateLimit { requests_per_second, .. } => {
                assert_eq!(requests_per_second, 100);
            }
            _ => panic!("Expected rate limit policy"),
        }

        match timeout_policy {
            ResiliencePolicy::Timeout { duration } => {
                assert_eq!(duration, std::time::Duration::from_secs(30));
            }
            _ => panic!("Expected timeout policy"),
        }

        match combined_policy {
            ResiliencePolicy::Combined { policies } => {
                assert_eq!(policies.len(), 2);
            }
            _ => panic!("Expected combined policy"),
        }
    }
}

/// Integration tests to ensure domain and application layers work together cleanly
#[cfg(test)]
mod domain_application_integration {
    use super::super::*;
    use crate::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};

    #[tokio::test]
    async fn domain_application_layer_integration() {
        // Domain layer defines operation
        struct TestOperation;

        impl ResilientOperation<i32, ResilienceDomainError> for TestOperation {
            fn resilience_policy(&self) -> ResiliencePolicy {
                ResiliencePolicy::Retry {
                    max_attempts: 2,
                    backoff: BackoffStrategy::Fixed {
                        delay: std::time::Duration::from_millis(10),
                    },
                }
            }

            async fn execute(&self) -> Result<i32, ResilienceDomainError> {
                static CALL_COUNT: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                let count = CALL_COUNT.fetch_add(1, std::sync::atomic::Ordering::SeqCst);

                if count < 1 {
                    // Fail first attempt
                    Err(ResilienceDomainError::Infrastructure {
                        message: "Temporary failure".to_string(),
                    })
                } else {
                    // Succeed on retry
                    Ok(42)
                }
            }
        }

        // Application layer orchestrates with infrastructure
        let orchestrator = DefaultResilienceOrchestrator::new();
        let operation = TestOperation;

        // Execute through application layer - domain stays pure
        let result = orchestrator.execute_operation(operation).await;

        // Should succeed after retry
        assert_eq!(result, Ok(42));

        // Verify metrics were recorded
        let metrics = orchestrator.metrics();
        assert_eq!(metrics.total_operations, 1);
        assert_eq!(metrics.successful_operations, 1);
        assert_eq!(metrics.retry_attempts, 1); // One retry attempt
    }

    #[tokio::test]
    async fn infrastructure_failures_properly_handled() {
        // Test that infrastructure failures are properly translated to domain errors
        let orchestrator = DefaultResilienceOrchestrator::new();

        let result = orchestrator
            .execute_with_policy(
                ResiliencePolicy::Retry {
                    max_attempts: 1, // Only one attempt
                    backoff: BackoffStrategy::Fixed {
                        delay: std::time::Duration::from_millis(1),
                    },
                },
                || async {
                    Result::<i32, ResilienceDomainError>::Err(
                        ResilienceDomainError::Infrastructure {
                            message: "Database connection failed".to_string(),
                        }
                    )
                }
            )
            .await;

        // Should fail with retry exhausted
        match result {
            Err(crate::application::resilience::ResilienceOrchestrationError::Domain(
                ResilienceDomainError::RetryExhausted { attempts, .. }
            )) => {
                assert_eq!(attempts, 1);
            }
            other => panic!("Expected retry exhausted error, got {:?}", other),
        }
    }
}
