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

use std::{fs, path::Path};

use anyhow::Result;

use crate::templates;

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
