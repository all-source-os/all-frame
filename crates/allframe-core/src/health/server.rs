//! HTTP health check server
//!
//! Provides a simple HTTP server that exposes health check endpoints.

use std::{convert::Infallible, future::Future, net::SocketAddr, pin::Pin, sync::Arc};

use hyper::{
    body::Incoming, server::conn::http1, service::service_fn, Method, Request, Response, StatusCode,
};
use hyper_util::rt::TokioIo;
use tokio::net::TcpListener;

use super::{HealthCheck, HealthReport};

/// HTTP health check server
///
/// Exposes `/health` and `/ready` endpoints for health monitoring.
pub struct HealthServer<H: HealthCheck + 'static> {
    health: Arc<H>,
    addr: SocketAddr,
}

impl<H: HealthCheck + 'static> HealthServer<H> {
    /// Create a new health server
    pub fn new(health: H) -> Self {
        Self {
            health: Arc::new(health),
            addr: ([0, 0, 0, 0], 8081).into(),
        }
    }

    /// Set the address to bind to
    pub fn addr(mut self, addr: impl Into<SocketAddr>) -> Self {
        self.addr = addr.into();
        self
    }

    /// Set the port (binds to 0.0.0.0)
    pub fn port(mut self, port: u16) -> Self {
        self.addr = ([0, 0, 0, 0], port).into();
        self
    }

    /// Start the health server
    ///
    /// This will block until the server is shut down.
    pub async fn serve(self) -> Result<(), HealthServerError> {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| HealthServerError::Bind(e.to_string()))?;

        loop {
            let (stream, _) = listener
                .accept()
                .await
                .map_err(|e| HealthServerError::Accept(e.to_string()))?;

            let io = TokioIo::new(stream);
            let health = Arc::clone(&self.health);

            tokio::spawn(async move {
                let service = service_fn(move |req| {
                    let health = Arc::clone(&health);
                    async move { handle_request(req, health).await }
                });

                if let Err(e) = http1::Builder::new().serve_connection(io, service).await {
                    // Connection errors are usually not fatal (client disconnected, etc.)
                    if !e.is_incomplete_message() {
                        #[cfg(feature = "otel")]
                        tracing::debug!(error = %e, "Health server connection error");
                    }
                }
            });
        }
    }

    /// Start the health server with graceful shutdown
    pub async fn serve_with_shutdown<F>(self, shutdown: F) -> Result<(), HealthServerError>
    where
        F: Future<Output = ()> + Send,
    {
        let listener = TcpListener::bind(self.addr)
            .await
            .map_err(|e| HealthServerError::Bind(e.to_string()))?;

        tokio::pin!(shutdown);

        loop {
            tokio::select! {
                _ = &mut shutdown => {
                    return Ok(());
                }
                result = listener.accept() => {
                    let (stream, _) = result
                        .map_err(|e| HealthServerError::Accept(e.to_string()))?;

                    let io = TokioIo::new(stream);
                    let health = Arc::clone(&self.health);

                    tokio::spawn(async move {
                        let service = service_fn(move |req| {
                            let health = Arc::clone(&health);
                            async move { handle_request(req, health).await }
                        });

                        let _ = http1::Builder::new().serve_connection(io, service).await;
                    });
                }
            }
        }
    }

    /// Get a health report without starting the server
    pub fn check(&self) -> Pin<Box<dyn Future<Output = HealthReport> + Send + '_>> {
        self.health.check_all()
    }
}

async fn handle_request<H: HealthCheck>(
    req: Request<Incoming>,
    health: Arc<H>,
) -> Result<Response<String>, Infallible> {
    let response = match (req.method(), req.uri().path()) {
        (&Method::GET, "/health") | (&Method::GET, "/healthz") => {
            let report = health.check_all().await;
            let status_code = match report.status.http_status_code() {
                200 => StatusCode::OK,
                503 => StatusCode::SERVICE_UNAVAILABLE,
                _ => StatusCode::INTERNAL_SERVER_ERROR,
            };
            let body = report
                .to_json()
                .unwrap_or_else(|_| r#"{"error":"serialization failed"}"#.to_string());

            Response::builder()
                .status(status_code)
                .header("Content-Type", "application/json")
                .body(body)
                .unwrap()
        }
        (&Method::GET, "/ready") | (&Method::GET, "/readyz") => {
            // Readiness check - just returns 200 if the server is running
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(r#"{"ready":true}"#.to_string())
                .unwrap()
        }
        (&Method::GET, "/live") | (&Method::GET, "/livez") => {
            // Liveness check - always returns 200 if the server is running
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(r#"{"alive":true}"#.to_string())
                .unwrap()
        }
        _ => Response::builder()
            .status(StatusCode::NOT_FOUND)
            .body("Not Found".to_string())
            .unwrap(),
    };

    Ok(response)
}

/// Errors that can occur when running the health server
#[derive(Debug, Clone)]
pub enum HealthServerError {
    /// Failed to bind to the address
    Bind(String),
    /// Failed to accept a connection
    Accept(String),
}

impl std::fmt::Display for HealthServerError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            HealthServerError::Bind(msg) => write!(f, "Failed to bind: {}", msg),
            HealthServerError::Accept(msg) => write!(f, "Failed to accept connection: {}", msg),
        }
    }
}

impl std::error::Error for HealthServerError {}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::health::{AlwaysHealthy, SimpleHealthCheck};

    #[tokio::test]
    async fn test_health_server_check() {
        let checker = SimpleHealthCheck::new().add_dependency(AlwaysHealthy::new("test"));
        let server = HealthServer::new(checker);
        let report = server.check().await;
        assert!(report.is_healthy());
    }

    #[test]
    fn test_health_server_error_display() {
        let err = HealthServerError::Bind("address in use".into());
        assert!(err.to_string().contains("address in use"));
    }
}
