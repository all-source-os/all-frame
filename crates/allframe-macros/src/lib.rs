//! AllFrame Procedural Macros
//!
//! This crate provides compile-time code generation for AllFrame,
//! including dependency injection, OpenAPI schema generation, and more.

#![deny(warnings)]

mod di;

use proc_macro::TokenStream;

/// Compile-time dependency injection container
///
/// Generates a container with automatic dependency resolution at compile time.
///
/// # Example
/// ```ignore
/// #[di_container]
/// struct AppContainer {
///     database: DatabaseService,
///     repository: UserRepository,
/// }
///
/// let container = AppContainer::new();
/// ```
#[proc_macro_attribute]
pub fn di_container(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    di::di_container_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// API handler with auto OpenAPI generation (not yet implemented)
#[proc_macro_attribute]
pub fn api_handler(_attr: TokenStream, item: TokenStream) -> TokenStream {
    // RED PHASE: Placeholder implementation
    item
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_crate_compiles() {
        assert!(true);
    }
}
