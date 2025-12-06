//! gRPC Server Infrastructure
//!
//! This module provides a fluent builder API for configuring and running
//! gRPC servers with support for:
//!
//! - TLS/mTLS encryption
//! - gRPC reflection (service discovery)
//! - Health checking
//! - Graceful shutdown
//!
//! # Quick Start
//!
//! ```rust,ignore
//! use allframe_core::grpc::GrpcServer;
//! use allframe_core::shutdown::GracefulShutdown;
//!
//! // Simple server
//! GrpcServer::builder()
//!     .addr("[::1]:50051")
//!     .serve(my_service)
//!     .await?;
//!
//! // Full-featured server
//! let shutdown = GracefulShutdown::new();
//!
//! GrpcServer::builder()
//!     .addr("0.0.0.0:50051")
//!     .reflection(FILE_DESCRIPTOR_SET)
//!     .health_check()
//!     .tls_from_env()
//!     .graceful_shutdown(shutdown)
//!     .serve(my_service)
//!     .await?;
//! ```
//!
//! # TLS Configuration
//!
//! TLS can be configured either directly or from environment variables:
//!
//! ```rust,ignore
//! use allframe_core::grpc::{GrpcServer, TlsConfig};
//!
//! // Direct configuration
//! let tls = TlsConfig::new("cert.pem", "key.pem")
//!     .with_client_ca("ca.pem"); // For mTLS
//!
//! GrpcServer::builder()
//!     .tls(tls)
//!     .serve(my_service)
//!     .await?;
//!
//! // From environment variables
//! // GRPC_TLS_CERT, GRPC_TLS_KEY, GRPC_TLS_CLIENT_CA
//! GrpcServer::builder()
//!     .tls_from_env()
//!     .serve(my_service)
//!     .await?;
//! ```
//!
//! # gRPC Reflection
//!
//! Enable service discovery with gRPC reflection:
//!
//! ```rust,ignore
//! // Include the file descriptor set from your build
//! pub const FILE_DESCRIPTOR_SET: &[u8] =
//!     include_bytes!(concat!(env!("OUT_DIR"), "/proto_descriptor.bin"));
//!
//! GrpcServer::builder()
//!     .reflection(FILE_DESCRIPTOR_SET)
//!     .serve(my_service)
//!     .await?;
//! ```
//!
//! # Health Checking
//!
//! Enable the standard gRPC health checking protocol:
//!
//! ```rust,ignore
//! GrpcServer::builder()
//!     .health_check()
//!     .serve(my_service)
//!     .await?;
//! ```

mod server;
mod tls;

pub use server::{GrpcServer, GrpcServerBuilder, GrpcServerError};
pub use tls::{TlsConfig, TlsError};
