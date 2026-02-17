//! tests/06_cqrs_events.rs
//!
//! GREEN PHASE: Tests passing with implementation
//!
//! Tests for CQRS Event handling.
//! Events represent things that have happened - immutable facts.
//!
//! Acceptance criteria from PRD:
//! - Events are immutable
//! - Events can be stored in event store
//! - Events can be replayed to rebuild state
//! - Events support versioning for schema evolution

// Allow dead code for test fixtures demonstrating event patterns:
// - variant Deleted: Shows deletion events in enum (not exercised in all tests)
// - field version in V1: Demonstrates versioning structure (used in conversion,
//   not directly read)
// - StreamUserEvent.user_id: Shows event streaming patterns (not fully
//   exercised)
// These fixtures document event sourcing patterns even when not every field is
// validated.
#[allow(dead_code)]
use allframe_core::cqrs::{Event, EventStore, EventTypeName};

#[derive(Clone, Debug, PartialEq, serde::Serialize, serde::Deserialize)]
enum UserEvent {
    Created { user_id: String, email: String },
    EmailUpdated { user_id: String, new_email: String },
    Deleted { user_id: String },
}

impl EventTypeName for UserEvent {}
impl Event for UserEvent {}

/// Test event store append - storing events
#[tokio::test]
async fn test_event_store_append() {
    let store = EventStore::new();

    // Append events
    store
        .append(
            "user-123",
            vec![UserEvent::Created {
                user_id: "123".to_string(),
                email: "user@example.com".to_string(),
            }],
        )
        .await
        .unwrap();

    // Retrieve events
    let events = store.get_events("user-123").await.unwrap();
    assert_eq!(events.len(), 1);
}

/// Test event store replay - rebuild state from events
#[tokio::test]
async fn test_event_store_replay() {
    #[derive(Default)]
    struct UserState {
        exists: bool,
        email: String,
    }

    impl UserState {
        fn apply(&mut self, event: &UserEvent) {
            match event {
                UserEvent::Created { email, .. } => {
                    self.exists = true;
                    self.email = email.clone();
                }
                UserEvent::EmailUpdated { new_email, .. } => {
                    self.email = new_email.clone();
                }
                UserEvent::Deleted { .. } => {
                    self.exists = false;
                }
            }
        }
    }

    let store = EventStore::new();

    // Store events
    store
        .append(
            "user-123",
            vec![
                UserEvent::Created {
                    user_id: "123".to_string(),
                    email: "old@example.com".to_string(),
                },
                UserEvent::EmailUpdated {
                    user_id: "123".to_string(),
                    new_email: "new@example.com".to_string(),
                },
            ],
        )
        .await
        .unwrap();

    // Replay events to rebuild state
    let events = store.get_events("user-123").await.unwrap();
    let mut state = UserState::default();
    for event in events {
        state.apply(&event);
    }

    assert!(state.exists);
    assert_eq!(state.email, "new@example.com");
}

/// Test event versioning for schema evolution
#[test]
fn test_event_versioning() {
    // Version 1 of event
    #[derive(Clone)]
    struct UserCreatedV1 {
        version: u32,
        user_id: String,
        email: String,
    }

    // Version 2 adds name field
    #[derive(Clone)]
    struct UserCreatedV2 {
        version: u32,
        user_id: String,
        email: String,
        name: String,
    }

    // Migration from V1 to V2
    impl From<UserCreatedV1> for UserCreatedV2 {
        fn from(v1: UserCreatedV1) -> Self {
            UserCreatedV2 {
                version: 2,
                user_id: v1.user_id,
                email: v1.email,
                name: "Unknown".to_string(), // Default for new field
            }
        }
    }

    let v1 = UserCreatedV1 {
        version: 1,
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
    };

    let v2: UserCreatedV2 = v1.into();
    assert_eq!(v2.version, 2);
    assert_eq!(v2.user_id, "123");
    assert_eq!(v2.email, "user@example.com");
    assert_eq!(v2.name, "Unknown");
}

/// Test event serialization - events persist correctly
#[tokio::test]
async fn test_event_serialization() {
    use serde::{Deserialize, Serialize};

    #[derive(Clone, Debug, Serialize, Deserialize, PartialEq)]
    enum SerializableUserEvent {
        Created { user_id: String, email: String },
    }

    impl EventTypeName for SerializableUserEvent {}
    impl Event for SerializableUserEvent {}

    let event = SerializableUserEvent::Created {
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
    };

    // Serialize
    let json = serde_json::to_string(&event).unwrap();

    // Deserialize
    let deserialized: SerializableUserEvent = serde_json::from_str(&json).unwrap();

    assert_eq!(event, deserialized);
}

/// Test event stream subscribe - real-time event streaming
#[tokio::test]
async fn test_event_stream_subscribe() {
    use tokio::sync::mpsc;

    #[derive(Clone, Debug, serde::Serialize, serde::Deserialize)]
    enum StreamUserEvent {
        Created { user_id: String },
    }

    impl EventTypeName for StreamUserEvent {}
    impl Event for StreamUserEvent {}

    let store = EventStore::new();

    // Subscribe to events
    let (tx, mut rx) = mpsc::channel(10);
    store.subscribe(tx).await;

    // Append event
    store
        .append(
            "user-123",
            vec![StreamUserEvent::Created {
                user_id: "123".to_string(),
            }],
        )
        .await
        .unwrap();

    // Subscriber receives event
    let received = rx.recv().await;
    assert!(received.is_some());
}
