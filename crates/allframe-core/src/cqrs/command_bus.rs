//! Command Bus for CQRS command dispatch and routing
//!
//! The CommandBus provides automatic command routing, validation, and error
//! handling.

use std::{
    any::{Any, TypeId},
    collections::HashMap,
    sync::Arc,
};

use async_trait::async_trait;
use tokio::sync::RwLock;

use super::Event;

/// Command trait marker
pub trait Command: Send + Sync + 'static {}

/// Validation error details
#[derive(Debug, Clone)]
pub struct ValidationError {
    /// Field that failed validation
    pub field: String,
    /// Error message
    pub message: String,
    /// Error code
    pub code: String,
}

impl ValidationError {
    /// Create a new validation error
    pub fn new(field: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: "validation_failed".to_string(),
        }
    }

    /// Create with custom error code
    pub fn with_code(
        field: impl Into<String>,
        message: impl Into<String>,
        code: impl Into<String>,
    ) -> Self {
        Self {
            field: field.into(),
            message: message.into(),
            code: code.into(),
        }
    }
}

/// Command execution result
pub type CommandResult<E> = Result<Vec<E>, CommandError>;

/// Command execution errors
#[derive(Debug, Clone)]
pub enum CommandError {
    /// Validation failed
    Validation(Vec<ValidationError>),
    /// Business logic error
    BusinessLogic(String),
    /// Handler not found
    NotFound(String),
    /// Idempotency violation (command already executed)
    AlreadyExecuted(String),
    /// Internal error
    Internal(String),
}

impl std::fmt::Display for CommandError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CommandError::Validation(errors) => {
                write!(f, "Validation failed: ")?;
                for (i, err) in errors.iter().enumerate() {
                    if i > 0 {
                        write!(f, ", ")?;
                    }
                    write!(f, "{}: {}", err.field, err.message)?;
                }
                Ok(())
            }
            CommandError::BusinessLogic(msg) => write!(f, "Business logic error: {}", msg),
            CommandError::NotFound(msg) => write!(f, "Handler not found: {}", msg),
            CommandError::AlreadyExecuted(msg) => write!(f, "Already executed: {}", msg),
            CommandError::Internal(msg) => write!(f, "Internal error: {}", msg),
        }
    }
}

impl std::error::Error for CommandError {}

/// Command handler trait
#[async_trait]
pub trait CommandHandler<C: Command, E: Event>: Send + Sync {
    /// Execute the command
    async fn handle(&self, command: C) -> CommandResult<E>;
}

/// Type-erased command handler wrapper
#[async_trait]
trait ErasedHandler<E: Event>: Send + Sync {
    async fn handle_erased(&self, command: Box<dyn Any + Send>) -> CommandResult<E>;
}

/// Wrapper to type-erase command handlers
struct HandlerWrapper<C: Command, E: Event, H: CommandHandler<C, E>> {
    handler: Arc<H>,
    _phantom: std::marker::PhantomData<(C, E)>,
}

#[async_trait]
impl<C: Command, E: Event, H: CommandHandler<C, E>> ErasedHandler<E> for HandlerWrapper<C, E, H> {
    async fn handle_erased(&self, command: Box<dyn Any + Send>) -> CommandResult<E> {
        match command.downcast::<C>() {
            Ok(cmd) => self.handler.handle(*cmd).await,
            Err(_) => Err(CommandError::Internal(
                "Type mismatch in command dispatch".to_string(),
            )),
        }
    }
}

/// Type alias for handler storage
type HandlerMap<E> = HashMap<TypeId, Arc<dyn ErasedHandler<E>>>;

/// Command Bus for dispatching commands to handlers
pub struct CommandBus<E: Event> {
    handlers: Arc<RwLock<HandlerMap<E>>>,
    idempotency_keys: Arc<RwLock<HashMap<String, Vec<E>>>>,
}

impl<E: Event> CommandBus<E> {
    /// Create a new command bus
    pub fn new() -> Self {
        Self {
            handlers: Arc::new(RwLock::new(HashMap::new())),
            idempotency_keys: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Register a command handler
    pub async fn register<C: Command, H: CommandHandler<C, E> + 'static>(&self, handler: H) {
        let type_id = TypeId::of::<C>();
        let wrapper = HandlerWrapper {
            handler: Arc::new(handler),
            _phantom: std::marker::PhantomData,
        };
        let mut handlers = self.handlers.write().await;
        handlers.insert(type_id, Arc::new(wrapper));
    }

    /// Dispatch a command
    pub async fn dispatch<C: Command>(&self, command: C) -> CommandResult<E> {
        let type_id = TypeId::of::<C>();
        let handlers = self.handlers.read().await;

        match handlers.get(&type_id) {
            Some(handler) => {
                let boxed_command: Box<dyn Any + Send> = Box::new(command);
                handler.handle_erased(boxed_command).await
            }
            None => Err(CommandError::NotFound(format!(
                "No handler registered for command type: {}",
                std::any::type_name::<C>()
            ))),
        }
    }

    /// Dispatch a command with idempotency key
    pub async fn dispatch_idempotent<C: Command>(
        &self,
        command: C,
        idempotency_key: String,
    ) -> CommandResult<E> {
        // Check if already executed
        {
            let keys = self.idempotency_keys.read().await;
            if let Some(events) = keys.get(&idempotency_key) {
                return Ok(events.clone());
            }
        }

        // Execute command
        let events = self.dispatch(command).await?;

        // Store result
        {
            let mut keys = self.idempotency_keys.write().await;
            keys.insert(idempotency_key, events.clone());
        }

        Ok(events)
    }

    /// Get number of registered handlers
    pub async fn handlers_count(&self) -> usize {
        self.handlers.read().await.len()
    }
}

impl<E: Event> Default for CommandBus<E> {
    fn default() -> Self {
        Self::new()
    }
}

impl<E: Event> Clone for CommandBus<E> {
    fn clone(&self) -> Self {
        Self {
            handlers: Arc::clone(&self.handlers),
            idempotency_keys: Arc::clone(&self.idempotency_keys),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[derive(Clone)]
    enum TestEvent {
        UserCreated { _id: String },
    }

    impl Event for TestEvent {}

    struct CreateUserCommand {
        email: String,
    }

    impl Command for CreateUserCommand {}

    struct CreateUserHandler;

    #[async_trait]
    impl CommandHandler<CreateUserCommand, TestEvent> for CreateUserHandler {
        async fn handle(&self, command: CreateUserCommand) -> CommandResult<TestEvent> {
            if command.email.is_empty() {
                return Err(CommandError::Validation(vec![ValidationError::new(
                    "email",
                    "Email is required",
                )]));
            }

            Ok(vec![TestEvent::UserCreated {
                _id: "123".to_string(),
            }])
        }
    }

    #[tokio::test]
    async fn test_command_dispatch() {
        let bus = CommandBus::new();
        bus.register(CreateUserHandler).await;

        let result = bus
            .dispatch(CreateUserCommand {
                email: "test@example.com".to_string(),
            })
            .await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_validation_error() {
        let bus = CommandBus::new();
        bus.register(CreateUserHandler).await;

        let result = bus
            .dispatch(CreateUserCommand {
                email: "".to_string(),
            })
            .await;

        assert!(matches!(result, Err(CommandError::Validation(_))));
    }

    #[tokio::test]
    async fn test_handler_not_found() {
        let bus: CommandBus<TestEvent> = CommandBus::new();

        let result = bus
            .dispatch(CreateUserCommand {
                email: "test@example.com".to_string(),
            })
            .await;

        assert!(matches!(result, Err(CommandError::NotFound(_))));
    }

    #[tokio::test]
    async fn test_idempotency() {
        let bus = CommandBus::new();
        bus.register(CreateUserHandler).await;

        let cmd = CreateUserCommand {
            email: "test@example.com".to_string(),
        };

        // First execution
        let result1 = bus
            .dispatch_idempotent(cmd, "key1".to_string())
            .await
            .unwrap();

        // Second execution with same key - should return cached result
        let cmd2 = CreateUserCommand {
            email: "different@example.com".to_string(),
        };
        let result2 = bus
            .dispatch_idempotent(cmd2, "key1".to_string())
            .await
            .unwrap();

        // Should be same events (idempotency)
        assert_eq!(result1.len(), result2.len());
    }
}
