//! # Protocol-Agnostic Router
//!
//! Write handlers once, expose them via REST, GraphQL, and gRPC.
//!
//! This is AllFrame's core differentiator - the same handler can serve
//! multiple protocols without code changes.
//!
//! ## Quick Start
//!
//! ```rust
//! use allframe_core::router::{Router, RestAdapter, GraphQLAdapter, GrpcAdapter};
//!
//! // Create router and register handlers
//! let mut router = Router::new();
//! router.register("get_user", || async {
//!     r#"{"id": 42, "name": "Alice"}"#.to_string()
//! });
//!
//! // Expose via REST
//! let mut rest = RestAdapter::new();
//! rest.route("GET", "/users/:id", "get_user");
//!
//! // Expose via GraphQL
//! let mut graphql = GraphQLAdapter::new();
//! graphql.query("user", "get_user");
//!
//! // Expose via gRPC
//! let mut grpc = GrpcAdapter::new();
//! grpc.unary("UserService", "GetUser", "get_user");
//! ```
//!
//! ## Key Types
//!
//! - `Router` - Central handler registry
//! - `RestAdapter` - REST protocol adapter
//! - `GraphQLAdapter` - GraphQL protocol adapter
//! - `GrpcAdapter` - gRPC protocol adapter
//! - `ProtocolAdapter` - Trait for custom protocol adapters
//!
//! ## API Documentation
//!
//! Generate beautiful API documentation automatically:
//!
//! - `scalar_html` - Scalar UI for REST APIs (<50KB)
//! - `graphiql_html` - GraphiQL playground for GraphQL
//! - `grpc_explorer_html` - gRPC Explorer for gRPC services
//! - `OpenApiGenerator` - OpenAPI 3.1 spec generation
//!
//! ## Configuration-Driven Protocol Selection
//!
//! Use TOML configuration to select protocols without code changes:
//!
//! ```toml
//! [server]
//! protocols = ["rest", "graphql", "grpc"]
//!
//! [server.rest]
//! port = 8080
//!
//! [server.graphql]
//! port = 8081
//! ```

use serde::de::DeserializeOwned;
use serde::Serialize;
use futures_core::Stream;
use std::{collections::HashMap, future::Future, pin::Pin, sync::Arc};

pub mod adapter;
pub mod builder;
#[cfg(feature = "router")]
pub mod config;
pub mod contract;
pub mod docs;
pub mod graphiql;
pub mod graphql;
pub mod grpc;
pub mod grpc_explorer;
pub mod handler;
pub mod metadata;
pub mod method;
pub mod openapi;
pub mod rest;
pub mod scalar;
pub mod schema;
pub mod ts_codegen;

// Production adapters (optional features)
#[cfg(feature = "router-graphql")]
pub mod graphql_prod;
#[cfg(feature = "router-grpc")]
pub mod grpc_prod;

pub use adapter::ProtocolAdapter;
pub use builder::RouteBuilder;
#[cfg(feature = "router")]
pub use config::{GraphQLConfig, GrpcConfig, RestConfig, RouterConfig, ServerConfig};
pub use contract::{
    ContractTestConfig, ContractTestResult, ContractTestResults, ContractTestable, ContractTester,
};
pub use docs::DocsConfig;
pub use graphiql::{graphiql_html, GraphiQLConfig, GraphiQLTheme};
pub use graphql::{GraphQLAdapter, GraphQLOperation, OperationType};
// Re-export production adapters when features are enabled
#[cfg(feature = "router-graphql")]
pub use graphql_prod::GraphQLProductionAdapter;
pub use grpc::{GrpcAdapter, GrpcMethod, GrpcMethodType, GrpcRequest, GrpcStatus};
pub use grpc_explorer::{grpc_explorer_html, GrpcExplorerConfig, GrpcExplorerTheme};
#[cfg(feature = "router-grpc")]
pub use grpc_prod::{protobuf, status, streaming, GrpcProductionAdapter, GrpcService};
pub use handler::{
    Handler, HandlerFn, HandlerWithArgs, HandlerWithState, HandlerWithStateOnly,
    IntoHandlerResult, IntoStreamItem, Json, SharedStateMap, State, StreamError, StreamHandler,
    StreamReceiver, StreamSender, StreamingHandlerFn, StreamingHandlerWithArgs,
    StreamingHandlerWithState, StreamingHandlerWithStateOnly, DEFAULT_STREAM_CAPACITY,
};
pub use metadata::RouteMetadata;
pub use method::Method;
pub use openapi::{OpenApiGenerator, OpenApiServer};
pub use rest::{RestAdapter, RestRequest, RestResponse, RestRoute};
pub use scalar::{scalar_html, ScalarConfig, ScalarLayout, ScalarTheme};
pub use schema::ToJsonSchema;
pub use ts_codegen::{generate_ts_client, HandlerMeta, TsField, TsType};

/// Drive a `Stream` to completion, forwarding items through a `StreamSender`.
///
/// Returns a JSON result: `"null"` on success, or a JSON error string if a
/// serialization error occurred.
async fn drive_stream<T, St>(stream: St, tx: &StreamSender) -> String
where
    T: IntoStreamItem,
    St: Stream<Item = T> + Send,
{
    tokio::pin!(stream);
    loop {
        let next = std::future::poll_fn(|cx| stream.as_mut().poll_next(cx)).await;
        match next {
            Some(item) => match tx.send(item).await {
                Ok(()) => {}
                Err(StreamError::Closed) => break,
                Err(StreamError::Serialize(e)) => {
                    return serde_json::json!({"error": e}).to_string();
                }
            },
            None => break,
        }
    }
    "null".to_string()
}

/// Router manages handler registration and protocol adapters
///
/// The router allows you to register handlers once and expose them via
/// multiple protocols based on configuration.
pub struct Router {
    handlers: HashMap<String, Box<dyn Handler>>,
    streaming_handlers: HashMap<String, Box<dyn StreamHandler>>,
    adapters: HashMap<String, Box<dyn ProtocolAdapter>>,
    routes: Vec<RouteMetadata>,
    states: SharedStateMap,
    handler_metas: HashMap<String, HandlerMeta>,
    #[cfg(feature = "router")]
    #[allow(dead_code)]
    config: Option<RouterConfig>,
}

impl Router {
    /// Create a new router
    pub fn new() -> Self {
        Self {
            handlers: HashMap::new(),
            streaming_handlers: HashMap::new(),
            adapters: HashMap::new(),
            routes: Vec::new(),
            states: Arc::new(std::sync::RwLock::new(HashMap::new())),
            handler_metas: HashMap::new(),
            #[cfg(feature = "router")]
            config: None,
        }
    }

    /// Create a new router with configuration
    #[cfg(feature = "router")]
    pub fn with_config(config: RouterConfig) -> Self {
        let mut router = Self {
            handlers: HashMap::new(),
            streaming_handlers: HashMap::new(),
            adapters: HashMap::new(),
            routes: Vec::new(),
            states: Arc::new(std::sync::RwLock::new(HashMap::new())),
            handler_metas: HashMap::new(),
            config: Some(config.clone()),
        };

        // Auto-register adapters based on config
        if config.has_protocol("rest") {
            router.add_adapter(Box::new(RestAdapter::new()));
        }
        if config.has_protocol("graphql") {
            router.add_adapter(Box::new(GraphQLAdapter::new()));
        }
        if config.has_protocol("grpc") {
            router.add_adapter(Box::new(GrpcAdapter::new()));
        }

        router
    }

    /// Add shared state for dependency injection (builder pattern).
    ///
    /// Can be called multiple times with different types. Each state type
    /// is stored independently and looked up by `TypeId` at registration time.
    /// Calling twice with the same `S` replaces the previous value.
    ///
    /// ```rust,ignore
    /// let router = Router::new()
    ///     .with_state(db_pool)       // handlers can request State<Arc<DbPool>>
    ///     .with_state(app_handle);   // handlers can request State<Arc<AppHandle>>
    /// ```
    pub fn with_state<S: Send + Sync + 'static>(mut self, state: S) -> Self {
        self.insert_state::<S>(state);
        self
    }

    /// Add shared state for dependency injection (mutable, non-builder).
    ///
    /// Like `with_state` but takes `&mut self` instead of consuming `self`.
    /// Useful when the state is not available at router construction time
    /// (e.g., Tauri's `AppHandle` which is only available during plugin setup).
    /// Calling twice with the same `S` replaces the previous value.
    pub fn inject_state<S: Send + Sync + 'static>(&mut self, state: S) {
        self.insert_state::<S>(state);
    }

    fn insert_state<S: Send + Sync + 'static>(&mut self, state: S) {
        let id = std::any::TypeId::of::<S>();
        let mut map = self.states.write().expect("state lock poisoned");
        if map.contains_key(&id) {
            #[cfg(debug_assertions)]
            eprintln!(
                "allframe: with_state called twice for type `{}` — previous value replaced",
                std::any::type_name::<S>()
            );
        }
        map.insert(id, Arc::new(state));
    }

    /// Returns a handle to the shared state map.
    ///
    /// This is the same `Arc` used internally, so state injected through
    /// the returned handle is visible to handlers at call time. Used by
    /// `BootContext` to inject state during async boot.
    pub fn shared_states(&self) -> SharedStateMap {
        self.states.clone()
    }

    /// Register a handler with a name (zero-arg, backward compatible)
    pub fn register<F, Fut>(&mut self, name: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        self.handlers
            .insert(name.to_string(), Box::new(HandlerFn::new(handler)));
    }

    /// Register a handler that receives typed, deserialized args
    pub fn register_with_args<T, F, Fut>(&mut self, name: &str, handler: F)
    where
        T: DeserializeOwned + Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithArgs::new(handler)));
    }

    /// Register a handler that receives injected state and typed args
    ///
    /// # Panics
    ///
    /// Panics at registration time if `with_state::<S>()` was not called.
    pub fn register_with_state<S, T, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        T: DeserializeOwned + Send + 'static,
        F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let state = self.states.clone();
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithState::new(handler, state)));
    }

    /// Register a handler that receives only injected state (no args)
    ///
    /// # Panics
    ///
    /// Panics at registration time if `with_state::<S>()` was not called.
    pub fn register_with_state_only<S, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        F: Fn(State<Arc<S>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let state = self.states.clone();
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithStateOnly::new(handler, state)));
    }

    // ─── Typed return registration (auto-serialize via Json wrapper) ─────

    /// Register a handler that returns `R: Serialize` (auto-serialized to JSON)
    pub fn register_typed<R, F, Fut>(&mut self, name: &str, handler: F)
    where
        R: Serialize + Send + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        let wrapped = move || {
            let fut = handler();
            async move { Json(fut.await) }
        };
        self.handlers
            .insert(name.to_string(), Box::new(HandlerFn::new(wrapped)));
    }

    /// Register a handler that accepts typed args and returns `R: Serialize`
    pub fn register_typed_with_args<T, R, F, Fut>(&mut self, name: &str, handler: F)
    where
        T: DeserializeOwned + Send + 'static,
        R: Serialize + Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        let wrapped = move |args: T| {
            let fut = handler(args);
            async move { Json(fut.await) }
        };
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithArgs::new(wrapped)));
    }

    /// Register a handler that receives state + typed args and returns `R: Serialize`
    pub fn register_typed_with_state<S, T, R, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        T: DeserializeOwned + Send + 'static,
        R: Serialize + Send + 'static,
        F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        let state = self.states.clone();
        let wrapped = move |s: State<Arc<S>>, args: T| {
            let fut = handler(s, args);
            async move { Json(fut.await) }
        };
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithState::new(wrapped, state)));
    }

    /// Register a handler that receives state only and returns `R: Serialize`
    pub fn register_typed_with_state_only<S, R, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        R: Serialize + Send + 'static,
        F: Fn(State<Arc<S>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
    {
        let state = self.states.clone();
        let wrapped = move |s: State<Arc<S>>| {
            let fut = handler(s);
            async move { Json(fut.await) }
        };
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithStateOnly::new(wrapped, state)));
    }

    // ─── Result return registration ─────────────────────────────────────

    /// Register a handler returning `Result<R, E>` (no args)
    ///
    /// On `Ok(value)`, `value` is serialized to JSON. On `Err(e)`, the error
    /// is returned as a string.
    pub fn register_result<R, E, F, Fut>(&mut self, name: &str, handler: F)
    where
        R: Serialize + Send + 'static,
        E: std::fmt::Display + Send + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        self.handlers
            .insert(name.to_string(), Box::new(HandlerFn::new(handler)));
    }

    /// Register a handler returning `Result<R, E>` with typed args
    pub fn register_result_with_args<T, R, E, F, Fut>(&mut self, name: &str, handler: F)
    where
        T: DeserializeOwned + Send + 'static,
        R: Serialize + Send + 'static,
        E: std::fmt::Display + Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithArgs::new(handler)));
    }

    /// Register a handler returning `Result<R, E>` with state + typed args
    pub fn register_result_with_state<S, T, R, E, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        T: DeserializeOwned + Send + 'static,
        R: Serialize + Send + 'static,
        E: std::fmt::Display + Send + 'static,
        F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        let state = self.states.clone();
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithState::new(handler, state)));
    }

    /// Register a handler returning `Result<R, E>` with state only
    pub fn register_result_with_state_only<S, R, E, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        R: Serialize + Send + 'static,
        E: std::fmt::Display + Send + 'static,
        F: Fn(State<Arc<S>>) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = Result<R, E>> + Send + 'static,
    {
        let state = self.states.clone();
        self.handlers
            .insert(name.to_string(), Box::new(HandlerWithStateOnly::new(handler, state)));
    }

    /// Get the number of registered handlers (request/response only)
    pub fn handlers_count(&self) -> usize {
        self.handlers.len()
    }

    // ─── Streaming handler registration ─────────────────────────────────

    /// Register a streaming handler (no args, receives StreamSender)
    pub fn register_streaming<F, Fut, R>(&mut self, name: &str, handler: F)
    where
        F: Fn(StreamSender) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoHandlerResult + 'static,
    {
        self.streaming_handlers
            .insert(name.to_string(), Box::new(StreamingHandlerFn::new(handler)));
    }

    /// Register a streaming handler with typed args
    pub fn register_streaming_with_args<T, F, Fut, R>(&mut self, name: &str, handler: F)
    where
        T: DeserializeOwned + Send + 'static,
        F: Fn(T, StreamSender) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoHandlerResult + 'static,
    {
        self.streaming_handlers
            .insert(name.to_string(), Box::new(StreamingHandlerWithArgs::new(handler)));
    }

    /// Register a streaming handler with state and typed args
    pub fn register_streaming_with_state<S, T, F, Fut, R>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        T: DeserializeOwned + Send + 'static,
        F: Fn(State<Arc<S>>, T, StreamSender) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoHandlerResult + 'static,
    {
        let state = self.states.clone();
        self.streaming_handlers
            .insert(name.to_string(), Box::new(StreamingHandlerWithState::new(handler, state)));
    }

    /// Register a streaming handler with state only (no args)
    pub fn register_streaming_with_state_only<S, F, Fut, R>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        F: Fn(State<Arc<S>>, StreamSender) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = R> + Send + 'static,
        R: IntoHandlerResult + 'static,
    {
        let state = self.states.clone();
        self.streaming_handlers
            .insert(name.to_string(), Box::new(StreamingHandlerWithStateOnly::new(handler, state)));
    }

    /// Register a handler that returns a `Stream` of items (no args).
    ///
    /// The stream is internally bridged to a `StreamSender` channel.
    /// Items are forwarded through a bounded channel (default capacity 64).
    /// Register a handler that returns a `Stream` of items (no args).
    ///
    /// The stream is internally bridged to a `StreamSender` channel.
    /// Items are forwarded through a bounded channel (default capacity 64).
    pub fn register_stream<T, St, F, Fut>(&mut self, name: &str, handler: F)
    where
        T: IntoStreamItem + 'static,
        St: Stream<Item = T> + Send + 'static,
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = St> + Send + 'static,
    {
        self.register_streaming(name, move |tx: StreamSender| {
            let stream_fut = handler();
            async move {
                drive_stream(stream_fut.await, &tx).await
            }
        });
    }

    /// Register a handler that takes typed args and returns a `Stream` of items.
    pub fn register_stream_with_args<T, Item, St, F, Fut>(&mut self, name: &str, handler: F)
    where
        T: DeserializeOwned + Send + 'static,
        Item: IntoStreamItem + 'static,
        St: Stream<Item = Item> + Send + 'static,
        F: Fn(T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = St> + Send + 'static,
    {
        self.register_streaming_with_args::<T, _, _, _>(name, move |args: T, tx: StreamSender| {
            let stream_fut = handler(args);
            async move {
                drive_stream(stream_fut.await, &tx).await
            }
        });
    }

    /// Register a handler that takes state + typed args and returns a `Stream` of items.
    pub fn register_stream_with_state<S, T, Item, St, F, Fut>(&mut self, name: &str, handler: F)
    where
        S: Send + Sync + 'static,
        T: DeserializeOwned + Send + 'static,
        Item: IntoStreamItem + 'static,
        St: Stream<Item = Item> + Send + 'static,
        F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = St> + Send + 'static,
    {
        self.register_streaming_with_state::<S, T, _, _, _>(name, move |state: State<Arc<S>>, args: T, tx: StreamSender| {
            let stream_fut = handler(state, args);
            async move {
                drive_stream(stream_fut.await, &tx).await
            }
        });
    }

    /// Check if a handler is a streaming handler
    pub fn is_streaming(&self, name: &str) -> bool {
        self.streaming_handlers.contains_key(name)
    }

    /// Call a streaming handler by name.
    ///
    /// Creates a bounded channel, passes the sender to the handler,
    /// and returns `(receiver, completion_future)`. The completion future
    /// resolves with the handler's final result.
    ///
    /// Note: the returned future borrows `self`. For use in contexts where
    /// a `'static` future is needed (e.g., `tokio::spawn`), use
    /// `spawn_streaming_handler` instead.
    #[allow(clippy::type_complexity)]
    pub fn call_streaming_handler(
        &self,
        name: &str,
        args: &str,
    ) -> Result<
        (
            StreamReceiver,
            Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>,
        ),
        String,
    > {
        let handler = self
            .streaming_handlers
            .get(name)
            .ok_or_else(|| format!("Streaming handler '{}' not found", name))?;

        let (tx, rx) = StreamSender::channel();
        let fut = handler.call_streaming(args, tx);
        Ok((rx, fut))
    }

    /// Spawn a streaming handler as a tokio task.
    ///
    /// Like `call_streaming_handler` but returns a `'static` receiver and
    /// `JoinHandle`, suitable for use with `tokio::spawn`.
    #[allow(clippy::type_complexity)]
    pub fn spawn_streaming_handler(
        self: &Arc<Self>,
        name: &str,
        args: &str,
    ) -> Result<
        (
            StreamReceiver,
            tokio::task::JoinHandle<Result<String, String>>,
        ),
        String,
    > {
        if !self.streaming_handlers.contains_key(name) {
            return Err(format!("Streaming handler '{}' not found", name));
        }

        let router = self.clone();
        let name = name.to_string();
        let args = args.to_string();

        let (tx, rx) = StreamSender::channel();

        let handle = tokio::spawn(async move {
            let handler = router
                .streaming_handlers
                .get(&name)
                .expect("handler verified to exist");
            handler.call_streaming(&args, tx).await
        });

        Ok((rx, handle))
    }

    // ─── TypeScript codegen metadata ────────────────────────────────────

    /// Attach type metadata to a handler for TypeScript client generation
    ///
    /// The metadata describes the handler's argument fields and return type,
    /// which `generate_ts_client()` uses to generate typed async functions.
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::{Router, TsField, TsType};
    ///
    /// let mut router = Router::new();
    /// router.register("get_user", || async { r#"{"id":1}"#.to_string() });
    /// router.describe_handler("get_user", vec![], TsType::Object(vec![
    ///     TsField::new("id", TsType::Number),
    ///     TsField::new("name", TsType::String),
    /// ]));
    /// ```
    pub fn describe_handler(
        &mut self,
        name: &str,
        args: Vec<TsField>,
        returns: TsType,
    ) {
        assert!(
            self.handlers.contains_key(name),
            "describe_handler: handler '{}' not registered",
            name
        );
        self.handler_metas
            .insert(name.to_string(), HandlerMeta::new(args, returns));
    }

    /// Attach type metadata to a streaming handler for TypeScript client generation
    pub fn describe_streaming_handler(
        &mut self,
        name: &str,
        args: Vec<TsField>,
        item_type: TsType,
        final_type: TsType,
    ) {
        assert!(
            self.streaming_handlers.contains_key(name),
            "describe_streaming_handler: streaming handler '{}' not registered",
            name
        );
        self.handler_metas
            .insert(name.to_string(), HandlerMeta::streaming(args, item_type, final_type));
    }

    /// Generate a complete TypeScript client module from all described handlers
    ///
    /// Returns a string containing TypeScript code with typed async functions
    /// for each handler that has metadata attached via `describe_handler()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::{Router, TsField, TsType};
    ///
    /// let mut router = Router::new();
    /// router.register("get_user", || async { r#"{"id":1}"#.to_string() });
    /// router.describe_handler("get_user", vec![
    ///     TsField::new("id", TsType::Number),
    /// ], TsType::Object(vec![
    ///     TsField::new("id", TsType::Number),
    ///     TsField::new("name", TsType::String),
    /// ]));
    ///
    /// let ts_code = router.generate_ts_client();
    /// assert!(ts_code.contains("export async function getUser"));
    /// ```
    pub fn generate_ts_client(&self) -> String {
        generate_ts_client(&self.handler_metas)
    }

    /// Get handler metadata (for inspection/testing)
    pub fn handler_meta(&self, name: &str) -> Option<&HandlerMeta> {
        self.handler_metas.get(name)
    }

    /// Add a protocol adapter
    pub fn add_adapter(&mut self, adapter: Box<dyn ProtocolAdapter>) {
        self.adapters.insert(adapter.name().to_string(), adapter);
    }

    /// Check if an adapter is registered
    pub fn has_adapter(&self, name: &str) -> bool {
        self.adapters.contains_key(name)
    }

    /// Get an adapter by name
    pub fn get_adapter(&self, name: &str) -> Option<&dyn ProtocolAdapter> {
        self.adapters.get(name).map(|b| &**b)
    }

    /// Route a request through the appropriate protocol adapter
    pub async fn route_request(&self, protocol: &str, request: &str) -> Result<String, String> {
        let adapter = self
            .get_adapter(protocol)
            .ok_or_else(|| format!("Adapter not found: {}", protocol))?;

        adapter.handle(request).await
    }

    /// Execute a handler by name (zero-arg shorthand)
    pub async fn execute(&self, name: &str) -> Result<String, String> {
        self.execute_with_args(name, "{}").await
    }

    /// Execute a handler by name with JSON args
    pub async fn execute_with_args(&self, name: &str, args: &str) -> Result<String, String> {
        match self.handlers.get(name) {
            Some(handler) => handler.call(args).await,
            None => Err(format!("Handler '{}' not found", name)),
        }
    }

    /// List all registered handler names (both request/response and streaming)
    ///
    /// Returns a vector of all handler names that have been registered
    /// with this router. Used by MCP server for tool discovery.
    pub fn list_handlers(&self) -> Vec<String> {
        let mut names: Vec<String> = self.handlers.keys().cloned().collect();
        names.extend(self.streaming_handlers.keys().cloned());
        names
    }

    /// Call a handler by name with request data (JSON args)
    ///
    /// Forwards the request string as args to the handler.
    /// Used by MCP server and Tauri plugin.
    pub async fn call_handler(&self, name: &str, request: &str) -> Result<String, String> {
        self.execute_with_args(name, request).await
    }

    /// Check if handler can be called via REST
    pub fn can_handle_rest(&self, _name: &str) -> bool {
        self.has_adapter("rest")
    }

    /// Check if handler can be called via GraphQL
    pub fn can_handle_graphql(&self, _name: &str) -> bool {
        self.has_adapter("graphql")
    }

    /// Check if handler can be called via gRPC
    pub fn can_handle_grpc(&self, _name: &str) -> bool {
        self.has_adapter("grpc")
    }

    /// Get list of enabled protocols
    pub fn enabled_protocols(&self) -> Vec<String> {
        self.adapters.keys().cloned().collect()
    }

    /// Add a route with metadata
    ///
    /// This stores route metadata that can be used to generate
    /// documentation (OpenAPI, GraphQL schemas, gRPC reflection).
    pub fn add_route(&mut self, metadata: RouteMetadata) {
        self.routes.push(metadata);
    }

    /// Get all registered routes
    ///
    /// Returns an immutable reference to all route metadata.
    /// This is used for documentation generation.
    pub fn routes(&self) -> &[RouteMetadata] {
        &self.routes
    }

    /// Register a GET route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a GET request. The handler name is automatically
    /// generated as "GET:{path}".
    pub fn get<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("GET:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::GET, "rest"));
    }

    /// Register a POST route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a POST request. The handler name is automatically
    /// generated as "POST:{path}".
    pub fn post<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("POST:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::POST, "rest"));
    }

    /// Register a PUT route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a PUT request. The handler name is automatically
    /// generated as "PUT:{path}".
    pub fn put<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("PUT:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::PUT, "rest"));
    }

    /// Register a DELETE route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a DELETE request. The handler name is automatically
    /// generated as "DELETE:{path}".
    pub fn delete<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("DELETE:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::DELETE, "rest"));
    }

    /// Register a PATCH route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a PATCH request. The handler name is automatically
    /// generated as "PATCH:{path}".
    pub fn patch<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("PATCH:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::PATCH, "rest"));
    }

    /// Register a HEAD route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for a HEAD request. The handler name is automatically
    /// generated as "HEAD:{path}".
    pub fn head<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("HEAD:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::HEAD, "rest"));
    }

    /// Register an OPTIONS route
    ///
    /// This is a convenience method that registers both a handler and route
    /// metadata for an OPTIONS request. The handler name is automatically
    /// generated as "OPTIONS:{path}".
    pub fn options<F, Fut>(&mut self, path: &str, handler: F)
    where
        F: Fn() -> Fut + Send + Sync + 'static,
        Fut: Future<Output = String> + Send + 'static,
    {
        let handler_name = format!("OPTIONS:{}", path);
        self.register(&handler_name, handler);
        self.add_route(RouteMetadata::new(path, Method::OPTIONS, "rest"));
    }

    /// Call handler via REST
    pub async fn call_rest(&self, method: &str, path: &str) -> Result<String, String> {
        let adapter = self
            .adapters
            .get("rest")
            .ok_or_else(|| "REST adapter not enabled".to_string())?;

        let request = format!("{} {}", method, path);
        adapter.handle(&request).await
    }

    /// Call handler via GraphQL
    pub async fn call_graphql(&self, query: &str) -> Result<String, String> {
        let adapter = self
            .adapters
            .get("graphql")
            .ok_or_else(|| "GraphQL adapter not enabled".to_string())?;

        adapter.handle(query).await
    }

    /// Call handler via gRPC
    pub async fn call_grpc(&self, method: &str, request: &str) -> Result<String, String> {
        let adapter = self
            .adapters
            .get("grpc")
            .ok_or_else(|| "gRPC adapter not enabled".to_string())?;

        let grpc_request = format!("{}:{}", method, request);
        adapter.handle(&grpc_request).await
    }

    /// Generate Scalar documentation HTML with default configuration
    ///
    /// This is a convenience method that generates a complete HTML page
    /// with Scalar UI for interactive API documentation.
    ///
    /// # Arguments
    ///
    /// * `title` - API title
    /// * `version` - API version
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::Router;
    ///
    /// let mut router = Router::new();
    /// router.get("/users", || async { "Users".to_string() });
    ///
    /// let html = router.scalar("My API", "1.0.0");
    /// // Serve this HTML at /docs endpoint
    /// ```
    pub fn scalar(&self, title: &str, version: &str) -> String {
        let config = scalar::ScalarConfig::default();
        self.scalar_docs(config, title, version)
    }

    /// Generate Scalar documentation HTML with custom configuration
    ///
    /// This method allows full customization of the Scalar UI appearance
    /// and behavior.
    ///
    /// # Arguments
    ///
    /// * `config` - Scalar configuration
    /// * `title` - API title
    /// * `version` - API version
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::{Router, ScalarConfig, ScalarTheme};
    ///
    /// let mut router = Router::new();
    /// router.get("/users", || async { "Users".to_string() });
    ///
    /// let config = ScalarConfig::new()
    ///     .theme(ScalarTheme::Light)
    ///     .show_sidebar(false);
    ///
    /// let html = router.scalar_docs(config, "My API", "1.0.0");
    /// ```
    pub fn scalar_docs(&self, config: scalar::ScalarConfig, title: &str, version: &str) -> String {
        // Generate OpenAPI spec
        let spec = OpenApiGenerator::new(title, version).generate(self);
        let spec_json = serde_json::to_string(&spec).unwrap_or_else(|_| "{}".to_string());

        // Generate Scalar HTML
        scalar::scalar_html(&config, title, &spec_json)
    }
}

impl Default for Router {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_router_creation() {
        let router = Router::new();
        assert_eq!(router.handlers_count(), 0);
    }

    #[tokio::test]
    async fn test_handler_registration() {
        let mut router = Router::new();
        router.register("test", || async { "Hello".to_string() });
        assert_eq!(router.handlers_count(), 1);
    }

    #[tokio::test]
    async fn test_handler_execution() {
        let mut router = Router::new();
        router.register("test", || async { "Hello".to_string() });
        let result = router.execute("test").await;
        assert_eq!(result, Ok("Hello".to_string()));
    }

    // New tests for route metadata tracking
    #[tokio::test]
    async fn test_router_starts_with_no_routes() {
        let router = Router::new();
        let routes = router.routes();
        assert_eq!(routes.len(), 0);
    }

    #[tokio::test]
    async fn test_add_route_metadata() {
        let mut router = Router::new();
        let metadata = RouteMetadata::new("/users", "GET", "rest");

        router.add_route(metadata.clone());

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[0].protocol, "rest");
    }

    #[tokio::test]
    async fn test_add_multiple_routes() {
        let mut router = Router::new();

        router.add_route(RouteMetadata::new("/users", "GET", "rest"));
        router.add_route(RouteMetadata::new("/users", "POST", "rest"));
        router.add_route(RouteMetadata::new("/posts", "GET", "rest"));

        let routes = router.routes();
        assert_eq!(routes.len(), 3);
    }

    #[tokio::test]
    async fn test_routes_with_different_protocols() {
        let mut router = Router::new();

        router.add_route(RouteMetadata::new("/users", "GET", "rest"));
        router.add_route(RouteMetadata::new("users", "query", "graphql"));
        router.add_route(RouteMetadata::new("UserService.GetUser", "unary", "grpc"));

        let routes = router.routes();
        assert_eq!(routes.len(), 3);

        assert_eq!(routes[0].protocol, "rest");
        assert_eq!(routes[1].protocol, "graphql");
        assert_eq!(routes[2].protocol, "grpc");
    }

    #[tokio::test]
    async fn test_routes_returns_immutable_reference() {
        let mut router = Router::new();
        router.add_route(RouteMetadata::new("/test", "GET", "rest"));

        let routes1 = router.routes();
        let routes2 = router.routes();

        // Both should have the same data
        assert_eq!(routes1.len(), routes2.len());
        assert_eq!(routes1[0].path, routes2[0].path);
    }

    // Tests for type-safe route registration
    #[tokio::test]
    async fn test_route_get_method() {
        let mut router = Router::new();
        router.get("/users", || async { "User list".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[0].protocol, "rest");
    }

    #[tokio::test]
    async fn test_route_post_method() {
        let mut router = Router::new();
        router.post("/users", || async { "User created".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].path, "/users");
        assert_eq!(routes[0].method, "POST");
        assert_eq!(routes[0].protocol, "rest");
    }

    #[tokio::test]
    async fn test_route_put_method() {
        let mut router = Router::new();
        router.put("/users/1", || async { "User updated".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "PUT");
    }

    #[tokio::test]
    async fn test_route_delete_method() {
        let mut router = Router::new();
        router.delete("/users/1", || async { "User deleted".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "DELETE");
    }

    #[tokio::test]
    async fn test_route_patch_method() {
        let mut router = Router::new();
        router.patch("/users/1", || async { "User patched".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 1);
        assert_eq!(routes[0].method, "PATCH");
    }

    #[tokio::test]
    async fn test_multiple_routes_different_methods() {
        let mut router = Router::new();
        router.get("/users", || async { "List".to_string() });
        router.post("/users", || async { "Create".to_string() });
        router.put("/users/1", || async { "Update".to_string() });
        router.delete("/users/1", || async { "Delete".to_string() });

        let routes = router.routes();
        assert_eq!(routes.len(), 4);

        assert_eq!(routes[0].method, "GET");
        assert_eq!(routes[1].method, "POST");
        assert_eq!(routes[2].method, "PUT");
        assert_eq!(routes[3].method, "DELETE");
    }

    #[tokio::test]
    async fn test_route_method_with_path_params() {
        let mut router = Router::new();
        router.get("/users/{id}", || async { "User details".to_string() });
        router.get("/users/{id}/posts/{post_id}", || async {
            "Post details".to_string()
        });

        let routes = router.routes();
        assert_eq!(routes.len(), 2);
        assert_eq!(routes[0].path, "/users/{id}");
        assert_eq!(routes[1].path, "/users/{id}/posts/{post_id}");
    }

    #[tokio::test]
    async fn test_route_registration_and_execution() {
        let mut router = Router::new();
        router.get("/test", || async { "GET response".to_string() });
        router.post("/test", || async { "POST response".to_string() });

        // Both route metadata and handler should be registered
        assert_eq!(router.routes().len(), 2);
        assert_eq!(router.handlers_count(), 2);

        // Handlers should be executable
        let result1 = router.execute("GET:/test").await;
        let result2 = router.execute("POST:/test").await;

        assert_eq!(result1, Ok("GET response".to_string()));
        assert_eq!(result2, Ok("POST response".to_string()));
    }

    // Tests for Scalar integration
    #[tokio::test]
    async fn test_scalar_generates_html() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });

        let html = router.scalar("Test API", "1.0.0");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("<title>Test API - API Documentation</title>"));
        assert!(html.contains("@scalar/api-reference"));
    }

    #[tokio::test]
    async fn test_scalar_contains_openapi_spec() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });
        router.post("/users", || async { "User created".to_string() });

        let html = router.scalar("Test API", "1.0.0");

        // Should contain the OpenAPI spec
        assert!(html.contains("openapi"));
        assert!(html.contains("Test API"));
        assert!(html.contains("1.0.0"));
    }

    #[tokio::test]
    async fn test_scalar_docs_with_custom_config() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });

        let config = scalar::ScalarConfig::new()
            .theme(scalar::ScalarTheme::Light)
            .show_sidebar(false);

        let html = router.scalar_docs(config, "Custom API", "2.0.0");

        assert!(html.contains("Custom API"));
        assert!(html.contains(r#""theme":"light""#));
        assert!(html.contains(r#""showSidebar":false"#));
    }

    #[tokio::test]
    async fn test_scalar_docs_with_custom_css() {
        let mut router = Router::new();
        router.get("/test", || async { "Test".to_string() });

        let config = scalar::ScalarConfig::new().custom_css("body { font-family: 'Inter'; }");

        let html = router.scalar_docs(config, "API", "1.0");

        assert!(html.contains("<style>body { font-family: 'Inter'; }</style>"));
    }

    #[tokio::test]
    async fn test_scalar_with_multiple_routes() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });
        router.post("/users", || async { "Create".to_string() });
        router.get("/users/{id}", || async { "User details".to_string() });
        router.delete("/users/{id}", || async { "Delete".to_string() });

        let html = router.scalar("API", "1.0.0");

        // Should contain all routes in the OpenAPI spec
        assert!(html.contains("/users"));
    }

    // Tests for protocol adapter management
    #[tokio::test]
    async fn test_get_adapter_returns_adapter() {
        let mut router = Router::new();
        router.add_adapter(Box::new(RestAdapter::new()));

        let adapter = router.get_adapter("rest");
        assert!(adapter.is_some());
        assert_eq!(adapter.unwrap().name(), "rest");
    }

    #[tokio::test]
    async fn test_get_adapter_returns_none_for_missing() {
        let router = Router::new();
        let adapter = router.get_adapter("rest");
        assert!(adapter.is_none());
    }

    #[tokio::test]
    async fn test_route_request_success() {
        let mut router = Router::new();
        router.register("test_handler", || async { "Success!".to_string() });

        // Register adapter with a route
        let mut rest_adapter = RestAdapter::new();
        rest_adapter.route("GET", "/test", "test_handler");
        router.add_adapter(Box::new(rest_adapter));

        let result = router.route_request("rest", "GET /test").await;
        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains("HTTP 200") || response.contains("test_handler"));
    }

    #[tokio::test]
    async fn test_route_request_unknown_adapter() {
        let router = Router::new();
        let result = router.route_request("unknown", "test").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Adapter not found"));
    }

    #[tokio::test]
    async fn test_enabled_protocols_empty() {
        let router = Router::new();
        let protocols = router.enabled_protocols();
        assert_eq!(protocols.len(), 0);
    }

    #[tokio::test]
    async fn test_enabled_protocols_multiple() {
        let mut router = Router::new();
        router.add_adapter(Box::new(RestAdapter::new()));
        router.add_adapter(Box::new(GraphQLAdapter::new()));
        router.add_adapter(Box::new(GrpcAdapter::new()));

        let protocols = router.enabled_protocols();
        assert_eq!(protocols.len(), 3);
        assert!(protocols.contains(&"rest".to_string()));
        assert!(protocols.contains(&"graphql".to_string()));
        assert!(protocols.contains(&"grpc".to_string()));
    }

    #[tokio::test]
    async fn test_can_handle_rest() {
        let mut router = Router::new();
        assert!(!router.can_handle_rest("test"));

        router.add_adapter(Box::new(RestAdapter::new()));
        assert!(router.can_handle_rest("test"));
    }

    #[tokio::test]
    async fn test_can_handle_graphql() {
        let mut router = Router::new();
        assert!(!router.can_handle_graphql("test"));

        router.add_adapter(Box::new(GraphQLAdapter::new()));
        assert!(router.can_handle_graphql("test"));
    }

    #[tokio::test]
    async fn test_can_handle_grpc() {
        let mut router = Router::new();
        assert!(!router.can_handle_grpc("test"));

        router.add_adapter(Box::new(GrpcAdapter::new()));
        assert!(router.can_handle_grpc("test"));
    }

    // ===== Integration Tests: Multi-Protocol Routing =====

    #[tokio::test]
    async fn test_integration_single_handler_rest() {
        // Test: Single handler exposed via REST
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        // Route REST request
        let response = router.route_request("rest", "GET /users/42").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_single_handler_graphql() {
        // Test: Single handler exposed via GraphQL
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        // Route GraphQL request
        let response = router.route_request("graphql", "query { user }").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_single_handler_grpc() {
        // Test: Single handler exposed via gRPC
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        // Route gRPC request
        let response = router
            .route_request("grpc", "UserService.GetUser:{\"id\":42}")
            .await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_single_handler_all_protocols() {
        // Test: Single handler exposed via ALL protocols
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        // Test REST
        let rest_response = router.route_request("rest", "GET /users/42").await;
        assert!(rest_response.is_ok());
        assert!(rest_response.unwrap().contains("get_user"));

        // Test GraphQL
        let graphql_response = router.route_request("graphql", "query { user }").await;
        assert!(graphql_response.is_ok());
        assert!(graphql_response.unwrap().contains("get_user"));

        // Test gRPC
        let grpc_response = router
            .route_request("grpc", "UserService.GetUser:{\"id\":42}")
            .await;
        assert!(grpc_response.is_ok());
        assert!(grpc_response.unwrap().contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_multiple_handlers_all_protocols() {
        // Test: Multiple handlers, each exposed via all protocols
        let mut router = Router::new();
        router.register("get_user", || async { "User data".to_string() });
        router.register("list_users", || async { "Users list".to_string() });
        router.register("create_user", || async { "Created user".to_string() });

        // Configure REST adapter
        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        rest.route("GET", "/users", "list_users");
        rest.route("POST", "/users", "create_user");
        router.add_adapter(Box::new(rest));

        // Configure GraphQL adapter
        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        graphql.query("users", "list_users");
        graphql.mutation("createUser", "create_user");
        router.add_adapter(Box::new(graphql));

        // Configure gRPC adapter
        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        grpc.unary("UserService", "ListUsers", "list_users");
        grpc.unary("UserService", "CreateUser", "create_user");
        router.add_adapter(Box::new(grpc));

        // Test each handler via each protocol
        assert!(router
            .route_request("rest", "GET /users/42")
            .await
            .unwrap()
            .contains("get_user"));
        assert!(router
            .route_request("graphql", "query { user }")
            .await
            .unwrap()
            .contains("get_user"));
        assert!(router
            .route_request("grpc", "UserService.GetUser:{}")
            .await
            .unwrap()
            .contains("get_user"));
    }

    #[tokio::test]
    async fn test_integration_error_handling_rest_404() {
        // Test: REST 404 error
        let mut router = Router::new();

        let mut rest = RestAdapter::new();
        rest.route("GET", "/users/:id", "get_user");
        router.add_adapter(Box::new(rest));

        let response = router.route_request("rest", "GET /posts/42").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("HTTP 404"));
    }

    #[tokio::test]
    async fn test_integration_error_handling_graphql_not_found() {
        // Test: GraphQL operation not found
        let mut router = Router::new();

        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        router.add_adapter(Box::new(graphql));

        let response = router.route_request("graphql", "query { post }").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("errors"));
    }

    #[tokio::test]
    async fn test_integration_error_handling_grpc_unimplemented() {
        // Test: gRPC method not implemented
        let mut router = Router::new();

        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        router.add_adapter(Box::new(grpc));

        let response = router.route_request("grpc", "UserService.GetPost:{}").await;
        assert!(response.is_ok());
        assert!(response.unwrap().contains("grpc-status: 12")); // UNIMPLEMENTED
    }

    #[tokio::test]
    async fn test_integration_unknown_protocol() {
        // Test: Unknown protocol error
        let router = Router::new();

        let response = router.route_request("unknown", "request").await;
        assert!(response.is_err());
        assert!(response.unwrap_err().contains("Adapter not found"));
    }

    #[tokio::test]
    async fn test_integration_protocol_specific_features_rest_methods() {
        // Test: REST-specific HTTP methods
        let mut router = Router::new();
        router.register("get_users", || async { "Users".to_string() });
        router.register("create_user", || async { "Created".to_string() });
        router.register("update_user", || async { "Updated".to_string() });
        router.register("delete_user", || async { "Deleted".to_string() });

        let mut rest = RestAdapter::new();
        rest.route("GET", "/users", "get_users");
        rest.route("POST", "/users", "create_user");
        rest.route("PUT", "/users/:id", "update_user");
        rest.route("DELETE", "/users/:id", "delete_user");
        router.add_adapter(Box::new(rest));

        // Test different HTTP methods
        assert!(router
            .route_request("rest", "GET /users")
            .await
            .unwrap()
            .contains("get_users"));
        assert!(router
            .route_request("rest", "POST /users")
            .await
            .unwrap()
            .contains("create_user"));
        assert!(router
            .route_request("rest", "PUT /users/42")
            .await
            .unwrap()
            .contains("update_user"));
        assert!(router
            .route_request("rest", "DELETE /users/42")
            .await
            .unwrap()
            .contains("delete_user"));
    }

    #[tokio::test]
    async fn test_integration_protocol_specific_features_graphql_types() {
        // Test: GraphQL-specific query vs mutation
        let mut router = Router::new();
        router.register("get_user", || async { "User".to_string() });
        router.register("create_user", || async { "Created".to_string() });

        let mut graphql = GraphQLAdapter::new();
        graphql.query("user", "get_user");
        graphql.mutation("createUser", "create_user");
        router.add_adapter(Box::new(graphql));

        // Test query
        assert!(router
            .route_request("graphql", "query { user }")
            .await
            .unwrap()
            .contains("get_user"));

        // Test mutation
        assert!(router
            .route_request("graphql", "mutation { createUser }")
            .await
            .unwrap()
            .contains("create_user"));
    }

    #[tokio::test]
    async fn test_integration_protocol_specific_features_grpc_streaming() {
        // Test: gRPC-specific streaming modes
        let mut router = Router::new();
        router.register("get_user", || async { "User".to_string() });
        router.register("list_users", || async { "Users".to_string() });

        let mut grpc = GrpcAdapter::new();
        grpc.unary("UserService", "GetUser", "get_user");
        grpc.server_streaming("UserService", "ListUsers", "list_users");
        router.add_adapter(Box::new(grpc));

        // Test unary
        let unary_response = router
            .route_request("grpc", "UserService.GetUser:{}")
            .await
            .unwrap();
        assert!(unary_response.contains("unary"));

        // Test server streaming
        let streaming_response = router
            .route_request("grpc", "UserService.ListUsers:{}")
            .await
            .unwrap();
        assert!(streaming_response.contains("server_streaming"));
    }

    // ===== Streaming handler registration tests =====

    #[tokio::test]
    async fn test_register_streaming_handler() {
        let mut router = Router::new();
        router.register_streaming("stream_data", |tx: StreamSender| async move {
            tx.send("item".to_string()).await.ok();
            "done".to_string()
        });
        assert!(router.is_streaming("stream_data"));
        assert!(!router.is_streaming("nonexistent"));
    }

    #[tokio::test]
    async fn test_register_streaming_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            count: usize,
        }

        let mut router = Router::new();
        router.register_streaming_with_args("stream_items", |args: Input, tx: StreamSender| async move {
            for i in 0..args.count {
                tx.send(format!("item-{i}")).await.ok();
            }
            "done".to_string()
        });
        assert!(router.is_streaming("stream_items"));
    }

    #[tokio::test]
    async fn test_streaming_handler_not_in_regular_handlers() {
        let mut router = Router::new();
        router.register_streaming("stream", |_tx: StreamSender| async move {
            "done".to_string()
        });
        // Streaming handlers are NOT in the regular handlers map
        assert_eq!(router.handlers_count(), 0);
    }

    #[tokio::test]
    async fn test_list_handlers_includes_streaming() {
        let mut router = Router::new();
        router.register("regular", || async { "ok".to_string() });
        router.register_streaming("stream", |_tx: StreamSender| async move {
            "ok".to_string()
        });

        let handlers = router.list_handlers();
        assert_eq!(handlers.len(), 2);
        assert!(handlers.contains(&"regular".to_string()));
        assert!(handlers.contains(&"stream".to_string()));
    }

    #[tokio::test]
    async fn test_call_streaming_handler() {
        let mut router = Router::new();
        router.register_streaming("stream", |tx: StreamSender| async move {
            tx.send("a".to_string()).await.ok();
            tx.send("b".to_string()).await.ok();
            "final".to_string()
        });

        let (mut rx, fut) = router.call_streaming_handler("stream", "{}").unwrap();
        let result = fut.await;

        assert_eq!(result, Ok("final".to_string()));
        assert_eq!(rx.recv().await, Some("a".to_string()));
        assert_eq!(rx.recv().await, Some("b".to_string()));
    }

    #[tokio::test]
    async fn test_call_streaming_handler_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            n: usize,
        }

        let mut router = Router::new();
        router.register_streaming_with_args("count", |args: Input, tx: StreamSender| async move {
            for i in 0..args.n {
                tx.send(format!("{i}")).await.ok();
            }
            format!("counted to {}", args.n)
        });

        let (mut rx, fut) = router.call_streaming_handler("count", r#"{"n":3}"#).unwrap();
        let result = fut.await;

        assert_eq!(result, Ok("counted to 3".to_string()));
        assert_eq!(rx.recv().await, Some("0".to_string()));
        assert_eq!(rx.recv().await, Some("1".to_string()));
        assert_eq!(rx.recv().await, Some("2".to_string()));
    }

    #[tokio::test]
    async fn test_call_streaming_handler_not_found() {
        let router = Router::new();
        let result = router.call_streaming_handler("missing", "{}");
        assert!(result.is_err());
        match result {
            Err(e) => assert!(e.contains("not found")),
            Ok(_) => panic!("expected error"),
        }
    }

    #[tokio::test]
    async fn test_is_streaming_false_for_regular() {
        let mut router = Router::new();
        router.register("regular", || async { "ok".to_string() });
        assert!(!router.is_streaming("regular"));
    }

    #[tokio::test]
    async fn test_mixed_router() {
        let mut router = Router::new();
        router.register("get_user", || async { "user".to_string() });
        router.register_streaming("stream_updates", |tx: StreamSender| async move {
            tx.send("update".to_string()).await.ok();
            "done".to_string()
        });

        // Regular handler works
        let result = router.execute("get_user").await;
        assert_eq!(result, Ok("user".to_string()));

        // Streaming handler works
        let (mut rx, fut) = router.call_streaming_handler("stream_updates", "{}").unwrap();
        let result = fut.await;
        assert_eq!(result, Ok("done".to_string()));
        assert_eq!(rx.recv().await, Some("update".to_string()));

        // Regular handler not found in streaming
        assert!(!router.is_streaming("get_user"));
        assert!(router.call_streaming_handler("get_user", "{}").is_err());
    }

    #[tokio::test]
    async fn test_register_streaming_with_state() {
        struct AppState {
            prefix: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        let mut router = Router::new().with_state(AppState {
            prefix: "Hello".to_string(),
        });
        router.register_streaming_with_state::<AppState, Input, _, _, _>(
            "greet_stream",
            |state: State<Arc<AppState>>, args: Input, tx: StreamSender| async move {
                tx.send(format!("{} {}", state.prefix, args.name))
                    .await
                    .ok();
                "done".to_string()
            },
        );

        let (mut rx, fut) = router
            .call_streaming_handler("greet_stream", r#"{"name":"Alice"}"#)
            .unwrap();
        let result = fut.await;

        assert_eq!(result, Ok("done".to_string()));
        assert_eq!(rx.recv().await, Some("Hello Alice".to_string()));
    }

    #[tokio::test]
    async fn test_register_streaming_with_state_only() {
        struct AppState {
            items: Vec<String>,
        }

        let mut router = Router::new().with_state(AppState {
            items: vec!["x".to_string(), "y".to_string()],
        });
        router.register_streaming_with_state_only::<AppState, _, _, _>(
            "list_stream",
            |state: State<Arc<AppState>>, tx: StreamSender| async move {
                for item in &state.items {
                    tx.send(item.clone()).await.ok();
                }
                format!("listed {}", state.items.len())
            },
        );

        let (mut rx, fut) = router
            .call_streaming_handler("list_stream", "{}")
            .unwrap();
        let result = fut.await;

        assert_eq!(result, Ok("listed 2".to_string()));
        assert_eq!(rx.recv().await, Some("x".to_string()));
        assert_eq!(rx.recv().await, Some("y".to_string()));
    }

    // ===== Stream return adapter tests =====

    #[tokio::test]
    async fn test_register_stream_no_args() {
        let mut router = Router::new();
        router.register_stream("items", || async {
            tokio_stream::iter(vec!["a".to_string(), "b".to_string(), "c".to_string()])
        });

        assert!(router.is_streaming("items"));

        let (mut rx, fut) = router.call_streaming_handler("items", "{}").unwrap();
        let _result = fut.await;

        assert_eq!(rx.recv().await, Some("a".to_string()));
        assert_eq!(rx.recv().await, Some("b".to_string()));
        assert_eq!(rx.recv().await, Some("c".to_string()));
    }

    #[tokio::test]
    async fn test_register_stream_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            count: usize,
        }

        let mut router = Router::new();
        router.register_stream_with_args("counting", |args: Input| async move {
            tokio_stream::iter((0..args.count).map(|i| format!("{i}")))
        });

        assert!(router.is_streaming("counting"));

        let (mut rx, fut) = router
            .call_streaming_handler("counting", r#"{"count":3}"#)
            .unwrap();
        let _result = fut.await;

        assert_eq!(rx.recv().await, Some("0".to_string()));
        assert_eq!(rx.recv().await, Some("1".to_string()));
        assert_eq!(rx.recv().await, Some("2".to_string()));
    }

    #[tokio::test]
    async fn test_register_stream_with_state() {
        struct AppState {
            items: Vec<String>,
        }

        let mut router = Router::new().with_state(AppState {
            items: vec!["x".to_string(), "y".to_string()],
        });
        router.register_stream_with_state::<AppState, serde_json::Value, _, _, _, _>(
            "state_stream",
            |state: State<Arc<AppState>>, _args: serde_json::Value| {
                let items = state.items.clone();
                async move { tokio_stream::iter(items) }
            },
        );

        assert!(router.is_streaming("state_stream"));
    }

    #[tokio::test]
    async fn test_stream_adapter_shows_in_is_streaming() {
        let mut router = Router::new();
        router.register_stream("my_stream", || async {
            tokio_stream::iter(vec!["done".to_string()])
        });

        assert!(router.is_streaming("my_stream"));
        assert!(!router.is_streaming("nonexistent"));
    }

    #[tokio::test]
    async fn test_multiple_state_types() {
        struct DbPool {
            url: String,
        }
        struct AppConfig {
            name: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            key: String,
        }

        let mut router = Router::new()
            .with_state(DbPool {
                url: "postgres://localhost".to_string(),
            })
            .with_state(AppConfig {
                name: "MyApp".to_string(),
            });

        // Handler using DbPool
        router.register_with_state::<DbPool, Input, _, _>(
            "db_query",
            |state: State<Arc<DbPool>>, args: Input| async move {
                format!("{}:{}", state.url, args.key)
            },
        );

        // Handler using AppConfig
        router.register_with_state_only::<AppConfig, _, _>(
            "app_name",
            |state: State<Arc<AppConfig>>| async move { state.name.clone() },
        );

        let result = router.call_handler("db_query", r#"{"key":"users"}"#).await;
        assert_eq!(result, Ok("postgres://localhost:users".to_string()));

        let result = router.call_handler("app_name", "{}").await;
        assert_eq!(result, Ok("MyApp".to_string()));
    }

    #[tokio::test]
    async fn test_inject_state_after_construction() {
        struct LateState {
            value: String,
        }

        let mut router = Router::new();
        router.inject_state(LateState {
            value: "injected".to_string(),
        });
        router.register_with_state_only::<LateState, _, _>(
            "get_value",
            |state: State<Arc<LateState>>| async move { state.value.clone() },
        );

        let result = router.call_handler("get_value", "{}").await;
        assert_eq!(result, Ok("injected".to_string()));
    }

    #[tokio::test]
    async fn test_multiple_state_streaming() {
        struct StreamConfig {
            prefix: String,
        }

        let mut router = Router::new().with_state(StreamConfig {
            prefix: "stream".to_string(),
        });

        router.register_streaming_with_state_only::<StreamConfig, _, _, _>(
            "prefixed_stream",
            |state: State<Arc<StreamConfig>>, tx: StreamSender| async move {
                tx.send(format!("{}:item", state.prefix)).await.ok();
                "done".to_string()
            },
        );

        let (mut rx, fut) = router
            .call_streaming_handler("prefixed_stream", "{}")
            .unwrap();
        let result = fut.await;
        assert_eq!(result, Ok("done".to_string()));
        assert_eq!(rx.recv().await, Some("stream:item".to_string()));
    }

    #[tokio::test]
    async fn test_with_state_duplicate_type_last_wins() {
        // Calling with_state twice with the same type replaces the previous value.
        let mut router = Router::new()
            .with_state("first".to_string())
            .with_state("second".to_string());

        router.register_with_state_only::<String, _, _>(
            "get",
            |state: State<Arc<String>>| async move { (**state).clone() },
        );

        let result = router.call_handler("get", "{}").await;
        assert_eq!(result, Ok("second".to_string()));
    }
}
