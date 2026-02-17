//! Complete example of a resilient payment service using Clean Architecture.
//!
//! This example demonstrates:
//! - Domain layer resilience contracts
//! - Application layer orchestration
//! - Configuration-based policies
//! - Observability and metrics
//! - Error handling and recovery

use allframe_core::domain::resilience::{ResilientOperation, ResiliencePolicy, ResilienceDomainError};
use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
use allframe_core::application::resilience_config::{ResilienceConfig, PolicyConfig, SimplePolicyConfig, RetryConfig, BackoffConfig};
use allframe_core::application::resilience_observability::{ResilienceObservability, InstrumentedResilienceOrchestrator};
use std::sync::Arc;
use tokio::sync::Mutex;

// ===== DOMAIN LAYER =====

/// Domain errors for payment operations
#[derive(thiserror::Error, Debug)]
pub enum PaymentError {
    #[error("Payment amount must be positive")]
    InvalidAmount,

    #[error("Insufficient funds")]
    InsufficientFunds,

    #[error("Payment method not supported")]
    UnsupportedMethod,

    #[error("Payment processing failed: {message}")]
    ProcessingFailed { message: String },

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] ResilienceDomainError),
}

/// Payment request data
#[derive(Clone, Debug)]
pub struct PaymentRequest {
    pub amount: u64,
    pub currency: String,
    pub method: PaymentMethod,
    pub customer_id: String,
}

/// Payment response data
#[derive(Clone, Debug)]
pub struct PaymentResponse {
    pub transaction_id: String,
    pub status: PaymentStatus,
    pub processed_at: std::time::SystemTime,
}

/// Payment methods
#[derive(Clone, Debug)]
pub enum PaymentMethod {
    CreditCard,
    DebitCard,
    BankTransfer,
    DigitalWallet,
}

/// Payment status
#[derive(Clone, Debug)]
pub enum PaymentStatus {
    Completed,
    Pending,
    Failed,
}

/// Domain service for payment processing
#[async_trait::async_trait]
pub trait PaymentService: Send + Sync {
    async fn process_payment(&self, request: PaymentRequest) -> Result<PaymentResponse, PaymentError>;
}

/// Domain entity that declares resilience requirements
pub struct PaymentProcessor {
    payment_service: Arc<dyn PaymentService>,
    request: PaymentRequest,
}

impl PaymentProcessor {
    pub fn new(payment_service: Arc<dyn PaymentService>, request: PaymentRequest) -> Self {
        Self {
            payment_service,
            request,
        }
    }
}

impl ResilientOperation<PaymentResponse, PaymentError> for PaymentProcessor {
    fn resilience_policy(&self) -> ResiliencePolicy {
        // Business rules determine resilience requirements
        match (&self.request.method, self.request.amount) {
            // High-value payments need strong resilience
            (_, amount) if amount > 10000 => ResiliencePolicy::Combined {
                policies: vec![
                    ResiliencePolicy::Retry {
                        max_attempts: 3,
                        backoff: allframe_core::domain::resilience::BackoffStrategy::Exponential {
                            initial_delay: std::time::Duration::from_millis(500),
                            multiplier: 2.0,
                            max_delay: Some(std::time::Duration::from_secs(30)),
                            jitter: true,
                        },
                    },
                    ResiliencePolicy::CircuitBreaker {
                        failure_threshold: 5,
                        recovery_timeout: std::time::Duration::from_secs(60),
                        success_threshold: 3,
                    },
                    ResiliencePolicy::Timeout {
                        duration: std::time::Duration::from_secs(120),
                    },
                ],
            },

            // Bank transfers are slow but reliable
            (PaymentMethod::BankTransfer, _) => ResiliencePolicy::Combined {
                policies: vec![
                    ResiliencePolicy::Retry {
                        max_attempts: 1, // Don't retry bank transfers
                        backoff: allframe_core::domain::resilience::BackoffStrategy::Fixed {
                            delay: std::time::Duration::from_secs(1),
                        },
                    },
                    ResiliencePolicy::Timeout {
                        duration: std::time::Duration::from_secs(300), // 5 minutes
                    },
                ],
            },

            // Card payments are fast but may fail
            (PaymentMethod::CreditCard | PaymentMethod::DebitCard, _) => ResiliencePolicy::Combined {
                policies: vec![
                    ResiliencePolicy::Retry {
                        max_attempts: 3,
                        backoff: allframe_core::domain::resilience::BackoffStrategy::Exponential {
                            initial_delay: std::time::Duration::from_millis(200),
                            multiplier: 1.5,
                            max_delay: Some(std::time::Duration::from_secs(10)),
                            jitter: true,
                        },
                    },
                    ResiliencePolicy::CircuitBreaker {
                        failure_threshold: 10,
                        recovery_timeout: std::time::Duration::from_secs(30),
                        success_threshold: 5,
                    },
                    ResiliencePolicy::RateLimit {
                        requests_per_second: 100,
                        burst_capacity: 20,
                    },
                    ResiliencePolicy::Timeout {
                        duration: std::time::Duration::from_secs(30),
                    },
                ],
            },

            // Digital wallets are fast and usually reliable
            (PaymentMethod::DigitalWallet, _) => ResiliencePolicy::Combined {
                policies: vec![
                    ResiliencePolicy::Retry {
                        max_attempts: 2,
                        backoff: allframe_core::domain::resilience::BackoffStrategy::Fixed {
                            delay: std::time::Duration::from_millis(500),
                        },
                    },
                    ResiliencePolicy::RateLimit {
                        requests_per_second: 200,
                        burst_capacity: 50,
                    },
                    ResiliencePolicy::Timeout {
                        duration: std::time::Duration::from_secs(15),
                    },
                ],
            },
        }
    }

    async fn execute(&self) -> Result<PaymentResponse, PaymentError> {
        // Domain logic - pure business rules
        self.validate_payment()?;

        // Delegate to infrastructure service
        self.payment_service.process_payment(self.request.clone()).await
    }

    fn operation_id(&self) -> &str {
        "payment_processor"
    }

    fn is_critical(&self) -> bool {
        // High-value payments are critical
        self.request.amount > 5000
    }
}

impl PaymentProcessor {
    fn validate_payment(&self) -> Result<(), PaymentError> {
        if self.request.amount == 0 {
            return Err(PaymentError::InvalidAmount);
        }

        match self.request.method {
            PaymentMethod::CreditCard | PaymentMethod::DebitCard => {
                // Card payments have limits
                if self.request.amount > 50000 {
                    return Err(PaymentError::InvalidAmount);
                }
            }
            PaymentMethod::BankTransfer => {
                // Bank transfers have different limits
                if self.request.amount < 100 {
                    return Err(PaymentError::InvalidAmount);
                }
            }
            PaymentMethod::DigitalWallet => {
                // Wallet payments are flexible
            }
        }

        Ok(())
    }
}

// ===== APPLICATION LAYER =====

/// Application service for payment processing
pub struct PaymentApplicationService {
    payment_service: Arc<dyn PaymentService>,
    orchestrator: Arc<dyn ResilienceOrchestrator>,
}

impl PaymentApplicationService {
    pub fn new(
        payment_service: Arc<dyn PaymentService>,
        orchestrator: Arc<dyn ResilienceOrchestrator>,
    ) -> Self {
        Self {
            payment_service,
            orchestrator,
        }
    }

    /// Process a payment with resilience
    pub async fn process_payment(
        &self,
        request: PaymentRequest,
    ) -> Result<PaymentResponse, PaymentError> {
        let processor = PaymentProcessor::new(self.payment_service.clone(), request);

        // Application layer orchestrates using domain contracts
        self.orchestrator
            .execute_operation(processor)
            .await
            .map_err(|e| match e {
                allframe_core::application::resilience::ResilienceOrchestrationError::Domain(domain_error) => {
                    PaymentError::Infrastructure(domain_error)
                }
                _ => PaymentError::ProcessingFailed {
                    message: "Orchestration failed".to_string(),
                },
            })
    }

    /// Get resilience metrics for monitoring
    pub fn get_resilience_metrics(&self) -> allframe_core::application::resilience::ResilienceMetrics {
        self.orchestrator.metrics()
    }
}

// ===== INFRASTRUCTURE LAYER =====

/// Mock payment service for demonstration
pub struct MockPaymentService {
    failure_rate: f64, // 0.0 to 1.0
    delay_ms: u64,
}

impl MockPaymentService {
    pub fn new(failure_rate: f64, delay_ms: u64) -> Self {
        Self {
            failure_rate,
            delay_ms,
        }
    }
}

#[async_trait::async_trait]
impl PaymentService for MockPaymentService {
    async fn process_payment(&self, request: PaymentRequest) -> Result<PaymentResponse, PaymentError> {
        // Simulate processing delay
        tokio::time::sleep(std::time::Duration::from_millis(self.delay_ms)).await;

        // Simulate failures
        if rand::random::<f64>() < self.failure_rate {
            return Err(PaymentError::ProcessingFailed {
                message: "Simulated payment failure".to_string(),
            });
        }

        // Simulate insufficient funds occasionally
        if rand::random::<f64>() < 0.1 {
            return Err(PaymentError::InsufficientFunds);
        }

        Ok(PaymentResponse {
            transaction_id: format!("txn_{}", rand::random::<u64>()),
            status: PaymentStatus::Completed,
            processed_at: std::time::SystemTime::now(),
        })
    }
}

// ===== CONFIGURATION =====

/// Load resilience configuration from file
pub fn load_resilience_config() -> Result<ResilienceConfig, Box<dyn std::error::Error>> {
    // In a real application, this would load from a config file
    // For this example, we'll create a configuration programmatically

    let mut config = ResilienceConfig {
        enabled: true,
        policies: std::collections::HashMap::new(),
        services: std::collections::HashMap::new(),
    };

    // Define named policies
    config.policies.insert(
        "high_value_payments".to_string(),
        PolicyConfig::Combined {
            retry: Some(RetryConfig {
                max_attempts: 3,
                backoff: BackoffConfig::Exponential {
                    initial_delay_ms: 500,
                    multiplier: 2.0,
                    max_delay_ms: Some(30000),
                    jitter: true,
                },
            }),
            circuit_breaker: Some(allframe_core::application::resilience_config::CircuitBreakerConfig {
                failure_threshold: 5,
                recovery_timeout_seconds: 60,
                success_threshold: 3,
            }),
            rate_limit: None,
            timeout: Some(allframe_core::application::resilience_config::TimeoutConfig {
                duration_seconds: 120,
            }),
        },
    );

    config.policies.insert(
        "card_payments".to_string(),
        PolicyConfig::Combined {
            retry: Some(RetryConfig {
                max_attempts: 3,
                backoff: BackoffConfig::Exponential {
                    initial_delay_ms: 200,
                    multiplier: 1.5,
                    max_delay_ms: Some(10000),
                    jitter: true,
                },
            }),
            circuit_breaker: Some(allframe_core::application::resilience_config::CircuitBreakerConfig {
                failure_threshold: 10,
                recovery_timeout_seconds: 30,
                success_threshold: 5,
            }),
            rate_limit: Some(allframe_core::application::resilience_config::RateLimitConfig {
                requests_per_second: 100,
                burst_capacity: 20,
            }),
            timeout: Some(allframe_core::application::resilience_config::TimeoutConfig {
                duration_seconds: 30,
            }),
        },
    );

    // Map services to policies
    config.services.insert("payment_processor".to_string(), "card_payments".to_string());

    Ok(config)
}

// ===== MAIN APPLICATION =====

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("🚀 Starting Resilient Payment Service");

    // Load configuration
    let resilience_config = load_resilience_config()?;
    println!("✅ Loaded resilience configuration");

    // Create infrastructure services
    let payment_service = Arc::new(MockPaymentService::new(0.2, 100)); // 20% failure rate, 100ms delay

    // Create orchestrator with observability
    let base_orchestrator = DefaultResilienceOrchestrator::new();
    let observability = ResilienceObservability::new();
    let orchestrator = Arc::new(InstrumentedResilienceOrchestrator::new(
        base_orchestrator,
        observability,
    ));

    // Create application service
    let payment_app = PaymentApplicationService::new(
        payment_service,
        orchestrator.clone(),
    );

    println!("🎯 Processing payments with resilience...");

    // Process multiple payments to demonstrate resilience
    let payment_requests = vec![
        PaymentRequest {
            amount: 100,
            currency: "USD".to_string(),
            method: PaymentMethod::CreditCard,
            customer_id: "customer_123".to_string(),
        },
        PaymentRequest {
            amount: 15000, // High value
            currency: "USD".to_string(),
            method: PaymentMethod::BankTransfer,
            customer_id: "customer_456".to_string(),
        },
        PaymentRequest {
            amount: 50,
            currency: "USD".to_string(),
            method: PaymentMethod::DigitalWallet,
            customer_id: "customer_789".to_string(),
        },
    ];

    for (i, request) in payment_requests.into_iter().enumerate() {
        println!("\n💳 Processing payment {}: ${} via {:?}", i + 1, request.amount, request.method);

        match payment_app.process_payment(request).await {
            Ok(response) => {
                println!("✅ Payment completed: {}", response.transaction_id);
            }
            Err(error) => {
                println!("❌ Payment failed: {}", error);
            }
        }

        // Show resilience metrics
        let metrics = payment_app.get_resilience_metrics();
        println!("📊 Resilience metrics: {} total, {} successful, {} failed",
                 metrics.total_operations, metrics.successful_operations, metrics.failed_operations);
    }

    // Show final observability data
    println!("\n📈 Final Observability Report");
    println!("==============================");

    let final_metrics = payment_app.get_resilience_metrics();
    println!("Total operations: {}", final_metrics.total_operations);
    println!("Successful operations: {}", final_metrics.successful_operations);
    println!("Failed operations: {}", final_metrics.failed_operations);
    println!("Retry attempts: {}", final_metrics.retry_attempts);
    println!("Circuit breaker trips: {}", final_metrics.circuit_breaker_trips);
    println!("Rate limit hits: {}", final_metrics.rate_limit_hits);
    println!("Timeouts: {}", final_metrics.timeout_count);

    // Show health status
    let health = observability.health_status();
    println!("Overall health: {:?}", health.overall_health);
    println!("Circuit breakers open: {}", health.circuit_breakers_open);

    println!("\n🎉 Resilient Payment Service demonstration complete!");
    println!("💡 Key takeaways:");
    println!("  • Domain layer defines WHAT resilience is needed");
    println!("  • Application layer orchestrates HOW it's implemented");
    println!("  • Infrastructure provides concrete implementations");
    println!("  • Observability enables monitoring and debugging");
    println!("  • Configuration allows runtime policy changes");

    Ok(())
}
