//! Resilience Patterns Example
//!
//! This example demonstrates AllFrame's comprehensive resilience features:
//! - Retry with exponential backoff
//! - Circuit breaker for fail-fast behavior
//! - Rate limiting with token bucket
//!
//! Run with: cargo run --example resilience --features resilience

use std::{
    sync::{
        atomic::{AtomicU32, Ordering},
        Arc,
    },
    time::Duration,
};

use allframe_core::resilience::{
    AdaptiveRateLimiter, AdaptiveRetry, CircuitBreaker, CircuitBreakerConfig,
    CircuitBreakerManager, KeyedRateLimiter, RateLimiter, RetryBudget, RetryConfig, RetryExecutor,
};

#[tokio::main]
async fn main() {
    println!("=== AllFrame Resilience Patterns Demo ===\n");

    // 1. Retry with Exponential Backoff
    demo_retry().await;

    // 2. Retry Budget (prevents retry storms)
    demo_retry_budget();

    // 3. Adaptive Retry (adjusts based on success rate)
    demo_adaptive_retry();

    // 4. Rate Limiting
    demo_rate_limiting();

    // 5. Adaptive Rate Limiting
    demo_adaptive_rate_limiting();

    // 6. Keyed Rate Limiting (per-endpoint/per-user)
    demo_keyed_rate_limiting();

    // 7. Circuit Breaker
    demo_circuit_breaker().await;

    // 8. Circuit Breaker Manager
    demo_circuit_breaker_manager();

    println!("\n=== Demo Complete ===");
}

async fn demo_retry() {
    println!("--- 1. Retry with Exponential Backoff ---");

    let config = RetryConfig::new(3)
        .with_initial_interval(Duration::from_millis(100))
        .with_max_interval(Duration::from_secs(5))
        .with_multiplier(2.0)
        .with_randomization_factor(0.1);

    let executor = RetryExecutor::new(config);

    // Simulate an operation that fails twice then succeeds
    let attempt_count = Arc::new(AtomicU32::new(0));
    let attempt_clone = attempt_count.clone();

    let result = executor
        .execute("fetch_data", || {
            let attempts = attempt_clone.clone();
            async move {
                let current = attempts.fetch_add(1, Ordering::SeqCst) + 1;
                println!("  Attempt {}", current);

                if current < 3 {
                    Err(std::io::Error::new(
                        std::io::ErrorKind::ConnectionRefused,
                        "Service unavailable",
                    ))
                } else {
                    Ok("Data fetched successfully!")
                }
            }
        })
        .await;

    match result {
        Ok(data) => println!("  Success: {}", data),
        Err(e) => println!("  Failed after {} attempts: {}", e.attempts, e.last_error),
    }
    println!();
}

fn demo_retry_budget() {
    println!("--- 2. Retry Budget (Storm Prevention) ---");

    // 10 tokens, recover 2 per second
    let budget = RetryBudget::new(10, 2.0);

    println!("  Initial tokens: {}", budget.remaining());

    // Consume some tokens
    for i in 1..=5 {
        if budget.try_consume(2) {
            println!(
                "  Request {}: Consumed 2 tokens, remaining: {}",
                i,
                budget.remaining()
            );
        } else {
            println!("  Request {}: Denied - insufficient tokens", i);
        }
    }

    // Reset to demonstrate recovery
    budget.reset();
    println!("  After reset: {} tokens", budget.remaining());
    println!();
}

fn demo_adaptive_retry() {
    println!("--- 3. Adaptive Retry ---");

    let base_config = RetryConfig::new(5).with_initial_interval(Duration::from_millis(500));

    let adaptive = AdaptiveRetry::new(base_config);

    // Simulate mixed outcomes
    adaptive.record_outcome(true); // success
    adaptive.record_outcome(false); // failure
    adaptive.record_outcome(false); // failure
    adaptive.record_outcome(false); // failure

    println!("  Success rate: {:.1}%", adaptive.success_rate() * 100.0);

    let adjusted = adaptive.get_adjusted_config();
    println!(
        "  Adjusted config: max_retries={}, initial_interval={:?}",
        adjusted.max_retries, adjusted.initial_interval
    );
    println!("  (Lower success rate = more conservative retry behavior)");
    println!();
}

fn demo_rate_limiting() {
    println!("--- 4. Rate Limiting ---");

    // 10 requests per second, burst of 5
    let limiter = RateLimiter::new(10, 5);

    // Try burst of requests
    let mut allowed = 0;
    let mut denied = 0;

    for _ in 0..10 {
        if limiter.check().is_ok() {
            allowed += 1;
        } else {
            denied += 1;
        }
    }

    println!("  Allowed: {}, Denied: {}", allowed, denied);

    let status = limiter.get_status();
    println!(
        "  Status: max_rps={}, burst={}, is_limited={}",
        status.max_rps, status.burst_size, status.is_limited
    );
    println!();
}

fn demo_adaptive_rate_limiting() {
    println!("--- 5. Adaptive Rate Limiting ---");

    let limiter = AdaptiveRateLimiter::new(100, 10)
        .with_min_rps(10)
        .with_backoff_factor(0.5);

    println!("  Initial RPS: {}", limiter.get_status().current_rps);

    // Simulate receiving 429 from external service
    limiter.record_rate_limit();
    println!("  After 1st 429: {} RPS", limiter.get_status().current_rps);

    limiter.record_rate_limit();
    println!("  After 2nd 429: {} RPS", limiter.get_status().current_rps);

    // Simulate successful requests (would recover over time)
    limiter.record_success();
    println!("  (RPS recovers gradually after successful requests)");
    println!();
}

fn demo_keyed_rate_limiting() {
    println!("--- 6. Keyed Rate Limiting ---");

    // Different limits per endpoint
    let limiter: KeyedRateLimiter<&str> = KeyedRateLimiter::new(10, 5);

    // Set custom limit for premium endpoint
    limiter.set_limit("premium_api", 100, 20);

    // Check different keys
    println!("  Standard endpoint:");
    let mut allowed = 0;
    for _ in 0..8 {
        if limiter.check(&"standard_api").is_ok() {
            allowed += 1;
        }
    }
    println!("    Allowed {} of 8 requests", allowed);

    println!("  Premium endpoint:");
    let mut allowed = 0;
    for _ in 0..25 {
        if limiter.check(&"premium_api").is_ok() {
            allowed += 1;
        }
    }
    println!("    Allowed {} of 25 requests", allowed);
    println!();
}

async fn demo_circuit_breaker() {
    println!("--- 7. Circuit Breaker ---");

    let config = CircuitBreakerConfig::new(3)
        .with_success_threshold(2)
        .with_timeout(Duration::from_millis(500));

    let cb = CircuitBreaker::new("external_service", config);

    println!("  Initial state: {:?}", cb.get_state());

    // Simulate failures
    for i in 1..=4 {
        let result = cb
            .call(|| async {
                Err::<(), _>(std::io::Error::new(
                    std::io::ErrorKind::ConnectionRefused,
                    "Service down",
                ))
            })
            .await;

        println!(
            "  Call {}: {:?}, State: {:?}",
            i,
            result.is_ok(),
            cb.get_state()
        );
    }

    // Wait for timeout to transition to half-open
    println!("  Waiting for timeout...");
    tokio::time::sleep(Duration::from_millis(600)).await;
    println!("  State after timeout: {:?}", cb.get_state());

    // Successful calls close the circuit
    for i in 1..=2 {
        cb.record_success();
        println!("  Success {}: State: {:?}", i, cb.get_state());
    }

    let stats = cb.get_stats();
    println!(
        "  Final stats: successes={}, failures={}, rejected={}",
        stats.success_count, stats.failure_count, stats.rejected_count
    );
    println!();
}

fn demo_circuit_breaker_manager() {
    println!("--- 8. Circuit Breaker Manager ---");

    let manager = CircuitBreakerManager::new(CircuitBreakerConfig::default());

    // Get or create circuit breakers for different services
    let api_cb = manager.get_or_create("payment_api");
    let db_cb = manager.get_or_create("database");
    let cache_cb = manager.get_or_create("redis_cache");

    // Simulate some activity
    api_cb.record_success();
    db_cb.record_failure();
    cache_cb.record_success();
    cache_cb.record_success();

    // Get all stats
    println!("  Circuit breaker states:");
    for (name, stats) in manager.get_all_stats() {
        println!(
            "    {}: state={:?}, successes={}, failures={}",
            name, stats.state, stats.success_count, stats.failure_count
        );
    }

    println!("  Total circuit breakers: {}", manager.len());
}
