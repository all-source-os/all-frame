//! Project scaffolding utilities
//!
//! This module handles the creation of directories and files
//! for new AllFrame projects following Clean Architecture principles.
//!
//! The scaffolding creates a complete project structure with:
//! - Domain layer (business logic, no dependencies)
//! - Application layer (use case orchestration)
//! - Infrastructure layer (external implementations)
//! - Presentation layer (HTTP/API handlers)
//!
//! ## Archetypes
//!
//! Different archetypes have different directory structures:
//!
//! ### Basic
//! Simple Clean Architecture project with greeter example.
//!
//! ### Gateway
//! API Gateway service with gRPC, including:
//! - Protocol buffer definitions
//! - HTTP client for external APIs
//! - Rate limiting and caching
//! - Authentication (HMAC, API Key, etc.)

use std::{fs, path::Path};

use anyhow::Result;

use crate::{
    config::ProjectConfig,
    templates::{self, acl, bff, consumer, gateway, producer, saga, scheduled, websocket},
};

/// Create the Clean Architecture directory structure
///
/// Creates all necessary directories for a Clean Architecture project:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (entities, repositories)
/// - `src/application/` - Application layer (services)
/// - `src/infrastructure/` - Infrastructure layer (repository implementations)
/// - `src/presentation/` - Presentation layer (HTTP handlers)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_directory_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all project files with Clean Architecture structure
///
/// Creates all necessary files for a complete AllFrame project:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with dependencies
/// - `src/main.rs` - Application entry point with tokio runtime
/// - `.gitignore` - Git ignore rules for Rust projects
/// - `README.md` - Project documentation
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/greeter.rs` - Greeter trait definition
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/greeting_service.rs` - Greeting service
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/console_greeter.rs` - Console greeter implementation
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module (placeholder)
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `project_name` - Name of the project (used in Cargo.toml and README)
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_files(project_path: &Path, project_name: &str) -> Result<()> {
    // Root files
    fs::write(
        project_path.join("Cargo.toml"),
        templates::cargo_toml(project_name),
    )?;
    fs::write(project_path.join("src/main.rs"), templates::main_rs())?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(
        project_path.join("README.md"),
        templates::readme(project_name),
    )?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        templates::domain_mod(),
    )?;
    fs::write(
        project_path.join("src/domain/greeter.rs"),
        templates::domain_greeter(),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        templates::application_mod(),
    )?;
    fs::write(
        project_path.join("src/application/greeting_service.rs"),
        templates::application_greeting_service(),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        templates::infrastructure_mod(),
    )?;
    fs::write(
        project_path.join("src/infrastructure/console_greeter.rs"),
        templates::infrastructure_console_greeter(),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        templates::presentation_mod(),
    )?;

    Ok(())
}

/// Create the Gateway Architecture directory structure
///
/// Creates all necessary directories for a Gateway service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (entities, repository traits)
/// - `src/application/` - Application layer (services, use cases)
/// - `src/infrastructure/` - Infrastructure layer (HTTP client, cache, auth)
/// - `src/presentation/` - Presentation layer (gRPC handlers)
/// - `proto/` - Protocol buffer definitions
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_gateway_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "proto",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all gateway project files
///
/// Creates all necessary files for a complete Gateway service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with gateway dependencies
/// - `build.rs` - Proto compilation build script
/// - `src/main.rs` - Application entry point with gRPC server
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Protocol Buffers
/// - `proto/{service}.proto` - gRPC service definition
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/entities.rs` - Domain entities
/// - `src/domain/repository.rs` - Repository trait
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/service.rs` - Gateway service implementation
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/http_client.rs` - External API HTTP client
/// - `src/infrastructure/auth.rs` - Authentication implementations
/// - `src/infrastructure/cache.rs` - Caching implementation
/// - `src/infrastructure/rate_limiter.rs` - Rate limiting
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module exports
/// - `src/presentation/grpc.rs` - gRPC service handlers
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_gateway_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(project_path.join("Cargo.toml"), gateway::cargo_toml(config))?;
    fs::write(project_path.join("build.rs"), gateway::build_rs(config))?;
    fs::write(project_path.join("src/main.rs"), gateway::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), gateway::readme(config))?;
    fs::write(project_path.join("Dockerfile"), gateway::dockerfile(config))?;

    // Protocol buffers
    let gateway_config = config.gateway.as_ref().expect("Gateway config required");
    fs::write(
        project_path.join(format!("proto/{}.proto", gateway_config.service_name)),
        gateway::proto_file(config),
    )?;

    // Configuration files
    fs::write(
        project_path.join("src/config.rs"),
        gateway::config_rs(config),
    )?;
    fs::write(project_path.join("src/error.rs"), gateway::error_rs(config))?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        gateway::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/entities.rs"),
        gateway::domain_entities(config),
    )?;
    fs::write(
        project_path.join("src/domain/repository.rs"),
        gateway::domain_repository(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        gateway::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/service.rs"),
        gateway::application_service(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        gateway::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/http_client.rs"),
        gateway::infrastructure_http_client(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/auth.rs"),
        gateway::infrastructure_auth(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/cache.rs"),
        gateway::infrastructure_cache(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/rate_limiter.rs"),
        gateway::infrastructure_rate_limiter(config),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        gateway::presentation_mod(config),
    )?;
    fs::write(
        project_path.join("src/presentation/grpc.rs"),
        gateway::presentation_grpc(config),
    )?;

    Ok(())
}

/// Create the Consumer Architecture directory structure
///
/// Creates all necessary directories for a Consumer (event handler) service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (events, handlers)
/// - `src/application/` - Application layer (consumer orchestration)
/// - `src/infrastructure/` - Infrastructure layer (broker, idempotency, health)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_consumer_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all consumer project files
///
/// Creates all necessary files for a complete Consumer service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with consumer dependencies
/// - `src/main.rs` - Application entry point with consumer setup
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/events.rs` - Event definitions with envelope pattern
/// - `src/domain/handlers.rs` - Event handler traits and registry
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/consumer.rs` - Consumer orchestration with
///   retry/idempotency
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/broker.rs` - Message broker implementation (Kafka)
/// - `src/infrastructure/idempotency.rs` - Idempotency store
/// - `src/infrastructure/health.rs` - Health check server
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_consumer_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(
        project_path.join("Cargo.toml"),
        consumer::cargo_toml(config),
    )?;
    fs::write(project_path.join("src/main.rs"), consumer::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), consumer::readme(config))?;
    fs::write(
        project_path.join("Dockerfile"),
        consumer::dockerfile(config),
    )?;

    // Configuration files
    fs::write(
        project_path.join("src/config.rs"),
        consumer::config_rs(config),
    )?;
    fs::write(
        project_path.join("src/error.rs"),
        consumer::error_rs(config),
    )?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        consumer::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/events.rs"),
        consumer::domain_events(config),
    )?;
    fs::write(
        project_path.join("src/domain/handlers.rs"),
        consumer::domain_handlers(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        consumer::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/consumer.rs"),
        consumer::application_consumer(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        consumer::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/broker.rs"),
        consumer::infrastructure_broker(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/idempotency.rs"),
        consumer::infrastructure_idempotency(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        consumer::infrastructure_health(config),
    )?;

    Ok(())
}

/// Create the Producer Architecture directory structure
///
/// Creates all necessary directories for a Producer (event publisher) service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (entities, events, repository)
/// - `src/application/` - Application layer (service, outbox)
/// - `src/infrastructure/` - Infrastructure layer (repository, outbox,
///   publisher)
/// - `src/presentation/` - Presentation layer (HTTP handlers)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_producer_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all producer project files
///
/// Creates all necessary files for a complete Producer service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with producer dependencies
/// - `src/main.rs` - Application entry point with API server and outbox
///   processor
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/entities.rs` - Domain entities
/// - `src/domain/events.rs` - Event definitions with envelope pattern
/// - `src/domain/repository.rs` - Repository trait
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/service.rs` - Application service with event publishing
/// - `src/application/outbox.rs` - Outbox trait
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/repository.rs` - PostgreSQL repository
/// - `src/infrastructure/outbox.rs` - PostgreSQL outbox
/// - `src/infrastructure/publisher.rs` - Kafka event publisher
/// - `src/infrastructure/outbox_processor.rs` - Outbox processor
/// - `src/infrastructure/health.rs` - Health check server
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module exports
/// - `src/presentation/handlers.rs` - HTTP API handlers
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_producer_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(
        project_path.join("Cargo.toml"),
        producer::cargo_toml(config),
    )?;
    fs::write(project_path.join("src/main.rs"), producer::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), producer::readme(config))?;
    fs::write(
        project_path.join("Dockerfile"),
        producer::dockerfile(config),
    )?;

    // Configuration files
    fs::write(
        project_path.join("src/config.rs"),
        producer::config_rs(config),
    )?;
    fs::write(
        project_path.join("src/error.rs"),
        producer::error_rs(config),
    )?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        producer::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/entities.rs"),
        producer::domain_entities(config),
    )?;
    fs::write(
        project_path.join("src/domain/events.rs"),
        producer::domain_events(config),
    )?;
    fs::write(
        project_path.join("src/domain/repository.rs"),
        producer::domain_repository(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        producer::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/service.rs"),
        producer::application_service(config),
    )?;
    fs::write(
        project_path.join("src/application/outbox.rs"),
        producer::application_outbox(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        producer::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/repository.rs"),
        producer::infrastructure_repository(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/outbox.rs"),
        producer::infrastructure_outbox(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/publisher.rs"),
        producer::infrastructure_publisher(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/outbox_processor.rs"),
        producer::infrastructure_outbox_processor(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        producer::infrastructure_health(config),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        producer::presentation_mod(config),
    )?;
    fs::write(
        project_path.join("src/presentation/handlers.rs"),
        producer::presentation_handlers(config),
    )?;

    Ok(())
}

/// Create the BFF (Backend for Frontend) Architecture directory structure
///
/// Creates all necessary directories for a BFF service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (models, aggregates)
/// - `src/application/` - Application layer (aggregator service)
/// - `src/infrastructure/` - Infrastructure layer (clients, cache, health)
/// - `src/presentation/` - Presentation layer (REST handlers, GraphQL)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_bff_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all BFF project files
///
/// Creates all necessary files for a complete BFF service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with BFF dependencies
/// - `src/main.rs` - Application entry point with API server
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/models.rs` - Domain models
/// - `src/domain/aggregates.rs` - Aggregate types for frontend views
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/aggregator.rs` - Aggregator service
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/clients.rs` - Backend HTTP clients
/// - `src/infrastructure/cache.rs` - Cache service
/// - `src/infrastructure/health.rs` - Health check server
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module exports
/// - `src/presentation/handlers.rs` - REST API handlers
/// - `src/presentation/graphql.rs` - GraphQL API (if enabled)
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_bff_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    let bff_config = config.bff.as_ref().expect("BFF config required");

    // Root files
    fs::write(project_path.join("Cargo.toml"), bff::cargo_toml(config))?;
    fs::write(project_path.join("src/main.rs"), bff::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), bff::readme(config))?;
    fs::write(project_path.join("Dockerfile"), bff::dockerfile(config))?;

    // Configuration files
    fs::write(project_path.join("src/config.rs"), bff::config_rs(config))?;
    fs::write(project_path.join("src/error.rs"), bff::error_rs(config))?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        bff::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/models.rs"),
        bff::domain_models(config),
    )?;
    fs::write(
        project_path.join("src/domain/aggregates.rs"),
        bff::domain_aggregates(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        bff::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/aggregator.rs"),
        bff::application_aggregator(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        bff::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/clients.rs"),
        bff::infrastructure_clients(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/cache.rs"),
        bff::infrastructure_cache(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        bff::infrastructure_health(config),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        bff::presentation_mod(config),
    )?;
    fs::write(
        project_path.join("src/presentation/handlers.rs"),
        bff::presentation_handlers(config),
    )?;

    // GraphQL (if enabled)
    if bff_config.graphql_enabled {
        fs::write(
            project_path.join("src/presentation/graphql.rs"),
            bff::presentation_graphql(config),
        )?;
    }

    Ok(())
}

/// Create the Scheduled Jobs Architecture directory structure
///
/// Creates all necessary directories for a Scheduled Jobs service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (job definitions)
/// - `src/application/` - Application layer (scheduler)
/// - `src/infrastructure/` - Infrastructure layer (health checks)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_scheduled_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all scheduled jobs project files
///
/// Creates all necessary files for a complete Scheduled Jobs service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with scheduler dependencies
/// - `src/main.rs` - Application entry point with scheduler setup
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/jobs.rs` - Job trait and context definitions
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/scheduler.rs` - Job scheduler and job implementations
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/health.rs` - Health check server
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_scheduled_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(
        project_path.join("Cargo.toml"),
        scheduled::cargo_toml(config),
    )?;
    fs::write(project_path.join("src/main.rs"), scheduled::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), scheduled::readme(config))?;
    fs::write(
        project_path.join("Dockerfile"),
        scheduled::dockerfile(config),
    )?;

    // Configuration files
    fs::write(
        project_path.join("src/config.rs"),
        scheduled::config_rs(config),
    )?;
    fs::write(
        project_path.join("src/error.rs"),
        scheduled::error_rs(config),
    )?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        scheduled::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/jobs.rs"),
        scheduled::domain_jobs(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        scheduled::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/scheduler.rs"),
        scheduled::application_scheduler(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        scheduled::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        scheduled::infrastructure_health(config),
    )?;

    Ok(())
}

/// Create the WebSocket Gateway Architecture directory structure
///
/// Creates all necessary directories for a WebSocket Gateway service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (messages, connections)
/// - `src/application/` - Application layer (hub)
/// - `src/infrastructure/` - Infrastructure layer (health checks)
/// - `src/presentation/` - Presentation layer (WebSocket handlers)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_websocket_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all WebSocket Gateway project files
///
/// Creates all necessary files for a complete WebSocket Gateway service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with WebSocket dependencies
/// - `src/main.rs` - Application entry point with WebSocket server
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/messages.rs` - WebSocket message types
/// - `src/domain/connection.rs` - Connection management
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/hub.rs` - WebSocket hub
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/health.rs` - Health check server
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module exports
/// - `src/presentation/handlers.rs` - WebSocket handlers
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_websocket_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(
        project_path.join("Cargo.toml"),
        websocket::cargo_toml(config),
    )?;
    fs::write(project_path.join("src/main.rs"), websocket::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), websocket::readme(config))?;
    fs::write(
        project_path.join("Dockerfile"),
        websocket::dockerfile(config),
    )?;

    // Configuration files
    fs::write(
        project_path.join("src/config.rs"),
        websocket::config_rs(config),
    )?;
    fs::write(
        project_path.join("src/error.rs"),
        websocket::error_rs(config),
    )?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        websocket::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/messages.rs"),
        websocket::domain_messages(config),
    )?;
    fs::write(
        project_path.join("src/domain/connection.rs"),
        websocket::domain_connection(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        websocket::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/hub.rs"),
        websocket::application_hub(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        websocket::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        websocket::infrastructure_health(config),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        websocket::presentation_mod(config),
    )?;
    fs::write(
        project_path.join("src/presentation/handlers.rs"),
        websocket::presentation_handlers(config),
    )?;

    Ok(())
}

/// Create the Saga Orchestrator Architecture directory structure
pub fn create_saga_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all Saga Orchestrator project files
pub fn generate_saga_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(project_path.join("Cargo.toml"), saga::cargo_toml(config))?;
    fs::write(project_path.join("src/main.rs"), saga::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), saga::readme(config))?;
    fs::write(project_path.join("Dockerfile"), saga::dockerfile(config))?;

    // Configuration files
    fs::write(project_path.join("src/config.rs"), saga::config_rs(config))?;
    fs::write(project_path.join("src/error.rs"), saga::error_rs(config))?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        saga::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/saga.rs"),
        saga::domain_saga(config),
    )?;
    fs::write(
        project_path.join("src/domain/step.rs"),
        saga::domain_step(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        saga::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/orchestrator.rs"),
        saga::application_orchestrator(config),
    )?;
    fs::write(
        project_path.join("src/application/steps.rs"),
        saga::application_steps(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        saga::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        saga::infrastructure_health(config),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        saga::presentation_mod(config),
    )?;
    fs::write(
        project_path.join("src/presentation/handlers.rs"),
        saga::presentation_handlers(config),
    )?;

    Ok(())
}

/// Create the Anti-Corruption Layer Architecture directory structure
///
/// Creates all necessary directories for an ACL service:
/// - `src/` - Source code root
/// - `src/domain/` - Domain layer (legacy models, modern models, transformer
///   traits)
/// - `src/application/` - Application layer (translator service)
/// - `src/infrastructure/` - Infrastructure layer (legacy client, health
///   checks)
/// - `src/presentation/` - Presentation layer (HTTP handlers)
/// - `tests/` - Integration tests
///
/// # Arguments
/// * `project_path` - Root path where the project directories will be created
///
/// # Errors
/// Returns an error if any directory creation fails
pub fn create_acl_structure(project_path: &Path) -> Result<()> {
    let dirs = vec![
        "src",
        "src/domain",
        "src/application",
        "src/infrastructure",
        "src/presentation",
        "tests",
    ];

    for dir in dirs {
        fs::create_dir_all(project_path.join(dir))?;
    }

    Ok(())
}

/// Generate all Anti-Corruption Layer project files
///
/// Creates all necessary files for a complete ACL service:
///
/// ## Root Files
/// - `Cargo.toml` - Project manifest with ACL dependencies
/// - `src/main.rs` - Application entry point with HTTP server
/// - `README.md` - Project documentation
/// - `Dockerfile` - Container build file
/// - `.gitignore` - Git ignore rules
///
/// ## Domain Layer
/// - `src/domain/mod.rs` - Domain module exports
/// - `src/domain/legacy.rs` - Legacy system models
/// - `src/domain/modern.rs` - Modern domain models
/// - `src/domain/transformer.rs` - Transformer trait for bidirectional
///   conversion
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/translator.rs` - Translation service
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/legacy_client.rs` - Legacy system HTTP client
/// - `src/infrastructure/health.rs` - Health check server
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module exports
/// - `src/presentation/handlers.rs` - HTTP API handlers
///
/// ## Configuration
/// - `src/config.rs` - Service configuration
/// - `src/error.rs` - Error types
///
/// # Arguments
/// * `project_path` - Root path where files will be created
/// * `config` - Project configuration
///
/// # Errors
/// Returns an error if any file write operation fails
pub fn generate_acl_files(project_path: &Path, config: &ProjectConfig) -> Result<()> {
    // Root files
    fs::write(project_path.join("Cargo.toml"), acl::cargo_toml(config))?;
    fs::write(project_path.join("src/main.rs"), acl::main_rs(config))?;
    fs::write(project_path.join(".gitignore"), templates::gitignore())?;
    fs::write(project_path.join("README.md"), acl::readme(config))?;
    fs::write(project_path.join("Dockerfile"), acl::dockerfile(config))?;

    // Configuration files
    fs::write(project_path.join("src/config.rs"), acl::config_rs(config))?;
    fs::write(project_path.join("src/error.rs"), acl::error_rs(config))?;

    // Domain layer
    fs::write(
        project_path.join("src/domain/mod.rs"),
        acl::domain_mod(config),
    )?;
    fs::write(
        project_path.join("src/domain/legacy.rs"),
        acl::domain_legacy(config),
    )?;
    fs::write(
        project_path.join("src/domain/modern.rs"),
        acl::domain_modern(config),
    )?;
    fs::write(
        project_path.join("src/domain/transformer.rs"),
        acl::domain_transformer(config),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        acl::application_mod(config),
    )?;
    fs::write(
        project_path.join("src/application/translator.rs"),
        acl::application_translator(config),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        acl::infrastructure_mod(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/legacy_client.rs"),
        acl::infrastructure_legacy_client(config),
    )?;
    fs::write(
        project_path.join("src/infrastructure/health.rs"),
        acl::infrastructure_health(config),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        acl::presentation_mod(config),
    )?;
    fs::write(
        project_path.join("src/presentation/handlers.rs"),
        acl::presentation_handlers(config),
    )?;

    Ok(())
}
