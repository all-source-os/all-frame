//! tests/07_otel_integration.rs
//!
//! RED PHASE: These tests MUST fail initially
//!
//! Integration tests for OpenTelemetry with other AllFrame features.
//! OTEL should work seamlessly with Router, Clean Architecture, and CQRS.
//!
//! Acceptance criteria from PRD:
//! - All router protocols (REST/GraphQL/gRPC) are traced
//! - Each architecture layer creates separate spans
//! - CQRS commands/events/queries are traced
//! - Performance overhead < 5%
//! - Configuration-driven setup works

/// Test OTEL with protocol-agnostic router
#[tokio::test]
async fn test_otel_with_router() {
    // This test will fail because router integration doesn't exist yet
    //
    // use allframe_core::router::{Router, RestAdapter};
    // use allframe_core::otel::{traced, SpanRecorder};
    //
    // #[traced]
    // async fn handle_request() -> String {
    //     "Hello".to_string()
    // }
    //
    // let router = Router::new()
    //     .protocol(RestAdapter::new());
    //
    // let recorder = SpanRecorder::new();
    //
    // // Simulate REST request
    // router.handle_rest("/api/users").await;
    //
    // let spans = recorder.spans();
    //
    // // Should have spans for: router -> adapter -> handler
    // assert!(spans.len() >= 3);
    // assert!(spans.iter().any(|s| s.name.contains("router")));
    // assert!(spans.iter().any(|s| s.name.contains("rest_adapter")));
    // assert!(spans.iter().any(|s| s.name.contains("handler")));

    panic!("OTEL with router not implemented yet - RED PHASE");
}

/// Test OTEL with Clean Architecture layers
#[tokio::test]
async fn test_otel_with_clean_arch() {
    // This test will fail because layer tracing doesn't exist yet
    //
    // use allframe_core::arch::{handler, use_case, repository, domain};
    // use allframe_core::otel::{traced, SpanRecorder};
    // use std::sync::Arc;
    //
    // #[domain]
    // struct User {
    //     id: String,
    // }
    //
    // #[repository]
    // #[async_trait::async_trait]
    // trait UserRepository: Send + Sync {
    //     #[traced]
    //     async fn find(&self, id: &str) -> Option<User>;
    // }
    //
    // struct InMemoryUserRepository;
    //
    // #[async_trait::async_trait]
    // impl UserRepository for InMemoryUserRepository {
    //     async fn find(&self, id: &str) -> Option<User> {
    //         Some(User { id: id.to_string() })
    //     }
    // }
    //
    // #[use_case]
    // struct GetUserUseCase {
    //     repo: Arc<dyn UserRepository>,
    // }
    //
    // impl GetUserUseCase {
    //     #[traced]
    //     async fn execute(&self, id: &str) -> Option<User> {
    //         self.repo.find(id).await
    //     }
    // }
    //
    // #[handler]
    // #[traced]
    // async fn get_user_handler(use_case: Arc<GetUserUseCase>) -> Option<User> {
    //     use_case.execute("123").await
    // }
    //
    // let recorder = SpanRecorder::new();
    // let repo = Arc::new(InMemoryUserRepository);
    // let use_case = Arc::new(GetUserUseCase { repo });
    //
    // get_user_handler(use_case).await;
    //
    // let spans = recorder.spans();
    //
    // // Each layer should create a span
    // assert!(spans.iter().any(|s| s.layer == "handler"));
    // assert!(spans.iter().any(|s| s.layer == "use_case"));
    // assert!(spans.iter().any(|s| s.layer == "repository"));

    panic!("OTEL with Clean Architecture not implemented yet - RED PHASE");
}

/// Test OTEL with CQRS
#[tokio::test]
async fn test_otel_with_cqrs() {
    // This test will fail because CQRS tracing doesn't exist yet
    //
    // use allframe_core::cqrs::{command, command_handler, Event, EventStore};
    // use allframe_core::otel::{traced, SpanRecorder};
    //
    // #[derive(Clone)]
    // enum UserEvent {
    //     Created { user_id: String },
    // }
    //
    // impl Event for UserEvent {}
    //
    // #[command]
    // struct CreateUserCommand {
    //     user_id: String,
    // }
    //
    // #[command_handler]
    // #[traced]
    // async fn handle_create_user(
    //     cmd: CreateUserCommand,
    //     store: &EventStore<UserEvent>
    // ) -> Result<(), String> {
    //     let event = UserEvent::Created {
    //         user_id: cmd.user_id.clone(),
    //     };
    //     store.append(&cmd.user_id, vec![event]).await?;
    //     Ok(())
    // }
    //
    // let recorder = SpanRecorder::new();
    // let store = EventStore::new();
    //
    // handle_create_user(
    //     CreateUserCommand { user_id: "123".to_string() },
    //     &store
    // ).await.unwrap();
    //
    // let spans = recorder.spans();
    //
    // // Should have spans for: command_handler -> event_store
    // assert!(spans.iter().any(|s| s.name.contains("handle_create_user")));
    // assert!(spans.iter().any(|s| s.name.contains("event_store.append")));

    panic!("OTEL with CQRS not implemented yet - RED PHASE");
}

/// Test OTEL performance overhead
#[tokio::test]
async fn test_otel_performance_overhead() {
    // This test will fail because performance testing doesn't exist yet
    //
    // use allframe_core::otel::{traced, enable_tracing, disable_tracing};
    // use std::time::Instant;
    //
    // #[traced]
    // async fn fast_operation() -> u64 {
    //     // Simulate light work
    //     let mut sum = 0u64;
    //     for i in 0..1000 {
    //         sum += i;
    //     }
    //     sum
    // }
    //
    // // Measure without tracing
    // disable_tracing();
    // let start = Instant::now();
    // for _ in 0..10000 {
    //     fast_operation().await;
    // }
    // let without_tracing = start.elapsed();
    //
    // // Measure with tracing
    // enable_tracing();
    // let start = Instant::now();
    // for _ in 0..10000 {
    //     fast_operation().await;
    // }
    // let with_tracing = start.elapsed();
    //
    // // Calculate overhead percentage
    // let overhead_pct = ((with_tracing.as_micros() as f64 - without_tracing.as_micros() as f64)
    //     / without_tracing.as_micros() as f64) * 100.0;
    //
    // // Overhead should be less than 5%
    // assert!(overhead_pct < 5.0, "Overhead: {}%", overhead_pct);

    panic!("OTEL performance overhead testing not implemented yet - RED PHASE");
}

/// Test OTEL configuration from config file
#[tokio::test]
async fn test_otel_configuration() {
    // This test will fail because config-driven setup doesn't exist yet
    //
    // use allframe_core::otel::{configure_from_file, get_config};
    //
    // // Load config from TOML
    // configure_from_file("tests/fixtures/otel_config.toml").await.unwrap();
    //
    // let config = get_config();
    //
    // assert_eq!(config.service_name, "allframe-test");
    // assert_eq!(config.exporter_type, "jaeger");
    // assert_eq!(config.sampling_rate, 1.0);
    // assert_eq!(config.batch_size, 512);

    panic!("OTEL configuration not implemented yet - RED PHASE");
}
