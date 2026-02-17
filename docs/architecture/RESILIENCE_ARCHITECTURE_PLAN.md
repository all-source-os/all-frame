# Resilience Architecture Refactoring Plan

**Status**: 🚧 Active Development
**Priority**: P0 (Critical for Clean Architecture compliance)
**Timeline**: Long-term (Q1 2025)

---

## Executive Summary

The current resilience implementation (`#[retry]`, `#[circuit_breaker]`, `#[rate_limited]` macros) violates Clean Architecture principles by injecting infrastructure-level code directly into domain and application layer functions. This refactoring will:

- **Move resilience logic to proper architectural layers**
- **Maintain full backward compatibility**
- **Establish patterns for future infrastructure features**
- **Improve testability and maintainability**

---

## Current Architectural Violation

### Problem
```rust
// Current: Infrastructure code injected into domain layer
#[retry(max_retries = 3)]
async fn business_operation(&self) -> Result<BusinessResult, BusinessError> {
    // This function now depends on RetryExecutor, RetryConfig, etc.
    // VIOLATION: Domain layer knows about infrastructure concerns
}
```

### Root Cause
- Macros inject infrastructure types directly into function bodies
- No separation between business logic and resilience policies
- Infrastructure concerns bleed into domain layer

---

## Target Architecture

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

### Resilience Flow

```
Domain Layer (Contracts)
    ↓
Application Layer (Orchestration)
    ↓
Infrastructure Layer (Implementation)
```

---

## Implementation Phases

### Phase 1: Domain Layer Resilience Contracts ✅

**Goal**: Define resilience contracts in the domain layer without implementation details.

**Deliverables**:
- `ResilientOperation` trait for operations that need resilience
- `ResiliencePolicy` enum for declaring resilience requirements
- `ResilienceError` types for domain-level error handling

**Example**:
```rust
// Domain layer - pure contracts
#[derive(ResiliencePolicy)]
pub enum PaymentResiliencePolicy {
    ProcessPayment {
        max_retries: u32,
        timeout_seconds: u64,
    },
    CheckStatus {
        circuit_breaker: bool,
    },
}

pub trait PaymentService: Send + Sync {
    async fn process_payment(
        &self,
        payment: Payment,
        policy: ResiliencePolicy,
    ) -> Result<PaymentResult, PaymentError>;
}
```

### Phase 2: Application Layer Orchestration 🚧

**Goal**: Application layer orchestrates resilience without domain knowing implementation details.

**Deliverables**:
- `ResilienceOrchestrator` trait for wiring policies to implementations
- Application-level resilience configuration
- Policy-to-implementation mapping

**Example**:
```rust
// Application layer - orchestration
pub struct PaymentUseCase {
    payment_service: Arc<dyn PaymentService>,
    resilience_orchestrator: Arc<ResilienceOrchestrator>,
}

impl PaymentUseCase {
    pub async fn process_payment(&self, payment: Payment) -> Result<PaymentResult, UseCaseError> {
        let policy = ResiliencePolicy::Retry {
            max_attempts: 3,
            backoff: ExponentialBackoff::default(),
        };

        // Orchestrator handles the resilience implementation
        self.resilience_orchestrator
            .execute_with_policy(policy, || async {
                self.payment_service.process_payment(payment).await
            })
            .await
    }
}
```

### Phase 3: Infrastructure Layer Implementation ✅

**Goal**: Infrastructure provides concrete resilience implementations.

**Deliverables**:
- `RetryExecutor` implementations
- `CircuitBreaker` implementations
- `RateLimiter` implementations
- `ResilienceOrchestrator` concrete implementation

### Phase 4: Macro Refactoring (Current Target)

**Goal**: Update macros to use the new architectural pattern.

**Deliverables**:
- Backward-compatible macro API
- New architectural macro variants
- Migration guide for existing code

**Migration Path**:
```rust
// Old way (infrastructure injection - VIOLATION)
#[retry(max_retries = 3)]
async fn business_logic() -> Result<(), Error> { /* ... */ }

// New way (architecturally clean)
#[resilient(policy = "retry(max_retries = 3)")]
async fn business_logic() -> Result<(), Error> { /* ... */ }

// Or at application layer
let result = orchestrator.execute_with_policy(
    ResiliencePolicy::Retry { max_attempts: 3 },
    || business_logic()
).await;
```

---

## Detailed Implementation Plan

### 1. Domain Layer Contracts (Week 1-2)

**Create `resilience` module in domain layer:**

```rust
// crates/allframe-core/src/domain/resilience.rs

/// Resilience policies that domain can declare without knowing implementation
#[derive(Clone, Debug)]
pub enum ResiliencePolicy {
    None,
    Retry {
        max_attempts: u32,
        backoff: BackoffStrategy,
    },
    CircuitBreaker {
        failure_threshold: u32,
        recovery_timeout: Duration,
    },
    RateLimit {
        requests_per_second: u32,
        burst_capacity: u32,
    },
    Timeout {
        duration: Duration,
    },
}

/// Domain-level error that can be converted to infrastructure errors
#[derive(thiserror::Error, Debug)]
pub enum ResilienceDomainError {
    #[error("Operation timed out")]
    Timeout,

    #[error("Operation failed after {attempts} attempts")]
    RetryExhausted { attempts: u32 },

    #[error("Circuit breaker is open")]
    CircuitOpen,

    #[error("Rate limit exceeded")]
    RateLimited,

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] Box<dyn std::error::Error + Send + Sync>),
}

/// Trait for operations that declare resilience requirements
pub trait ResilientOperation<T, E> {
    fn resilience_policy(&self) -> ResiliencePolicy;
    async fn execute(&self) -> Result<T, E>;
}
```

### 2. Application Layer Orchestration (Week 3-4)

**Create resilience orchestrator:**

```rust
// crates/allframe-core/src/application/resilience.rs

/// Orchestrates resilience policies across infrastructure implementations
#[async_trait::async_trait]
pub trait ResilienceOrchestrator: Send + Sync {
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        operation: F,
    ) -> Result<T, ResilienceError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: Future<Output = Result<T, E>> + Send,
        E: Into<ResilienceError> + Send;

    fn get_circuit_breaker(&self, name: &str) -> Option<&CircuitBreaker>;
    fn get_rate_limiter(&self, name: &str) -> Option<&RateLimiter>;
}

/// Default implementation using infrastructure layer
pub struct DefaultResilienceOrchestrator {
    retry_executor: Arc<RetryExecutor>,
    circuit_breakers: HashMap<String, CircuitBreaker>,
    rate_limiters: HashMap<String, RateLimiter>,
}
```

### 3. Infrastructure Layer Implementation (Week 5-6)

**Refactor existing infrastructure to implement orchestrator:**

```rust
// crates/allframe-core/src/infrastructure/resilience.rs

impl ResilienceOrchestrator for DefaultResilienceOrchestrator {
    async fn execute_with_policy<T, F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        operation: F,
    ) -> Result<T, ResilienceError>
    where
        // ... trait bounds
    {
        match policy {
            ResiliencePolicy::None => operation().await.map_err(Into::into),
            ResiliencePolicy::Retry { max_attempts, backoff } => {
                self.retry_executor
                    .execute_with_config(
                        RetryConfig::new(max_attempts).with_backoff(backoff),
                        operation,
                    )
                    .await
            }
            ResiliencePolicy::CircuitBreaker { failure_threshold, recovery_timeout } => {
                let cb = self.get_or_create_circuit_breaker("default", failure_threshold, recovery_timeout);
                cb.call(operation).await
            }
            // ... other policy implementations
        }
    }
}
```

### 4. Macro Backward Compatibility (Week 7-8)

**Create new macros that use architectural pattern:**

```rust
// crates/allframe-macros/src/resilience.rs

/// New architectural macro - injects orchestration at application layer
#[proc_macro_attribute]
pub fn resilient(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse policy from attribute
    // Generate code that uses ResilienceOrchestrator
    // Maintains domain layer purity
}

/// Keep old macros for backward compatibility with deprecation warnings
#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Add deprecation warning
    // Delegate to new resilient macro with retry policy
}
```

---

## Testing Strategy

### 1. Architectural Compliance Tests
- **Domain Layer Isolation**: Ensure domain layer has no infrastructure dependencies
- **Dependency Direction**: Verify dependencies only flow inward
- **Mock Infrastructure**: Test domain logic with mocked infrastructure

### 2. Backward Compatibility Tests
- **Existing Code**: All existing `#[retry]` usage continues to work
- **Deprecation Warnings**: Old macros show warnings pointing to new patterns
- **Performance**: New architecture doesn't degrade performance

### 3. Integration Tests
- **End-to-End**: Full request flow with resilience policies
- **Failure Scenarios**: Circuit breaker opens, retries exhausted
- **Policy Configuration**: Runtime policy changes work correctly

---

## Migration Guide

### For Existing Users

**Immediate (No Changes Required):**
```rust
// This continues to work unchanged
#[retry(max_retries = 3)]
async fn my_function() -> Result<(), Error> { /* ... */ }
```

**Recommended (Architecturally Clean):**
```rust
// New pattern - resilience declared at domain layer
#[derive(ResiliencePolicy)]
struct MyPolicy {
    retry: RetryPolicy,
    circuit_breaker: CircuitBreakerPolicy,
}

impl ResilientOperation for MyOperation {
    fn resilience_policy(&self) -> ResiliencePolicy {
        ResiliencePolicy::Retry { max_attempts: 3 }
    }
}

// Application layer orchestration
let result = orchestrator.execute_with_policy(
    operation.resilience_policy(),
    || operation.execute()
).await;
```

---

## Success Metrics

### 1. Architectural Compliance
- ✅ Domain layer has zero infrastructure dependencies
- ✅ Dependencies flow inward only
- ✅ Infrastructure can be swapped without domain changes

### 2. Performance
- ✅ No performance degradation (<5% overhead)
- ✅ Memory usage remains stable
- ✅ Compilation time impact minimal

### 3. Developer Experience
- ✅ Backward compatibility maintained
- ✅ Clear migration path provided
- ✅ Better error messages and debugging

### 4. Testability
- ✅ Domain logic can be tested without infrastructure
- ✅ Infrastructure can be tested in isolation
- ✅ Integration tests cover full resilience flows

---

## Risk Assessment

### High Risk
- **Breaking Changes**: Must maintain 100% backward compatibility
- **Performance Impact**: New abstraction layer could add overhead
- **Complexity**: More layers increase cognitive load

### Mitigation
- **Gradual Migration**: Old macros continue working with warnings
- **Performance Benchmarking**: Measure and optimize abstraction overhead
- **Documentation**: Comprehensive guides for new patterns

---

## Timeline

- **Week 1-2**: Domain layer contracts ✅
- **Week 3-4**: Application layer orchestration 🚧
- **Week 5-6**: Infrastructure implementation ✅
- **Week 7-8**: Macro refactoring and testing
- **Week 9-10**: Integration testing and documentation
- **Week 11-12**: Performance optimization and final validation

**Target Completion**: End of Q1 2025
**Risk Level**: Medium (architectural change with backward compatibility requirements)

---

**Status**: Phase 2 in progress - Application layer orchestration implementation.
