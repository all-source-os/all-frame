//! # AllFrame Core
//!
//! The composable Rust API framework.
//!
//! AllFrame is the first Rust web framework designed, built, and evolved
//! exclusively through Test-Driven Development (TDD).
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use allframe_core::prelude::*;
//!
//! #[tokio::main]
//! async fn main() {
//!     println!("AllFrame - One frame. Infinite transformations.");
//! }
//! ```

#![deny(warnings)]
#![deny(missing_docs)]
#![deny(unsafe_code)]

/// Prelude module for convenient imports
pub mod prelude {
    //! Commonly used imports for AllFrame applications
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_allframe_core_exists() {
        // This test verifies the crate compiles
        assert!(true);
    }
}
