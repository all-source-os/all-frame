//! OpenTelemetry automatic instrumentation
//!
//! This module provides automatic distributed tracing, metrics, and context propagation
//! for AllFrame applications with zero manual instrumentation required.

use std::collections::HashMap;
use std::sync::{Arc, RwLock};

// Re-export macros
#[cfg(feature = "otel")]
pub use allframe_macros::traced;

/// Span represents a unit of work in distributed tracing
#[derive(Debug, Clone)]
pub struct Span {
    /// Span ID
    pub span_id: String,
    /// Parent span ID (if any)
    pub parent_span_id: Option<String>,
    /// Trace ID
    pub trace_id: String,
    /// Span name
    pub name: String,
    /// Span attributes
    pub attributes: HashMap<String, String>,
    /// Span status
    pub status: String,
    /// Error message (if failed)
    pub error_message: String,
    /// Duration in milliseconds
    pub duration_ms: f64,
    /// Architecture layer (if applicable)
    pub layer: String,
}

/// SpanRecorder for testing - records all spans
#[derive(Clone, Default)]
pub struct SpanRecorder {
    spans: Arc<RwLock<Vec<Span>>>,
}

impl SpanRecorder {
    /// Create a new span recorder
    pub fn new() -> Self {
        Self {
            spans: Arc::new(RwLock::new(Vec::new())),
        }
    }

    /// Record a span
    pub fn record(&self, span: Span) {
        let mut spans = self.spans.write().unwrap();
        spans.push(span);
    }

    /// Get all recorded spans
    pub fn spans(&self) -> Vec<Span> {
        self.spans.read().unwrap().clone()
    }
}

/// Get the current span ID (placeholder)
pub fn current_span_id() -> String {
    "span-placeholder".to_string()
}

/// Get the current trace ID (placeholder)
pub fn current_trace_id() -> String {
    "trace-placeholder".to_string()
}

/// Start a new trace (placeholder)
pub fn start_trace(_trace_id: &str) {
    // Placeholder for MVP
}

/// Set baggage value (placeholder)
pub fn set_baggage(_key: &str, _value: &str) {
    // Placeholder for MVP
}

/// Get baggage value (placeholder)
pub fn get_baggage(_key: &str) -> Option<String> {
    // Placeholder for MVP
    None
}

/// SpanContext for distributed tracing
#[derive(Debug, Clone)]
pub struct SpanContext {
    /// Trace ID
    pub trace_id: String,
    /// Parent span ID
    pub parent_span_id: String,
    /// Sampled flag
    pub sampled: bool,
}

impl SpanContext {
    /// Create a new span context
    pub fn new(trace_id: &str, parent_span_id: &str) -> Self {
        Self {
            trace_id: trace_id.to_string(),
            parent_span_id: parent_span_id.to_string(),
            sampled: true,
        }
    }
}

/// Inject context into headers (placeholder)
pub fn inject_context(_context: &SpanContext) -> HashMap<String, String> {
    let mut headers = HashMap::new();
    headers.insert("traceparent".to_string(), "placeholder".to_string());
    headers
}

/// Extract context from headers (placeholder)
pub fn extract_context(headers: &HashMap<String, String>) -> Option<SpanContext> {
    headers.get("traceparent").map(|_| SpanContext {
        trace_id: "extracted-trace".to_string(),
        parent_span_id: "extracted-span".to_string(),
        sampled: true,
    })
}

/// MetricsRecorder for testing - records all metrics
#[derive(Clone, Default)]
pub struct MetricsRecorder {
    counters: Arc<RwLock<HashMap<String, u64>>>,
    gauges: Arc<RwLock<HashMap<String, i64>>>,
    histograms: Arc<RwLock<HashMap<String, Vec<f64>>>>,
}

impl MetricsRecorder {
    /// Create a new metrics recorder
    pub fn new() -> Self {
        Self {
            counters: Arc::new(RwLock::new(HashMap::new())),
            gauges: Arc::new(RwLock::new(HashMap::new())),
            histograms: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Get current instance (placeholder)
    pub fn current() -> Self {
        Self::new()
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters.read().unwrap().get(name).copied().unwrap_or(0)
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> i64 {
        self.gauges.read().unwrap().get(name).copied().unwrap_or(0)
    }

    /// Get histogram
    pub fn get_histogram(&self, name: &str) -> Histogram {
        let values = self.histograms.read().unwrap()
            .get(name)
            .cloned()
            .unwrap_or_default();
        Histogram::new(values)
    }

    /// Get counter with labels (placeholder)
    pub fn get_counter_with_labels(&self, name: &str, _labels: &[(&str, &str)]) -> u64 {
        self.get_counter(name)
    }
}

/// Histogram for latency measurements
pub struct Histogram {
    values: Vec<f64>,
}

impl Histogram {
    /// Create new histogram
    pub fn new(values: Vec<f64>) -> Self {
        Self { values }
    }

    /// Get count of measurements
    pub fn count(&self) -> usize {
        self.values.len()
    }

    /// Get sum of all values
    pub fn sum(&self) -> f64 {
        self.values.iter().sum()
    }

    /// Get p50 percentile
    pub fn p50(&self) -> f64 {
        self.percentile(0.5)
    }

    /// Get p95 percentile
    pub fn p95(&self) -> f64 {
        self.percentile(0.95)
    }

    /// Get p99 percentile
    pub fn p99(&self) -> f64 {
        self.percentile(0.99)
    }

    fn percentile(&self, p: f64) -> f64 {
        if self.values.is_empty() {
            return 0.0;
        }
        let mut sorted = self.values.clone();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        let index = ((self.values.len() as f64 - 1.0) * p) as usize;
        sorted[index]
    }
}

/// Exporter type
#[derive(Debug, Clone, PartialEq)]
pub enum ExporterType {
    /// Stdout console exporter
    Stdout,
    /// Jaeger exporter
    Jaeger {
        /// Jaeger endpoint URL
        endpoint: String
    },
    /// OTLP exporter
    Otlp {
        /// OTLP endpoint URL
        endpoint: String
    },
}

/// Configure exporter (placeholder)
pub fn configure_exporter(_exporter: ExporterType) {
    // Placeholder for MVP
}

/// Configure batch export (placeholder)
pub fn configure_batch_export(_batch_size: usize, _flush_interval_ms: u64) {
    // Placeholder for MVP
}

/// Get export count (placeholder)
pub fn get_export_count() -> usize {
    0
}

/// Configure sampling rate (placeholder)
pub fn configure_sampling(_rate: f64) {
    // Placeholder for MVP
}

/// Enable tracing (placeholder)
pub fn enable_tracing() {
    // Placeholder for MVP
}

/// Disable tracing (placeholder)
pub fn disable_tracing() {
    // Placeholder for MVP
}

/// OTel configuration
#[derive(Debug, Clone)]
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
pub async fn configure_from_file(_path: &str) -> Result<(), String> {
    // Placeholder for MVP
    Ok(())
}

/// Get current config (placeholder)
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
            attributes: HashMap::new(),
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
}
