//! tests/04_router_config.rs
//!
//! GREEN PHASE: Tests now pass with implementation
//!
//! Tests for configuration-driven protocol selection.
//! This is the key differentiator - same handler, multiple protocols via config.
//!
//! Acceptance criteria from PRD:
//! - "Same handler works as REST, GraphQL, gRPC via config"
//! - Config file determines which protocols are enabled
//! - No code changes needed to switch protocols
//! - Single handler callable via all configured protocols

use allframe_core::router::RouterConfig;

/// Test basic config loading
#[test]
fn test_load_router_config() {
    let config_toml = r#"
        [server]
        protocols = ["rest", "graphql"]

        [server.rest]
        port = 8080

        [server.graphql]
        port = 8081
    "#;

    let config = RouterConfig::from_str(config_toml).unwrap();

    assert_eq!(config.server.protocols.len(), 2);
    assert!(config.has_protocol("rest"));
    assert!(config.has_protocol("graphql"));
    assert!(!config.has_protocol("grpc"));
}

/// Test single handler with multiple protocols
#[test]
fn test_single_handler_multiple_protocols() {
    use allframe_core::router::Router;

    let config_toml = r#"
        [server]
        protocols = ["rest", "graphql", "grpc"]
    "#;

    let config = RouterConfig::from_str(config_toml).unwrap();
    let mut router = Router::with_config(config);

    // Register handler once
    router.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    // Should be callable via all configured protocols
    assert!(router.can_handle_rest("get_user"));
    assert!(router.can_handle_graphql("get_user"));
    assert!(router.can_handle_grpc("get_user"));
}

/// Test protocol-specific configuration
#[test]
fn test_protocol_specific_config() {
    let config_toml = r#"
        [server]
        protocols = ["rest", "graphql"]

        [server.rest]
        port = 8080
        path_prefix = "/api/v1"

        [server.graphql]
        port = 8081
        path = "/graphql"
        playground = true
    "#;

    let config = RouterConfig::from_str(config_toml).unwrap();

    let rest_config = config.server.rest.as_ref().unwrap();
    assert_eq!(rest_config.port, 8080);
    assert_eq!(rest_config.path_prefix, "/api/v1");

    let graphql_config = config.server.graphql.as_ref().unwrap();
    assert_eq!(graphql_config.port, 8081);
    assert_eq!(graphql_config.path, "/graphql");
    assert!(graphql_config.playground);
}

/// Test protocol enablement/disablement
#[test]
fn test_protocol_enablement() {
    use allframe_core::router::Router;

    // Config 1: Only REST enabled
    let config1 = r#"
        [server]
        protocols = ["rest"]
    "#;

    let mut router1 = Router::with_config(RouterConfig::from_str(config1).unwrap());
    router1.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    assert!(router1.can_handle_rest("get_user"));
    assert!(!router1.can_handle_graphql("get_user"));

    // Config 2: Only GraphQL enabled
    let config2 = r#"
        [server]
        protocols = ["graphql"]
    "#;

    let mut router2 = Router::with_config(RouterConfig::from_str(config2).unwrap());
    router2.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    assert!(!router2.can_handle_rest("get_user"));
    assert!(router2.can_handle_graphql("get_user"));
}

/// Test end-to-end multi-protocol execution
#[tokio::test]
async fn test_e2e_multi_protocol() {
    use allframe_core::router::Router;

    let config_toml = r#"
        [server]
        protocols = ["rest", "graphql", "grpc"]

        [server.rest]
        port = 8080

        [server.graphql]
        port = 8081

        [server.grpc]
        port = 9090
    "#;

    let config = RouterConfig::from_str(config_toml).unwrap();
    let mut router = Router::with_config(config);

    // Register handler once
    router.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });

    // Call via REST
    let rest_response = router.call_rest("GET", "/users/42").await.unwrap();
    // MVP implementation returns mock response
    assert!(rest_response.contains("REST handled"));
    assert!(rest_response.contains("/users/42"));

    // Call via GraphQL
    let graphql_query = r#"query { user(id: 42) { name } }"#;
    let graphql_response = router.call_graphql(graphql_query).await.unwrap();
    // MVP implementation returns mock JSON response
    assert!(graphql_response.contains("data"));
    assert!(graphql_response.contains("John Doe"));

    // Call via gRPC
    let grpc_request = r#"{"id": 42}"#;
    let grpc_response = router.call_grpc("GetUser", grpc_request).await.unwrap();
    // MVP implementation returns mock JSON response
    assert!(grpc_response.contains("John Doe"));
    assert!(grpc_response.contains("id"));
}

/// Test that changing config doesn't require code changes
#[test]
fn test_config_change_no_code_change() {
    // This test demonstrates that we can switch from one protocol to another
    // just by changing the config, without touching handler code
    use allframe_core::router::Router;

    // Config 1: REST only
    let config1 = RouterConfig::from_str(r#"
        [server]
        protocols = ["rest"]
    "#).unwrap();

    let mut router1 = Router::with_config(config1);
    router1.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });
    assert!(router1.enabled_protocols().contains(&"rest".to_string()));
    assert_eq!(router1.enabled_protocols().len(), 1);

    // Config 2: All protocols (same handler code)
    let config2 = RouterConfig::from_str(r#"
        [server]
        protocols = ["rest", "graphql", "grpc"]
    "#).unwrap();

    let mut router2 = Router::with_config(config2);
    router2.register("get_user", || async move {
        r#"{"id": 42, "name": "John Doe"}"#.to_string()
    });
    assert_eq!(router2.enabled_protocols().len(), 3);
}
