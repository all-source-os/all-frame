//! Handler trait and implementations for protocol-agnostic request handling

use serde::de::DeserializeOwned;
use serde::Serialize;
use std::{
    any::Any,
    fmt,
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
};
use tokio::sync::mpsc;
use tokio_util::sync::CancellationToken;

// ─── Output conversion trait ────────────────────────────────────────────────

/// Trait for converting handler return values into `Result<String, String>`.
///
/// This is the handler equivalent of axum's `IntoResponse`. By implementing
/// this trait for different return types, a single set of handler structs
/// can support `String` passthrough, `Json<T>` auto-serialization, and
/// `Result<T, E>` error handling.
pub trait IntoHandlerResult: Send {
    /// Convert this value into the handler's wire result.
    fn into_handler_result(self) -> Result<String, String>;
}

/// `String` passes through verbatim (backwards-compatible with existing handlers).
impl IntoHandlerResult for String {
    fn into_handler_result(self) -> Result<String, String> {
        Ok(self)
    }
}

/// Wrapper that auto-serializes `T: Serialize` to JSON.
///
/// Used internally by `register_typed*` methods — users return `T` directly,
/// the registration method wraps it in `Json`.
pub struct Json<T>(pub T);

impl<T: Serialize + Send> IntoHandlerResult for Json<T> {
    fn into_handler_result(self) -> Result<String, String> {
        serde_json::to_string(&self.0)
            .map_err(|e| format!("Failed to serialize response: {e}"))
    }
}

/// `Result<T, E>` serializes `Ok(T)` to JSON and stringifies `Err(E)`.
impl<T: Serialize + Send, E: fmt::Display + Send> IntoHandlerResult for Result<T, E> {
    fn into_handler_result(self) -> Result<String, String> {
        match self {
            Ok(value) => serde_json::to_string(&value)
                .map_err(|e| format!("Failed to serialize response: {e}")),
            Err(e) => Err(e.to_string()),
        }
    }
}

// ─── Stream item conversion trait ───────────────────────────────────────────

/// Trait for converting stream items into JSON strings.
///
/// Parallel to `IntoHandlerResult` but for individual stream messages.
pub trait IntoStreamItem: Send {
    /// Convert this value into a JSON string for streaming.
    fn into_stream_item(self) -> Result<String, String>;
}

/// `String` passes through verbatim.
impl IntoStreamItem for String {
    fn into_stream_item(self) -> Result<String, String> {
        Ok(self)
    }
}

/// `Json<T>` auto-serializes to JSON.
impl<T: Serialize + Send> IntoStreamItem for Json<T> {
    fn into_stream_item(self) -> Result<String, String> {
        serde_json::to_string(&self.0)
            .map_err(|e| format!("Failed to serialize stream item: {e}"))
    }
}

/// `Result<T, E>` serializes `Ok(T)` to JSON and stringifies `Err(E)`.
impl<T: Serialize + Send, E: fmt::Display + Send> IntoStreamItem for Result<T, E> {
    fn into_stream_item(self) -> Result<String, String> {
        match self {
            Ok(value) => serde_json::to_string(&value)
                .map_err(|e| format!("Failed to serialize stream item: {e}")),
            Err(e) => Err(e.to_string()),
        }
    }
}

// ─── Stream error type ─────────────────────────────────────────────────────

/// Errors that can occur when sending stream items.
#[derive(Debug, Clone, PartialEq)]
pub enum StreamError {
    /// The receiver was dropped (stream cancelled or consumer disconnected).
    Closed,
    /// Failed to serialize the stream item.
    Serialize(String),
}

impl fmt::Display for StreamError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            StreamError::Closed => write!(f, "stream closed: receiver dropped"),
            StreamError::Serialize(e) => write!(f, "stream serialization error: {e}"),
        }
    }
}

impl std::error::Error for StreamError {}

// ─── StreamSender ───────────────────────────────────────────────────────────

/// Default bounded channel capacity for streaming handlers.
pub const DEFAULT_STREAM_CAPACITY: usize = 64;

/// Sender half for streaming handlers.
///
/// Wraps a bounded `tokio::sync::mpsc::Sender<String>` and provides
/// ergonomic methods for sending typed items and checking cancellation.
///
/// The associated `CancellationToken` is **automatically cancelled** when
/// the `StreamReceiver` is dropped, enabling explicit cancellation checks
/// via `tokio::select!` in addition to `is_closed()`.
///
/// # Example
///
/// ```rust,ignore
/// async fn my_streaming_handler(args: MyArgs, tx: StreamSender) -> String {
///     let token = tx.cancellation_token();
///     loop {
///         tokio::select! {
///             _ = token.cancelled() => break,
///             item = next_item() => { tx.send(item).await.ok(); }
///         }
///     }
///     r#"{"done": true}"#.to_string()
/// }
/// ```
///
/// **Note on `Clone`:** Cloning a `StreamSender` shares the same underlying
/// channel and `CancellationToken`. Calling `cancel()` on any clone cancels
/// all of them.
#[derive(Clone)]
pub struct StreamSender {
    tx: mpsc::Sender<String>,
    cancel: CancellationToken,
}

/// Receiver half for streaming handlers.
///
/// Wraps `mpsc::Receiver<String>` and holds a `CancellationToken` guard.
/// When this receiver is dropped, the `CancellationToken` is automatically
/// cancelled, signalling the handler that the consumer has disconnected.
pub struct StreamReceiver {
    rx: mpsc::Receiver<String>,
    cancel: CancellationToken,
}

impl StreamReceiver {
    /// Receive the next stream item, or `None` if the stream is complete.
    pub async fn recv(&mut self) -> Option<String> {
        self.rx.recv().await
    }

}

impl Drop for StreamReceiver {
    fn drop(&mut self) {
        self.cancel.cancel();
    }
}

impl fmt::Debug for StreamReceiver {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamReceiver")
            .field("cancelled", &self.cancel.is_cancelled())
            .finish()
    }
}

impl StreamSender {
    /// Create a new stream channel with the default capacity (64).
    ///
    /// Returns `(sender, receiver)` pair. The `CancellationToken` is
    /// automatically cancelled when the `StreamReceiver` is dropped.
    pub fn channel() -> (Self, StreamReceiver) {
        Self::with_capacity(DEFAULT_STREAM_CAPACITY)
    }

    /// Create a new stream channel with a custom capacity.
    ///
    /// Returns `(sender, receiver)` pair.
    pub fn with_capacity(capacity: usize) -> (Self, StreamReceiver) {
        let (tx, rx) = mpsc::channel(capacity);
        let cancel = CancellationToken::new();
        (
            Self { tx, cancel: cancel.clone() },
            StreamReceiver { rx, cancel },
        )
    }

    /// Get the cancellation token for this stream.
    ///
    /// The token is automatically cancelled when the `StreamReceiver` is
    /// dropped, and can also be cancelled explicitly via `cancel()`.
    /// Use in `tokio::select!` for cooperative cancellation:
    /// ```rust,ignore
    /// let token = tx.cancellation_token();
    /// tokio::select! {
    ///     _ = token.cancelled() => { /* stream cancelled */ }
    ///     result = do_work() => { tx.send(result).await?; }
    /// }
    /// ```
    pub fn cancellation_token(&self) -> CancellationToken {
        self.cancel.clone()
    }

    /// Explicitly cancel the stream.
    ///
    /// This cancels the `CancellationToken`, signalling handlers
    /// that are using `token.cancelled()` in `select!`.
    pub fn cancel(&self) {
        self.cancel.cancel();
    }

    /// Send a stream item.
    ///
    /// The item is converted to a JSON string via `IntoStreamItem`.
    /// Returns `StreamError::Closed` if the receiver has been dropped,
    /// or `StreamError::Serialize` if serialization fails.
    pub async fn send(&self, item: impl IntoStreamItem) -> Result<(), StreamError> {
        let serialized = item.into_stream_item().map_err(StreamError::Serialize)?;
        self.tx
            .send(serialized)
            .await
            .map_err(|_| StreamError::Closed)
    }

    /// Check if the receiver has been dropped (stream cancelled).
    ///
    /// Useful for cooperative cancellation in loops:
    /// ```rust,ignore
    /// while !tx.is_closed() {
    ///     tx.send(next_item()).await?;
    /// }
    /// ```
    pub fn is_closed(&self) -> bool {
        self.tx.is_closed()
    }
}

impl fmt::Debug for StreamSender {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("StreamSender")
            .field("closed", &self.is_closed())
            .field("cancelled", &self.cancel.is_cancelled())
            .finish()
    }
}

// ─── Core handler trait ─────────────────────────────────────────────────────

/// Handler trait for protocol-agnostic request handling
///
/// Handlers implement this trait to provide a unified interface
/// that can be called from any protocol adapter.
pub trait Handler: Send + Sync {
    /// Call the handler with JSON args and return a result
    fn call(&self, args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
}

// ─── State wrapper ──────────────────────────────────────────────────────────

/// Newtype wrapper for injected state
///
/// Handlers receive `State<Arc<S>>` to access shared application state.
#[derive(Debug, Clone)]
pub struct State<S>(pub S);

impl<S> Deref for State<S> {
    type Target = S;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

// ─── Handler structs (4 total, generic over R: IntoHandlerResult) ───────────

/// Wrapper for function-based handlers with no arguments
pub struct HandlerFn<F, Fut, R>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    _marker: std::marker::PhantomData<fn() -> R>,
}

impl<F, Fut, R> HandlerFn<F, Fut, R>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new handler from a function
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, Fut, R> Handler for HandlerFn<F, Fut, R>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call(&self, _args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let fut = (self.func)();
        Box::pin(async move { fut.await.into_handler_result() })
    }
}

/// Wrapper for handlers that accept typed, deserialized arguments
#[allow(clippy::type_complexity)]
pub struct HandlerWithArgs<F, T, Fut, R>
where
    F: Fn(T) -> Fut + Send + Sync,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    // fn() -> T is covariant and auto-implements Send + Sync regardless of T,
    // which is correct because T is only deserialized transiently, never stored.
    _marker: std::marker::PhantomData<(fn() -> T, fn() -> R)>,
}

impl<F, T, Fut, R> HandlerWithArgs<F, T, Fut, R>
where
    F: Fn(T) -> Fut + Send + Sync,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new handler that deserializes JSON args into `T`
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, T, Fut, R> Handler for HandlerWithArgs<F, T, Fut, R>
where
    F: Fn(T) -> Fut + Send + Sync + 'static,
    T: DeserializeOwned + Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call(&self, args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let parsed: Result<T, _> = serde_json::from_str(args);
        match parsed {
            Ok(value) => {
                let fut = (self.func)(value);
                Box::pin(async move { fut.await.into_handler_result() })
            }
            Err(e) => Box::pin(async move {
                Err(format!("Failed to deserialize args: {e}"))
            }),
        }
    }
}

/// Wrapper for handlers that receive injected state and typed args
#[allow(clippy::type_complexity)]
pub struct HandlerWithState<F, S, T, Fut, R>
where
    F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    state: Arc<dyn Any + Send + Sync>,
    _marker: std::marker::PhantomData<(fn() -> S, fn() -> T, fn() -> R)>,
}

impl<F, S, T, Fut, R> HandlerWithState<F, S, T, Fut, R>
where
    F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new handler with state injection and typed args
    pub fn new(func: F, state: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            func,
            state,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, S, T, Fut, R> Handler for HandlerWithState<F, S, T, Fut, R>
where
    F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync + 'static,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call(&self, args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let state_arc = match self.state.clone().downcast::<S>() {
            Ok(s) => s,
            Err(_) => {
                let msg = format!(
                    "State type mismatch: expected {}",
                    std::any::type_name::<S>()
                );
                return Box::pin(async move { Err(msg) });
            }
        };

        let parsed: Result<T, _> = serde_json::from_str(args);
        match parsed {
            Ok(value) => {
                let fut = (self.func)(State(state_arc), value);
                Box::pin(async move { fut.await.into_handler_result() })
            }
            Err(e) => Box::pin(async move {
                Err(format!("Failed to deserialize args: {e}"))
            }),
        }
    }
}

/// Wrapper for handlers that receive only injected state (no args)
#[allow(clippy::type_complexity)]
pub struct HandlerWithStateOnly<F, S, Fut, R>
where
    F: Fn(State<Arc<S>>) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    state: Arc<dyn Any + Send + Sync>,
    _marker: std::marker::PhantomData<(fn() -> S, fn() -> R)>,
}

impl<F, S, Fut, R> HandlerWithStateOnly<F, S, Fut, R>
where
    F: Fn(State<Arc<S>>) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new handler with state injection only
    pub fn new(func: F, state: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            func,
            state,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, S, Fut, R> Handler for HandlerWithStateOnly<F, S, Fut, R>
where
    F: Fn(State<Arc<S>>) -> Fut + Send + Sync + 'static,
    S: Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call(&self, _args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let state_arc = match self.state.clone().downcast::<S>() {
            Ok(s) => s,
            Err(_) => {
                let msg = format!(
                    "State type mismatch: expected {}",
                    std::any::type_name::<S>()
                );
                return Box::pin(async move { Err(msg) });
            }
        };

        let fut = (self.func)(State(state_arc));
        Box::pin(async move { fut.await.into_handler_result() })
    }
}

// ─── Streaming handler trait ────────────────────────────────────────────────

/// Trait for streaming handlers that send incremental updates during execution.
///
/// Parallel to `Handler` but receives a `StreamSender` for emitting intermediate
/// messages. The handler returns a final result after streaming completes.
pub trait StreamHandler: Send + Sync {
    /// Call the streaming handler with JSON args and a stream sender.
    ///
    /// The handler sends intermediate messages via `tx` and returns a final
    /// result when execution completes.
    fn call_streaming(
        &self,
        args: &str,
        tx: StreamSender,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
}

// ─── Streaming handler structs (4 variants) ─────────────────────────────────

/// Streaming handler with no arguments (receives only StreamSender)
pub struct StreamingHandlerFn<F, Fut, R>
where
    F: Fn(StreamSender) -> Fut + Send + Sync,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    _marker: std::marker::PhantomData<fn() -> R>,
}

impl<F, Fut, R> StreamingHandlerFn<F, Fut, R>
where
    F: Fn(StreamSender) -> Fut + Send + Sync,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new streaming handler from a function
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, Fut, R> StreamHandler for StreamingHandlerFn<F, Fut, R>
where
    F: Fn(StreamSender) -> Fut + Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call_streaming(
        &self,
        _args: &str,
        tx: StreamSender,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let fut = (self.func)(tx);
        Box::pin(async move { fut.await.into_handler_result() })
    }
}

/// Streaming handler that accepts typed, deserialized arguments
#[allow(clippy::type_complexity)]
pub struct StreamingHandlerWithArgs<F, T, Fut, R>
where
    F: Fn(T, StreamSender) -> Fut + Send + Sync,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    _marker: std::marker::PhantomData<(fn() -> T, fn() -> R)>,
}

impl<F, T, Fut, R> StreamingHandlerWithArgs<F, T, Fut, R>
where
    F: Fn(T, StreamSender) -> Fut + Send + Sync,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new streaming handler with typed args
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, T, Fut, R> StreamHandler for StreamingHandlerWithArgs<F, T, Fut, R>
where
    F: Fn(T, StreamSender) -> Fut + Send + Sync + 'static,
    T: DeserializeOwned + Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call_streaming(
        &self,
        args: &str,
        tx: StreamSender,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let parsed: Result<T, _> = serde_json::from_str(args);
        match parsed {
            Ok(value) => {
                let fut = (self.func)(value, tx);
                Box::pin(async move { fut.await.into_handler_result() })
            }
            Err(e) => Box::pin(async move {
                Err(format!("Failed to deserialize args: {e}"))
            }),
        }
    }
}

/// Streaming handler that receives injected state and typed args
#[allow(clippy::type_complexity)]
pub struct StreamingHandlerWithState<F, S, T, Fut, R>
where
    F: Fn(State<Arc<S>>, T, StreamSender) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    state: Arc<dyn Any + Send + Sync>,
    _marker: std::marker::PhantomData<(fn() -> S, fn() -> T, fn() -> R)>,
}

impl<F, S, T, Fut, R> StreamingHandlerWithState<F, S, T, Fut, R>
where
    F: Fn(State<Arc<S>>, T, StreamSender) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new streaming handler with state and typed args
    pub fn new(func: F, state: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            func,
            state,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, S, T, Fut, R> StreamHandler for StreamingHandlerWithState<F, S, T, Fut, R>
where
    F: Fn(State<Arc<S>>, T, StreamSender) -> Fut + Send + Sync + 'static,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call_streaming(
        &self,
        args: &str,
        tx: StreamSender,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let state_arc = match self.state.clone().downcast::<S>() {
            Ok(s) => s,
            Err(_) => {
                let msg = format!(
                    "State type mismatch: expected {}",
                    std::any::type_name::<S>()
                );
                return Box::pin(async move { Err(msg) });
            }
        };

        let parsed: Result<T, _> = serde_json::from_str(args);
        match parsed {
            Ok(value) => {
                let fut = (self.func)(State(state_arc), value, tx);
                Box::pin(async move { fut.await.into_handler_result() })
            }
            Err(e) => Box::pin(async move {
                Err(format!("Failed to deserialize args: {e}"))
            }),
        }
    }
}

/// Streaming handler that receives only injected state (no args)
#[allow(clippy::type_complexity)]
pub struct StreamingHandlerWithStateOnly<F, S, Fut, R>
where
    F: Fn(State<Arc<S>>, StreamSender) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    func: F,
    state: Arc<dyn Any + Send + Sync>,
    _marker: std::marker::PhantomData<(fn() -> S, fn() -> R)>,
}

impl<F, S, Fut, R> StreamingHandlerWithStateOnly<F, S, Fut, R>
where
    F: Fn(State<Arc<S>>, StreamSender) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    Fut: Future<Output = R> + Send,
    R: IntoHandlerResult,
{
    /// Create a new streaming handler with state only
    pub fn new(func: F, state: Arc<dyn Any + Send + Sync>) -> Self {
        Self {
            func,
            state,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, S, Fut, R> StreamHandler for StreamingHandlerWithStateOnly<F, S, Fut, R>
where
    F: Fn(State<Arc<S>>, StreamSender) -> Fut + Send + Sync + 'static,
    S: Send + Sync + 'static,
    Fut: Future<Output = R> + Send + 'static,
    R: IntoHandlerResult + 'static,
{
    fn call_streaming(
        &self,
        _args: &str,
        tx: StreamSender,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let state_arc = match self.state.clone().downcast::<S>() {
            Ok(s) => s,
            Err(_) => {
                let msg = format!(
                    "State type mismatch: expected {}",
                    std::any::type_name::<S>()
                );
                return Box::pin(async move { Err(msg) });
            }
        };

        let fut = (self.func)(State(state_arc), tx);
        Box::pin(async move { fut.await.into_handler_result() })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // ─── String return (backwards compat) ───────────────────────────────

    #[tokio::test]
    async fn test_handler_fn() {
        let handler = HandlerFn::new(|| async { "test".to_string() });
        let result = handler.call("{}").await;
        assert_eq!(result, Ok("test".to_string()));
    }

    #[tokio::test]
    async fn test_handler_fn_ignores_args() {
        let handler = HandlerFn::new(|| async { "no-args".to_string() });
        let result = handler.call(r#"{"unexpected": true}"#).await;
        assert_eq!(result, Ok("no-args".to_string()));
    }

    #[tokio::test]
    async fn test_handler_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        let handler = HandlerWithArgs::new(|args: Input| async move {
            format!("hello {}", args.name)
        });

        let result = handler.call(r#"{"name":"Alice"}"#).await;
        assert_eq!(result, Ok("hello Alice".to_string()));
    }

    #[tokio::test]
    async fn test_handler_with_args_bad_json() {
        #[derive(serde::Deserialize)]
        struct Input {
            _name: String,
        }

        let handler = HandlerWithArgs::new(|_args: Input| async move {
            "unreachable".to_string()
        });

        let result = handler.call("not-json").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to deserialize args"));
    }

    #[tokio::test]
    async fn test_handler_with_args_missing_field() {
        #[derive(serde::Deserialize)]
        struct Input {
            _name: String,
        }

        let handler = HandlerWithArgs::new(|_args: Input| async move {
            "unreachable".to_string()
        });

        let result = handler.call(r#"{"age": 30}"#).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to deserialize args"));
    }

    #[tokio::test]
    async fn test_handler_with_state() {
        struct AppState {
            greeting: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState {
            greeting: "Hi".to_string(),
        });

        let handler = HandlerWithState::new(
            |state: State<Arc<AppState>>, args: Input| async move {
                format!("{} {}", state.greeting, args.name)
            },
            state,
        );

        let result = handler.call(r#"{"name":"Bob"}"#).await;
        assert_eq!(result, Ok("Hi Bob".to_string()));
    }

    #[tokio::test]
    async fn test_handler_with_state_only() {
        struct AppState {
            value: i32,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState { value: 42 });

        let handler = HandlerWithStateOnly::new(
            |state: State<Arc<AppState>>| async move {
                format!("value={}", state.value)
            },
            state,
        );

        let result = handler.call("{}").await;
        assert_eq!(result, Ok("value=42".to_string()));
    }

    #[tokio::test]
    async fn test_handler_with_state_deser_error() {
        struct AppState;

        #[derive(serde::Deserialize)]
        struct Input {
            _x: i32,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState);

        let handler = HandlerWithState::new(
            |_state: State<Arc<AppState>>, _args: Input| async move {
                "unreachable".to_string()
            },
            state,
        );

        let result = handler.call("bad").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to deserialize args"));
    }

    // ─── Json<T> return (typed handlers via IntoHandlerResult) ──────────

    #[tokio::test]
    async fn test_json_handler_fn_struct() {
        #[derive(serde::Serialize)]
        struct User {
            id: u32,
            name: String,
        }

        let handler = HandlerFn::new(|| async {
            Json(User {
                id: 1,
                name: "Alice".to_string(),
            })
        });

        let result = handler.call("{}").await;
        assert_eq!(result, Ok(r#"{"id":1,"name":"Alice"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_json_handler_fn_vec() {
        let handler = HandlerFn::new(|| async { Json(vec![1, 2, 3]) });
        let result = handler.call("{}").await;
        assert_eq!(result, Ok("[1,2,3]".to_string()));
    }

    #[tokio::test]
    async fn test_json_handler_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        #[derive(serde::Serialize)]
        struct Output {
            greeting: String,
        }

        let handler = HandlerWithArgs::new(|args: Input| async move {
            Json(Output {
                greeting: format!("Hello {}", args.name),
            })
        });

        let result = handler.call(r#"{"name":"Bob"}"#).await;
        assert_eq!(result, Ok(r#"{"greeting":"Hello Bob"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_json_handler_with_args_bad_json() {
        #[derive(serde::Deserialize)]
        struct Input {
            _x: i32,
        }

        let handler = HandlerWithArgs::new(|_: Input| async move { Json(42) });
        let result = handler.call("bad").await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to deserialize args"));
    }

    #[tokio::test]
    async fn test_json_handler_with_state() {
        struct AppState {
            prefix: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        #[derive(serde::Serialize)]
        struct Output {
            message: String,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState {
            prefix: "Hi".to_string(),
        });

        let handler = HandlerWithState::new(
            |state: State<Arc<AppState>>, args: Input| async move {
                Json(Output {
                    message: format!("{} {}", state.prefix, args.name),
                })
            },
            state,
        );

        let result = handler.call(r#"{"name":"Charlie"}"#).await;
        assert_eq!(result, Ok(r#"{"message":"Hi Charlie"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_json_handler_with_state_only() {
        struct AppState {
            version: String,
        }

        #[derive(serde::Serialize)]
        struct Info {
            version: String,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState {
            version: "1.0".to_string(),
        });

        let handler = HandlerWithStateOnly::new(
            |state: State<Arc<AppState>>| async move {
                Json(Info {
                    version: state.version.clone(),
                })
            },
            state,
        );

        let result = handler.call("{}").await;
        assert_eq!(result, Ok(r#"{"version":"1.0"}"#.to_string()));
    }

    // ─── Result<T, E> return (via IntoHandlerResult) ────────────────────

    #[tokio::test]
    async fn test_result_handler_fn_ok() {
        #[derive(serde::Serialize)]
        struct Data {
            value: i32,
        }

        let handler = HandlerFn::new(|| async {
            Ok::<_, String>(Data { value: 42 })
        });

        let result = handler.call("{}").await;
        assert_eq!(result, Ok(r#"{"value":42}"#.to_string()));
    }

    #[tokio::test]
    async fn test_result_handler_fn_err() {
        #[derive(serde::Serialize)]
        struct Data {
            value: i32,
        }

        let handler = HandlerFn::new(|| async {
            Err::<Data, String>("something went wrong".to_string())
        });

        let result = handler.call("{}").await;
        assert_eq!(result, Err("something went wrong".to_string()));
    }

    #[tokio::test]
    async fn test_result_handler_with_args_ok() {
        #[derive(serde::Deserialize)]
        struct Input {
            x: i32,
        }

        #[derive(serde::Serialize)]
        struct Output {
            doubled: i32,
        }

        let handler = HandlerWithArgs::new(|args: Input| async move {
            Ok::<_, String>(Output { doubled: args.x * 2 })
        });

        let result = handler.call(r#"{"x":21}"#).await;
        assert_eq!(result, Ok(r#"{"doubled":42}"#.to_string()));
    }

    #[tokio::test]
    async fn test_result_handler_with_args_err() {
        #[derive(serde::Deserialize)]
        struct Input {
            x: i32,
        }

        let handler = HandlerWithArgs::new(|args: Input| async move {
            if args.x < 0 {
                Err::<i32, String>("negative".to_string())
            } else {
                Ok(args.x)
            }
        });

        let result = handler.call(r#"{"x":-1}"#).await;
        assert_eq!(result, Err("negative".to_string()));
    }

    #[tokio::test]
    async fn test_result_handler_with_state() {
        struct AppState {
            threshold: i32,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            value: i32,
        }

        #[derive(serde::Serialize)]
        struct Output {
            accepted: bool,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState { threshold: 10 });

        let handler = HandlerWithState::new(
            |state: State<Arc<AppState>>, args: Input| async move {
                if args.value >= state.threshold {
                    Ok::<_, String>(Output { accepted: true })
                } else {
                    Err("below threshold".to_string())
                }
            },
            state,
        );

        let ok_result = handler.call(r#"{"value":15}"#).await;
        assert_eq!(ok_result, Ok(r#"{"accepted":true}"#.to_string()));

        let err_result = handler.call(r#"{"value":5}"#).await;
        assert_eq!(err_result, Err("below threshold".to_string()));
    }

    #[tokio::test]
    async fn test_result_handler_with_state_only() {
        struct AppState {
            ready: bool,
        }

        #[derive(serde::Serialize)]
        struct Status {
            ok: bool,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState { ready: true });

        let handler = HandlerWithStateOnly::new(
            |state: State<Arc<AppState>>| async move {
                if state.ready {
                    Ok::<_, String>(Status { ok: true })
                } else {
                    Err("not ready".to_string())
                }
            },
            state,
        );

        let result = handler.call("{}").await;
        assert_eq!(result, Ok(r#"{"ok":true}"#.to_string()));
    }

    // ─── IntoStreamItem tests ───────────────────────────────────────────

    #[test]
    fn test_into_stream_item_string() {
        let item = "hello".to_string();
        assert_eq!(item.into_stream_item(), Ok("hello".to_string()));
    }

    #[test]
    fn test_into_stream_item_json() {
        #[derive(serde::Serialize)]
        struct Token {
            text: String,
        }
        let item = Json(Token {
            text: "hi".to_string(),
        });
        assert_eq!(
            item.into_stream_item(),
            Ok(r#"{"text":"hi"}"#.to_string())
        );
    }

    #[test]
    fn test_into_stream_item_json_vec() {
        let item = Json(vec![1, 2, 3]);
        assert_eq!(item.into_stream_item(), Ok("[1,2,3]".to_string()));
    }

    #[test]
    fn test_into_stream_item_result_ok() {
        #[derive(serde::Serialize)]
        struct Data {
            v: i32,
        }
        let item: Result<Data, String> = Ok(Data { v: 42 });
        assert_eq!(item.into_stream_item(), Ok(r#"{"v":42}"#.to_string()));
    }

    #[test]
    fn test_into_stream_item_result_err() {
        let item: Result<i32, String> = Err("bad".to_string());
        assert_eq!(item.into_stream_item(), Err("bad".to_string()));
    }

    // ─── StreamError tests ──────────────────────────────────────────────

    #[test]
    fn test_stream_error_display_closed() {
        let err = StreamError::Closed;
        assert_eq!(err.to_string(), "stream closed: receiver dropped");
    }

    #[test]
    fn test_stream_error_display_serialize() {
        let err = StreamError::Serialize("bad json".to_string());
        assert_eq!(err.to_string(), "stream serialization error: bad json");
    }

    #[test]
    fn test_stream_error_is_std_error() {
        let err: Box<dyn std::error::Error> = Box::new(StreamError::Closed);
        assert!(err.to_string().contains("closed"));
    }

    // ─── StreamSender tests ─────────────────────────────────────────────

    #[tokio::test]
    async fn test_stream_sender_send_and_receive() {
        let (tx, mut rx) = StreamSender::channel();
        tx.send("hello".to_string()).await.unwrap();
        tx.send("world".to_string()).await.unwrap();
        drop(tx);

        assert_eq!(rx.recv().await, Some("hello".to_string()));
        assert_eq!(rx.recv().await, Some("world".to_string()));
        assert_eq!(rx.recv().await, None);
    }

    #[tokio::test]
    async fn test_stream_sender_send_json() {
        #[derive(serde::Serialize)]
        struct Token {
            t: String,
        }
        let (tx, mut rx) = StreamSender::channel();
        tx.send(Json(Token {
            t: "hi".to_string(),
        }))
        .await
        .unwrap();
        drop(tx);

        assert_eq!(rx.recv().await, Some(r#"{"t":"hi"}"#.to_string()));
    }

    #[tokio::test]
    async fn test_stream_sender_closed_detection() {
        let (tx, rx) = StreamSender::channel();
        assert!(!tx.is_closed());
        drop(rx);
        assert!(tx.is_closed());
    }

    #[tokio::test]
    async fn test_stream_sender_send_after_close() {
        let (tx, rx) = StreamSender::channel();
        drop(rx);
        let result = tx.send("late".to_string()).await;
        assert_eq!(result, Err(StreamError::Closed));
    }

    #[tokio::test]
    async fn test_stream_sender_custom_capacity() {
        let (tx, mut rx) = StreamSender::with_capacity(2);

        // Fill the buffer
        tx.send("a".to_string()).await.unwrap();
        tx.send("b".to_string()).await.unwrap();

        // Drain and verify order
        assert_eq!(rx.recv().await, Some("a".to_string()));
        assert_eq!(rx.recv().await, Some("b".to_string()));

        // Can send more after draining
        tx.send("c".to_string()).await.unwrap();
        assert_eq!(rx.recv().await, Some("c".to_string()));
    }

    #[tokio::test]
    async fn test_stream_sender_default_capacity() {
        assert_eq!(DEFAULT_STREAM_CAPACITY, 64);
    }

    #[tokio::test]
    async fn test_stream_sender_clone() {
        let (tx, mut rx) = StreamSender::channel();
        let tx2 = tx.clone();

        tx.send("from-tx1".to_string()).await.unwrap();
        tx2.send("from-tx2".to_string()).await.unwrap();
        drop(tx);
        drop(tx2);

        assert_eq!(rx.recv().await, Some("from-tx1".to_string()));
        assert_eq!(rx.recv().await, Some("from-tx2".to_string()));
        assert_eq!(rx.recv().await, None);
    }

    #[test]
    fn test_stream_sender_debug() {
        let (tx, _rx) = StreamSender::channel();
        let debug = format!("{:?}", tx);
        assert!(debug.contains("StreamSender"));
    }

    // ─── CancellationToken tests ────────────────────────────────────────

    #[tokio::test]
    async fn test_cancellation_token_not_cancelled_initially() {
        let (tx, _rx) = StreamSender::channel();
        let token = tx.cancellation_token();
        assert!(!token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_token_cancelled_on_explicit_cancel() {
        let (tx, _rx) = StreamSender::channel();
        let token = tx.cancellation_token();
        assert!(!token.is_cancelled());
        tx.cancel();
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_token_cancelled_future_resolves() {
        let (tx, _rx) = StreamSender::channel();
        let token = tx.cancellation_token();

        // Cancel in a spawned task
        let tx2 = tx.clone();
        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            tx2.cancel();
        });

        // cancelled() future should resolve
        tokio::time::timeout(std::time::Duration::from_secs(1), token.cancelled())
            .await
            .expect("cancelled future should resolve");
    }

    #[tokio::test]
    async fn test_cancellation_token_shared_across_clones() {
        let (tx, _rx) = StreamSender::channel();
        let token1 = tx.cancellation_token();
        let token2 = tx.cancellation_token();
        let tx2 = tx.clone();
        let token3 = tx2.cancellation_token();

        tx.cancel();
        assert!(token1.is_cancelled());
        assert!(token2.is_cancelled());
        assert!(token3.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_token_auto_cancelled_on_receiver_drop() {
        let (tx, rx) = StreamSender::channel();
        let token = tx.cancellation_token();

        assert!(!token.is_cancelled());
        drop(rx); // Dropping StreamReceiver should auto-cancel the token
        assert!(token.is_cancelled());
    }

    #[tokio::test]
    async fn test_cancellation_token_auto_cancel_future_resolves_on_drop() {
        let (tx, rx) = StreamSender::channel();
        let token = tx.cancellation_token();

        tokio::spawn(async move {
            tokio::time::sleep(std::time::Duration::from_millis(10)).await;
            drop(rx);
        });

        tokio::time::timeout(std::time::Duration::from_secs(1), token.cancelled())
            .await
            .expect("cancelled future should resolve when receiver is dropped");
    }

    // ─── StreamHandler trait tests ──────────────────────────────────────

    #[tokio::test]
    async fn test_streaming_handler_fn() {
        let handler = StreamingHandlerFn::new(|tx: StreamSender| async move {
            tx.send("item1".to_string()).await.ok();
            tx.send("item2".to_string()).await.ok();
            "done".to_string()
        });

        let (tx, mut rx) = StreamSender::channel();
        let result = handler.call_streaming("{}", tx).await;

        assert_eq!(result, Ok("done".to_string()));
        assert_eq!(rx.recv().await, Some("item1".to_string()));
        assert_eq!(rx.recv().await, Some("item2".to_string()));
    }

    #[tokio::test]
    async fn test_streaming_handler_with_args() {
        #[derive(serde::Deserialize)]
        struct Input {
            count: usize,
        }

        let handler =
            StreamingHandlerWithArgs::new(|args: Input, tx: StreamSender| async move {
                for i in 0..args.count {
                    tx.send(format!("item-{i}")).await.ok();
                }
                format!("sent {}", args.count)
            });

        let (tx, mut rx) = StreamSender::channel();
        let result = handler.call_streaming(r#"{"count":3}"#, tx).await;

        assert_eq!(result, Ok("sent 3".to_string()));
        assert_eq!(rx.recv().await, Some("item-0".to_string()));
        assert_eq!(rx.recv().await, Some("item-1".to_string()));
        assert_eq!(rx.recv().await, Some("item-2".to_string()));
    }

    #[tokio::test]
    async fn test_streaming_handler_with_args_bad_json() {
        #[derive(serde::Deserialize)]
        struct Input {
            _x: i32,
        }

        let handler =
            StreamingHandlerWithArgs::new(|_args: Input, _tx: StreamSender| async move {
                "unreachable".to_string()
            });

        let (tx, _rx) = StreamSender::channel();
        let result = handler.call_streaming("bad-json", tx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Failed to deserialize args"));
    }

    #[tokio::test]
    async fn test_streaming_handler_with_state() {
        struct AppState {
            prefix: String,
        }

        #[derive(serde::Deserialize)]
        struct Input {
            name: String,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState {
            prefix: "Hi".to_string(),
        });

        let handler = StreamingHandlerWithState::new(
            |state: State<Arc<AppState>>, args: Input, tx: StreamSender| async move {
                tx.send(format!("{} {}", state.prefix, args.name))
                    .await
                    .ok();
                "done".to_string()
            },
            state,
        );

        let (tx, mut rx) = StreamSender::channel();
        let result = handler.call_streaming(r#"{"name":"Alice"}"#, tx).await;

        assert_eq!(result, Ok("done".to_string()));
        assert_eq!(rx.recv().await, Some("Hi Alice".to_string()));
    }

    #[tokio::test]
    async fn test_streaming_handler_with_state_only() {
        struct AppState {
            items: Vec<String>,
        }

        let state: Arc<dyn Any + Send + Sync> = Arc::new(AppState {
            items: vec!["a".to_string(), "b".to_string()],
        });

        let handler = StreamingHandlerWithStateOnly::new(
            |state: State<Arc<AppState>>, tx: StreamSender| async move {
                for item in &state.items {
                    tx.send(item.clone()).await.ok();
                }
                format!("sent {}", state.items.len())
            },
            state,
        );

        let (tx, mut rx) = StreamSender::channel();
        let result = handler.call_streaming("{}", tx).await;

        assert_eq!(result, Ok("sent 2".to_string()));
        assert_eq!(rx.recv().await, Some("a".to_string()));
        assert_eq!(rx.recv().await, Some("b".to_string()));
    }

    #[tokio::test]
    async fn test_streaming_handler_with_state_type_mismatch() {
        struct WrongState;
        struct AppState;

        let state: Arc<dyn Any + Send + Sync> = Arc::new(WrongState);

        let handler = StreamingHandlerWithStateOnly::new(
            |_state: State<Arc<AppState>>, _tx: StreamSender| async move {
                "unreachable".to_string()
            },
            state,
        );

        let (tx, _rx) = StreamSender::channel();
        let result = handler.call_streaming("{}", tx).await;
        assert!(result.is_err());
        assert!(result.unwrap_err().contains("State type mismatch"));
    }

    #[tokio::test]
    async fn test_streaming_handler_json_return() {
        #[derive(serde::Serialize)]
        struct Summary {
            count: usize,
        }

        let handler = StreamingHandlerFn::new(|tx: StreamSender| async move {
            tx.send("item".to_string()).await.ok();
            Json(Summary { count: 1 })
        });

        let (tx, mut rx) = StreamSender::channel();
        let result = handler.call_streaming("{}", tx).await;

        assert_eq!(result, Ok(r#"{"count":1}"#.to_string()));
        assert_eq!(rx.recv().await, Some("item".to_string()));
    }

    #[tokio::test]
    async fn test_streaming_handler_result_return() {
        let handler = StreamingHandlerFn::new(|tx: StreamSender| async move {
            tx.send("progress".to_string()).await.ok();
            Ok::<_, String>(42)
        });

        let (tx, mut rx) = StreamSender::channel();
        let result = handler.call_streaming("{}", tx).await;

        assert_eq!(result, Ok("42".to_string()));
        assert_eq!(rx.recv().await, Some("progress".to_string()));
    }
}
