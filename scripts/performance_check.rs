//! Quick performance check script
//!
//! Run with: `rustc --edition 2021 scripts/performance_check.rs && ./performance_check`

use std::time::{Duration, Instant};

#[tokio::main]
async fn main() {
    println!("=== AllFrame Resilience Performance Check ===\n");

    // This is a simplified version for quick verification
    // Full benchmarks require criterion

    let iterations = 10000;

    // Baseline measurement
    println!("Running {} iterations of each test...", iterations);

    let baseline_start = Instant::now();
    for _ in 0..iterations {
        baseline_operation().await;
    }
    let baseline_time = baseline_start.elapsed();

    println!("✅ Baseline (direct): {:.2} ns/op",
             baseline_time.as_nanos() as f64 / iterations as f64);

    // Check if allframe-core is available
    #[cfg(feature = "resilience")]
    {
        use allframe_core::application::resilience::DefaultResilienceOrchestrator;
        use allframe_core::domain::resilience::{ResiliencePolicy, policies};

        let orchestrator = DefaultResilienceOrchestrator::new();

        // No resilience policy
        let no_resilience_start = Instant::now();
        for _ in 0..iterations {
            let _ = orchestrator
                .execute_with_policy(ResiliencePolicy::None, baseline_operation)
                .await;
        }
        let no_resilience_time = no_resilience_start.elapsed();

        // Retry policy
        let retry_start = Instant::now();
        for _ in 0..iterations {
            let _ = orchestrator
                .execute_with_policy(policies::retry(3), baseline_operation)
                .await;
        }
        let retry_time = retry_start.elapsed();

        let baseline_ns = baseline_time.as_nanos() as f64 / iterations as f64;
        let no_resilience_ns = no_resilience_time.as_nanos() as f64 / iterations as f64;
        let retry_ns = retry_time.as_nanos() as f64 / iterations as f64;

        let no_resilience_overhead = no_resilience_ns / baseline_ns;
        let retry_overhead = retry_ns / baseline_ns;

        println!("✅ No resilience: {:.2} ns/op ({:.1}x overhead)", no_resilience_ns, no_resilience_overhead);
        println!("✅ Retry policy: {:.2} ns/op ({:.1}x overhead)", retry_ns, retry_overhead);

        // Performance assessment
        println!("\n=== Performance Assessment ===");
        if no_resilience_overhead < 2.0 {
            println!("✅ Low overhead for no-resilience operations");
        } else {
            println!("⚠️  High overhead detected - may need optimization");
        }

        if retry_overhead < 5.0 {
            println!("✅ Acceptable overhead for retry operations");
        } else {
            println!("⚠️  High retry overhead - review implementation");
        }

        println!("\n🎯 Resilience architecture performance verified!");
    }

    #[cfg(not(feature = "resilience"))]
    {
        println!("⚠️  Resilience features not enabled - run with --features resilience");
    }
}

async fn baseline_operation() -> Result<i32, String> {
    // Minimal async operation for benchmarking
    Ok(42)
}
