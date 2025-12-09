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
/// Note: The `allframe` crate dependency will be added once it's published to
/// crates.io.
///
/// # Arguments
/// * `project_name` - Name of the project (used for package name and binary
///   name)
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
/// - Working example demonstrating layer connections
///
/// # Returns
/// A static string containing the main.rs template
pub fn main_rs() -> &'static str {
    r#"//! AllFrame Application
//!
//! This is an AllFrame application following Clean Architecture principles.

mod application;
mod domain;
mod infrastructure;
mod presentation;

use application::GreetingService;
use infrastructure::ConsoleGreeter;

#[tokio::main]
async fn main() {
    println!("AllFrame - One frame. Infinite transformations.");
    println!();

    // Wire up dependencies (Dependency Injection)
    let greeter = ConsoleGreeter;
    let service = GreetingService::new(greeter);

    // Execute use case
    service.greet("World").await;

    println!();
    println!("Your application is ready!");
}

#[cfg(test)]
mod tests {
    use super::*;
    use domain::Greeter;

    struct MockGreeter {
        messages: std::cell::RefCell<Vec<String>>,
    }

    impl MockGreeter {
        fn new() -> Self {
            Self {
                messages: std::cell::RefCell::new(Vec::new()),
            }
        }

        fn get_messages(&self) -> Vec<String> {
            self.messages.borrow().clone()
        }
    }

    impl Greeter for MockGreeter {
        fn greet(&self, name: &str) {
            self.messages.borrow_mut().push(format!("Hello, {}!", name));
        }
    }

    #[tokio::test]
    async fn test_greeting_service() {
        let greeter = MockGreeter::new();
        let service = GreetingService::new(greeter);

        service.greet("Test").await;

        // Verify the greeter was called correctly
        // In a real test, you'd check the mock's recorded calls
    }

    #[test]
    fn test_mock_greeter() {
        let greeter = MockGreeter::new();
        greeter.greet("Alice");
        greeter.greet("Bob");

        let messages = greeter.get_messages();
        assert_eq!(messages.len(), 2);
        assert_eq!(messages[0], "Hello, Alice!");
        assert_eq!(messages[1], "Hello, Bob!");
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

mod greeter;

pub use greeter::Greeter;
"#
}

/// Generate domain/greeter.rs content (replaces entities.rs)
pub fn domain_greeter() -> &'static str {
    r#"//! Greeter Domain Trait
//!
//! This trait defines the contract for greeting functionality.
//! Implementations live in the infrastructure layer.

/// A trait for greeting functionality.
///
/// This is a domain-level abstraction that allows different
/// implementations (console, HTTP, mock for testing, etc.)
pub trait Greeter {
    /// Greet someone by name.
    fn greet(&self, name: &str);
}
"#
}

/// Generate application/mod.rs content
pub fn application_mod() -> &'static str {
    r#"//! Application Layer
//!
//! This layer orchestrates domain objects to fulfill use cases.
//! It depends only on the domain layer.

mod greeting_service;

pub use greeting_service::GreetingService;
"#
}

/// Generate application/greeting_service.rs content
pub fn application_greeting_service() -> &'static str {
    r#"//! Greeting Service
//!
//! Application service that orchestrates the greeting use case.

use crate::domain::Greeter;

/// Service for greeting users.
///
/// This service demonstrates dependency injection - it accepts
/// any implementation of the Greeter trait.
pub struct GreetingService<G: Greeter> {
    greeter: G,
}

impl<G: Greeter> GreetingService<G> {
    /// Create a new GreetingService with the given greeter implementation.
    pub fn new(greeter: G) -> Self {
        Self { greeter }
    }

    /// Greet someone by name.
    pub async fn greet(&self, name: &str) {
        self.greeter.greet(name);
    }
}
"#
}

/// Generate infrastructure/mod.rs content
pub fn infrastructure_mod() -> &'static str {
    r#"//! Infrastructure Layer
//!
//! This layer implements domain traits and handles external concerns.
//! It depends on the domain layer (implements traits defined there).

mod console_greeter;

pub use console_greeter::ConsoleGreeter;
"#
}

/// Generate infrastructure/console_greeter.rs content
pub fn infrastructure_console_greeter() -> &'static str {
    r#"//! Console Greeter Implementation
//!
//! A concrete implementation of the Greeter trait that prints to the console.

use crate::domain::Greeter;

/// A greeter that prints greetings to the console.
pub struct ConsoleGreeter;

impl Greeter for ConsoleGreeter {
    fn greet(&self, name: &str) {
        println!("Hello, {}!", name);
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
//!
//! Add your HTTP handlers, GraphQL resolvers, or gRPC services here.
//! Example:
//!
//! ```ignore
//! mod handlers;
//! pub use handlers::*;
//! ```
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
