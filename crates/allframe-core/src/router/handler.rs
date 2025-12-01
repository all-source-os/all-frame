//! Handler trait and implementations for protocol-agnostic request handling

use std::{future::Future, pin::Pin};

/// Handler trait for protocol-agnostic request handling
///
/// Handlers implement this trait to provide a unified interface
/// that can be called from any protocol adapter.
pub trait Handler: Send + Sync {
    /// Call the handler and return a result
    fn call(&self) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>>;
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
    fn call(&self) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        let fut = (self.func)();
        Box::pin(async move { Ok(fut.await) })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_handler_fn() {
        let handler = HandlerFn::new(|| async { "test".to_string() });
        let result = handler.call().await;
        assert_eq!(result, Ok("test".to_string()));
    }
}
