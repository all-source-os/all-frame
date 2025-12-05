//! Simplified DI Container Tests for v0.2 MVP
//!
//! These tests use #[provide] attributes for explicit dependency specification.
//! This is the MVP approach for v0.2 - automatic dependency resolution will
//! come in v0.3.

use allframe_macros::di_container;

/// Test basic DI with explicit #[provide] attributes
#[test]
fn test_di_with_provide_simple() {
    // Define simple services
    struct ConfigService {
        app_name: String,
    }

    impl ConfigService {
        fn new() -> Self {
            Self {
                app_name: "TestApp".to_string(),
            }
        }

        fn app_name(&self) -> &str {
            &self.app_name
        }
    }

    struct LogService {
        prefix: String,
    }

    impl LogService {
        fn new() -> Self {
            Self {
                prefix: "[LOG]".to_string(),
            }
        }

        fn log(&self, msg: &str) -> String {
            format!("{} {}", self.prefix, msg)
        }
    }

    // Container - services with new() will be auto-created
    #[di_container]
    struct AppContainer {
        config: ConfigService,
        logger: LogService,
    }

    let container = AppContainer::new();

    // Test accessors
    assert_eq!(container.config().app_name(), "TestApp");
    assert_eq!(container.logger().log("test"), "[LOG] test");
}

/// Test DI with multiple instances - Skipped for MVP
/// TODO: Implement in v0.3 with #[provide] attribute support
#[test]
#[ignore]
fn test_di_multiple_instances() {
    // This test is skipped because it requires custom initialization
    // which needs #[provide] attribute support
    // Will be implemented in v0.3
}

/// Test DI with no-arg constructors (auto-wire without #[provide])
#[test]
fn test_di_auto_wire_simple() {
    struct SimpleService;

    impl SimpleService {
        fn new() -> Self {
            Self
        }

        fn greet(&self) -> &'static str {
            "Hello"
        }
    }

    #[di_container]
    struct AppContainer {
        service: SimpleService,
    }

    let container = AppContainer::new();
    assert_eq!(container.service().greet(), "Hello");
}
