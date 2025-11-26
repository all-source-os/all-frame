//! Template strings for project scaffolding
//!
//! This module contains all the template strings used when generating
//! new AllFrame projects with the `allframe ignite` command.
//!
//! All templates follow Clean Architecture principles and include:
//! - Example code demonstrating proper layer separation
//! - Documentation comments explaining each layer's purpose
//! - Trait-based abstractions for dependency inversion

/// Generate Cargo.toml content for a new AllFrame project
///
/// Creates a minimal `Cargo.toml` with:
/// - Project name and metadata
/// - Core dependencies: tokio, serde, anyhow, async-trait
/// - Binary configuration
///
/// Note: The `allframe` crate dependency will be added once it's published to crates.io.
///
/// # Arguments
/// * `project_name` - Name of the project (used for package name and binary name)
///
/// # Returns
/// A formatted Cargo.toml file content as a String
pub fn cargo_toml(project_name: &str) -> String {
    // For now, we generate a minimal Cargo.toml without allframe dependency
    // since allframe is not published to crates.io yet.
    // When allframe is published, we'll add the features and allframe dependency.
    format!(
        r#"[package]
name = "{}"
version = "0.1.0"
edition = "2021"

[dependencies]
tokio = {{ version = "1", features = ["full"] }}
serde = {{ version = "1", features = ["derive"] }}
anyhow = "1.0"
async-trait = "0.1"

[[bin]]
name = "{}"
path = "src/main.rs"
"#,
        project_name, project_name
    )
}

/// Generate src/main.rs content with tokio runtime
///
/// Creates the application entry point with:
/// - Module declarations for all Clean Architecture layers
/// - Async main function with tokio runtime
/// - Basic test structure
///
/// # Returns
/// A static string containing the main.rs template
pub fn main_rs() -> &'static str {
    r#"//! AllFrame Application
//!
//! This is an AllFrame application following Clean Architecture principles.

mod domain;
mod application;
mod infrastructure;
mod presentation;

#[tokio::main]
async fn main() {
    println!("AllFrame - One frame. Infinite transformations.");
    println!("Your application is ready!");
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_application_compiles() {
        // This test verifies the application structure compiles
        assert!(true);
    }
}
"#
}

/// Generate domain/mod.rs content
pub fn domain_mod() -> &'static str {
    r#"//! Domain Layer
//!
//! This layer contains the core business logic and has NO external dependencies.
//!
//! Structure:
//! - entities/: Business entities with behavior
//! - repositories/: Repository trait definitions (interfaces)
//! - services/: Domain services for complex business logic
//! - value_objects/: Immutable value types

pub mod entities;
pub mod repositories;

// Re-export commonly used types
pub use entities::*;
pub use repositories::*;
"#
}

/// Generate domain/entities.rs content
pub fn domain_entities() -> &'static str {
    r#"//! Domain Entities
//!
//! Business entities with behavior live here.

// Example entity (remove and add your own)
pub struct ExampleEntity {
    id: String,
    name: String,
}

impl ExampleEntity {
    pub fn new(id: String, name: String) -> Self {
        Self { id, name }
    }

    pub fn id(&self) -> &str {
        &self.id
    }

    pub fn name(&self) -> &str {
        &self.name
    }
}
"#
}

/// Generate domain/repositories.rs content
pub fn domain_repositories() -> &'static str {
    r#"//! Repository Traits
//!
//! Repository interfaces (traits) are defined here.
//! Implementations live in the infrastructure layer.

// Example repository trait (remove and add your own)
#[async_trait::async_trait]
pub trait ExampleRepository: Send + Sync {
    async fn find_by_id(&self, id: &str) -> anyhow::Result<Option<String>>;
    async fn save(&self, id: &str, data: String) -> anyhow::Result<()>;
}
"#
}

/// Generate application/mod.rs content
pub fn application_mod() -> &'static str {
    r#"//! Application Layer
//!
//! This layer orchestrates domain objects to fulfill use cases.
//! It depends only on the domain layer.

pub mod services;

pub use services::*;
"#
}

/// Generate application/services.rs content
pub fn application_services() -> &'static str {
    r#"//! Application Services
//!
//! Use case orchestration lives here.

// Example service (remove and add your own)
pub struct ExampleService;

impl ExampleService {
    pub fn new() -> Self {
        Self
    }

    pub async fn do_something(&self) -> anyhow::Result<String> {
        Ok("Service executed successfully".to_string())
    }
}
"#
}

/// Generate infrastructure/mod.rs content
pub fn infrastructure_mod() -> &'static str {
    r#"//! Infrastructure Layer
//!
//! This layer implements domain repository traits and handles external concerns.
//! It depends on the domain layer (implements traits defined there).

pub mod repositories;

pub use repositories::*;
"#
}

/// Generate infrastructure/repositories.rs content
pub fn infrastructure_repositories() -> &'static str {
    r#"//! Repository Implementations
//!
//! Concrete implementations of domain repository traits.

// Example repository implementation (remove and add your own)
pub struct InMemoryExampleRepository;

impl InMemoryExampleRepository {
    pub fn new() -> Self {
        Self
    }
}

#[async_trait::async_trait]
impl crate::domain::repositories::ExampleRepository for InMemoryExampleRepository {
    async fn find_by_id(&self, _id: &str) -> anyhow::Result<Option<String>> {
        Ok(None)
    }

    async fn save(&self, _id: &str, _data: String) -> anyhow::Result<()> {
        Ok(())
    }
}
"#
}

/// Generate presentation/mod.rs content
pub fn presentation_mod() -> &'static str {
    r#"//! Presentation Layer
//!
//! This layer handles HTTP/gRPC/GraphQL requests and responses.
//! It depends on the application and domain layers.

pub mod handlers;

pub use handlers::*;
"#
}

/// Generate presentation/handlers.rs content
pub fn presentation_handlers() -> &'static str {
    r#"//! HTTP/API Handlers
//!
//! Thin handlers that delegate to application services.

// Example handler (remove and add your own)
pub async fn example_handler() -> String {
    "Hello from AllFrame!".to_string()
}
"#
}

/// Generate .gitignore content
pub fn gitignore() -> &'static str {
    r#"# Rust
target/
Cargo.lock

# IDE
.vscode/
.idea/
*.swp
*.swo

# OS
.DS_Store
Thumbs.db

# Environment
.env
.env.local

# Logs
*.log
"#
}

/// Generate README.md content for the project
///
/// Creates comprehensive project documentation including:
/// - Quick start guide
/// - Project structure explanation
/// - Feature highlights
///
/// # Arguments
/// * `project_name` - Name of the project (used in title)
///
/// # Returns
/// A formatted README.md file content as a String
pub fn readme(project_name: &str) -> String {
    format!(
        r#"# {}

An AllFrame application following Clean Architecture principles.

## Quick Start

```bash
# Run the application
cargo run

# Run tests
cargo test

# Run with all features
cargo run --all-features
```

## Structure

This project follows Clean Architecture:

```
src/
├── domain/          # Core business logic (no external dependencies)
├── application/     # Use case orchestration
├── infrastructure/  # External implementations (database, HTTP clients)
└── presentation/    # HTTP/gRPC/GraphQL handlers
```

## Features

- Clean Architecture enforced
- TDD-first development
- Protocol-agnostic (REST/GraphQL/gRPC)

## Built with AllFrame

[AllFrame](https://github.com/all-source-os/all-frame) - The composable Rust API framework.

*One frame. Infinite transformations.*
"#,
        project_name
    )
}
