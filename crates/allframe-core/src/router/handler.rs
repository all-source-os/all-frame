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
}
