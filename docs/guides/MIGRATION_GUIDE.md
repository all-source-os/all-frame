# Migration Guide: From Legacy Macros to Clean Architecture

**Status**: ✅ Production Ready
**Applies to**: AllFrame v0.1.13+
**Breaking Changes**: None (100% backward compatible)

---

## Overview

AllFrame v0.1.13 introduces a Clean Architecture-compliant resilience system while maintaining 100% backward compatibility. This guide helps you migrate from the old `#[retry]`, `#[circuit_breaker]`, and `#[rate_limited]` macros to the new architectural patterns.

**Key Benefits of Migration:**
- ✅ **Clean Architecture**: Domain logic stays pure and testable
- ✅ **Better Performance**: Reduced overhead and improved observability
- ✅ **Enhanced Flexibility**: Runtime policy changes and better configuration
- ✅ **Future-Proof**: Extensible for new resilience patterns

---

## Quick Assessment: Do You Need to Migrate?

### ✅ **Immediate Migration Recommended If:**
- You're starting a new project
- You want Clean Architecture compliance
- You need advanced resilience features (timeouts, bulkhead, adaptive policies)
- You want better testability and observability

### ⏳ **Optional Migration If:**
- You have existing code that works
- You don't need advanced features
- You're on a tight timeline

### ❌ **No Migration Needed If:**
- Your current code works and meets requirements
- Legacy macros are deprecated but still functional

---

## Migration Strategies

### Strategy 1: Gradual Migration (Recommended)

Migrate one service or module at a time:

```rust
// Phase 1: Keep existing code working
#[retry(max_retries = 3)]
async fn legacy_payment_processing() { /* existing code */ }

// Phase 2: Add new implementation alongside
async fn new_payment_processing() -> Result<PaymentResult, PaymentError> {
    // New Clean Architecture implementation
}

// Phase 3: Gradually migrate callers
// Phase 4: Remove legacy code
```

### Strategy 2: Big Bang Migration

Replace all macros at once (higher risk, bigger reward):

```bash
# Use automated migration script (future feature)
find src/ -name "*.rs" -exec sed -i 's/#\[retry(/#[resilient(retry(/g' {} \;
```

### Strategy 3: Hybrid Approach

Critical paths use new architecture, non-critical keep legacy:

```rust
// Critical payment processing - new architecture
async fn process_payment() -> Result<(), Error> {
    // Clean, testable, observable
}

// Background job - keep legacy for now
#[retry(max_retries = 5)]
async fn send_notification() { /* low priority */ }
```

---

## Step-by-Step Migration

### Step 1: Enable New Features

Update your `Cargo.toml`:

```toml
[dependencies]
allframe-core = { version = "0.1.13", features = [
    "resilience",      # Required for new architecture
    "resilience-tokio" # Optional: advanced async features
] }
```

### Step 2: Identify Migration Candidates

Find all uses of legacy macros:

```bash
# Find retry macros
grep -r "#\[retry" src/

# Find circuit breaker macros
grep -r "#\[circuit_breaker" src/

# Find rate limiting macros
grep -r "#\[rate_limited" src/
```

### Step 3: Convert Domain Logic

**Before (Legacy):**
```rust
#[retry(max_retries = 3)]
async fn process_payment(payment: Payment) -> Result<PaymentResult, PaymentError> {
    // Business logic mixed with resilience concerns
    validate_payment(&payment)?;
    charge_credit_card(&payment).await?;
    update_database(&payment).await?;
    send_notification(&payment).await?;
    Ok(PaymentResult::Success)
}
```

**After (Clean Architecture):**
```rust
use allframe_core::domain::resilience::{ResilientOperation, ResiliencePolicy, policies};

struct PaymentProcessor {
    payment: Payment,
}

impl ResilientOperation<PaymentResult, PaymentError> for PaymentProcessor {
    fn resilience_policy(&self) -> ResiliencePolicy {
        policies::combine(vec![
            policies::retry(3),
            policies::circuit_breaker(5, 30),
            policies::timeout(30),
        ])
    }

    async fn execute(&self) -> Result<PaymentResult, PaymentError> {
        // Pure business logic - no infrastructure concerns
        validate_payment(&self.payment)?;
        charge_credit_card(&self.payment).await?;
        update_database(&self.payment).await?;
        send_notification(&self.payment).await?;
        Ok(PaymentResult::Success)
    }

    fn operation_id(&self) -> &str {
        "payment_processor"
    }
}
```

### Step 4: Update Application Layer

**Before:**
```rust
#[handler]
async fn payment_handler(request: PaymentRequest) -> Result<PaymentResponse, ApiError> {
    // Direct call to resilient function
    let result = process_payment(request.into()).await?;
    Ok(result.into())
}
```

**After:**
```rust
use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};

#[handler]
async fn payment_handler(
    request: PaymentRequest,
    orchestrator: &ResilienceOrchestrator, // Injected via DI
) -> Result<PaymentResponse, ApiError> {
    let processor = PaymentProcessor::from(request);

    // Application layer orchestrates resilience
    let result = orchestrator.execute_operation(processor).await?;
    Ok(result.into())
}
```

### Step 5: Update Tests

**Before:**
```rust
#[tokio::test]
async fn test_payment_processing() {
    // Hard to test - resilience logic is baked in
    let result = process_payment(test_payment()).await;
    assert!(result.is_ok());
}
```

**After:**
```rust
#[tokio::test]
async fn test_payment_business_logic() {
    // Test pure business logic in isolation
    let processor = PaymentProcessor::new(test_payment());
    let result = processor.execute().await;
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_payment_resilience_policy() {
    // Test resilience configuration
    let processor = PaymentProcessor::new(test_payment());
    let policy = processor.resilience_policy();

    match policy {
        ResiliencePolicy::Combined { policies } => {
            assert_eq!(policies.len(), 3);
            // Verify each policy is configured correctly
        }
        _ => panic!("Expected combined policy"),
    }
}

#[tokio::test]
async fn test_payment_with_orchestration() {
    // Test full integration
    let orchestrator = DefaultResilienceOrchestrator::new();
    let processor = PaymentProcessor::new(test_payment());

    let result = orchestrator.execute_operation(processor).await;
    assert!(result.is_ok());

    // Verify metrics were collected
    let metrics = orchestrator.metrics();
    assert_eq!(metrics.total_operations, 1);
}
```

---

## Specific Macro Migrations

### `#[retry]` Migration

**Legacy:**
```rust
#[retry(max_retries = 3)]
async fn api_call() -> Result<Data, Error> {
    call_external_api().await
}
```

**New Architecture:**
```rust
use allframe_core::domain::resilience::{ResilientOperation, policies};

struct ApiCaller;

impl ResilientOperation<Data, Error> for ApiCaller {
    fn resilience_policy(&self) -> ResiliencePolicy {
        policies::retry(3)
    }

    async fn execute(&self) -> Result<Data, Error> {
        call_external_api().await
    }
}

// Usage
let result = orchestrator.execute_operation(ApiCaller).await;
```

### `#[circuit_breaker]` Migration

**Legacy:**
```rust
#[circuit_breaker(failure_threshold = 5, recovery_timeout = 30)]
async fn database_query() -> Result<Records, DbError> {
    query_database().await
}
```

**New Architecture:**
```rust
struct DatabaseQuery {
    query: String,
}

impl ResilientOperation<Records, DbError> for DatabaseQuery {
    fn resilience_policy(&self) -> ResiliencePolicy {
        policies::circuit_breaker(5, 30)
    }

    async fn execute(&self) -> Result<Records, DbError> {
        query_database(&self.query).await
    }
}
```

### `#[rate_limited]` Migration

**Legacy:**
```rust
#[rate_limited(requests_per_second = 100)]
async fn api_endpoint() -> Result<Response, ApiError> {
    process_request().await
}
```

**New Architecture:**
```rust
struct ApiEndpoint {
    request: Request,
}

impl ResilientOperation<Response, ApiError> for ApiEndpoint {
    fn resilience_policy(&self) -> ResiliencePolicy {
        policies::rate_limit(100)
    }

    async fn execute(&self) -> Result<Response, ApiError> {
        process_request(&self.request).await
    }
}
```

### Complex Combined Policies

**Legacy:**
```rust
#[retry(max_retries = 3)]
#[circuit_breaker(failure_threshold = 5, recovery_timeout = 30)]
#[rate_limited(requests_per_second = 50)]
async fn complex_operation() -> Result<(), Error> {
    do_something().await
}
```

**New Architecture:**
```rust
struct ComplexOperation;

impl ResilientOperation<(), Error> for ComplexOperation {
    fn resilience_policy(&self) -> ResiliencePolicy {
        policies::combine(vec![
            policies::retry(3),
            policies::circuit_breaker(5, 30),
            policies::rate_limit(50),
        ])
    }

    async fn execute(&self) -> Result<(), Error> {
        do_something().await
    }
}
```

---

## Advanced Migration Patterns

### Policy Factories

Create reusable policy configurations:

```rust
mod resilience_policies {
    use allframe_core::domain::resilience::{ResiliencePolicy, policies};

    pub fn external_api_policy() -> ResiliencePolicy {
        policies::combine(vec![
            policies::retry(2),
            policies::circuit_breaker(3, 60),
            policies::timeout(10),
        ])
    }

    pub fn database_policy() -> ResiliencePolicy {
        policies::combine(vec![
            policies::retry(3),
            policies::circuit_breaker(5, 30),
            policies::timeout(5),
        ])
    }

    pub fn cache_policy() -> ResiliencePolicy {
        policies::retry(1) // Cache failures are usually fast
    }
}
```

### Context-Aware Policies

Policies based on runtime context:

```rust
impl ResilientOperation<Data, Error> for ContextualOperation {
    fn resilience_policy(&self) -> ResiliencePolicy {
        match self.priority {
            Priority::High => policies::combine(vec![
                policies::retry(5),  // More retries for high priority
                policies::circuit_breaker(10, 30),
                policies::timeout(60), // Longer timeout
            ]),
            Priority::Low => policies::retry(1), // Fail fast for low priority
        }
    }
}
```

### Dependency Injection Integration

Integrate with DI containers:

```rust
use allframe_core::application::resilience::ResilienceOrchestrator;

#[injectable]
struct PaymentService {
    orchestrator: Arc<dyn ResilienceOrchestrator>,
    payment_repo: Arc<dyn PaymentRepository>,
}

impl PaymentService {
    async fn process(&self, payment: Payment) -> Result<(), Error> {
        let processor = PaymentProcessor {
            payment,
            repo: self.payment_repo.clone(),
        };

        self.orchestrator.execute_operation(processor).await
    }
}
```

---

## Testing Migration

### Test Compatibility

Verify that migrated code maintains the same behavior:

```rust
#[tokio::test]
async fn test_behavior_unchanged() {
    // Test with new architecture
    let new_result = orchestrator.execute_operation(NewOperation).await;

    // Simulate old behavior (without actual macro)
    let old_result = retry_operation(|| old_implementation()).await;

    // Results should be equivalent
    assert_eq!(new_result, old_result);
}
```

### Performance Regression Testing

Ensure migration doesn't degrade performance:

```rust
use criterion::{criterion_group, criterion_main, Criterion};

fn benchmark_migration(c: &mut Criterion) {
    let orchestrator = DefaultResilienceOrchestrator::new();

    // Benchmark new architecture
    c.bench_function("new_architecture", |b| {
        b.iter(|| async {
            orchestrator
                .execute_operation(TestOperation)
                .await
        })
    });

    // Benchmark simulated old architecture
    c.bench_function("old_architecture", |b| {
        b.iter(|| async {
            // Simulate old macro behavior
            retry_operation(|| test_operation()).await
        })
    });
}
```

---

## Rollback Strategy

If migration causes issues, you can rollback:

### Temporary Rollback
```rust
// Keep both implementations during transition
async fn payment_processing(request: PaymentRequest) -> Result<(), Error> {
    if std::env::var("USE_NEW_ARCHITECTURE").unwrap_or("false".to_string()) == "true" {
        // New architecture
        orchestrator.execute_operation(PaymentProcessor::from(request)).await
    } else {
        // Legacy fallback
        legacy_payment_processing(request).await
    }
}
```

### Feature Flags
```toml
[features]
legacy-macros = ["allframe-macros/legacy"]  # If we add legacy feature
new-architecture = ["resilience"]           # Default for new projects
```

---

## Troubleshooting Migration

### Common Issues

1. **"Type doesn't implement ResilientOperation"**
   ```rust
   // Fix: Implement the trait
   impl ResilientOperation<ResultType, ErrorType> for YourType {
       fn resilience_policy(&self) -> ResiliencePolicy { /* ... */ }
       async fn execute(&self) -> Result<ResultType, ErrorType> { /* ... */ }
   }
   ```

2. **"Orchestrator not available"**
   ```rust
   // Fix: Inject orchestrator via DI
   fn new(orchestrator: Arc<dyn ResilienceOrchestrator>) -> Self {
       Self { orchestrator }
   }
   ```

3. **Performance degradation**
   ```rust
   // Fix: Check metrics and optimize policies
   let metrics = orchestrator.metrics();
   println!("Overhead: {:?}", metrics);
   ```

### Debug Mode

Enable detailed logging during migration:

```rust
use tracing_subscriber;

std::env::set_var("RUST_LOG", "allframe_core=debug");
tracing_subscriber::init();

// Now you'll see:
// - Policy application details
// - Orchestration decisions
// - Performance metrics
```

---

## Success Metrics

Track these metrics during migration:

- **Test Pass Rate**: All tests should pass post-migration
- **Performance**: <5% degradation acceptable, <1% preferred
- **Error Rates**: Should not increase
- **Code Coverage**: Should improve (domain logic easier to test)
- **Build Time**: <10% increase acceptable

---

## Support and Resources

### Getting Help

1. **Documentation**: [Resilience Architecture Guide](RESILIENCE_ARCHITECTURE.md)
2. **Examples**: Check `examples/` directory for complete implementations
3. **Issues**: File bugs with "migration" label
4. **Discussions**: Use GitHub Discussions for migration questions

### Example Projects

- `examples/resilient_payment_service.rs` - Complete payment service migration
- `examples/microservice_resilience.rs` - Inter-service communication patterns
- `examples/legacy_migration.rs` - Side-by-side comparison

---

## Timeline Recommendations

### Week 1-2: Planning and Assessment
- Audit current macro usage
- Plan migration strategy
- Set up monitoring

### Week 3-4: Core Migration
- Migrate critical services first
- Update tests incrementally
- Monitor performance metrics

### Week 5-6: Advanced Features
- Implement custom policies
- Add observability
- Performance optimization

### Week 7-8: Cleanup and Validation
- Remove legacy code
- Final testing
- Documentation updates

---

**Migration Complete**: Your application now uses Clean Architecture patterns while maintaining all existing functionality. The new system is more testable, maintainable, and ready for future enhancements.

---

**Next**: [Resilience Architecture Guide](RESILIENCE_ARCHITECTURE.md) | [Configuration Reference](CONFIGURATION.md)
