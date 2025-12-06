//! TLS configuration for gRPC servers
//!
//! This module provides TLS configuration options for secure gRPC connections.

use std::env;
use std::path::PathBuf;

/// TLS configuration for gRPC servers
#[derive(Debug, Clone)]
pub struct TlsConfig {
    /// Path to the server certificate file (PEM format)
    pub cert_path: PathBuf,
    /// Path to the server private key file (PEM format)
    pub key_path: PathBuf,
    /// Optional path to client CA certificate for mTLS
    pub client_ca_path: Option<PathBuf>,
}

impl TlsConfig {
    /// Create a new TLS configuration
    pub fn new(cert_path: impl Into<PathBuf>, key_path: impl Into<PathBuf>) -> Self {
        Self {
            cert_path: cert_path.into(),
            key_path: key_path.into(),
            client_ca_path: None,
        }
    }

    /// Enable mutual TLS (mTLS) with a client CA certificate
    pub fn with_client_ca(mut self, client_ca_path: impl Into<PathBuf>) -> Self {
        self.client_ca_path = Some(client_ca_path.into());
        self
    }

    /// Create TLS configuration from environment variables
    ///
    /// Reads from:
    /// - `GRPC_TLS_CERT` or `TLS_CERT_PATH` - Certificate path
    /// - `GRPC_TLS_KEY` or `TLS_KEY_PATH` - Key path
    /// - `GRPC_TLS_CLIENT_CA` or `TLS_CLIENT_CA_PATH` - Optional client CA path
    pub fn from_env() -> Option<Self> {
        let cert_path = env::var("GRPC_TLS_CERT")
            .or_else(|_| env::var("TLS_CERT_PATH"))
            .ok()?;

        let key_path = env::var("GRPC_TLS_KEY")
            .or_else(|_| env::var("TLS_KEY_PATH"))
            .ok()?;

        let client_ca_path = env::var("GRPC_TLS_CLIENT_CA")
            .or_else(|_| env::var("TLS_CLIENT_CA_PATH"))
            .ok()
            .map(PathBuf::from);

        Some(Self {
            cert_path: PathBuf::from(cert_path),
            key_path: PathBuf::from(key_path),
            client_ca_path,
        })
    }

    /// Load the certificate and key files
    #[cfg(feature = "grpc-tls")]
    pub fn load(&self) -> Result<(Vec<u8>, Vec<u8>), TlsError> {
        let cert = std::fs::read(&self.cert_path).map_err(|e| TlsError::CertificateLoad {
            path: self.cert_path.clone(),
            source: e.to_string(),
        })?;

        let key = std::fs::read(&self.key_path).map_err(|e| TlsError::KeyLoad {
            path: self.key_path.clone(),
            source: e.to_string(),
        })?;

        Ok((cert, key))
    }

    /// Load the client CA certificate if configured
    #[cfg(feature = "grpc-tls")]
    pub fn load_client_ca(&self) -> Result<Option<Vec<u8>>, TlsError> {
        match &self.client_ca_path {
            Some(path) => {
                let ca = std::fs::read(path).map_err(|e| TlsError::ClientCaLoad {
                    path: path.clone(),
                    source: e.to_string(),
                })?;
                Ok(Some(ca))
            }
            None => Ok(None),
        }
    }
}

/// Errors that can occur during TLS configuration
#[derive(Debug)]
pub enum TlsError {
    /// Failed to load certificate file
    CertificateLoad {
        /// Path that failed to load
        path: PathBuf,
        /// Error details
        source: String,
    },
    /// Failed to load key file
    KeyLoad {
        /// Path that failed to load
        path: PathBuf,
        /// Error details
        source: String,
    },
    /// Failed to load client CA file
    ClientCaLoad {
        /// Path that failed to load
        path: PathBuf,
        /// Error details
        source: String,
    },
    /// Failed to configure TLS
    Configuration(String),
}

impl std::fmt::Display for TlsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TlsError::CertificateLoad { path, source } => {
                write!(f, "Failed to load certificate from {:?}: {}", path, source)
            }
            TlsError::KeyLoad { path, source } => {
                write!(f, "Failed to load key from {:?}: {}", path, source)
            }
            TlsError::ClientCaLoad { path, source } => {
                write!(f, "Failed to load client CA from {:?}: {}", path, source)
            }
            TlsError::Configuration(msg) => write!(f, "TLS configuration error: {}", msg),
        }
    }
}

impl std::error::Error for TlsError {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tls_config_new() {
        let config = TlsConfig::new("/path/to/cert.pem", "/path/to/key.pem");
        assert_eq!(config.cert_path, PathBuf::from("/path/to/cert.pem"));
        assert_eq!(config.key_path, PathBuf::from("/path/to/key.pem"));
        assert!(config.client_ca_path.is_none());
    }

    #[test]
    fn test_tls_config_with_client_ca() {
        let config = TlsConfig::new("/path/to/cert.pem", "/path/to/key.pem")
            .with_client_ca("/path/to/ca.pem");

        assert_eq!(
            config.client_ca_path,
            Some(PathBuf::from("/path/to/ca.pem"))
        );
    }

    #[test]
    fn test_tls_error_display() {
        let err = TlsError::CertificateLoad {
            path: PathBuf::from("/path/to/cert.pem"),
            source: "file not found".to_string(),
        };
        assert!(err.to_string().contains("cert.pem"));
        assert!(err.to_string().contains("file not found"));
    }
}
