//! AllFrame Procedural Macros
//!
//! This crate provides compile-time code generation for AllFrame,
//! including dependency injection, OpenAPI schema generation, and more.

#![deny(warnings)]

mod api;
mod arch;
mod cqrs;
mod di;
mod otel;

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

/// Marks a type as part of the Domain layer (Layer 1)
///
/// Domain entities contain pure business logic with no infrastructure dependencies.
///
/// # Example
/// ```ignore
/// #[domain]
/// struct User {
///     id: UserId,
///     email: Email,
/// }
/// ```
#[proc_macro_attribute]
pub fn domain(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    arch::domain_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a type as part of the Repository layer (Layer 2)
///
/// Repositories handle data access and can depend on Domain entities.
///
/// # Example
/// ```ignore
/// #[repository]
/// trait UserRepository: Send + Sync {
///     async fn find(&self, id: UserId) -> Option<User>;
/// }
/// ```
#[proc_macro_attribute]
pub fn repository(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    arch::repository_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a type as part of the Use Case layer (Layer 3)
///
/// Use cases orchestrate application logic and can depend on Repositories and Domain.
///
/// # Example
/// ```ignore
/// #[use_case]
/// struct GetUserUseCase {
///     repo: Arc<dyn UserRepository>,
/// }
/// ```
#[proc_macro_attribute]
pub fn use_case(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    arch::use_case_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a type as part of the Handler layer (Layer 4)
///
/// Handlers are entry points (HTTP/gRPC/GraphQL) and can only depend on Use Cases.
/// Handlers CANNOT depend on Repositories directly - they must go through Use Cases.
///
/// # Example
/// ```ignore
/// #[handler]
/// struct GetUserHandler {
///     use_case: Arc<GetUserUseCase>,
/// }
/// ```
#[proc_macro_attribute]
pub fn handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    arch::handler_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a struct as a Command (CQRS write operation)
///
/// Commands represent write operations that change state and produce events.
///
/// # Example
/// ```ignore
/// #[command]
/// struct CreateUserCommand {
///     email: String,
///     name: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn command(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    cqrs::command_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a struct as a Query (CQRS read operation)
///
/// Queries represent read operations that don't change state.
///
/// # Example
/// ```ignore
/// #[query]
/// struct GetUserQuery {
///     user_id: String,
/// }
/// ```
#[proc_macro_attribute]
pub fn query(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    cqrs::query_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks an enum or struct as an Event
///
/// Events represent immutable facts that have occurred in the system.
///
/// # Example
/// ```ignore
/// #[event]
/// enum UserEvent {
///     Created { user_id: String, email: String },
///     Updated { user_id: String, email: String },
/// }
/// ```
#[proc_macro_attribute]
pub fn event(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    cqrs::event_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a function as a Command Handler
///
/// Command handlers process commands and produce events.
///
/// # Example
/// ```ignore
/// #[command_handler]
/// async fn handle_create_user(cmd: CreateUserCommand) -> Result<Vec<Event>, String> {
///     Ok(vec![Event::UserCreated { ... }])
/// }
/// ```
#[proc_macro_attribute]
pub fn command_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    cqrs::command_handler_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a function as a Query Handler
///
/// Query handlers process queries and return data from projections.
///
/// # Example
/// ```ignore
/// #[query_handler]
/// async fn handle_get_user(query: GetUserQuery) -> Result<Option<User>, String> {
///     Ok(Some(user))
/// }
/// ```
#[proc_macro_attribute]
pub fn query_handler(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    cqrs::query_handler_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Marks a function to be automatically traced with OpenTelemetry
///
/// Automatically creates spans with proper context propagation.
///
/// # Example
/// ```ignore
/// #[traced]
/// async fn fetch_user(user_id: String) -> Result<User, Error> {
///     // Span created automatically with name "fetch_user"
///     // Span includes function arguments as attributes
///     Ok(user)
/// }
/// ```
#[proc_macro_attribute]
pub fn traced(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    otel::traced_impl(attr, item)
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
