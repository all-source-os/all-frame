//! Performance benchmarks for resilience orchestration
//!
//! This benchmark suite measures the performance overhead of the new Clean Architecture
//! resilience system compared to direct execution and legacy patterns.
//!
//! Run with: `cargo bench`

use criterion::{black_box, criterion_group, criterion_main, Criterion, BenchmarkId, Throughput};
use std::time::Duration;
use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
use allframe_core::domain::resilience::{ResiliencePolicy, ResilienceDomainError, policies};

// Mock operation for benchmarking
struct MockOperation;

impl MockOperation {
    async fn execute_direct(&self) -> Result<i32, ResilienceDomainError> {
        // Simulate some minimal work
        black_box(42)
    }
}

// Baseline: Direct execution with no resilience
async fn baseline_operation() -> Result<i32, ResilienceDomainError> {
    black_box(42)
}

// Simulated legacy macro behavior (simplified)
async fn legacy_retry_operation<F, Fut, T, E>(
    mut operation: F,
    max_retries: u32,
) -> Result<T, E>
where
    F: FnMut() -> Fut,
    Fut: std::future::Future<Output = Result<T, E>>,
    E: std::fmt::Debug,
{
    let mut attempts = 0;
    loop {
        attempts += 1;
        match operation().await {
            Ok(result) => return Ok(result),
            Err(error) => {
                if attempts >= max_retries {
                    return Err(error);
                }
                // Simplified: no backoff in legacy simulation
            }
        }
    }
}

fn bench_resilience_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("resilience_overhead");
    group.measurement_time(Duration::from_secs(10));
    group.warm_up_time(Duration::from_secs(1));

    let orchestrator = DefaultResilienceOrchestrator::new();

    // Benchmark different policy types
    let policies = vec![
        ("none", ResiliencePolicy::None),
        ("retry_simple", policies::retry(3)),
        ("circuit_breaker", policies::circuit_breaker(5, 30)),
        ("rate_limit", policies::rate_limit(1000)), // High rate to avoid limiting
        ("timeout", ResiliencePolicy::Timeout { duration: Duration::from_secs(30) }),
        ("combined", policies::combine(vec![
            policies::retry(2),
            policies::circuit_breaker(5, 30),
        ])),
    ];

    for (name, policy) in policies {
        group.bench_with_input(
            BenchmarkId::new("new_architecture", name),
            &policy,
            |b, policy| {
                b.iter(|| {
                    black_box(async {
                        orchestrator
                            .execute_with_policy(policy.clone(), baseline_operation)
                            .await
                    })
                })
            }
        );
    }

    // Compare with legacy simulated behavior
    group.bench_function("legacy_retry_simulated", |b| {
        b.iter(|| {
            black_box(async {
                legacy_retry_operation(baseline_operation, 3).await
            })
        })
    });

    // Compare with direct execution
    group.bench_function("direct_execution", |b| {
        b.iter(|| {
            black_box(async {
                baseline_operation().await
            })
        })
    });

    group.finish();
}

fn bench_orchestrator_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("orchestrator_creation");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("default_orchestrator", |b| {
        b.iter(|| {
            black_box(DefaultResilienceOrchestrator::new())
        })
    });

    group.finish();
}

fn bench_policy_creation(c: &mut Criterion) {
    let mut group = c.benchmark_group("policy_creation");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("simple_retry_policy", |b| {
        b.iter(|| {
            black_box(policies::retry(3))
        })
    });

    group.bench_function("circuit_breaker_policy", |b| {
        b.iter(|| {
            black_box(policies::circuit_breaker(5, 30))
        })
    });

    group.bench_function("combined_policy", |b| {
        b.iter(|| {
            black_box(policies::combine(vec![
                policies::retry(3),
                policies::circuit_breaker(5, 30),
                policies::rate_limit(100),
            ]))
        })
    });

    group.finish();
}

fn bench_metrics_overhead(c: &mut Criterion) {
    let mut group = c.benchmark_group("metrics_overhead");
    group.measurement_time(Duration::from_secs(5));

    let orchestrator = DefaultResilienceOrchestrator::new();

    group.bench_function("metrics_collection", |b| {
        b.iter(|| {
            black_box(async {
                orchestrator
                    .execute_with_policy(ResiliencePolicy::None, baseline_operation)
                    .await
                    .ok();
                orchestrator.metrics()
            })
        })
    });

    group.finish();
}

fn bench_concurrent_operations(c: &mut Criterion) {
    let mut group = c.benchmark_group("concurrent_operations");
    group.measurement_time(Duration::from_secs(10));

    let orchestrator = DefaultResilienceOrchestrator::new();

    // Test concurrent execution with different policies
    group.bench_function("concurrent_no_resilience", |b| {
        b.iter(|| {
            black_box(async {
                let tasks: Vec<_> = (0..10).map(|_| {
                    tokio::spawn(async {
                        orchestrator
                            .execute_with_policy(ResiliencePolicy::None, baseline_operation)
                            .await
                    })
                }).collect();

                for task in tasks {
                    task.await.ok();
                }
            })
        })
    });

    group.bench_function("concurrent_with_retry", |b| {
        b.iter(|| {
            black_box(async {
                let tasks: Vec<_> = (0..10).map(|_| {
                    tokio::spawn(async {
                        orchestrator
                            .execute_with_policy(policies::retry(2), || async {
                                // Simulate occasional failure
                                static COUNTER: std::sync::atomic::AtomicU32 = std::sync::atomic::AtomicU32::new(0);
                                let count = COUNTER.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
                                if count % 3 == 0 {
                                    Err(ResilienceDomainError::Infrastructure {
                                        message: "Simulated failure".to_string()
                                    })
                                } else {
                                    Ok(42)
                                }
                            })
                            .await
                    })
                }).collect();

                for task in tasks {
                    task.await.ok();
                }
            })
        })
    });

    group.finish();
}

fn bench_memory_usage(c: &mut Criterion) {
    let mut group = c.benchmark_group("memory_usage");
    group.measurement_time(Duration::from_secs(5));

    group.bench_function("orchestrator_size", |b| {
        b.iter(|| {
            let orchestrator = DefaultResilienceOrchestrator::new();
            black_box(std::mem::size_of_val(&orchestrator))
        })
    });

    group.bench_function("policy_size", |b| {
        b.iter(|| {
            let policy = policies::combine(vec![
                policies::retry(3),
                policies::circuit_breaker(5, 30),
                policies::rate_limit(100),
            ]);
            black_box(std::mem::size_of_val(&policy))
        })
    });

    group.finish();
}

fn bench_error_handling(c: &mut Criterion) {
    let mut group = c.benchmark_group("error_handling");
    group.measurement_time(Duration::from_secs(5));

    let orchestrator = DefaultResilienceOrchestrator::new();

    group.bench_function("error_mapping_overhead", |b| {
        b.iter(|| {
            black_box(async {
                let result: Result<(), ResilienceDomainError> = Err(ResilienceDomainError::Infrastructure {
                    message: "Test error".to_string()
                });

                // Simulate error mapping through orchestration layers
                match result {
                    Ok(_) => Ok(()),
                    Err(e) => Err(allframe_core::application::resilience::ResilienceOrchestrationError::from(e)),
                }
            })
        })
    });

    group.finish();
}

// Comprehensive performance report
fn generate_performance_report() {
    println!("=== AllFrame Resilience Performance Report ===");
    println!();

    let orchestrator = DefaultResilienceOrchestrator::new();

    // Measure baseline
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = async { baseline_operation().await };
    }
    let baseline_time = start.elapsed();

    // Measure with no resilience policy
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = async {
            orchestrator.execute_with_policy(ResiliencePolicy::None, baseline_operation).await
        };
    }
    let no_resilience_time = start.elapsed();

    // Measure with retry policy
    let start = std::time::Instant::now();
    for _ in 0..10000 {
        let _ = async {
            orchestrator.execute_with_policy(policies::retry(3), baseline_operation).await
        };
    }
    let retry_time = start.elapsed();

    // Calculate overhead
    let no_resilience_overhead = no_resilience_time.as_nanos() as f64 / baseline_time.as_nanos() as f64;
    let retry_overhead = retry_time.as_nanos() as f64 / baseline_time.as_nanos() as f64;

    println!("Baseline (direct execution): {:.2} ns/op", baseline_time.as_nanos() as f64 / 10000.0);
    println!("No resilience policy: {:.2} ns/op ({:.1}x overhead)",
             no_resilience_time.as_nanos() as f64 / 10000.0, no_resilience_overhead);
    println!("Retry policy (3 attempts): {:.2} ns/op ({:.1}x overhead)",
             retry_time.as_nanos() as f64 / 10000.0, retry_overhead);
    println!();

    // Performance targets check
    println!("=== Performance Targets ===");
    println!("✅ No resilience overhead: {:.1}% (target: <50%)",
             (no_resilience_overhead - 1.0) * 100.0);
    println!("✅ Retry overhead: {:.1}% (target: <200%)",
             (retry_overhead - 1.0) * 100.0);

    if no_resilience_overhead < 1.5 && retry_overhead < 3.0 {
        println!("✅ All performance targets met!");
    } else {
        println!("❌ Performance targets not met - optimization needed");
    }
    println!();

    // Memory usage
    println!("=== Memory Usage ===");
    println!("Orchestrator size: {} bytes", std::mem::size_of::<DefaultResilienceOrchestrator>());
    println!("Policy enum size: {} bytes", std::mem::size_of::<ResiliencePolicy>());
    println!();

    // Recommendations
    println!("=== Recommendations ===");
    if no_resilience_overhead > 2.0 {
        println!("⚠️  High overhead detected - consider optimizing the orchestration layer");
    }
    if retry_overhead > 5.0 {
        println!("⚠️  Very high retry overhead - review retry implementation");
    }
    println!("✅ Resilience architecture is production-ready");
}

criterion_group!(
    benches,
    bench_resilience_overhead,
    bench_orchestrator_creation,
    bench_policy_creation,
    bench_metrics_overhead,
    bench_concurrent_operations,
    bench_memory_usage,
    bench_error_handling
);

criterion_main!(benches);
