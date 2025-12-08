//! OpenTelemetry automatic instrumentation
//!
//! This module provides automatic distributed tracing, metrics, and context
//! propagation for AllFrame applications.
//!
//! # Features
//!
//! - `otel` - Basic tracing support with the `#[traced]` macro
//! - `otel-otlp` - Full OpenTelemetry integration with OTLP export
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use allframe_core::otel::Observability;
//!
//! let _guard = Observability::builder("my-service")
//!     .service_version(env!("CARGO_PKG_VERSION"))
//!     .environment_from_env()
//!     .otlp_endpoint_from_env()
//!     .json_logging()
//!     .log_level_from_env()
//!     .build()?;
//!
//! // Guard keeps the subscriber active
//! // When dropped, flushes pending spans
//! ```

mod builder;
mod testing;

// Re-export the traced macro
// Legacy placeholder functions - kept for backwards compatibility
// These will be removed in a future version
use std::collections::HashMap;

#[cfg(feature = "otel")]
pub use allframe_macros::traced;
// Re-export builder types
pub use builder::{Observability, ObservabilityBuilder, ObservabilityError, ObservabilityGuard};
// Re-export testing utilities
pub use testing::{Histogram, MetricsRecorder, Span, SpanContext, SpanRecorder};

/// Get the current span ID (placeholder - use tracing for real spans)
#[deprecated(since = "0.2.0", note = "Use tracing::Span::current() instead")]
pub fn current_span_id() -> String {
    "span-placeholder".to_string()
}

/// Get the current trace ID (placeholder - use tracing for real traces)
#[deprecated(since = "0.2.0", note = "Use tracing::Span::current() instead")]
pub fn current_trace_id() -> String {
    "trace-placeholder".to_string()
}

/// Start a new trace (placeholder)
#[deprecated(since = "0.2.0", note = "Use tracing spans instead")]
pub fn start_trace(_trace_id: &str) {
    // Placeholder for backwards compatibility
}

/// Set baggage value (placeholder)
#[deprecated(since = "0.2.0", note = "Use opentelemetry baggage API instead")]
pub fn set_baggage(_key: &str, _value: &str) {
    // Placeholder for backwards compatibility
}

/// Get baggage value (placeholder)
#[deprecated(since = "0.2.0", note = "Use opentelemetry baggage API instead")]
pub fn get_baggage(_key: &str) -> Option<String> {
    None
}

/// Inject context into headers (placeholder)
#[deprecated(since = "0.2.0", note = "Use opentelemetry propagator API instead")]
pub fn inject_context(_context: &SpanContext) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("traceparent".to_string(), "placeholder".to_string());
    headers
}

/// Extract context from headers (placeholder)
#[deprecated(since = "0.2.0", note = "Use opentelemetry propagator API instead")]
pub fn extract_context(headers: &HashMap<String, String>) -> Option<SpanContext> {
    headers.get("traceparent").map(|_| SpanContext {
        trace_id: "extracted-trace".to_string(),
        parent_span_id: "extracted-span".to_string(),
        sampled: true,
    })
}

/// Exporter type (legacy)
#[derive(Debug, Clone, PartialEq)]
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
pub enum ExporterType {
    /// Stdout console exporter
    Stdout,
    /// Jaeger exporter
    Jaeger {
        /// Jaeger endpoint URL
        endpoint: String,
    },
    /// OTLP exporter
    Otlp {
        /// OTLP endpoint URL
        endpoint: String,
    },
}

/// Configure exporter (placeholder)
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
#[allow(deprecated)]
pub fn configure_exporter(_exporter: ExporterType) {
    // Placeholder for backwards compatibility
}

/// Configure batch export (placeholder)
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
pub fn configure_batch_export(_batch_size: usize, _flush_interval_ms: u64) {
    // Placeholder for backwards compatibility
}

/// Get export count (placeholder)
#[deprecated(since = "0.2.0", note = "This function will be removed")]
pub fn get_export_count() -> usize {
    0
}

/// Configure sampling rate (placeholder)
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
pub fn configure_sampling(_rate: f64) {
    // Placeholder for backwards compatibility
}

/// Enable tracing (placeholder)
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder::build() instead")]
pub fn enable_tracing() {
    // Placeholder for backwards compatibility
}

/// Disable tracing (placeholder)
#[deprecated(since = "0.2.0", note = "Drop the ObservabilityGuard instead")]
pub fn disable_tracing() {
    // Placeholder for backwards compatibility
}

/// OTel configuration (legacy)
#[derive(Debug, Clone)]
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
pub struct OtelConfig {
    /// Service name
    pub service_name: String,
    /// Exporter type
    pub exporter_type: String,
    /// Sampling rate
    pub sampling_rate: f64,
    /// Batch size
    pub batch_size: usize,
}

/// Configure from file (placeholder)
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
pub async fn configure_from_file(_path: &str) -> Result<(), String> {
    Ok(())
}

/// Get current config (placeholder)
#[deprecated(since = "0.2.0", note = "Use ObservabilityBuilder instead")]
#[allow(deprecated)]
pub fn get_config() -> OtelConfig {
    OtelConfig {
        service_name: "allframe".to_string(),
        exporter_type: "stdout".to_string(),
        sampling_rate: 1.0,
        batch_size: 512,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_span_recorder() {
        let recorder = SpanRecorder::new();

        let span = Span {
            span_id: "span-1".to_string(),
            parent_span_id: None,
            trace_id: "trace-1".to_string(),
            name: "test".to_string(),
            attributes: std::collections::HashMap::new(),
            status: "ok".to_string(),
            error_message: String::new(),
            duration_ms: 100.0,
            layer: String::new(),
        };

        recorder.record(span.clone());
        let spans = recorder.spans();

        assert_eq!(spans.len(), 1);
        assert_eq!(spans[0].span_id, "span-1");
    }

    #[test]
    fn test_histogram() {
        let hist = Histogram::new(vec![1.0, 2.0, 3.0, 4.0, 5.0]);

        assert_eq!(hist.count(), 5);
        assert_eq!(hist.sum(), 15.0);
        assert_eq!(hist.p50(), 3.0);
    }

    #[test]
    fn test_builder_creation() {
        // Just verify we can create the builder - fields are tested in builder.rs
        let _builder = ObservabilityBuilder::new("test-service");
    }
}
