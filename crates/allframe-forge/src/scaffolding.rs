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

use crate::config::ProjectConfig;
use crate::templates::{self, gateway};

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
    fs::write(
        project_path.join("Cargo.toml"),
        gateway::cargo_toml(config),
    )?;
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
    fs::write(project_path.join("src/config.rs"), gateway::config_rs(config))?;
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
