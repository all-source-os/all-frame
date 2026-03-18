//! Integration tests for allframe-forge scaffolding
//!
//! Tests that each archetype scaffolds a valid project structure.

use allframe_forge::config::{
    AntiCorruptionLayerConfig, ConsumerConfig, GatewayConfig, ProducerConfig, ProjectConfig,
    SagaOrchestratorConfig, ScheduledConfig, WebSocketGatewayConfig,
};
use allframe_forge::scaffolding;
use allframe_forge::validation::validate_project_name;
use tempfile::TempDir;

fn make_config(name: &str) -> ProjectConfig {
    let mut config = ProjectConfig::default();
    config.name = name.to_string();
    config
}

// --- Basic archetype ---

#[test]
fn test_scaffold_basic_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-basic-app");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_directory_structure(&project_path).unwrap();
    scaffolding::generate_files(&project_path, "my-basic-app").unwrap();

    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src/main.rs").exists());
    assert!(project_path.join("src/domain").is_dir());
    assert!(project_path.join("src/application").is_dir());
    assert!(project_path.join("src/infrastructure").is_dir());
    assert!(project_path.join("src/presentation").is_dir());

    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains(r#"edition = "2021""#));
    assert!(cargo_toml.contains(r#"rust-version = "1.89""#));
    assert!(cargo_toml.contains("my-basic-app"));
}

// --- Gateway archetype ---

#[test]
fn test_scaffold_gateway_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-gateway");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_gateway_structure(&project_path).unwrap();

    let mut config = make_config("my-gateway");
    config.gateway = Some(GatewayConfig::default());
    scaffolding::generate_gateway_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src/main.rs").exists());
    assert!(project_path.join("src/domain").is_dir());
    assert!(project_path.join("proto").is_dir());

    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(
        cargo_toml.contains(r#"edition = "2021""#),
        "Gateway must use edition 2021"
    );
    assert!(
        cargo_toml.contains(r#"rust-version = "1.89""#),
        "Gateway must use MSRV 1.89"
    );
}

// --- Consumer archetype ---

#[test]
fn test_scaffold_consumer_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-consumer");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_consumer_structure(&project_path).unwrap();

    let mut config = make_config("my-consumer");
    config.consumer = Some(ConsumerConfig::default());
    scaffolding::generate_consumer_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src/main.rs").exists());

    let cargo_toml = std::fs::read_to_string(project_path.join("Cargo.toml")).unwrap();
    assert!(cargo_toml.contains(r#"edition = "2021""#));
    assert!(cargo_toml.contains(r#"rust-version = "1.89""#));
}

// --- Producer archetype ---

#[test]
fn test_scaffold_producer_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-producer");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_producer_structure(&project_path).unwrap();

    let mut config = make_config("my-producer");
    config.producer = Some(ProducerConfig::default());
    scaffolding::generate_producer_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src/main.rs").exists());
}

// --- Error handling: missing config ---

#[test]
fn test_gateway_without_config_panics() {
    // When gateway config is missing, template functions unwrap() internally
    // before scaffolding's error handling is reached. This documents the behavior.
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("bad-gateway");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_gateway_structure(&project_path).unwrap();

    let config = make_config("bad-gateway"); // No gateway config
    let result = std::panic::catch_unwind(|| {
        let _ = scaffolding::generate_gateway_files(&project_path, &config);
    });
    assert!(result.is_err(), "Should panic when gateway config is missing");
}

#[test]
fn test_bff_without_config_returns_error() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("bad-bff");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_bff_structure(&project_path).unwrap();

    let config = make_config("bad-bff"); // No BFF config
    let result = scaffolding::generate_bff_files(&project_path, &config);
    assert!(result.is_err(), "Should error when BFF config is missing");
}

// --- Validation tests ---

#[test]
fn test_valid_project_names() {
    assert!(validate_project_name("my-app").is_ok());
    assert!(validate_project_name("my_app").is_ok());
    assert!(validate_project_name("app123").is_ok());
    assert!(validate_project_name("a").is_ok());
}

#[test]
fn test_invalid_project_names() {
    assert!(validate_project_name("123start").is_err());
    assert!(validate_project_name("has space").is_err());
    assert!(validate_project_name("special!chars").is_err());
}

// --- Additional archetype structure tests ---

#[test]
fn test_scaffold_scheduled_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-scheduler");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_scheduled_structure(&project_path).unwrap();

    let mut config = make_config("my-scheduler");
    config.scheduled = Some(ScheduledConfig::default());
    scaffolding::generate_scheduled_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
    assert!(project_path.join("src/main.rs").exists());
}

#[test]
fn test_scaffold_websocket_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-ws");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_websocket_structure(&project_path).unwrap();

    let mut config = make_config("my-ws");
    config.websocket_gateway = Some(WebSocketGatewayConfig::default());
    scaffolding::generate_websocket_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
}

#[test]
fn test_scaffold_saga_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-saga");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_saga_structure(&project_path).unwrap();

    let mut config = make_config("my-saga");
    config.saga_orchestrator = Some(SagaOrchestratorConfig::default());
    scaffolding::generate_saga_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
}

#[test]
fn test_scaffold_acl_structure() {
    let tmp = TempDir::new().unwrap();
    let project_path = tmp.path().join("my-acl");
    std::fs::create_dir_all(&project_path).unwrap();

    scaffolding::create_acl_structure(&project_path).unwrap();

    let mut config = make_config("my-acl");
    config.acl = Some(AntiCorruptionLayerConfig::default());
    scaffolding::generate_acl_files(&project_path, &config).unwrap();

    assert!(project_path.join("Cargo.toml").exists());
}
