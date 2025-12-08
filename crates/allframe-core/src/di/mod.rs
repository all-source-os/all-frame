//! Dependency Injection Infrastructure
//!
//! This module provides compile-time dependency injection with support for:
//! - Async initialization
//! - Explicit dependency declaration
//! - Singleton and transient scoping
//! - Environment-based configuration
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::di::{Provider, Scope, DependencyError};
//! use allframe_macros::di_container;
//!
//! #[di_container]
//! struct AppContainer {
//!     #[provide(from_env)]
//!     config: Config,
//!
//!     #[provide(singleton, async)]
//!     #[depends(config)]
//!     database: DatabasePool,
//!
//!     #[provide(transient)]
//!     service: MyService,
//! }
//!
//! // Generated async build method
//! let container = AppContainer::build().await?;
//! ```

use std::{any::Any, collections::HashMap, fmt, sync::Arc};

/// Error type for dependency injection operations
#[derive(Debug)]
pub enum DependencyError {
    /// A required dependency was not found
    NotFound(String),
    /// Circular dependency detected
    CircularDependency(Vec<String>),
    /// Failed to initialize a dependency
    InitializationFailed {
        /// Name of the dependency that failed
        name: String,
        /// The underlying error
        source: Box<dyn std::error::Error + Send + Sync>,
    },
    /// Configuration error (e.g., missing environment variable)
    ConfigError(String),
    /// Type mismatch when resolving dependency
    TypeMismatch {
        /// Expected type name
        expected: String,
        /// Actual type name
        actual: String,
    },
}

impl fmt::Display for DependencyError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            DependencyError::NotFound(name) => {
                write!(f, "Dependency not found: {}", name)
            }
            DependencyError::CircularDependency(chain) => {
                write!(f, "Circular dependency detected: {}", chain.join(" -> "))
            }
            DependencyError::InitializationFailed { name, source } => {
                write!(f, "Failed to initialize '{}': {}", name, source)
            }
            DependencyError::ConfigError(msg) => {
                write!(f, "Configuration error: {}", msg)
            }
            DependencyError::TypeMismatch { expected, actual } => {
                write!(f, "Type mismatch: expected {}, got {}", expected, actual)
            }
        }
    }
}

impl std::error::Error for DependencyError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            DependencyError::InitializationFailed { source, .. } => Some(source.as_ref()),
            _ => None,
        }
    }
}

/// Scope determines the lifecycle of a dependency
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum Scope {
    /// Single instance shared across all requests (wrapped in Arc)
    #[default]
    Singleton,
    /// New instance created for each request
    Transient,
}

/// Trait for types that can provide dependencies
///
/// This trait is implemented automatically by the `#[di_container]` macro
/// for container types, but can also be implemented manually for custom
/// providers.
#[crate::async_trait::async_trait]
pub trait Provider<T>: Send + Sync {
    /// Provide an instance of the dependency
    async fn provide(&self) -> Result<T, DependencyError>;
}

/// Trait for types that can be loaded from environment variables
pub trait FromEnv: Sized {
    /// Load configuration from environment variables
    fn from_env() -> Result<Self, DependencyError>;
}

/// Trait for async initialization
#[crate::async_trait::async_trait]
pub trait AsyncInit: Sized {
    /// Initialize the type asynchronously
    async fn init() -> Result<Self, DependencyError>;
}

/// Trait for types that can be initialized with dependencies
#[crate::async_trait::async_trait]
pub trait AsyncInitWith<Deps>: Sized {
    /// Initialize the type asynchronously with dependencies
    async fn init_with(deps: Deps) -> Result<Self, DependencyError>;
}

/// A type-erased container for storing dependencies
pub struct DependencyRegistry {
    singletons: HashMap<std::any::TypeId, Arc<dyn Any + Send + Sync>>,
}

impl Default for DependencyRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl DependencyRegistry {
    /// Create a new empty registry
    pub fn new() -> Self {
        Self {
            singletons: HashMap::new(),
        }
    }

    /// Store a singleton instance
    pub fn store_singleton<T: Send + Sync + 'static>(&mut self, value: T) {
        let type_id = std::any::TypeId::of::<T>();
        self.singletons.insert(type_id, Arc::new(value));
    }

    /// Get a singleton instance
    pub fn get_singleton<T: Send + Sync + 'static>(&self) -> Option<Arc<T>> {
        let type_id = std::any::TypeId::of::<T>();
        self.singletons
            .get(&type_id)
            .and_then(|any| any.clone().downcast::<T>().ok())
    }

    /// Check if a singleton exists
    pub fn has_singleton<T: 'static>(&self) -> bool {
        let type_id = std::any::TypeId::of::<T>();
        self.singletons.contains_key(&type_id)
    }
}

/// Builder for constructing dependency containers with explicit ordering
pub struct ContainerBuilder {
    initialization_order: Vec<String>,
    initialized: std::collections::HashSet<String>,
}

impl Default for ContainerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl ContainerBuilder {
    /// Create a new container builder
    pub fn new() -> Self {
        Self {
            initialization_order: Vec::new(),
            initialized: std::collections::HashSet::new(),
        }
    }

    /// Mark a dependency as initialized
    pub fn mark_initialized(&mut self, name: &str) {
        self.initialized.insert(name.to_string());
        self.initialization_order.push(name.to_string());
    }

    /// Check if a dependency has been initialized
    pub fn is_initialized(&self, name: &str) -> bool {
        self.initialized.contains(name)
    }

    /// Get the initialization order
    pub fn initialization_order(&self) -> &[String] {
        &self.initialization_order
    }

    /// Validate that all dependencies for a type are initialized
    pub fn validate_dependencies(&self, deps: &[&str]) -> Result<(), DependencyError> {
        for dep in deps {
            if !self.is_initialized(dep) {
                return Err(DependencyError::NotFound((*dep).to_string()));
            }
        }
        Ok(())
    }
}

/// Helper to load a value from an environment variable
pub fn env_var(name: &str) -> Result<String, DependencyError> {
    std::env::var(name).map_err(|_| {
        DependencyError::ConfigError(format!("Environment variable '{}' not set", name))
    })
}

/// Helper to load an optional value from an environment variable
pub fn env_var_opt(name: &str) -> Option<String> {
    std::env::var(name).ok()
}

/// Helper to load a value from an environment variable with a default
pub fn env_var_or(name: &str, default: &str) -> String {
    std::env::var(name).unwrap_or_else(|_| default.to_string())
}

/// Helper to parse a value from an environment variable
pub fn env_var_parse<T: std::str::FromStr>(name: &str) -> Result<T, DependencyError>
where
    T::Err: std::fmt::Display,
{
    let value = env_var(name)?;
    value.parse().map_err(|e: T::Err| {
        DependencyError::ConfigError(format!(
            "Failed to parse environment variable '{}': {}",
            name, e
        ))
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dependency_error_display() {
        let err = DependencyError::NotFound("database".to_string());
        assert!(err.to_string().contains("database"));

        let err = DependencyError::CircularDependency(vec![
            "a".to_string(),
            "b".to_string(),
            "a".to_string(),
        ]);
        assert!(err.to_string().contains("a -> b -> a"));

        let err = DependencyError::ConfigError("missing var".to_string());
        assert!(err.to_string().contains("missing var"));
    }

    #[test]
    fn test_scope_default() {
        assert_eq!(Scope::default(), Scope::Singleton);
    }

    #[test]
    fn test_dependency_registry() {
        let mut registry = DependencyRegistry::new();

        registry.store_singleton(42i32);
        assert!(registry.has_singleton::<i32>());

        let value = registry.get_singleton::<i32>();
        assert_eq!(*value.unwrap(), 42);

        assert!(!registry.has_singleton::<String>());
    }

    #[test]
    fn test_container_builder() {
        let mut builder = ContainerBuilder::new();

        assert!(!builder.is_initialized("config"));
        builder.mark_initialized("config");
        assert!(builder.is_initialized("config"));

        builder.mark_initialized("database");
        assert_eq!(builder.initialization_order(), &["config", "database"]);
    }

    #[test]
    fn test_container_builder_validate() {
        let mut builder = ContainerBuilder::new();
        builder.mark_initialized("config");

        assert!(builder.validate_dependencies(&["config"]).is_ok());
        assert!(builder.validate_dependencies(&["database"]).is_err());
    }

    #[test]
    fn test_env_helpers() {
        // Test env_var_or with default
        let value = env_var_or("NONEXISTENT_VAR_12345", "default");
        assert_eq!(value, "default");

        // Test env_var_opt
        let value = env_var_opt("NONEXISTENT_VAR_12345");
        assert!(value.is_none());
    }
}
