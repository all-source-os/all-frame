//! Clean Architecture enforcement
//!
//! This module provides compile-time and runtime support for Clean Architecture.
//! AllFrame enforces architectural boundaries at compile time, preventing
//! violations like handlers directly accessing repositories.
//!
//! ## Architecture Layers
//!
//! ```text
//! ┌─────────────────────────────────────┐
//! │  Layer 4: Handler                   │  ← HTTP/gRPC/GraphQL endpoints
//! │  (depends on Use Cases)             │
//! └─────────────────────────────────────┘
//!              ↓
//! ┌─────────────────────────────────────┐
//! │  Layer 3: Use Case                  │  ← Application logic
//! │  (depends on Repositories, Domain)  │
//! └─────────────────────────────────────┘
//!              ↓
//! ┌─────────────────────────────────────┐
//! │  Layer 2: Repository                │  ← Data access
//! │  (depends on Domain)                │
//! └─────────────────────────────────────┘
//!              ↓
//! ┌─────────────────────────────────────┐
//! │  Layer 1: Domain                    │  ← Business logic (no deps)
//! └─────────────────────────────────────┘
//! ```
//!
//! ## Usage
//!
//! ```rust,ignore
//! use allframe_core::arch::{domain, repository, use_case, handler};
//!
//! // Domain entities - pure business logic
//! #[domain]
//! struct User {
//!     id: String,
//!     email: String,
//! }
//!
//! // Repository - data access
//! #[repository]
//! trait UserRepository: Send + Sync {
//!     async fn find(&self, id: &str) -> Option<User>;
//! }
//!
//! // Use case - application logic
//! #[use_case]
//! struct GetUserUseCase {
//!     repo: Arc<dyn UserRepository>,  // ✅ Can depend on repository
//! }
//!
//! // Handler - entry point
//! #[handler]
//! struct GetUserHandler {
//!     use_case: Arc<GetUserUseCase>,  // ✅ Can depend on use case
//!     // repo: Arc<dyn UserRepository>, // ❌ COMPILE ERROR - cannot skip use case!
//! }
//! ```

// Re-export the architecture macros
#[cfg(feature = "di")]
pub use allframe_macros::{domain, handler, repository, use_case};

/// Layer metadata for runtime introspection
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LayerMetadata {
    /// Layer name
    pub layer_name: &'static str,
    /// Layer number (1-4)
    pub layer_number: u8,
    /// Type name
    pub type_name: &'static str,
}

impl LayerMetadata {
    /// Create new layer metadata
    pub const fn new(layer_name: &'static str, layer_number: u8, type_name: &'static str) -> Self {
        Self {
            layer_name,
            layer_number,
            type_name,
        }
    }

    /// Get the layer name
    pub fn layer_name(&self) -> &'static str {
        self.layer_name
    }

    /// Get the layer number
    pub fn layer_number(&self) -> u8 {
        self.layer_number
    }

    /// Get the type name
    pub fn type_name(&self) -> &'static str {
        self.type_name
    }

    /// Check if this layer can depend on another layer
    pub fn can_depend_on(&self, other_layer: &str) -> bool {
        let other_number = match other_layer {
            "domain" => 1,
            "repository" => 2,
            "use_case" => 3,
            "handler" => 4,
            _ => return false,
        };

        // A layer can only depend on layers with lower numbers
        self.layer_number > other_number
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_metadata_creation() {
        let metadata = LayerMetadata::new("domain", 1, "User");

        assert_eq!(metadata.layer_name(), "domain");
        assert_eq!(metadata.layer_number(), 1);
        assert_eq!(metadata.type_name(), "User");
    }

    #[test]
    fn test_can_depend_on() {
        let domain = LayerMetadata::new("domain", 1, "User");
        let repository = LayerMetadata::new("repository", 2, "UserRepository");
        let use_case = LayerMetadata::new("use_case", 3, "GetUserUseCase");
        let handler = LayerMetadata::new("handler", 4, "GetUserHandler");

        // Domain cannot depend on anything
        assert!(!domain.can_depend_on("repository"));
        assert!(!domain.can_depend_on("use_case"));
        assert!(!domain.can_depend_on("handler"));

        // Repository can only depend on domain
        assert!(repository.can_depend_on("domain"));
        assert!(!repository.can_depend_on("use_case"));
        assert!(!repository.can_depend_on("handler"));

        // Use case can depend on repository and domain
        assert!(use_case.can_depend_on("domain"));
        assert!(use_case.can_depend_on("repository"));
        assert!(!use_case.can_depend_on("handler"));

        // Handler can depend on all lower layers
        assert!(handler.can_depend_on("domain"));
        assert!(handler.can_depend_on("repository"));
        assert!(handler.can_depend_on("use_case"));
    }
}
