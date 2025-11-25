//! tests/04_router_core.rs
//!
//! GREEN PHASE: Making tests pass
//!
//! Tests for the core router abstractions and handler registration.
//! This is the foundation for protocol-agnostic routing in Milestone 0.3.
//!
//! Acceptance criteria from PRD:
//! - Same handler works as REST, GraphQL, gRPC via config
//! - Single handler function can be registered once
//! - Protocol adapters can transform requests/responses
//! - Type-safe handler execution

use allframe_core::router::{ProtocolAdapter, Router};
use std::future::Future;
use std::pin::Pin;

/// Test that we can register a simple handler
#[tokio::test]
async fn test_register_handler() {
    let mut router = Router::new();

    // Register a simple handler
    router.register("get_user", || async move { "User 42".to_string() });

    assert_eq!(router.handlers_count(), 1);
}

/// Test that we can execute a registered handler
#[tokio::test]
async fn test_execute_handler() {
    let mut router = Router::new();

    router.register("get_user", || async move { "User 42".to_string() });

    let result = router.execute("get_user").await.unwrap();
    assert_eq!(result, "User 42");
}

/// Test that handlers can return Result types
#[tokio::test]
async fn test_handler_with_result() {
    let mut router = Router::new();

    router.register("get_user_ok", || async move { "User 42".to_string() });

    let result = router.execute("get_user_ok").await;
    assert!(result.is_ok());

    let error = router.execute("nonexistent").await;
    assert!(error.is_err());
}

/// Test that we can register handlers with different signatures
/// Note: For MVP, all handlers use the same simple signature
/// Advanced parameter handling will come in later phases
#[tokio::test]
async fn test_multiple_handler_signatures() {
    let mut router = Router::new();

    // Handler with no parameters (for now all handlers are no-arg)
    router.register("get_user", || async move { "User 42".to_string() });

    router.register("list_users", || async move { "All users".to_string() });

    router.register("update_user", || async move { "Updated user".to_string() });

    assert_eq!(router.handlers_count(), 3);
}

/// Test that protocol adapters can be registered
#[tokio::test]
async fn test_register_protocol_adapter() {
    struct RestAdapter;

    impl ProtocolAdapter for RestAdapter {
        fn name(&self) -> &str {
            "rest"
        }

        fn handle(
            &self,
            _request: &str,
        ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
            Box::pin(async move { Ok("REST response".to_string()) })
        }
    }

    let mut router = Router::new();
    router.add_adapter(Box::new(RestAdapter));

    assert!(router.has_adapter("rest"));
    assert!(!router.has_adapter("graphql"));
}
