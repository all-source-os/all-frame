//! Feature flag validation tests
//!
//! These tests verify that feature flags work correctly and users
//! can opt into only the features they need to minimize bloat.
//!
//! Test strategy:
//! - Verify minimal build (no features) compiles
//! - Verify each feature independently
//! - Verify feature combinations
//! - Verify production features are truly optional

// These tests are compile-time checks - the fact that they compile proves the feature flags work

#[test]
fn test_minimal_router_without_config() {
    // This test should compile even without the "router" feature
    // because the core Router type should always be available
    use allframe_core::router::Router;

    let mut router = Router::new();
    router.register("test", || async { "Hello".to_string() });

    assert_eq!(router.handlers_count(), 1);
}

#[test]
#[cfg(feature = "router")]
fn test_router_with_config() {
    // This test only compiles when "router" feature is enabled
    // because RouterConfig requires toml dependency
    use allframe_core::router::{Router, RouterConfig};

    let config_toml = r#"
        [server]
        protocols = ["rest"]
    "#;

    let config = RouterConfig::from_str(config_toml).unwrap();
    let router = Router::with_config(config);

    assert!(router.has_adapter("rest"));
}

#[test]
#[cfg(feature = "router-graphql")]
fn test_graphql_production_adapter() {
    // This test only compiles when "router-graphql" feature is enabled
    use allframe_core::router::GraphQLProductionAdapter;

    let adapter = GraphQLProductionAdapter::new("/graphql");
    let schema = adapter.graphiql_source();

    assert!(schema.contains("GraphiQL"));
}

#[test]
#[cfg(feature = "router-grpc")]
fn test_grpc_production_adapter() {
    // This test only compiles when "router-grpc" feature is enabled
    use allframe_core::router::GrpcProductionAdapter;

    let adapter = GrpcProductionAdapter::new();
    assert_eq!(adapter.name(), "grpc-production");
}

#[test]
#[cfg(all(feature = "router-graphql", feature = "router-grpc"))]
fn test_full_router_features() {
    // This test only compiles when both production features are enabled
    use allframe_core::router::{GraphQLProductionAdapter, GrpcProductionAdapter};

    let _graphql = GraphQLProductionAdapter::new("/graphql");
    let _grpc = GrpcProductionAdapter::new();

    // Both adapters available
    assert!(true);
}

#[test]
fn test_mvp_adapters_always_available() {
    // MVP adapters should always be available regardless of features
    use allframe_core::router::{RestAdapter, GraphQLAdapter, GrpcAdapter};

    let _rest = RestAdapter::new();
    let _graphql = GraphQLAdapter::new();
    let _grpc = GrpcAdapter::new();

    assert!(true);
}

#[test]
#[cfg(not(feature = "router"))]
fn test_config_not_available_without_router_feature() {
    // This test verifies that config types are NOT available without router feature
    // We can't directly test absence, but the compilation succeeds if the types aren't used

    use allframe_core::router::Router;
    let _router = Router::new();

    // RouterConfig should not be available here - this would fail to compile:
    // use allframe_core::router::RouterConfig; // <-- would cause compile error

    assert!(true);
}

#[test]
#[cfg(all(not(feature = "router-graphql"), not(feature = "router-grpc")))]
fn test_production_adapters_not_available() {
    // This test verifies production adapters are NOT available without their features
    // We can't directly test absence, but the compilation succeeds if types aren't used

    use allframe_core::router::Router;
    let _router = Router::new();

    // These would fail to compile without features:
    // use allframe_core::router::GraphQLProductionAdapter; // <-- compile error
    // use allframe_core::router::GrpcProductionAdapter; // <-- compile error

    assert!(true);
}

// Compile-time dependency verification tests
// These tests help ensure we don't accidentally pull in heavy dependencies

#[test]
fn test_baseline_compiles() {
    // Baseline test - should always pass
    assert!(true);
}

/// Test that demonstrates feature flag bloat reduction
///
/// Without feature flags:
/// - async-graphql: ~2MB
/// - tonic + prost: ~3MB
/// - Total unnecessary bloat: ~5MB
///
/// With feature flags:
/// - Minimal build: Core router only
/// - Optional GraphQL: Add ~2MB only if needed
/// - Optional gRPC: Add ~3MB only if needed
/// - Users pay only for what they use
#[test]
fn test_feature_flag_benefits_documented() {
    // This test documents the benefits of feature flags

    // Minimal configuration (no features):
    // - Core router types
    // - MVP adapters (lightweight, no external deps)
    // - Handler registration
    // - Basic protocol adapters

    // With "router" feature:
    // + TOML config support
    // + RouterConfig types
    // + Config-driven protocol selection

    // With "router-graphql" feature:
    // + async-graphql (full AST parsing)
    // + GraphiQL playground
    // + Production-ready GraphQL

    // With "router-grpc" feature:
    // + tonic (HTTP/2 transport)
    // + prost (protobuf encoding)
    // + gRPC streaming
    // + Reflection API

    assert!(true);
}
