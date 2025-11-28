//! tests/06_cqrs_queries.rs
//!
//! GREEN PHASE: Tests passing with implementation
//!
//! Tests for CQRS Query handling.
//! Queries represent read operations - they don't change state.
//!
//! Acceptance criteria from PRD:
//! - Queries are read-only
//! - Queries read from projections (read models)
//! - Projections are updated from events
//! - Multiple projections can exist for same events

use allframe_core::cqrs::{query, query_handler, Event, EventStore, Projection};
use std::collections::HashMap;

#[derive(Clone, Debug)]
enum UserEvent {
    Created { user_id: String, email: String, country: String },
    Updated { user_id: String, email: String },
}

impl Event for UserEvent {}

#[derive(Clone, Debug)]
struct User {
    id: String,
    email: String,
}

/// Test query handler execution returns data
#[tokio::test]
async fn test_query_handler_execution() {
    #[query]
    struct GetUserQuery {
        user_id: String,
    }

    #[query_handler]
    async fn handle_get_user(query: GetUserQuery) -> Result<Option<User>, String> {
        // Read from projection
        Ok(Some(User {
            id: query.user_id,
            email: "user@example.com".to_string(),
        }))
    }

    let query = GetUserQuery {
        user_id: "123".to_string(),
    };

    let user = handle_get_user(query).await.unwrap();
    assert!(user.is_some());
    assert_eq!(user.as_ref().unwrap().id, "123");
}

/// Test projection update from events
#[tokio::test]
async fn test_query_projection_update() {
    struct UserProjection {
        users: HashMap<String, User>,
    }

    impl Projection for UserProjection {
        type Event = UserEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                UserEvent::Created { user_id, email, .. } => {
                    self.users.insert(user_id.clone(), User {
                        id: user_id.clone(),
                        email: email.clone(),
                    });
                }
                UserEvent::Updated { user_id, email } => {
                    if let Some(user) = self.users.get_mut(user_id) {
                        user.email = email.clone();
                    }
                }
            }
        }
    }

    let mut projection = UserProjection {
        users: HashMap::new(),
    };

    // Apply events
    projection.apply(&UserEvent::Created {
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
        country: "US".to_string(),
    });

    assert_eq!(projection.users.len(), 1);
}

/// Test eventual consistency - read after write
#[tokio::test]
async fn test_query_eventual_consistency() {
    struct UserProjection {
        users: HashMap<String, User>,
    }

    impl UserProjection {
        fn new() -> Self {
            Self { users: HashMap::new() }
        }

        fn get_user(&self, id: &str) -> Option<&User> {
            self.users.get(id)
        }
    }

    impl Projection for UserProjection {
        type Event = UserEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                UserEvent::Created { user_id, email, .. } => {
                    self.users.insert(user_id.clone(), User {
                        id: user_id.clone(),
                        email: email.clone(),
                    });
                }
                UserEvent::Updated { user_id, email } => {
                    if let Some(user) = self.users.get_mut(user_id) {
                        user.email = email.clone();
                    }
                }
            }
        }
    }

    // Write side: Events stored
    let event_store = EventStore::new();
    event_store.append("user-123", vec![
        UserEvent::Created {
            user_id: "123".to_string(),
            email: "user@example.com".to_string(),
            country: "US".to_string(),
        },
    ]).await.unwrap();

    let events = event_store.get_events("user-123").await.unwrap();
    assert_eq!(events.len(), 1);

    // Read side: Projection updated from events
    let mut projection = UserProjection::new();
    for event in events {
        projection.apply(&event);
    }

    // Query reads from projection
    let user = projection.get_user("123");
    assert!(user.is_some());
}

/// Test multiple projections from same events
#[tokio::test]
async fn test_multiple_projections() {
    // Projection 1: All users by ID
    struct UserByIdProjection {
        users: HashMap<String, User>,
    }

    impl Projection for UserByIdProjection {
        type Event = UserEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                UserEvent::Created { user_id, email, .. } => {
                    self.users.insert(user_id.clone(), User {
                        id: user_id.clone(),
                        email: email.clone(),
                    });
                }
                UserEvent::Updated { user_id, email } => {
                    if let Some(user) = self.users.get_mut(user_id) {
                        user.email = email.clone();
                    }
                }
            }
        }
    }

    // Projection 2: Users grouped by country
    struct UsersByCountryProjection {
        countries: HashMap<String, Vec<User>>,
    }

    impl Projection for UsersByCountryProjection {
        type Event = UserEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                UserEvent::Created { user_id, email, country } => {
                    let users = self.countries.entry(country.clone()).or_insert_with(Vec::new);
                    users.push(User {
                        id: user_id.clone(),
                        email: email.clone(),
                    });
                }
                UserEvent::Updated { .. } => {
                    // For simplicity, country updates not handled
                }
            }
        }
    }

    let event = UserEvent::Created {
        user_id: "123".to_string(),
        email: "user@example.com".to_string(),
        country: "US".to_string(),
    };

    let mut projection1 = UserByIdProjection { users: HashMap::new() };
    let mut projection2 = UsersByCountryProjection { countries: HashMap::new() };

    // Same event updates both projections
    projection1.apply(&event);
    projection2.apply(&event);

    assert_eq!(projection1.users.len(), 1);
    assert_eq!(projection2.countries.get("US").unwrap().len(), 1);
}

/// Test projection rebuild from event history
#[tokio::test]
async fn test_projection_rebuild() {
    struct UserProjection {
        users: HashMap<String, User>,
    }

    impl UserProjection {
        fn new() -> Self {
            Self { users: HashMap::new() }
        }

        fn get_user(&self, id: &str) -> Option<&User> {
            self.users.get(id)
        }
    }

    impl Projection for UserProjection {
        type Event = UserEvent;

        fn apply(&mut self, event: &Self::Event) {
            match event {
                UserEvent::Created { user_id, email, .. } => {
                    self.users.insert(user_id.clone(), User {
                        id: user_id.clone(),
                        email: email.clone(),
                    });
                }
                UserEvent::Updated { user_id, email } => {
                    if let Some(user) = self.users.get_mut(user_id) {
                        user.email = email.clone();
                    }
                }
            }
        }
    }

    let event_store = EventStore::new();

    // Store many events over time
    event_store.append("user-123", vec![
        UserEvent::Created {
            user_id: "123".to_string(),
            email: "v1@example.com".to_string(),
            country: "US".to_string(),
        },
    ]).await.unwrap();

    event_store.append("user-123", vec![
        UserEvent::Updated {
            user_id: "123".to_string(),
            email: "v2@example.com".to_string(),
        },
    ]).await.unwrap();

    event_store.append("user-123", vec![
        UserEvent::Updated {
            user_id: "123".to_string(),
            email: "v3@example.com".to_string(),
        },
    ]).await.unwrap();

    // Rebuild projection from scratch by replaying all events
    let all_events = event_store.get_all_events().await.unwrap();
    let mut projection = UserProjection::new();

    for event in all_events {
        projection.apply(&event);
    }

    // Final state reflects all events
    let user = projection.get_user("123").unwrap();
    assert_eq!(user.email, "v3@example.com");
}
