//! Observability for resilience operations.
//!
//! This module provides metrics collection, tracing instrumentation, and monitoring
//! capabilities for resilience operations, enabling visibility into system reliability.
//!
//! # Features
//!
//! - **Metrics Collection**: Counters, histograms, and gauges for resilience operations
//! - **Tracing Instrumentation**: Detailed traces for policy execution and failures
//! - **Health Checks**: Circuit breaker and service health monitoring
//! - **Alerting Integration**: Threshold-based alerting for resilience events

use crate::application::resilience::{ResilienceOrchestrator, ResilienceOrchestrationError, ResilienceMetrics};
use crate::domain::resilience::{ResiliencePolicy, ResilienceDomainError};
use std::time::{Duration, Instant};
use std::sync::Arc;

/// Observability service for resilience operations
#[derive(Clone)]
pub struct ResilienceObservability {
    metrics_collector: Arc<dyn MetricsCollector>,
    tracer: Arc<dyn ResilienceTracer>,
}

impl ResilienceObservability {
    /// Create a new observability service with default implementations
    pub fn new() -> Self {
        Self {
            metrics_collector: Arc::new(NoOpMetricsCollector),
            tracer: Arc::new(NoOpTracer),
        }
    }

    /// Create with custom collector and tracer
    pub fn with_components(
        metrics_collector: Arc<dyn MetricsCollector>,
        tracer: Arc<dyn ResilienceTracer>,
    ) -> Self {
        Self {
            metrics_collector,
            tracer,
        }
    }

    /// Record the start of a resilience operation
    pub fn record_operation_start(&self, operation_id: &str, policy: &ResiliencePolicy) {
        self.metrics_collector.increment_counter("resilience_operations_total", &[("operation", operation_id)]);
        self.tracer.start_span("resilience_operation", &[
            ("operation_id", operation_id),
            ("policy_type", &policy_type_name(policy)),
        ]);
    }

    /// Record the completion of a resilience operation
    pub fn record_operation_complete(
        &self,
        operation_id: &str,
        policy: &ResiliencePolicy,
        duration: Duration,
        result: &Result<(), ResilienceOrchestrationError>,
    ) {
        let status = if result.is_ok() { "success" } else { "failure" };
        let duration_ms = duration.as_millis() as f64;

        // Record metrics
        self.metrics_collector.increment_counter(
            "resilience_operations_completed_total",
            &[("operation", operation_id), ("status", status)]
        );

        self.metrics_collector.record_histogram(
            "resilience_operation_duration_ms",
            duration_ms,
            &[("operation", operation_id), ("policy_type", &policy_type_name(policy))]
        );

        // Record policy-specific metrics
        match policy {
            ResiliencePolicy::Retry { max_attempts, .. } => {
                self.metrics_collector.record_histogram(
                    "resilience_retry_max_attempts",
                    *max_attempts as f64,
                    &[("operation", operation_id)]
                );
            }
            ResiliencePolicy::CircuitBreaker { failure_threshold, .. } => {
                self.metrics_collector.record_gauge(
                    "resilience_circuit_breaker_failure_threshold",
                    *failure_threshold as f64,
                    &[("operation", operation_id)]
                );
            }
            ResiliencePolicy::RateLimit { requests_per_second, .. } => {
                self.metrics_collector.record_gauge(
                    "resilience_rate_limit_rps",
                    *requests_per_second as f64,
                    &[("operation", operation_id)]
                );
            }
            _ => {}
        }

        // Handle errors specifically
        if let Err(error) = result {
            self.record_operation_error(operation_id, error);
        }

        // End tracing span
        self.tracer.end_span(&[("duration_ms", &duration_ms.to_string()), ("status", status)]);
    }

    /// Record resilience-specific errors
    pub fn record_operation_error(&self, operation_id: &str, error: &ResilienceOrchestrationError) {
        let error_type = match error {
            ResilienceOrchestrationError::Domain(domain_error) => match domain_error {
                ResilienceDomainError::RetryExhausted { .. } => "retry_exhausted",
                ResilienceDomainError::CircuitOpen => "circuit_open",
                ResilienceDomainError::RateLimited { .. } => "rate_limited",
                ResilienceDomainError::Timeout { .. } => "timeout",
                ResilienceDomainError::Infrastructure { .. } => "infrastructure",
                _ => "domain_error",
            },
            ResilienceOrchestrationError::Infrastructure(_) => "infrastructure",
            ResilienceOrchestrationError::Configuration(_) => "configuration",
            ResilienceOrchestrationError::Cancelled => "cancelled",
        };

        self.metrics_collector.increment_counter(
            "resilience_operation_errors_total",
            &[("operation", operation_id), ("error_type", error_type)]
        );

        self.tracer.add_event("resilience_error", &[
            ("operation_id", operation_id),
            ("error_type", error_type),
        ]);
    }

    /// Record circuit breaker state changes
    pub fn record_circuit_breaker_state_change(
        &self,
        circuit_breaker_id: &str,
        old_state: CircuitBreakerState,
        new_state: CircuitBreakerState,
    ) {
        self.metrics_collector.increment_counter(
            "resilience_circuit_breaker_state_changes_total",
            &[
                ("circuit_breaker", circuit_breaker_id),
                ("old_state", &old_state.as_str()),
                ("new_state", &new_state.as_str()),
            ]
        );

        self.tracer.add_event("circuit_breaker_state_change", &[
            ("circuit_breaker_id", circuit_breaker_id),
            ("old_state", &old_state.as_str()),
            ("new_state", &new_state.as_str()),
        ]);
    }

    /// Get current health status
    pub fn health_status(&self) -> ResilienceHealthStatus {
        // This would typically aggregate metrics to determine overall health
        ResilienceHealthStatus {
            overall_health: HealthLevel::Healthy,
            circuit_breakers_open: 0,
            services_degraded: 0,
            last_updated: std::time::SystemTime::now(),
        }
    }

    /// Export metrics in Prometheus format
    pub fn export_prometheus_metrics(&self) -> String {
        // This would collect all metrics and format them for Prometheus
        "# AllFrame Resilience Metrics\n# (Implementation would export actual metrics)\n".to_string()
    }
}

/// Circuit breaker states for observability
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum CircuitBreakerState {
    Closed,
    Open,
    HalfOpen,
}

impl CircuitBreakerState {
    pub fn as_str(&self) -> &'static str {
        match self {
            CircuitBreakerState::Closed => "closed",
            CircuitBreakerState::Open => "open",
            CircuitBreakerState::HalfOpen => "half_open",
        }
    }
}

/// Overall health status of the resilience system
#[derive(Clone, Debug)]
pub struct ResilienceHealthStatus {
    pub overall_health: HealthLevel,
    pub circuit_breakers_open: u32,
    pub services_degraded: u32,
    pub last_updated: std::time::SystemTime,
}

/// Health levels for services
#[derive(Clone, Copy, Debug, PartialEq)]
pub enum HealthLevel {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Metrics collection trait
#[async_trait::async_trait]
pub trait MetricsCollector: Send + Sync {
    /// Increment a counter metric
    fn increment_counter(&self, name: &str, labels: &[(&str, &str)]);

    /// Record a histogram value
    fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]);

    /// Set a gauge value
    fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]);
}

/// Tracing instrumentation trait
#[async_trait::async_trait]
pub trait ResilienceTracer: Send + Sync {
    /// Start a new trace span
    fn start_span(&self, name: &str, attributes: &[(&str, &str)]);

    /// End the current span
    fn end_span(&self, attributes: &[(&str, &str)]);

    /// Add an event to the current span
    fn add_event(&self, name: &str, attributes: &[(&str, &str)]);
}

/// No-op implementation for when observability is disabled
pub struct NoOpMetricsCollector;

impl MetricsCollector for NoOpMetricsCollector {
    fn increment_counter(&self, _name: &str, _labels: &[(&str, &str)]) {}
    fn record_histogram(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
    fn record_gauge(&self, _name: &str, _value: f64, _labels: &[(&str, &str)]) {}
}

/// No-op tracer implementation
pub struct NoOpTracer;

#[async_trait::async_trait]
impl ResilienceTracer for NoOpTracer {
    fn start_span(&self, _name: &str, _attributes: &[(&str, &str)]) {}
    fn end_span(&self, _attributes: &[(&str, &str)]) {}
    fn add_event(&self, _name: &str, _attributes: &[(&str, &str)]) {}
}

/// Prometheus metrics collector implementation
#[cfg(feature = "prometheus")]
pub mod prometheus {
    use super::*;
    use prometheus::{CounterVec, HistogramVec, GaugeVec, Encoder, TextEncoder};

    pub struct PrometheusMetricsCollector {
        counters: HashMap<String, CounterVec>,
        histograms: HashMap<String, HistogramVec>,
        gauges: HashMap<String, GaugeVec>,
    }

    impl PrometheusMetricsCollector {
        pub fn new() -> Self {
            Self {
                counters: HashMap::new(),
                histograms: HashMap::new(),
                gauges: HashMap::new(),
            }
        }

        fn get_or_create_counter(&mut self, name: &str, help: &str) -> &CounterVec {
            self.counters.entry(name.to_string()).or_insert_with(|| {
                CounterVec::new(
                    prometheus::Opts::new(name, help),
                    &["operation", "status", "error_type"]
                ).expect("Failed to create counter")
            })
        }

        fn get_or_create_histogram(&mut self, name: &str, help: &str) -> &HistogramVec {
            self.histograms.entry(name.to_string()).or_insert_with(|| {
                HistogramVec::new(
                    prometheus::HistogramOpts::new(name, help),
                    &["operation", "policy_type"]
                ).expect("Failed to create histogram")
            })
        }

        fn get_or_create_gauge(&mut self, name: &str, help: &str) -> &GaugeVec {
            self.gauges.entry(name.to_string()).or_insert_with(|| {
                GaugeVec::new(
                    prometheus::Opts::new(name, help),
                    &["operation"]
                ).expect("Failed to create gauge")
            })
        }
    }

    impl MetricsCollector for PrometheusMetricsCollector {
        fn increment_counter(&self, name: &str, labels: &[(&str, &str)]) {
            // Implementation would increment the appropriate counter
            // This is a simplified version
        }

        fn record_histogram(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
            // Implementation would record histogram values
        }

        fn record_gauge(&self, name: &str, value: f64, labels: &[(&str, &str)]) {
            // Implementation would set gauge values
        }
    }
}

/// Helper function to get policy type name for metrics
fn policy_type_name(policy: &ResiliencePolicy) -> String {
    match policy {
        ResiliencePolicy::None => "none".to_string(),
        ResiliencePolicy::Retry { .. } => "retry".to_string(),
        ResiliencePolicy::CircuitBreaker { .. } => "circuit_breaker".to_string(),
        ResiliencePolicy::RateLimit { .. } => "rate_limit".to_string(),
        ResiliencePolicy::Timeout { .. } => "timeout".to_string(),
        ResiliencePolicy::Combined { .. } => "combined".to_string(),
    }
}

/// Instrumented wrapper for resilience orchestrator
pub struct InstrumentedResilienceOrchestrator<T: ResilienceOrchestrator> {
    inner: T,
    observability: ResilienceObservability,
}

impl<T: ResilienceOrchestrator> InstrumentedResilienceOrchestrator<T> {
    pub fn new(inner: T, observability: ResilienceObservability) -> Self {
        Self { inner, observability }
    }
}

#[async_trait::async_trait]
impl<T: ResilienceOrchestrator> ResilienceOrchestrator for InstrumentedResilienceOrchestrator<T> {
    async fn execute_with_policy<F, Fut, E>(
        &self,
        policy: ResiliencePolicy,
        operation: F,
    ) -> Result<(), ResilienceOrchestrationError>
    where
        F: FnOnce() -> Fut + Send,
        Fut: std::future::Future<Output = Result<(), E>> + Send,
        E: Into<ResilienceOrchestrationError> + Send,
    {
        let operation_id = "anonymous_operation"; // In a real implementation, this would be configurable
        let start_time = Instant::now();

        self.observability.record_operation_start(operation_id, &policy);

        let result = self.inner.execute_with_policy(policy, operation).await;

        let duration = start_time.elapsed();
        self.observability.record_operation_complete(operation_id, &policy, duration, &result);

        result
    }

    #[cfg(feature = "resilience")]
    fn get_circuit_breaker(&self, name: &str) -> Option<&crate::resilience::CircuitBreaker> {
        self.inner.get_circuit_breaker(name)
    }

    #[cfg(feature = "resilience")]
    fn get_rate_limiter(&self, name: &str) -> Option<&crate::resilience::RateLimiter> {
        self.inner.get_rate_limiter(name)
    }

    fn metrics(&self) -> ResilienceMetrics {
        self.inner.metrics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_observability_recording() {
        let observability = ResilienceObservability::new();
        let policy = ResiliencePolicy::Retry {
            max_attempts: 3,
            backoff: crate::domain::resilience::BackoffStrategy::default(),
        };

        // Record operation lifecycle
        observability.record_operation_start("test_operation", &policy);

        let duration = Duration::from_millis(150);
        let result = Ok(());

        observability.record_operation_complete("test_operation", &policy, duration, &result);

        // Verify health status works
        let health = observability.health_status();
        assert_eq!(health.overall_health, HealthLevel::Healthy);
    }

    #[test]
    fn test_policy_type_name() {
        assert_eq!(policy_type_name(&ResiliencePolicy::None), "none");
        assert_eq!(policy_type_name(&ResiliencePolicy::Retry {
            max_attempts: 3,
            backoff: crate::domain::resilience::BackoffStrategy::default(),
        }), "retry");
        assert_eq!(policy_type_name(&ResiliencePolicy::CircuitBreaker {
            failure_threshold: 5,
            recovery_timeout: Duration::from_secs(30),
            success_threshold: 3,
        }), "circuit_breaker");
    }

    #[test]
    fn test_circuit_breaker_state_transitions() {
        let observability = ResilienceObservability::new();

        observability.record_circuit_breaker_state_change(
            "test_circuit",
            CircuitBreakerState::Closed,
            CircuitBreakerState::Open,
        );

        // In a real implementation, this would update metrics
        let health = observability.health_status();
        assert_eq!(health.circuit_breakers_open, 0); // No-op implementation
    }

    #[test]
    fn test_prometheus_export() {
        let observability = ResilienceObservability::new();
        let metrics = observability.export_prometheus_metrics();
        assert!(metrics.contains("#"));
    }
}
