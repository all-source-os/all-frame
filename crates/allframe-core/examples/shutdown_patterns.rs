//! Shutdown Patterns Example
//!
//! Shows common patterns for graceful shutdown in different scenarios.
//!
//! Run with: cargo run --example shutdown_patterns

use std::{sync::Arc, time::Duration};

use allframe_core::shutdown::{
    GracefulShutdown, GracefulShutdownExt, ShutdownAwareTaskSpawner, ShutdownExt,
};

#[tokio::main]
async fn main() {
    println!("=== Shutdown Patterns ===\n");

    // Pattern 1: Basic task spawning with automatic cancellation
    pattern_basic_spawner().await;

    // Pattern 2: Tasks that return results
    pattern_with_results().await;

    // Pattern 3: Cleanup with error handling
    pattern_cleanup().await;

    // Pattern 4: Using ShutdownExt trait on futures
    pattern_shutdown_ext().await;

    // Pattern 5: Multiple workers with shared spawner
    pattern_multiple_workers().await;

    println!("\n=== All patterns demonstrated ===");
}

/// Pattern 1: Basic task spawning
///
/// Use `ShutdownAwareTaskSpawner` to spawn tasks that automatically
/// cancel when shutdown is triggered.
async fn pattern_basic_spawner() {
    println!("--- Pattern 1: Basic Task Spawning ---\n");

    let shutdown = Arc::new(GracefulShutdown::new());
    let spawner = ShutdownAwareTaskSpawner::new(shutdown.clone());

    // Spawn a task using a closure that returns a future
    let handle = spawner.spawn("worker", || async {
        println!("  Worker started");
        tokio::time::sleep(Duration::from_millis(50)).await;
        println!("  Worker completed");
    });

    // Wait for task to complete
    handle.await.unwrap();
    println!();
}

/// Pattern 2: Tasks with results
///
/// Use `spawn_with_result` when you need the task's return value.
/// Returns `None` if cancelled before completion.
async fn pattern_with_results() {
    println!("--- Pattern 2: Tasks with Results ---\n");

    let shutdown = Arc::new(GracefulShutdown::new());
    let spawner = ShutdownAwareTaskSpawner::new(shutdown.clone());

    // Task that completes normally
    let handle = spawner.spawn_with_result("compute", || async {
        tokio::time::sleep(Duration::from_millis(50)).await;
        42
    });

    match handle.await.unwrap() {
        Some(result) => println!("  Completed with result: {}", result),
        None => println!("  Task was cancelled"),
    }

    // Task that gets cancelled
    let handle = spawner.spawn_with_result("long_compute", || async {
        tokio::time::sleep(Duration::from_secs(60)).await;
        999
    });

    // Cancel immediately
    shutdown.shutdown();

    match handle.await.unwrap() {
        Some(result) => println!("  Completed with result: {}", result),
        None => println!("  Task was cancelled (as expected)"),
    }
    println!();
}

/// Pattern 3: Cleanup with GracefulShutdownExt
///
/// Use `perform_shutdown` to run cleanup logic with error logging.
async fn pattern_cleanup() {
    println!("--- Pattern 3: Cleanup with Error Handling ---\n");

    let shutdown = GracefulShutdown::new();

    // Successful cleanup
    let result: Result<(), &str> = shutdown
        .perform_shutdown(|| async {
            println!("  Running cleanup...");
            tokio::time::sleep(Duration::from_millis(50)).await;
            println!("  Cleanup done");
            Ok(())
        })
        .await;
    println!("  Result: {:?}", result);

    // Cleanup with error (error is logged and returned)
    let result: Result<(), &str> = shutdown
        .perform_shutdown(|| async {
            println!("  Running cleanup that fails...");
            Err("connection timeout")
        })
        .await;
    println!("  Result: {:?}", result);
    println!();
}

/// Pattern 4: Using ShutdownExt and run_until_shutdown
///
/// Futures can be made cancellable using `with_shutdown()` trait method
/// or `run_until_shutdown()` on GracefulShutdown.
async fn pattern_shutdown_ext() {
    println!("--- Pattern 4: Cancellable Futures ---\n");

    // Using ShutdownExt trait - future completes normally
    let shutdown = GracefulShutdown::new();
    let result = async { "completed" }.with_shutdown(&shutdown).await;
    println!("  with_shutdown (completes): {:?}", result);

    // Using run_until_shutdown - future completes normally
    let shutdown = GracefulShutdown::new();
    let result = shutdown.run_until_shutdown(async { "done" }).await;
    println!("  run_until_shutdown (completes): {:?}", result);

    // Using run_until_shutdown with concurrent shutdown
    let shutdown = Arc::new(GracefulShutdown::new());
    let shutdown_clone = shutdown.clone();

    // Spawn shutdown trigger
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(50)).await;
        shutdown_clone.shutdown();
    });

    // This future will be cancelled mid-flight
    let result = shutdown
        .run_until_shutdown(async {
            tokio::time::sleep(Duration::from_millis(200)).await;
            "never reached"
        })
        .await;
    println!("  run_until_shutdown (cancelled): {:?}", result);
    println!();
}

/// Pattern 5: Multiple workers with shared spawner
///
/// Clone the spawner to share it across components.
async fn pattern_multiple_workers() {
    println!("--- Pattern 5: Multiple Workers ---\n");

    let shutdown = Arc::new(GracefulShutdown::new());
    let spawner = ShutdownAwareTaskSpawner::new(shutdown.clone());

    // Spawn multiple workers
    let mut handles = vec![];
    for i in 1..=3 {
        let spawner = spawner.clone(); // Clone spawner for each worker
        let handle = spawner.spawn(&format!("worker_{}", i), move || async move {
            println!("  Worker {} started", i);
            tokio::time::sleep(Duration::from_millis(50 * i as u64)).await;
            println!("  Worker {} done", i);
        });
        handles.push(handle);
    }

    // Wait for all workers
    for handle in handles {
        handle.await.unwrap();
    }
    println!();
}
