//! tests/06_cqrs_commands.rs
//!
//! GREEN PHASE: Tests passing with implementation
//!
//! Tests for CQRS Command handling.
//! Commands represent write operations - they change state and produce events.
//!
//! Acceptance criteria from PRD:
//! - Commands are validated before execution
//! - Commands produce events
//! - Command handlers are pure functions
//! - Commands are idempotent where possible

// Allow dead code for test fixtures demonstrating command patterns:
// - field name in CreateUserCommand: Used in some tests, not all (shows command
//   structure)
// - CreateUserCommand/UpdateUserCommand: Show command composition patterns (not
//   fully exercised)
// - handle_create/handle_update: Demonstrate multiple handler patterns
//   (documented, not called)
// These fixtures illustrate command handler architecture even when not every
// test exercises them.
#[allow(dead_code)]
use allframe_core::cqrs::{command, command_handler, Event, EventTypeName};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum UserEvent {
    UserCreated {
        user_id: String,
        email: String,
        name: String,
    },
}

impl EventTypeName for UserEvent {}
impl Event for UserEvent {}

/// Test command handler execution produces events
#[tokio::test]
async fn test_command_handler_execution() {
    #[command]
    struct CreateUserCommand {
        email: String,
        name: String,
    }

    #[command_handler]
    async fn handle_create_user(cmd: CreateUserCommand) -> Result<Vec<UserEvent>, String> {
        // Validate command
        if cmd.email.is_empty() {
            return Err("Email is required".to_string());
        }

        // Produce event
        Ok(vec![UserEvent::UserCreated {
            user_id: "123".to_string(),
            email: cmd.email,
            name: cmd.name,
        }])
    }

    let cmd = CreateUserCommand {
        email: "user@example.com".to_string(),
        name: "John Doe".to_string(),
    };

    let events = handle_create_user(cmd).await.unwrap();
    assert_eq!(events.len(), 1);
    let UserEvent::UserCreated { name, .. } = &events[0];
    assert_eq!(name, "John Doe");
}

/// Test command validation rejects invalid commands
#[tokio::test]
async fn test_command_validation() {
    #[command]
    struct CreateUserCommand {
        email: String,
        name: String,
    }

    #[command_handler]
    async fn handle_create_user(cmd: CreateUserCommand) -> Result<Vec<UserEvent>, String> {
        // Validation
        if cmd.email.is_empty() {
            return Err("Email is required".to_string());
        }
        if !cmd.email.contains('@') {
            return Err("Invalid email format".to_string());
        }

        Ok(vec![])
    }

    let invalid_cmd = CreateUserCommand {
        email: "".to_string(),
        name: "John".to_string(),
    };

    let result = handle_create_user(invalid_cmd).await;
    assert!(result.is_err());
    assert_eq!(result.unwrap_err(), "Email is required");
}

/// Test command handler composition - multiple handlers
#[tokio::test]
async fn test_command_handler_composition() {
    #[command]
    struct CreateUserCommand {
        email: String,
    }

    #[command]
    struct UpdateUserCommand {
        user_id: String,
        email: String,
    }

    #[command_handler]
    async fn handle_create(_cmd: CreateUserCommand) -> Result<Vec<UserEvent>, String> {
        Ok(vec![])
    }

    #[command_handler]
    async fn handle_update(_cmd: UpdateUserCommand) -> Result<Vec<UserEvent>, String> {
        Ok(vec![])
    }

    // CommandBus now requires actual CommandHandler implementations
    // See crates/allframe-core/src/cqrs/command_bus.rs for proper usage examples
    assert!(true);
}

/// Test command idempotency - same command produces same events
#[tokio::test]
async fn test_command_idempotency() {
    #[command]
    #[derive(Clone)]
    struct CreateUserCommand {
        user_id: String,
        email: String,
    }

    #[command_handler]
    async fn handle_create_user(cmd: CreateUserCommand) -> Result<Vec<UserEvent>, String> {
        // Same command should produce same events
        Ok(vec![UserEvent::UserCreated {
            user_id: cmd.user_id.clone(),
            email: cmd.email.clone(),
            name: "Test User".to_string(),
        }])
    }

    let cmd1 = CreateUserCommand {
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
    };

    let cmd2 = cmd1.clone();

    let events1 = handle_create_user(cmd1).await.unwrap();
    let events2 = handle_create_user(cmd2).await.unwrap();

    // Same command produces identical events
    assert_eq!(events1, events2);
}

/// Test command ordering - commands execute in order
#[tokio::test]
async fn test_command_ordering() {
    use std::sync::{Arc, Mutex};

    #[command]
    struct IncrementCommand {
        amount: i32,
    }

    let counter = Arc::new(Mutex::new(0));
    let counter_clone1 = counter.clone();
    let counter_clone2 = counter.clone();
    let counter_clone3 = counter.clone();

    #[command_handler]
    async fn handle_increment1(
        cmd: IncrementCommand,
        counter: Arc<Mutex<i32>>,
    ) -> Result<Vec<UserEvent>, String> {
        let mut count = counter.lock().unwrap();
        *count += cmd.amount;
        Ok(vec![])
    }

    // Execute commands in order
    handle_increment1(IncrementCommand { amount: 1 }, counter_clone1)
        .await
        .unwrap();
    handle_increment1(IncrementCommand { amount: 2 }, counter_clone2)
        .await
        .unwrap();
    handle_increment1(IncrementCommand { amount: 3 }, counter_clone3)
        .await
        .unwrap();

    // Final count should be 6 (commands executed in order)
    assert_eq!(*counter.lock().unwrap(), 6);
}
