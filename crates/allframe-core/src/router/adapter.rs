//! Protocol adapter trait for supporting multiple protocols

use std::future::Future;
use std::pin::Pin;

/// Protocol adapter trait
///
/// Adapters implement this trait to provide protocol-specific
/// request/response handling while using the unified handler interface.
pub trait ProtocolAdapter: Send + Sync {
    /// Get the name of this protocol adapter
    fn name(&self) -> &str;

    /// Handle a protocol-specific request
    ///
    /// The adapter is responsible for:
    /// - Parsing the incoming request format
    /// - Extracting parameters for the handler
    /// - Calling the appropriate handler
    /// - Formatting the response in protocol-specific format
    fn handle(
        &self,
        request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
}

#[cfg(test)]
mod tests {
    use super::*;

    struct TestAdapter;

    impl ProtocolAdapter for TestAdapter {
        fn name(&self) -> &str {
            "test"
        }

        fn handle(
            &self,
            request: &str,
        ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
            let response = format!("Handled: {}", request);
            Box::pin(async move { Ok(response) })
        }
    }

    #[tokio::test]
    async fn test_protocol_adapter() {
        let adapter = TestAdapter;
        assert_eq!(adapter.name(), "test");

        let result = adapter.handle("test request").await;
        assert_eq!(result, Ok("Handled: test request".to_string()));
    }
}
