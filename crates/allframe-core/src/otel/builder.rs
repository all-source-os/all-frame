//! Observability builder for easy setup of tracing and metrics
//!
//! This module provides a fluent builder API for configuring OpenTelemetry
//! tracing with OTLP export, structured logging, and more.
//!
//! # Example
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

use std::env;

/// Builder for configuring observability (tracing, metrics, logging)
pub struct ObservabilityBuilder {
    service_name: String,
    service_version: Option<String>,
    environment: Option<String>,
    otlp_endpoint: Option<String>,
    json_logging: bool,
    log_level: String,
}

impl ObservabilityBuilder {
    /// Create a new observability builder
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
            service_version: None,
            environment: None,
            otlp_endpoint: None,
            json_logging: false,
            log_level: "info".to_string(),
        }
    }

    /// Set the service version
    pub fn service_version(mut self, version: impl Into<String>) -> Self {
        self.service_version = Some(version.into());
        self
    }

    /// Set the environment (e.g., "production", "staging", "development")
    pub fn environment(mut self, env: impl Into<String>) -> Self {
        self.environment = Some(env.into());
        self
    }

    /// Read environment from ENVIRONMENT or ENV env var
    pub fn environment_from_env(mut self) -> Self {
        self.environment = env::var("ENVIRONMENT")
            .or_else(|_| env::var("ENV"))
            .ok();
        self
    }

    /// Set the OTLP endpoint for exporting traces
    pub fn otlp_endpoint(mut self, endpoint: impl Into<String>) -> Self {
        self.otlp_endpoint = Some(endpoint.into());
        self
    }

    /// Read OTLP endpoint from OTEL_EXPORTER_OTLP_ENDPOINT env var
    pub fn otlp_endpoint_from_env(mut self) -> Self {
        self.otlp_endpoint = env::var("OTEL_EXPORTER_OTLP_ENDPOINT").ok();
        self
    }

    /// Enable JSON-formatted log output (for production)
    pub fn json_logging(mut self) -> Self {
        self.json_logging = true;
        self
    }

    /// Set the log level (trace, debug, info, warn, error)
    pub fn log_level(mut self, level: impl Into<String>) -> Self {
        self.log_level = level.into();
        self
    }

    /// Read log level from RUST_LOG env var
    pub fn log_level_from_env(mut self) -> Self {
        if let Ok(level) = env::var("RUST_LOG") {
            self.log_level = level;
        }
        self
    }

    /// Build and initialize the observability stack
    ///
    /// Returns a guard that must be kept alive for the duration of the program.
    /// When the guard is dropped, pending spans are flushed.
    #[cfg(feature = "otel-otlp")]
    pub fn build(self) -> Result<ObservabilityGuard, ObservabilityError> {
        use opentelemetry::trace::TracerProvider as _;
        use opentelemetry_otlp::WithExportConfig;
        use opentelemetry_sdk::trace::TracerProvider;
        use tracing_subscriber::layer::SubscriberExt;
        use tracing_subscriber::util::SubscriberInitExt;
        use tracing_subscriber::EnvFilter;

        // Build resource attributes
        let mut resource_attrs = vec![opentelemetry::KeyValue::new(
            "service.name",
            self.service_name.clone(),
        )];

        if let Some(version) = &self.service_version {
            resource_attrs.push(opentelemetry::KeyValue::new(
                "service.version",
                version.clone(),
            ));
        }

        if let Some(env) = &self.environment {
            resource_attrs.push(opentelemetry::KeyValue::new(
                "deployment.environment",
                env.clone(),
            ));
        }

        let resource = opentelemetry_sdk::Resource::new(resource_attrs);

        // Build tracer provider
        let tracer_provider = if let Some(endpoint) = &self.otlp_endpoint {
            // OTLP exporter with batch processing
            let exporter = opentelemetry_otlp::SpanExporter::builder()
                .with_tonic()
                .with_endpoint(endpoint)
                .build()
                .map_err(|e| ObservabilityError::ExporterInit(e.to_string()))?;

            TracerProvider::builder()
                .with_batch_exporter(exporter, opentelemetry_sdk::runtime::Tokio)
                .with_resource(resource)
                .build()
        } else {
            // No exporter, just create a basic provider
            TracerProvider::builder().with_resource(resource).build()
        };

        let tracer = tracer_provider.tracer(self.service_name.clone());

        // Build env filter
        let env_filter =
            EnvFilter::try_new(&self.log_level).unwrap_or_else(|_| EnvFilter::new("info"));

        // Build subscriber based on logging format
        // Note: telemetry layer must be added last so it sees all events
        if self.json_logging {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .json()
                .with_target(true)
                .with_thread_ids(true)
                .with_file(true)
                .with_line_number(true);

            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(telemetry_layer)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| ObservabilityError::SubscriberInit(e.to_string()))?;
        } else {
            let fmt_layer = tracing_subscriber::fmt::layer()
                .with_target(true)
                .with_thread_ids(false)
                .with_file(false)
                .with_line_number(false);

            let telemetry_layer = tracing_opentelemetry::layer().with_tracer(tracer);

            tracing_subscriber::registry()
                .with(env_filter)
                .with(telemetry_layer)
                .with(fmt_layer)
                .try_init()
                .map_err(|e| ObservabilityError::SubscriberInit(e.to_string()))?;
        }

        Ok(ObservabilityGuard {
            _tracer_provider: Some(tracer_provider),
        })
    }

    /// Build without OTLP - just tracing subscriber
    #[cfg(not(feature = "otel-otlp"))]
    pub fn build(self) -> Result<ObservabilityGuard, ObservabilityError> {
        // Without otel-otlp, we just set up basic tracing
        #[cfg(feature = "otel")]
        {
            // Just log a warning that OTLP is not enabled
            eprintln!(
                "Warning: otel-otlp feature not enabled, OTLP export disabled for {}",
                self.service_name
            );
        }

        Ok(ObservabilityGuard {
            #[cfg(feature = "otel-otlp")]
            _tracer_provider: None,
        })
    }
}

/// Guard that keeps the observability stack active
///
/// When dropped, flushes any pending spans to the exporter.
pub struct ObservabilityGuard {
    #[cfg(feature = "otel-otlp")]
    _tracer_provider: Option<opentelemetry_sdk::trace::TracerProvider>,
}

impl Drop for ObservabilityGuard {
    fn drop(&mut self) {
        #[cfg(feature = "otel-otlp")]
        if let Some(provider) = self._tracer_provider.take() {
            // Flush and shutdown the tracer provider
            if let Err(e) = provider.shutdown() {
                eprintln!("Error shutting down tracer provider: {:?}", e);
            }
        }
    }
}

/// Errors that can occur during observability setup
#[derive(Debug)]
pub enum ObservabilityError {
    /// Failed to initialize the exporter
    ExporterInit(String),
    /// Failed to initialize the subscriber
    SubscriberInit(String),
    /// Configuration error
    Config(String),
}

impl std::fmt::Display for ObservabilityError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ObservabilityError::ExporterInit(msg) => {
                write!(f, "Failed to initialize exporter: {}", msg)
            }
            ObservabilityError::SubscriberInit(msg) => {
                write!(f, "Failed to initialize subscriber: {}", msg)
            }
            ObservabilityError::Config(msg) => write!(f, "Configuration error: {}", msg),
        }
    }
}

impl std::error::Error for ObservabilityError {}

/// Type alias for the builder
pub type Observability = ObservabilityBuilder;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_creation() {
        let builder = ObservabilityBuilder::new("test-service");
        assert_eq!(builder.service_name, "test-service");
        assert!(builder.service_version.is_none());
        assert!(builder.environment.is_none());
        assert!(builder.otlp_endpoint.is_none());
        assert!(!builder.json_logging);
        assert_eq!(builder.log_level, "info");
    }

    #[test]
    fn test_builder_fluent_api() {
        let builder = ObservabilityBuilder::new("test-service")
            .service_version("1.0.0")
            .environment("production")
            .otlp_endpoint("http://localhost:4317")
            .json_logging()
            .log_level("debug");

        assert_eq!(builder.service_version, Some("1.0.0".to_string()));
        assert_eq!(builder.environment, Some("production".to_string()));
        assert_eq!(
            builder.otlp_endpoint,
            Some("http://localhost:4317".to_string())
        );
        assert!(builder.json_logging);
        assert_eq!(builder.log_level, "debug");
    }

    #[test]
    fn test_observability_error_display() {
        let err = ObservabilityError::ExporterInit("connection refused".into());
        assert!(err.to_string().contains("connection refused"));
    }
}
