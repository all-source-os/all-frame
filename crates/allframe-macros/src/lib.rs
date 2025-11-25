//! AllFrame Procedural Macros
//!
//! This crate provides compile-time code generation for AllFrame,
//! including dependency injection, OpenAPI schema generation, and more.

#![deny(warnings)]

mod api;
mod di;

use proc_macro::TokenStream;

// Note: The `provide` attribute is handled directly by `di_container` macro
// It doesn't need a separate proc_macro_attribute since it's consumed during parsing

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

/// API handler with auto OpenAPI generation
///
/// Generates OpenAPI 3.1 schema for the annotated function.
///
/// # Example
/// ```ignore
/// #[api_handler(path = "/users", method = "POST", description = "Create user")]
/// async fn create_user(req: CreateUserRequest) -> CreateUserResponse {
///     // handler implementation
/// }
///
/// // Generated function:
/// // fn create_user_openapi_schema() -> String { /* JSON schema */ }
/// ```
#[proc_macro_attribute]
pub fn api_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    api::api_handler_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_macros_crate_compiles() {
        assert!(true);
    }
}
