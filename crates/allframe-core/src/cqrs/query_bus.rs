//! Query Bus for CQRS query dispatch and routing
//!
//! The QueryBus provides automatic query routing and error handling,
//! mirroring the CommandBus pattern for the read side.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::sync::RwLock;

/// Query trait marker
pub trait Query: Send + Sync + 'static {}

/// Query execution result
pub type QueryResult<R> = Result<R, QueryError>;

/// Query execution errors
#[derive(Debug, Clone)]
pub enum QueryError {
    /// Query handler not found
    NotFound(String),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for QueryError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            QueryError::NotFound(msg) => write!(f, "Handler not found: {}", msg),
            QueryError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for QueryError {}

/// Query handler trait
#[async_trait]
pub trait QueryHandler<Q: Query, R: Send + Sync + 'static>: Send + Sync {
    /// Execute the query
    async fn handle(&self, query: Q) -> QueryResult<R>;
}

/// Type-erased query handler wrapper
#[async_trait]
trait ErasedQueryHandler: Send + Sync {
    async fn handle_erased(&self, query: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>, QueryError>;
}

/// Wrapper to type-erase query handlers
struct QueryHandlerWrapper<Q: Query, R: Send + Sync + 'static, H: QueryHandler<Q, R>> {
    handler: Arc<H>,
    _phantom: std::marker::PhantomData<(Q, R)>,
}

#[async_trait]
impl<Q: Query, R: Send + Sync + 'static, H: QueryHandler<Q, R>> ErasedQueryHandler
    for QueryHandlerWrapper<Q, R, H>
{
    async fn handle_erased(&self, query: Box<dyn Any + Send>) -> Result<Box<dyn Any + Send>, QueryError> {
        match query.downcast::<Q>() {
            Ok(q) => {
                let result = self.handler.handle(*q).await?;
                Ok(Box::new(result))
            }
            Err(_) => Err(QueryError::Internal(
                "Type mismatch in query dispatch".to_string(),
            )),
        }
    }
}

/// Type alias for handler storage
type HandlerMap = HashMap<TypeId, Arc<dyn ErasedQueryHandler>>;

/// Query Bus for dispatching queries to handlers
pub struct QueryBus {
    handlers: Arc<RwLock<HandlerMap>>,
}

impl QueryBus {
    /// Create a new query bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a query handler
    pub async fn register<Q: Query, R: Send + Sync + 'static, H: QueryHandler<Q, R> + 'static>(
        &self,
        handler: H,
    ) {
        let type_id = TypeId::of::<Q>();
        let wrapper = QueryHandlerWrapper {
            handler: Arc::new(handler),
            _phantom: std::marker::PhantomData,
        };
        let mut handlers = self.handlers.write().await;
        handlers.insert(type_id, Arc::new(wrapper));
    }

    /// Dispatch a query
    pub async fn dispatch<Q: Query, R: Send + Sync + 'static>(&self, query: Q) -> QueryResult<R> {
        let type_id = TypeId::of::<Q>();
        let handlers = self.handlers.read().await;

        match handlers.get(&type_id) {
            Some(handler) => {
                let boxed_query: Box<dyn Any + Send> = Box::new(query);
                let result = handler.handle_erased(boxed_query).await?;
                match result.downcast::<R>() {
                    Ok(r) => Ok(*r),
                    Err(_) => Err(QueryError::Internal(
                        "Type mismatch in query result".to_string(),
                    )),
                }
            }
            None => Err(QueryError::NotFound(format!(
                "No handler registered for query type: {}",
                std::any::type_name::<Q>()
            ))),
        }
    }

    /// Get number of registered handlers
    pub async fn handlers_count(&self) -> usize {
        self.handlers.read().await.len()
    }
}

impl Default for QueryBus {
    fn default() -> Self {
        Self::new()
    }
}

impl Clone for QueryBus {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    struct GetUserQuery {
        id: String,
    }

    impl Query for GetUserQuery {}

    #[derive(Debug, PartialEq)]
    struct UserResult {
        id: String,
        name: String,
    }

    struct GetUserHandler;

    #[async_trait]
    impl QueryHandler<GetUserQuery, UserResult> for GetUserHandler {
        async fn handle(&self, query: GetUserQuery) -> QueryResult<UserResult> {
            Ok(UserResult {
                id: query.id,
                name: "Test User".to_string(),
            })
        }
    }

    #[tokio::test]
    async fn test_query_dispatch() {
        let bus = QueryBus::new();
        bus.register(GetUserHandler).await;

        let result = bus
            .dispatch::<GetUserQuery, UserResult>(GetUserQuery {
                id: "123".to_string(),
            })
            .await;

        assert!(result.is_ok());
        let user = result.unwrap();
        assert_eq!(user.id, "123");
        assert_eq!(user.name, "Test User");
    }

    #[tokio::test]
    async fn test_query_handler_not_found() {
        let bus = QueryBus::new();

        let result = bus
            .dispatch::<GetUserQuery, UserResult>(GetUserQuery {
                id: "123".to_string(),
            })
            .await;

        assert!(matches!(result, Err(QueryError::NotFound(_))));
    }
}
