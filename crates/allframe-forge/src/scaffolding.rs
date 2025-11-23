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

use crate::templates;
use anyhow::Result;
use std::fs;
use std::path::Path;

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
/// - `src/domain/entities.rs` - Example business entity
/// - `src/domain/repositories.rs` - Example repository trait
///
/// ## Application Layer
/// - `src/application/mod.rs` - Application module exports
/// - `src/application/services.rs` - Example application service
///
/// ## Infrastructure Layer
/// - `src/infrastructure/mod.rs` - Infrastructure module exports
/// - `src/infrastructure/repositories.rs` - Example repository implementation
///
/// ## Presentation Layer
/// - `src/presentation/mod.rs` - Presentation module exports
/// - `src/presentation/handlers.rs` - Example HTTP handler
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
        project_path.join("src/domain/entities.rs"),
        templates::domain_entities(),
    )?;
    fs::write(
        project_path.join("src/domain/repositories.rs"),
        templates::domain_repositories(),
    )?;

    // Application layer
    fs::write(
        project_path.join("src/application/mod.rs"),
        templates::application_mod(),
    )?;
    fs::write(
        project_path.join("src/application/services.rs"),
        templates::application_services(),
    )?;

    // Infrastructure layer
    fs::write(
        project_path.join("src/infrastructure/mod.rs"),
        templates::infrastructure_mod(),
    )?;
    fs::write(
        project_path.join("src/infrastructure/repositories.rs"),
        templates::infrastructure_repositories(),
    )?;

    // Presentation layer
    fs::write(
        project_path.join("src/presentation/mod.rs"),
        templates::presentation_mod(),
    )?;
    fs::write(
        project_path.join("src/presentation/handlers.rs"),
        templates::presentation_handlers(),
    )?;

    Ok(())
}
