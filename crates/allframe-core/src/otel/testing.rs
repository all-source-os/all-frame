//! Testing utilities for OpenTelemetry
//!
//! This module provides in-memory recorders for spans and metrics
//! that are useful for testing instrumented code.

use std::{
    collections::HashMap,
    sync::{Arc, RwLock},
};

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

    /// Clear all recorded spans
    pub fn clear(&self) {
        let mut spans = self.spans.write().unwrap();
        spans.clear();
    }
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

    /// Increment a counter
    pub fn increment_counter(&self, name: &str, value: u64) {
        let mut counters = self.counters.write().unwrap();
        *counters.entry(name.to_string()).or_insert(0) += value;
    }

    /// Set a gauge value
    pub fn set_gauge(&self, name: &str, value: i64) {
        let mut gauges = self.gauges.write().unwrap();
        gauges.insert(name.to_string(), value);
    }

    /// Record a histogram value
    pub fn record_histogram(&self, name: &str, value: f64) {
        let mut histograms = self.histograms.write().unwrap();
        histograms
            .entry(name.to_string())
            .or_insert_with(Vec::new)
            .push(value);
    }

    /// Get counter value
    pub fn get_counter(&self, name: &str) -> u64 {
        self.counters
            .read()
            .unwrap()
            .get(name)
            .copied()
            .unwrap_or(0)
    }

    /// Get gauge value
    pub fn get_gauge(&self, name: &str) -> i64 {
        self.gauges.read().unwrap().get(name).copied().unwrap_or(0)
    }

    /// Get histogram
    pub fn get_histogram(&self, name: &str) -> Histogram {
        let values = self
            .histograms
            .read()
            .unwrap()
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

    #[test]
    fn test_metrics_recorder() {
        let recorder = MetricsRecorder::new();

        recorder.increment_counter("requests", 1);
        recorder.increment_counter("requests", 2);
        assert_eq!(recorder.get_counter("requests"), 3);

        recorder.set_gauge("active_connections", 42);
        assert_eq!(recorder.get_gauge("active_connections"), 42);

        recorder.record_histogram("latency", 100.0);
        recorder.record_histogram("latency", 200.0);
        let hist = recorder.get_histogram("latency");
        assert_eq!(hist.count(), 2);
    }
}
