//! Health check infrastructure for AllFrame applications
//!
//! This module provides a standardized health check pattern with support for
//! dependency health monitoring, configurable timeouts, and critical vs
//! non-critical dependency classification.
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::health::{HealthCheck, Dependency, DependencyStatus, HealthReport};
//! use std::time::Duration;
//!
//! struct RedisHealth {
//!     client: redis::Client,
//! }
//!
//! impl Dependency for RedisHealth {
//!     fn name(&self) -> &str { "redis" }
//!
//!     async fn check(&self) -> DependencyStatus {
//!         match self.client.ping().await {
//!             Ok(_) => DependencyStatus::Healthy,
//!             Err(e) => DependencyStatus::Unhealthy(e.to_string()),
//!         }
//!     }
//!
//!     fn is_critical(&self) -> bool { true }
//!     fn timeout(&self) -> Duration { Duration::from_secs(5) }
//! }
//! ```

mod server;
mod types;

use std::{
    future::Future,
    pin::Pin,
    sync::Arc,
    time::{Duration, Instant},
};

pub use server::HealthServer;
pub use types::*;

/// A dependency that can be health-checked
///
/// Implement this trait for each external dependency (database, cache, API,
/// etc.) that your application relies on.
pub trait Dependency: Send + Sync {
    /// The name of this dependency (e.g., "redis", "postgres", "kraken-api")
    fn name(&self) -> &str;

    /// Check the health of this dependency
    ///
    /// This method should perform an actual health check (e.g., ping, query)
    /// and return the current status.
    fn check(&self) -> Pin<Box<dyn Future<Output = DependencyStatus> + Send + '_>>;

    /// Whether this dependency is critical for the application to function
    ///
    /// If a critical dependency is unhealthy, the overall health status will be
    /// Unhealthy. Non-critical dependencies being unhealthy will result in
    /// a Degraded status.
    fn is_critical(&self) -> bool {
        true
    }

    /// The timeout for health checks on this dependency
    ///
    /// If the check takes longer than this duration, it will be considered
    /// unhealthy.
    fn timeout(&self) -> Duration {
        Duration::from_secs(5)
    }
}

/// A collection of dependencies that can be health-checked together
pub trait HealthCheck: Send + Sync {
    /// Get all dependencies to check
    fn dependencies(&self) -> Vec<Arc<dyn Dependency>>;

    /// Check all dependencies and return a comprehensive health report
    fn check_all(&self) -> Pin<Box<dyn Future<Output = HealthReport> + Send + '_>> {
        let deps = self.dependencies();
        Box::pin(async move {
            let start = Instant::now();
            let mut reports = Vec::with_capacity(deps.len());
            let mut has_critical_failure = false;
            let mut has_degradation = false;

            for dep in deps {
                let dep_start = Instant::now();
                let timeout = dep.timeout();
                let is_critical = dep.is_critical();
                let name = dep.name().to_string();

                let status = match tokio::time::timeout(timeout, dep.check()).await {
                    Ok(status) => status,
                    Err(_) => DependencyStatus::Unhealthy(format!(
                        "Health check timed out after {:?}",
                        timeout
                    )),
                };

                let duration = dep_start.elapsed();

                match &status {
                    DependencyStatus::Unhealthy(_) if is_critical => {
                        has_critical_failure = true;
                    }
                    DependencyStatus::Unhealthy(_) | DependencyStatus::Degraded(_) => {
                        has_degradation = true;
                    }
                    _ => {}
                }

                reports.push(DependencyReport {
                    name,
                    status,
                    duration,
                    critical: is_critical,
                });
            }

            let overall_status = if has_critical_failure {
                OverallStatus::Unhealthy
            } else if has_degradation {
                OverallStatus::Degraded
            } else {
                OverallStatus::Healthy
            };

            HealthReport {
                status: overall_status,
                dependencies: reports,
                total_duration: start.elapsed(),
                timestamp: std::time::SystemTime::now(),
            }
        })
    }
}

/// A simple health checker that holds a list of dependencies
pub struct SimpleHealthCheck {
    dependencies: Vec<Arc<dyn Dependency>>,
}

impl SimpleHealthCheck {
    /// Create a new health checker with no dependencies
    pub fn new() -> Self {
        Self {
            dependencies: Vec::new(),
        }
    }

    /// Add a dependency to check
    pub fn add_dependency<D: Dependency + 'static>(mut self, dep: D) -> Self {
        self.dependencies.push(Arc::new(dep));
        self
    }

    /// Add an already Arc-wrapped dependency
    pub fn add_arc_dependency(mut self, dep: Arc<dyn Dependency>) -> Self {
        self.dependencies.push(dep);
        self
    }
}

impl Default for SimpleHealthCheck {
    fn default() -> Self {
        Self::new()
    }
}

impl HealthCheck for SimpleHealthCheck {
    fn dependencies(&self) -> Vec<Arc<dyn Dependency>> {
        self.dependencies.clone()
    }
}

/// A dependency that always reports healthy (useful for testing)
pub struct AlwaysHealthy {
    name: String,
}

impl AlwaysHealthy {
    /// Create a new always-healthy dependency
    pub fn new(name: impl Into<String>) -> Self {
        Self { name: name.into() }
    }
}

impl Dependency for AlwaysHealthy {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = DependencyStatus> + Send + '_>> {
        Box::pin(async { DependencyStatus::Healthy })
    }
}

/// A dependency that always reports unhealthy (useful for testing)
pub struct AlwaysUnhealthy {
    name: String,
    message: String,
}

impl AlwaysUnhealthy {
    /// Create a new always-unhealthy dependency
    pub fn new(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            message: message.into(),
        }
    }
}

impl Dependency for AlwaysUnhealthy {
    fn name(&self) -> &str {
        &self.name
    }

    fn check(&self) -> Pin<Box<dyn Future<Output = DependencyStatus> + Send + '_>> {
        let msg = self.message.clone();
        Box::pin(async move { DependencyStatus::Unhealthy(msg) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_always_healthy() {
        let dep = AlwaysHealthy::new("test");
        assert_eq!(dep.name(), "test");
        assert!(matches!(dep.check().await, DependencyStatus::Healthy));
    }

    #[tokio::test]
    async fn test_always_unhealthy() {
        let dep = AlwaysUnhealthy::new("test", "connection failed");
        assert_eq!(dep.name(), "test");
        match dep.check().await {
            DependencyStatus::Unhealthy(msg) => assert_eq!(msg, "connection failed"),
            _ => panic!("Expected unhealthy status"),
        }
    }

    #[tokio::test]
    async fn test_simple_health_check_empty() {
        let checker = SimpleHealthCheck::new();
        let report = checker.check_all().await;
        assert_eq!(report.status, OverallStatus::Healthy);
        assert!(report.dependencies.is_empty());
    }

    #[tokio::test]
    async fn test_simple_health_check_all_healthy() {
        let checker = SimpleHealthCheck::new()
            .add_dependency(AlwaysHealthy::new("dep1"))
            .add_dependency(AlwaysHealthy::new("dep2"));

        let report = checker.check_all().await;
        assert_eq!(report.status, OverallStatus::Healthy);
        assert_eq!(report.dependencies.len(), 2);
    }

    #[tokio::test]
    async fn test_simple_health_check_critical_failure() {
        let checker = SimpleHealthCheck::new()
            .add_dependency(AlwaysHealthy::new("healthy"))
            .add_dependency(AlwaysUnhealthy::new("critical", "down"));

        let report = checker.check_all().await;
        assert_eq!(report.status, OverallStatus::Unhealthy);
    }

    #[tokio::test]
    async fn test_default_timeout() {
        let dep = AlwaysHealthy::new("test");
        assert_eq!(dep.timeout(), Duration::from_secs(5));
    }

    #[tokio::test]
    async fn test_default_critical() {
        let dep = AlwaysHealthy::new("test");
        assert!(dep.is_critical());
    }
}
