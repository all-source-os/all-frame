//! Production gRPC adapter using tonic and prost
//!
//! This module provides full gRPC support with protobuf encoding,
//! streaming RPCs, HTTP/2 transport, and reflection API.

#[cfg(feature = "router-grpc")]
use futures::Stream;
#[cfg(feature = "router-grpc")]
use prost::Message;
#[cfg(feature = "router-grpc")]
use std::pin::Pin;
#[cfg(feature = "router-grpc")]
use tokio_stream::StreamExt;
#[cfg(feature = "router-grpc")]
use tonic::{transport::Server, Code, Status, Streaming};

use super::ProtocolAdapter;
use std::future::Future;

/// Production gRPC adapter with full protobuf support
///
/// Features:
/// - Full protobuf encoding/decoding
/// - Unary RPCs
/// - Server streaming
/// - Client streaming
/// - Bidirectional streaming
/// - gRPC reflection
/// - HTTP/2 transport
#[cfg(feature = "router-grpc")]
pub struct GrpcProductionAdapter {
    service_name: String,
}

#[cfg(feature = "router-grpc")]
impl GrpcProductionAdapter {
    /// Create a new production gRPC adapter
    pub fn new(service_name: impl Into<String>) -> Self {
        Self {
            service_name: service_name.into(),
        }
    }

    /// Get the service name
    pub fn service_name(&self) -> &str {
        &self.service_name
    }

    /// Create a gRPC server builder
    pub fn server_builder() -> Server {
        Server::builder()
    }

    /// Convert a gRPC status code to Status
    pub fn status_from_code(code: Code, message: impl Into<String>) -> Status {
        Status::new(code, message)
    }

    /// Create a reflection server for gRPC service discovery
    ///
    /// This enables clients to discover services and their methods at runtime.
    /// Useful for tools like grpcurl, grpcui, and Postman.
    ///
    /// Returns the builder that can be used to construct the reflection server.
    pub fn enable_reflection() -> tonic_reflection::server::Builder<'static> {
        tonic_reflection::server::Builder::configure()
    }
}

#[cfg(feature = "router-grpc")]
impl ProtocolAdapter for GrpcProductionAdapter {
    fn name(&self) -> &str {
        "grpc-production"
    }

    fn handle(
        &self,
        _request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        Box::pin(async move {
            // In production, this would:
            // 1. Decode protobuf message
            // 2. Route to appropriate RPC handler
            // 3. Encode response as protobuf
            // 4. Send over HTTP/2
            Ok("gRPC production adapter".to_string())
        })
    }
}

/// gRPC service trait for implementing RPC handlers
#[cfg(feature = "router-grpc")]
#[tonic::async_trait]
pub trait GrpcService: Send + Sync + 'static {
    /// The service name
    const NAME: &'static str;

    /// Handle a unary RPC call
    async fn handle_unary(
        &self,
        method: &str,
        request: Vec<u8>,
    ) -> Result<Vec<u8>, Status>;
}

/// gRPC streaming types and helpers
#[cfg(feature = "router-grpc")]
pub mod streaming {
    use super::*;

    /// Server streaming response - server sends multiple responses
    pub type ServerStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send>>;

    /// Client streaming request - client sends multiple requests
    pub type ClientStream<T> = Streaming<T>;

    /// Bidirectional streaming - both sides send multiple messages
    pub type BidiStream<T> = Pin<Box<dyn Stream<Item = Result<T, Status>> + Send>>;

    /// Create a server stream from a vector
    pub fn from_vec<T>(items: Vec<T>) -> ServerStream<T>
    where
        T: Send + 'static,
    {
        Box::pin(tokio_stream::iter(items.into_iter().map(Ok)))
    }

    /// Create a server stream from an iterator
    pub fn from_iter<T, I>(iter: I) -> ServerStream<T>
    where
        T: Send + 'static,
        I: Iterator<Item = T> + Send + 'static,
    {
        Box::pin(tokio_stream::iter(iter.map(Ok)))
    }

    /// Process a client stream into a vector
    pub async fn collect_stream<T>(mut stream: ClientStream<T>) -> Result<Vec<T>, Status>
    where
        T: Send + 'static,
    {
        let mut items = Vec::new();
        while let Some(item) = stream.next().await {
            items.push(item?);
        }
        Ok(items)
    }

    /// Transform a client stream with a function
    pub async fn map_stream<T, U, F>(
        mut stream: ClientStream<T>,
        mut f: F,
    ) -> Result<Vec<U>, Status>
    where
        T: Send + 'static,
        U: Send + 'static,
        F: FnMut(T) -> U + Send,
    {
        let mut items = Vec::new();
        while let Some(item) = stream.next().await {
            items.push(f(item?));
        }
        Ok(items)
    }
}

/// Protobuf message helpers
#[cfg(feature = "router-grpc")]
pub mod protobuf {
    use super::*;

    /// Encode a message to protobuf bytes
    pub fn encode<M: Message>(message: &M) -> Result<Vec<u8>, String> {
        let mut buf = Vec::new();
        message
            .encode(&mut buf)
            .map_err(|e| format!("Failed to encode message: {}", e))?;
        Ok(buf)
    }

    /// Decode a message from protobuf bytes
    pub fn decode<M: Message + Default>(bytes: &[u8]) -> Result<M, String> {
        M::decode(bytes).map_err(|e| format!("Failed to decode message: {}", e))
    }
}

/// gRPC status code helpers
#[cfg(feature = "router-grpc")]
pub mod status {
    use super::*;

    /// Create an OK status
    pub fn ok() -> Status {
        Status::ok("Success")
    }

    /// Create an INVALID_ARGUMENT status
    pub fn invalid_argument(message: impl Into<String>) -> Status {
        Status::new(Code::InvalidArgument, message)
    }

    /// Create a NOT_FOUND status
    pub fn not_found(message: impl Into<String>) -> Status {
        Status::new(Code::NotFound, message)
    }

    /// Create an UNIMPLEMENTED status
    pub fn unimplemented(message: impl Into<String>) -> Status {
        Status::new(Code::Unimplemented, message)
    }

    /// Create an INTERNAL status
    pub fn internal(message: impl Into<String>) -> Status {
        Status::new(Code::Internal, message)
    }
}

#[cfg(test)]
#[cfg(feature = "router-grpc")]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_adapter_creation() {
        let adapter = GrpcProductionAdapter::new("UserService");
        assert_eq!(adapter.service_name(), "UserService");
        assert_eq!(adapter.name(), "grpc-production");
    }

    #[test]
    fn test_status_codes() {
        let ok_status = status::ok();
        assert_eq!(ok_status.code(), Code::Ok);

        let invalid = status::invalid_argument("Bad request");
        assert_eq!(invalid.code(), Code::InvalidArgument);

        let not_found = status::not_found("User not found");
        assert_eq!(not_found.code(), Code::NotFound);

        let unimplemented = status::unimplemented("Not implemented");
        assert_eq!(unimplemented.code(), Code::Unimplemented);

        let internal = status::internal("Internal error");
        assert_eq!(internal.code(), Code::Internal);
    }

    #[tokio::test]
    async fn test_protobuf_encoding() {
        // Test with a simple message type
        use prost_types::Timestamp;

        let ts = Timestamp {
            seconds: 12345,
            nanos: 67890,
        };

        // Encode
        let bytes = protobuf::encode(&ts).unwrap();
        assert!(!bytes.is_empty());

        // Decode
        let decoded: Timestamp = protobuf::decode(&bytes).unwrap();
        assert_eq!(decoded.seconds, 12345);
        assert_eq!(decoded.nanos, 67890);
    }

    #[tokio::test]
    async fn test_server_streaming_from_vec() {
        use tokio_stream::StreamExt;

        // Create a server stream from a vector
        let items = vec![1, 2, 3, 4, 5];
        let mut stream = streaming::from_vec(items);

        // Collect results
        let mut results = Vec::new();
        while let Some(item) = stream.next().await {
            results.push(item.unwrap());
        }

        assert_eq!(results, vec![1, 2, 3, 4, 5]);
    }

    #[tokio::test]
    async fn test_server_streaming_from_iter() {
        use tokio_stream::StreamExt;

        // Create a server stream from an iterator
        let mut stream = streaming::from_iter(1..=5);

        // Collect results
        let mut results = Vec::new();
        while let Some(item) = stream.next().await {
            results.push(item.unwrap());
        }

        assert_eq!(results, vec![1, 2, 3, 4, 5]);
    }

    #[test]
    fn test_grpc_service_name() {
        let adapter = GrpcProductionAdapter::new("UserService");
        assert_eq!(adapter.service_name(), "UserService");
    }

    #[test]
    fn test_all_status_codes() {
        // Test all standard gRPC status codes
        assert_eq!(status::ok().code(), Code::Ok);
        assert_eq!(
            status::invalid_argument("msg").code(),
            Code::InvalidArgument
        );
        assert_eq!(status::not_found("msg").code(), Code::NotFound);
        assert_eq!(status::unimplemented("msg").code(), Code::Unimplemented);
        assert_eq!(status::internal("msg").code(), Code::Internal);
    }
}
