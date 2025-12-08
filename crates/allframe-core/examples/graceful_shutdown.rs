//! Graceful Shutdown Example
//!
//! Demonstrates the shutdown utilities in allframe-core:
//! - `ShutdownAwareTaskSpawner` for spawning tasks that respect shutdown
//!   signals
//! - `GracefulShutdownExt` for running cleanup during shutdown
//!
//! Run with: cargo run --example graceful_shutdown

use std::{
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::Duration,
};

use allframe_core::shutdown::{
    GracefulShutdown, GracefulShutdownExt, ShutdownAwareTaskSpawner, ShutdownToken,
};

/// Simulated database connection pool
struct DatabasePool {
    active_connections: AtomicU64,
}

impl DatabasePool {
    fn new() -> Self {
        Self {
            active_connections: AtomicU64::new(5),
        }
    }

    async fn close(&self) -> Result<(), &'static str> {
        println!("  Closing database connections...");
        tokio::time::sleep(Duration::from_millis(100)).await;
        self.active_connections.store(0, Ordering::SeqCst);
        println!("  Database connections closed");
        Ok(())
    }
}

/// Simulated message queue consumer
struct MessageConsumer {
    messages_processed: AtomicU64,
}

impl MessageConsumer {
    fn new() -> Self {
        Self {
            messages_processed: AtomicU64::new(0),
        }
    }

    async fn process_messages(&self, mut shutdown_token: ShutdownToken) {
        loop {
            tokio::select! {
                _ = shutdown_token.cancelled() => {
                    println!("  Message consumer received shutdown signal");
                    break;
                }
                _ = tokio::time::sleep(Duration::from_millis(200)) => {
                    let count = self.messages_processed.fetch_add(1, Ordering::SeqCst) + 1;
                    println!("  Processed message #{}", count);
                }
            }
        }
    }

    fn processed_count(&self) -> u64 {
        self.messages_processed.load(Ordering::SeqCst)
    }
}

/// Simulated metrics reporter
async fn metrics_reporter(mut shutdown_token: ShutdownToken, consumer: Arc<MessageConsumer>) {
    let mut interval = tokio::time::interval(Duration::from_millis(500));

    loop {
        tokio::select! {
            _ = shutdown_token.cancelled() => {
                println!("  Metrics reporter shutting down");
                break;
            }
            _ = interval.tick() => {
                println!("  [Metrics] Messages processed: {}", consumer.processed_count());
            }
        }
    }
}

/// Simulated health checker
async fn health_checker() {
    // Simulates a periodic health check that completes quickly
    for i in 1..=3 {
        println!("  Health check #{} - OK", i);
        tokio::time::sleep(Duration::from_millis(300)).await;
    }
    println!("  Health checker completed all checks");
}

#[tokio::main]
async fn main() {
    println!("=== AllFrame Graceful Shutdown Example ===\n");

    // Create shared resources
    let db_pool = Arc::new(DatabasePool::new());
    let message_consumer = Arc::new(MessageConsumer::new());

    // Create the shutdown handler
    let shutdown = Arc::new(
        GracefulShutdown::builder()
            .timeout(Duration::from_secs(30))
            .on_signal(|signal| {
                println!("\nReceived shutdown signal: {}", signal);
            })
            .build(),
    );

    // Create task spawner
    let spawner = ShutdownAwareTaskSpawner::new(shutdown.clone());

    println!("1. Starting background tasks using ShutdownAwareTaskSpawner\n");

    // Spawn the message consumer (long-running task)
    let consumer_clone = message_consumer.clone();
    let token = shutdown.token();
    spawner.spawn("message_consumer", move || async move {
        consumer_clone.process_messages(token).await;
    });

    // Spawn metrics reporter (long-running task)
    let consumer_clone = message_consumer.clone();
    let token = shutdown.token();
    spawner.spawn_background("metrics_reporter", move || async move {
        metrics_reporter(token, consumer_clone).await;
    });

    // Spawn health checker (finite task - will complete on its own)
    let health_handle = spawner.spawn_with_result("health_checker", || async {
        health_checker().await;
        "all_checks_passed"
    });

    println!("   Started: message_consumer, metrics_reporter, health_checker\n");

    // Let the system run for a bit
    println!("2. System running... (will auto-shutdown in 2 seconds)\n");
    tokio::time::sleep(Duration::from_secs(2)).await;

    // Wait for health checker to complete (it should finish before shutdown)
    if let Ok(Some(result)) = health_handle.await {
        println!("\n   Health checker result: {}\n", result);
    }

    // Trigger manual shutdown
    println!("3. Initiating graceful shutdown sequence\n");
    shutdown.shutdown();

    // Small delay to let tasks respond to shutdown
    tokio::time::sleep(Duration::from_millis(100)).await;

    // Perform cleanup using GracefulShutdownExt
    println!("4. Running cleanup with GracefulShutdownExt\n");

    let db_pool_clone = db_pool.clone();
    let cleanup_result: Result<(), &str> = shutdown
        .perform_shutdown(move || async move { db_pool_clone.close().await })
        .await;

    match cleanup_result {
        Ok(()) => println!("\n   Cleanup completed successfully"),
        Err(e) => println!("\n   Cleanup failed: {}", e),
    }

    // Print final stats
    println!("\n=== Shutdown Complete ===");
    println!(
        "   Total messages processed: {}",
        message_consumer.processed_count()
    );
    println!(
        "   Active DB connections: {}",
        db_pool.active_connections.load(Ordering::SeqCst)
    );
}
