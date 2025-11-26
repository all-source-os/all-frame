//! tests/06_cqrs_integration.rs
//!
//! GREEN PHASE: Tests passing with implementation
//!
//! Integration tests for CQRS + Event Sourcing.
//! These tests verify the full CQRS flow works end-to-end.

use allframe_core::cqrs::{
    command, command_handler, query, query_handler,
    Event, EventStore, Projection, Aggregate, Saga, Snapshot,
};
use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq)]
enum UserEvent {
    Created { user_id: String, email: String },
    EmailUpdated { new_email: String },
    Deleted,
    Incremented { amount: i32 },
}

impl Event for UserEvent {}

#[derive(Clone, Debug)]
struct User {
    id: String,
    email: String,
}

/// Test full CQRS flow: Command → Event → Projection → Query
#[tokio::test]
async fn test_full_cqrs_flow() {
    // Command side
    #[command]
    struct CreateUserCommand {
        user_id: String,
        email: String,
    }

    #[command_handler]
    async fn handle_create_user(cmd: CreateUserCommand, store: &EventStore<UserEvent>) -> Result<(), String> {
        let event = UserEvent::Created {
            user_id: cmd.user_id.clone(),
            email: cmd.email.clone(),
        };

        store.append(&cmd.user_id, vec![event]).await?;
        Ok(())
    }

    // Projection
    struct UserProjection {
        users: HashMap<String, User>,
    }

    impl Projection for UserProjection {
        type Event = UserEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                UserEvent::Created { user_id, email } => {
                    self.users.insert(user_id.clone(), User {
                        id: user_id.clone(),
                        email: email.clone(),
                    });
                }
                _ => {}
            }
        }
    }

    // Query side
    #[query]
    struct GetUserQuery {
        user_id: String,
    }

    #[query_handler]
    async fn handle_get_user(query: GetUserQuery, projection: &UserProjection) -> Option<User> {
        projection.users.get(&query.user_id).cloned()
    }

    // Execute full flow
    let event_store = EventStore::new();
    let mut projection = UserProjection { users: HashMap::new() };

    // 1. Execute command
    handle_create_user(CreateUserCommand {
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
    }, &event_store).await.unwrap();

    // 2. Get events from store
    let events = event_store.get_events("123").await.unwrap();

    // 3. Update projection
    for event in events {
        projection.apply(&event);
    }

    // 4. Execute query
    let user = handle_get_user(GetUserQuery {
        user_id: "123".to_string(),
    }, &projection).await;

    assert!(user.is_some());
    assert_eq!(user.unwrap().email, "user@example.com");
}

/// Test CQRS with Clean Architecture layers
#[tokio::test]
async fn test_cqrs_with_clean_architecture() {
    use allframe_core::arch::{domain, use_case};

    // Domain layer
    #[domain]
    #[derive(Clone)]
    struct ArchUser {
        id: String,
        email: String,
    }

    // Events are domain layer
    #[domain]
    #[derive(Clone)]
    enum ArchUserEvent {
        Created { user_id: String, email: String },
    }

    impl Event for ArchUserEvent {}

    // Commands are use case layer
    #[use_case]
    #[command]
    struct CreateArchUserCommand {
        email: String,
    }

    // Command handlers are use case layer
    #[command_handler]
    async fn handle_create_arch_user(cmd: CreateArchUserCommand) -> Vec<ArchUserEvent> {
        vec![ArchUserEvent::Created {
            user_id: "123".to_string(),
            email: cmd.email,
        }]
    }

    // Queries are use case layer
    #[use_case]
    #[query]
    struct GetArchUserQuery {
        user_id: String,
    }

    let _query = GetArchUserQuery {
        user_id: "123".to_string(),
    };

    let cmd = CreateArchUserCommand {
        email: "user@example.com".to_string(),
    };

    let events = handle_create_arch_user(cmd).await;
    assert_eq!(events.len(), 1);
}

/// Test event sourcing aggregate - rebuild from events
#[tokio::test]
async fn test_event_sourcing_aggregate() {
    #[derive(Default)]
    struct UserAggregate {
        id: String,
        email: String,
        is_deleted: bool,
        version: u64,
    }

    impl Aggregate for UserAggregate {
        type Event = UserEvent;

        fn apply_event(&mut self, event: &Self::Event) {
            self.version += 1;
            match event {
                UserEvent::Created { user_id, email } => {
                    self.id = user_id.clone();
                    self.email = email.clone();
                }
                UserEvent::EmailUpdated { new_email } => {
                    self.email = new_email.clone();
                }
                UserEvent::Deleted => {
                    self.is_deleted = true;
                }
                _ => {}
            }
        }
    }

    let events = vec![
        UserEvent::Created {
            user_id: "123".to_string(),
            email: "old@example.com".to_string(),
        },
        UserEvent::EmailUpdated {
            new_email: "new@example.com".to_string(),
        },
    ];

    let mut aggregate = UserAggregate::default();
    for event in events {
        aggregate.apply_event(&event);
    }

    assert_eq!(aggregate.email, "new@example.com");
    assert_eq!(aggregate.version, 2);
}

/// Test snapshot optimization for performance
#[tokio::test]
async fn test_snapshot_optimization() {
    #[derive(Default, Clone)]
    struct CounterAggregate {
        count: i32,
        version: u64,
    }

    impl Aggregate for CounterAggregate {
        type Event = UserEvent;

        fn apply_event(&mut self, event: &Self::Event) {
            self.version += 1;
            match event {
                UserEvent::Incremented { amount } => {
                    self.count += amount;
                }
                _ => {}
            }
        }
    }

    let event_store = EventStore::new();

    // Store many events (expensive to replay)
    let mut events = Vec::new();
    for _ in 0..1000 {
        events.push(UserEvent::Incremented { amount: 1 });
    }
    event_store.append("counter-123", events).await.unwrap();

    // Create snapshot at version 1000
    let mut aggregate = CounterAggregate::default();
    let all_events = event_store.get_events("counter-123").await.unwrap();
    for event in all_events {
        aggregate.apply_event(&event);
    }
    let snapshot = Snapshot::create(aggregate.clone(), 1000);

    // Add more events after snapshot
    event_store.append("counter-123", vec![
        UserEvent::Incremented { amount: 1 },
    ]).await.unwrap();

    // Rebuild: Load snapshot + replay only new events (much faster)
    let mut rebuilt_aggregate = snapshot.into_aggregate();
    let new_events = event_store.get_events_after("counter-123", 1000).await.unwrap();

    for event in new_events {
        rebuilt_aggregate.apply_event(&event);
    }

    assert_eq!(rebuilt_aggregate.version, 1001);
    assert_eq!(rebuilt_aggregate.count, 1001);
}

/// Test saga coordination for multi-aggregate transactions
#[tokio::test]
async fn test_saga_coordination() {
    use allframe_core::cqrs::SagaStep;

    // Saga: Transfer money between two accounts
    struct TransferMoneySaga {
        from_account: String,
        to_account: String,
        amount: f64,
        steps_executed: std::sync::Arc<tokio::sync::Mutex<Vec<String>>>,
    }

    #[async_trait::async_trait]
    impl Saga for TransferMoneySaga {
        async fn execute(&self) -> Result<(), String> {
            // Step 1: Debit from account
            self.execute_step(SagaStep::DebitAccount {
                account_id: self.from_account.clone(),
                amount: self.amount,
            }).await?;

            let mut steps = self.steps_executed.lock().await;
            steps.push(format!("Debited {} from {}", self.amount, self.from_account));
            drop(steps);

            // Step 2: Credit to account
            self.execute_step(SagaStep::CreditAccount {
                account_id: self.to_account.clone(),
                amount: self.amount,
            }).await?;

            let mut steps = self.steps_executed.lock().await;
            steps.push(format!("Credited {} to {}", self.amount, self.to_account));

            Ok(())
        }

        async fn compensate(&self, failed_step: usize) -> Result<(), String> {
            // If step 2 fails, compensate step 1
            if failed_step == 1 {
                self.execute_step(SagaStep::CreditAccount {
                    account_id: self.from_account.clone(),
                    amount: self.amount,
                }).await?;
            }
            Ok(())
        }
    }

    let steps_executed = std::sync::Arc::new(tokio::sync::Mutex::new(Vec::new()));
    let saga = TransferMoneySaga {
        from_account: "account-1".to_string(),
        to_account: "account-2".to_string(),
        amount: 100.0,
        steps_executed: steps_executed.clone(),
    };

    saga.execute().await.unwrap();

    let steps = steps_executed.lock().await;
    assert_eq!(steps.len(), 2);
    assert!(steps[0].contains("Debited"));
    assert!(steps[1].contains("Credited"));
}
