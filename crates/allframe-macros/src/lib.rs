//! AllFrame Procedural Macros
//!
//! This crate provides compile-time code generation for AllFrame,
//! including dependency injection, OpenAPI schema generation, and more.

#![deny(warnings)]

mod api;
mod arch;
mod cqrs;
mod di;
mod error;
mod health;
mod otel;
mod resilience;
mod security;

use proc_macro::TokenStream;

// Note: The `provide` attribute is handled directly by `di_container` macro
// It doesn't need a separate proc_macro_attribute since it's consumed during
// parsing

/// Compile-time dependency injection container
///
/// Generates a container with automatic dependency resolution at compile time.
///
/// # Attributes
///
/// - `#[provide(expr)]` - Use custom expression for initialization
/// - `#[provide(from_env)]` - Load from environment using `FromEnv` trait
/// - `#[provide(singleton)]` - Shared instance (default)
/// - `#[provide(transient)]` - New instance on each access
/// - `#[provide(async)]` - Async initialization using `AsyncInit` trait
/// - `#[depends(field1, field2)]` - Explicit dependencies
///
/// # Example (Sync)
/// ```ignore
/// #[di_container]
/// struct AppContainer {
///     database: DatabaseService,
///     repository: UserRepository,
/// }
///
/// let container = AppContainer::new();
/// ```
///
/// # Example (Async)
/// ```ignore
/// #[di_container]
/// struct AppContainer {
///     #[provide(from_env)]
///     config: Config,
///
///     #[provide(singleton, async)]
///     #[depends(config)]
///     database: DatabasePool,
///
///     #[provide(transient)]
///     service: MyService,
/// }
///
/// let container = AppContainer::build().await?;
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
/// Domain entities contain pure business logic with no infrastructure
/// dependencies.
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
/// Use cases orchestrate application logic and can depend on Repositories and
/// Domain.
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
/// Handlers are entry points (HTTP/gRPC/GraphQL) and can only depend on Use
/// Cases. Handlers CANNOT depend on Repositories directly - they must go
/// through Use Cases.
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

/// Derive macro for automatic gRPC status conversion
///
/// Generates `From<Error> for tonic::Status` implementation.
/// Use `#[grpc(CODE)]` on variants to specify the gRPC status code.
///
/// # Example
/// ```ignore
/// use allframe_macros::GrpcError;
/// use thiserror::Error;
///
/// #[derive(Error, Debug, GrpcError)]
/// pub enum AppError {
///     #[error("Unauthenticated: {0}")]
///     #[grpc(UNAUTHENTICATED)]
///     Unauthenticated(String),
///
///     #[error("Rate limited")]
///     #[grpc(RESOURCE_EXHAUSTED)]
///     RateLimited,
///
///     #[error("Not found: {0}")]
///     #[grpc(NOT_FOUND)]
///     NotFound(String),
///
///     #[error("Internal error: {0}")]
///     #[grpc(INTERNAL)]
///     Internal(String),
/// }
///
/// // Auto-generates: impl From<AppError> for tonic::Status
/// ```
///
/// # Supported gRPC Codes
/// - `OK`, `CANCELLED`, `UNKNOWN`, `INVALID_ARGUMENT`
/// - `DEADLINE_EXCEEDED`, `NOT_FOUND`, `ALREADY_EXISTS`
/// - `PERMISSION_DENIED`, `RESOURCE_EXHAUSTED`, `FAILED_PRECONDITION`
/// - `ABORTED`, `OUT_OF_RANGE`, `UNIMPLEMENTED`, `INTERNAL`
/// - `UNAVAILABLE`, `DATA_LOSS`, `UNAUTHENTICATED`
#[proc_macro_derive(GrpcError, attributes(grpc))]
pub fn grpc_error(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    error::grpc_error_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive macro for automatic HealthCheck implementation
///
/// Generates the `HealthCheck` trait implementation by collecting all fields
/// that implement the `Dependency` trait.
///
/// # Example
/// ```ignore
/// use allframe_macros::HealthCheck;
/// use allframe_core::health::{Dependency, DependencyStatus};
///
/// struct RedisDependency { /* ... */ }
/// impl Dependency for RedisDependency { /* ... */ }
///
/// struct DatabaseDependency { /* ... */ }
/// impl Dependency for DatabaseDependency { /* ... */ }
///
/// #[derive(HealthCheck)]
/// struct AppHealth {
///     #[health(timeout = "5s", critical = true)]
///     redis: RedisDependency,
///
///     #[health(timeout = "10s", critical = false)]
///     database: DatabaseDependency,
///
///     #[health(skip)]
///     config: Config, // Not a dependency
/// }
///
/// // Auto-generates:
/// // impl HealthCheck for AppHealth { ... }
/// ```
///
/// # Attributes
/// - `#[health(skip)]` - Skip this field from health checks
/// - `#[health(critical = true)]` - Mark dependency as critical (default: true)
/// - `#[health(timeout = "5s")]` - Set check timeout (default: 5s)
#[proc_macro_derive(HealthCheck, attributes(health))]
pub fn health_check(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    health::health_check_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Derive macro for automatic Obfuscate implementation
///
/// Generates the `Obfuscate` trait implementation that safely logs struct
/// fields, obfuscating those marked with `#[sensitive]`.
///
/// # Example
/// ```ignore
/// use allframe_macros::Obfuscate;
///
/// #[derive(Obfuscate)]
/// struct DatabaseConfig {
///     host: String,
///     port: u16,
///     #[sensitive]
///     password: String,
///     #[sensitive]
///     api_key: String,
/// }
///
/// let config = DatabaseConfig {
///     host: "localhost".to_string(),
///     port: 5432,
///     password: "secret".to_string(),
///     api_key: "sk_live_abc123".to_string(),
/// };
///
/// // Output: "DatabaseConfig { host: "localhost", port: 5432, password: ***, api_key: *** }"
/// println!("{}", config.obfuscate());
/// ```
///
/// # Attributes
/// - `#[sensitive]` - Mark field as sensitive, will be displayed as `***`
/// - `#[obfuscate(with = "function_name")]` - Use custom function to obfuscate
#[proc_macro_derive(Obfuscate, attributes(sensitive, obfuscate))]
pub fn obfuscate(input: TokenStream) -> TokenStream {
    let input = proc_macro2::TokenStream::from(input);

    security::obfuscate_impl(input)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for automatic retry with exponential backoff
///
/// Wraps an async function with retry logic using `RetryExecutor`.
///
/// # Example
/// ```ignore
/// use allframe_macros::retry;
///
/// #[retry(max_retries = 3, initial_interval_ms = 100)]
/// async fn fetch_data() -> Result<String, std::io::Error> {
///     // This will be retried up to 3 times on failure
///     reqwest::get("https://api.example.com/data")
///         .await?
///         .text()
///         .await
/// }
/// ```
///
/// # Parameters
/// - `max_retries` - Maximum retry attempts (default: 3)
/// - `initial_interval_ms` - Initial backoff in milliseconds (default: 500)
/// - `max_interval_ms` - Maximum backoff in milliseconds (default: 30000)
/// - `multiplier` - Backoff multiplier (default: 2.0)
#[proc_macro_attribute]
pub fn retry(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    resilience::retry_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for circuit breaker pattern
///
/// Wraps a function with circuit breaker logic for fail-fast behavior.
///
/// # Example
/// ```ignore
/// use allframe_macros::circuit_breaker;
///
/// #[circuit_breaker(name = "external_api", failure_threshold = 5)]
/// async fn call_external_api() -> Result<String, std::io::Error> {
///     // After 5 failures, the circuit opens and calls fail fast
///     external_service::call().await
/// }
/// ```
///
/// # Parameters
/// - `name` - Circuit breaker name (default: function name)
/// - `failure_threshold` - Failures before opening (default: 5)
/// - `success_threshold` - Successes to close in half-open (default: 3)
/// - `timeout_ms` - Time before half-open in milliseconds (default: 30000)
#[proc_macro_attribute]
pub fn circuit_breaker(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    resilience::circuit_breaker_impl(attr, item)
        .unwrap_or_else(|err| err.to_compile_error())
        .into()
}

/// Attribute macro for rate limiting
///
/// Wraps a function with rate limiting using token bucket algorithm.
///
/// # Example
/// ```ignore
/// use allframe_macros::rate_limited;
///
/// #[rate_limited(rps = 100, burst = 10)]
/// fn handle_request() -> Result<Response, std::io::Error> {
///     // Limited to 100 requests per second with burst of 10
///     process_request()
/// }
/// ```
///
/// # Parameters
/// - `rps` - Requests per second (default: 100)
/// - `burst` - Burst capacity (default: 10)
#[proc_macro_attribute]
pub fn rate_limited(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr = proc_macro2::TokenStream::from(attr);
    let item = proc_macro2::TokenStream::from(item);

    resilience::rate_limited_impl(attr, item)
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
