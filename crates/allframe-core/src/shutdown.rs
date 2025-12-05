//! Graceful shutdown utilities
//!
//! This module provides utilities for handling graceful shutdown of services,
//! including signal handling, timeout management, and task cancellation.
//!
//! # Example
//!
//! ```rust,no_run
//! use allframe_core::shutdown::{GracefulShutdown, ShutdownSignal};
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() {
//!     let shutdown = GracefulShutdown::new();
//!
//!     // Spawn a task that will be cancelled on shutdown
//!     let token = shutdown.token();
//!     tokio::spawn(async move {
//!         loop {
//!             tokio::select! {
//!                 _ = token.cancelled() => {
//!                     println!("Task cancelled");
//!                     break;
//!                 }
//!                 _ = tokio::time::sleep(Duration::from_secs(1)) => {
//!                     println!("Working...");
//!                 }
//!             }
//!         }
//!     });
//!
//!     // Wait for shutdown signal
//!     shutdown.wait().await;
//! }
//! ```

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    time::Duration,
};

use tokio::sync::{broadcast, watch};

/// Shutdown signal types
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ShutdownSignal {
    /// SIGINT (Ctrl+C)
    Interrupt,
    /// SIGTERM
    Terminate,
    /// Manual shutdown request
    Manual,
}

impl std::fmt::Display for ShutdownSignal {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ShutdownSignal::Interrupt => write!(f, "SIGINT"),
            ShutdownSignal::Terminate => write!(f, "SIGTERM"),
            ShutdownSignal::Manual => write!(f, "Manual"),
        }
    }
}

/// A token that can be used to check if shutdown has been requested
#[derive(Clone)]
pub struct ShutdownToken {
    receiver: watch::Receiver<bool>,
}

impl ShutdownToken {
    /// Check if shutdown has been requested
    pub fn is_shutdown(&self) -> bool {
        *self.receiver.borrow()
    }

    /// Wait until shutdown is requested
    pub async fn cancelled(&mut self) {
        // Wait for the value to become true
        let _ = self.receiver.wait_for(|v| *v).await;
    }
}

/// Builder for configuring graceful shutdown
pub struct GracefulShutdownBuilder {
    timeout: Duration,
    on_signal: Option<Box<dyn Fn(ShutdownSignal) + Send + Sync>>,
}

impl Default for GracefulShutdownBuilder {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(30),
            on_signal: None,
        }
    }
}

impl GracefulShutdownBuilder {
    /// Create a new builder
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the shutdown timeout
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.timeout = timeout;
        self
    }

    /// Set a callback to be called when a shutdown signal is received
    pub fn on_signal<F>(mut self, callback: F) -> Self
    where
        F: Fn(ShutdownSignal) + Send + Sync + 'static,
    {
        self.on_signal = Some(Box::new(callback));
        self
    }

    /// Build the graceful shutdown handler
    pub fn build(self) -> GracefulShutdown {
        let on_signal: Option<Arc<dyn Fn(ShutdownSignal) + Send + Sync>> =
            self.on_signal.map(|f| Arc::from(f) as Arc<dyn Fn(ShutdownSignal) + Send + Sync>);
        GracefulShutdown {
            timeout: self.timeout,
            on_signal,
            shutdown_tx: watch::channel(false).0,
            signal_tx: broadcast::channel(1).0,
        }
    }
}

/// Graceful shutdown handler
///
/// Provides utilities for handling graceful shutdown of services.
pub struct GracefulShutdown {
    timeout: Duration,
    on_signal: Option<Arc<dyn Fn(ShutdownSignal) + Send + Sync>>,
    shutdown_tx: watch::Sender<bool>,
    signal_tx: broadcast::Sender<ShutdownSignal>,
}

impl GracefulShutdown {
    /// Create a new graceful shutdown handler with default settings
    pub fn new() -> Self {
        GracefulShutdownBuilder::new().build()
    }

    /// Create a builder for configuring the shutdown handler
    pub fn builder() -> GracefulShutdownBuilder {
        GracefulShutdownBuilder::new()
    }

    /// Get the shutdown timeout
    pub fn timeout(&self) -> Duration {
        self.timeout
    }

    /// Get a token that can be used to check shutdown status
    pub fn token(&self) -> ShutdownToken {
        ShutdownToken {
            receiver: self.shutdown_tx.subscribe(),
        }
    }

    /// Subscribe to shutdown signals
    pub fn subscribe(&self) -> broadcast::Receiver<ShutdownSignal> {
        self.signal_tx.subscribe()
    }

    /// Trigger a manual shutdown
    pub fn shutdown(&self) {
        let _ = self.shutdown_tx.send(true);
        let _ = self.signal_tx.send(ShutdownSignal::Manual);
        if let Some(ref callback) = self.on_signal {
            callback(ShutdownSignal::Manual);
        }
    }

    /// Wait for a shutdown signal (SIGINT or SIGTERM)
    ///
    /// Returns the signal that triggered the shutdown.
    pub async fn wait(&self) -> ShutdownSignal {
        let signal = wait_for_signal().await;

        // Notify subscribers
        let _ = self.shutdown_tx.send(true);
        let _ = self.signal_tx.send(signal);

        // Call the callback if set
        if let Some(ref callback) = self.on_signal {
            callback(signal);
        }

        signal
    }

    /// Wait for shutdown with a timeout
    ///
    /// If the timeout expires before shutdown completes, returns None.
    pub async fn wait_with_timeout(&self) -> Option<ShutdownSignal> {
        tokio::select! {
            signal = self.wait() => Some(signal),
            _ = tokio::time::sleep(self.timeout) => None,
        }
    }

    /// Run a future until shutdown is requested
    ///
    /// Returns the result of the future if it completes before shutdown,
    /// or None if shutdown was requested first.
    pub async fn run_until_shutdown<F, T>(&self, future: F) -> Option<T>
    where
        F: Future<Output = T>,
    {
        let mut token = self.token();
        tokio::select! {
            result = future => Some(result),
            _ = token.cancelled() => None,
        }
    }

    /// Spawn a task that will be cancelled on shutdown
    pub fn spawn<F>(&self, name: &str, future: F) -> tokio::task::JoinHandle<Option<()>>
    where
        F: Future<Output = ()> + Send + 'static,
    {
        let mut token = self.token();
        let name = name.to_string();
        tokio::spawn(async move {
            tokio::select! {
                _ = future => {
                    Some(())
                }
                _ = token.cancelled() => {
                    #[cfg(feature = "otel")]
                    tracing::info!(task = %name, "Task cancelled due to shutdown");
                    #[cfg(not(feature = "otel"))]
                    let _ = name;
                    None
                }
            }
        })
    }
}

impl Default for GracefulShutdown {
    fn default() -> Self {
        Self::new()
    }
}

/// Wait for a shutdown signal (SIGINT or SIGTERM)
async fn wait_for_signal() -> ShutdownSignal {
    #[cfg(unix)]
    {
        use tokio::signal::unix::{signal, SignalKind};

        let mut sigint = signal(SignalKind::interrupt()).expect("Failed to register SIGINT handler");
        let mut sigterm = signal(SignalKind::terminate()).expect("Failed to register SIGTERM handler");

        tokio::select! {
            _ = sigint.recv() => ShutdownSignal::Interrupt,
            _ = sigterm.recv() => ShutdownSignal::Terminate,
        }
    }

    #[cfg(not(unix))]
    {
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to register Ctrl+C handler");
        ShutdownSignal::Interrupt
    }
}

/// Shutdown guard that triggers shutdown when dropped
///
/// Useful for ensuring cleanup happens even on panic.
pub struct ShutdownGuard {
    shutdown: Arc<GracefulShutdown>,
}

impl ShutdownGuard {
    /// Create a new shutdown guard
    pub fn new(shutdown: Arc<GracefulShutdown>) -> Self {
        Self { shutdown }
    }
}

impl Drop for ShutdownGuard {
    fn drop(&mut self) {
        self.shutdown.shutdown();
    }
}

/// Extension trait for futures that adds shutdown awareness
pub trait ShutdownExt: Future + Sized {
    /// Run this future until completion or shutdown
    fn with_shutdown(
        self,
        shutdown: &GracefulShutdown,
    ) -> Pin<Box<dyn Future<Output = Option<Self::Output>> + Send + '_>>
    where
        Self: Send + 'static,
        Self::Output: Send,
    {
        let future = self;
        let mut token = shutdown.token();
        Box::pin(async move {
            tokio::select! {
                result = future => Some(result),
                _ = token.cancelled() => None,
            }
        })
    }
}

impl<F: Future> ShutdownExt for F {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_shutdown_signal_display() {
        assert_eq!(ShutdownSignal::Interrupt.to_string(), "SIGINT");
        assert_eq!(ShutdownSignal::Terminate.to_string(), "SIGTERM");
        assert_eq!(ShutdownSignal::Manual.to_string(), "Manual");
    }

    #[tokio::test]
    async fn test_shutdown_token() {
        let shutdown = GracefulShutdown::new();
        let token = shutdown.token();

        assert!(!token.is_shutdown());

        shutdown.shutdown();

        assert!(token.is_shutdown());
    }

    #[tokio::test]
    async fn test_manual_shutdown() {
        let shutdown = GracefulShutdown::new();
        let mut rx = shutdown.subscribe();

        shutdown.shutdown();

        let signal = rx.recv().await.unwrap();
        assert_eq!(signal, ShutdownSignal::Manual);
    }

    #[tokio::test]
    async fn test_shutdown_callback() {
        use std::sync::atomic::{AtomicBool, Ordering};

        let called = Arc::new(AtomicBool::new(false));
        let called_clone = called.clone();

        let shutdown = GracefulShutdown::builder()
            .on_signal(move |_| {
                called_clone.store(true, Ordering::SeqCst);
            })
            .build();

        shutdown.shutdown();

        assert!(called.load(Ordering::SeqCst));
    }

    #[tokio::test]
    async fn test_run_until_shutdown() {
        let shutdown = GracefulShutdown::new();

        // Future completes before shutdown
        let result = shutdown
            .run_until_shutdown(async { 42 })
            .await;
        assert_eq!(result, Some(42));

        // Trigger shutdown
        shutdown.shutdown();

        // Future doesn't complete because shutdown was triggered
        let token = shutdown.token();
        assert!(token.is_shutdown());
    }

    #[tokio::test]
    async fn test_builder_timeout() {
        let shutdown = GracefulShutdown::builder()
            .timeout(Duration::from_secs(60))
            .build();

        assert_eq!(shutdown.timeout(), Duration::from_secs(60));
    }

    #[tokio::test]
    async fn test_spawn_task() {
        let shutdown = GracefulShutdown::new();
        let counter = Arc::new(std::sync::atomic::AtomicU32::new(0));
        let counter_clone = counter.clone();

        let handle = shutdown.spawn("test_task", async move {
            counter_clone.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        });

        // Wait for the task to complete
        let result = handle.await.unwrap();
        assert_eq!(result, Some(()));
        assert_eq!(counter.load(std::sync::atomic::Ordering::SeqCst), 1);
    }
}
