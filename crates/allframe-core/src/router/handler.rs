//! Handler trait and implementations for protocol-agnostic request handling

use serde::de::DeserializeOwned;
use std::{
    any::Any,
    future::Future,
    ops::Deref,
    pin::Pin,
    sync::Arc,
};

/// Handler trait for protocol-agnostic request handling
///
/// Handlers implement this trait to provide a unified interface
/// that can be called from any protocol adapter.
pub trait Handler: Send + Sync {
    /// Call the handler with JSON args and return a result
    fn call(&self, args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
}

/// Wrapper for function-based handlers with no arguments
pub struct HandlerFn<F, Fut>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = String> + Send,
{
    func: F,
}

impl<F, Fut> HandlerFn<F, Fut>
where
    F: Fn() -> Fut + Send + Sync,
    Fut: Future<Output = String> + Send,
{
    /// Create a new handler from a function
    pub fn new(func: F) -> Self {
        Self { func }
    }
}

impl<F, Fut> Handler for HandlerFn<F, Fut>
where
    F: Fn() -> Fut + Send + Sync + 'static,
    Fut: Future<Output = String> + Send + 'static,
{
    fn call(&self, _args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let fut = (self.func)();
        Box::pin(async move { Ok(fut.await) })
    }
}

/// Wrapper for handlers that accept typed, deserialized arguments
pub struct HandlerWithArgs<F, T, Fut>
where
    F: Fn(T) -> Fut + Send + Sync,
    T: DeserializeOwned + Send,
    Fut: Future<Output = String> + Send,
{
    func: F,
    // fn() -> T is covariant and auto-implements Send + Sync regardless of T,
    // which is correct because T is only deserialized transiently, never stored.
    _marker: std::marker::PhantomData<fn() -> T>,
}

impl<F, T, Fut> HandlerWithArgs<F, T, Fut>
where
    F: Fn(T) -> Fut + Send + Sync,
    T: DeserializeOwned + Send,
    Fut: Future<Output = String> + Send,
{
    /// Create a new handler that deserializes JSON args into `T`
    pub fn new(func: F) -> Self {
        Self {
            func,
            _marker: std::marker::PhantomData,
        }
    }
}

impl<F, T, Fut> Handler for HandlerWithArgs<F, T, Fut>
where
    F: Fn(T) -> Fut + Send + Sync + 'static,
    T: DeserializeOwned + Send + 'static,
    Fut: Future<Output = String> + Send + 'static,
{
    fn call(&self, args: &str) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let parsed: Result<T, _> = serde_json::from_str(args);
        match parsed {
            Ok(value) => {
                let fut = (self.func)(value);
                Box::pin(async move { Ok(fut.await) })
            }
            Err(e) => Box::pin(async move {
                Err(format!("Failed to deserialize args: {e}"))
            }),
        }
    }
}

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

/// Wrapper for handlers that receive injected state and typed args
pub struct HandlerWithState<F, S, T, Fut>
where
    F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send,
    Fut: Future<Output = String> + Send,
{
    func: F,
    state: Arc<dyn Any + Send + Sync>,
    // fn() -> X is covariant and auto-implements Send + Sync (see HandlerWithArgs).
    _marker: std::marker::PhantomData<(fn() -> S, fn() -> T)>,
}

impl<F, S, T, Fut> HandlerWithState<F, S, T, Fut>
where
    F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send,
    Fut: Future<Output = String> + Send,
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

impl<F, S, T, Fut> Handler for HandlerWithState<F, S, T, Fut>
where
    F: Fn(State<Arc<S>>, T) -> Fut + Send + Sync + 'static,
    S: Send + Sync + 'static,
    T: DeserializeOwned + Send + 'static,
    Fut: Future<Output = String> + Send + 'static,
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
                Box::pin(async move { Ok(fut.await) })
            }
            Err(e) => Box::pin(async move {
                Err(format!("Failed to deserialize args: {e}"))
            }),
        }
    }
}

/// Wrapper for handlers that receive only injected state (no args)
pub struct HandlerWithStateOnly<F, S, Fut>
where
    F: Fn(State<Arc<S>>) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    Fut: Future<Output = String> + Send,
{
    func: F,
    state: Arc<dyn Any + Send + Sync>,
    _marker: std::marker::PhantomData<fn() -> S>,
}

impl<F, S, Fut> HandlerWithStateOnly<F, S, Fut>
where
    F: Fn(State<Arc<S>>) -> Fut + Send + Sync,
    S: Send + Sync + 'static,
    Fut: Future<Output = String> + Send,
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

impl<F, S, Fut> Handler for HandlerWithStateOnly<F, S, Fut>
where
    F: Fn(State<Arc<S>>) -> Fut + Send + Sync + 'static,
    S: Send + Sync + 'static,
    Fut: Future<Output = String> + Send + 'static,
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
        Box::pin(async move { Ok(fut.await) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
}
