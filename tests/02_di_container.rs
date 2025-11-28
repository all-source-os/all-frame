//! tests/02_di_container.rs
//!
//! RED PHASE: This test MUST fail initially
//!
//! Tests for the #[di_container] procedural macro as specified in PRD_01.md.
//! The macro provides compile-time dependency injection with:
//! - Zero runtime reflection
//! - Type-safe dependency resolution
//! - Compile-time detection of circular dependencies
//! - Support for nested service dependencies
//!
//! Acceptance criteria from PRD:
//! - Inject 50+ nested services, zero runtime reflection
//! - All dependencies resolved at compile time
//! - Circular dependency detection at compile time

use allframe_macros::di_container;

/// Test basic dependency injection with simple services
#[test]
fn test_di_basic_injection() {
    // Define a simple service
    struct DatabaseService {
        connection_string: String,
    }

    impl DatabaseService {
        fn new() -> Self {
            Self {
                connection_string: "postgresql://localhost/test".to_string(),
            }
        }

        fn query(&self, sql: &str) -> String {
            format!("Querying: {} | SQL: {}", self.connection_string, sql)
        }
    }

    // Define a service that depends on DatabaseService
    struct UserRepository {
        db: std::sync::Arc<DatabaseService>,
    }

    impl UserRepository {
        fn new(db: std::sync::Arc<DatabaseService>) -> Self {
            Self { db }
        }

        fn find_user(&self, id: i32) -> String {
            self.db
                .query(&format!("SELECT * FROM users WHERE id = {}", id))
        }
    }

    // Define a service that depends on UserRepository
    struct UserService {
        repo: std::sync::Arc<UserRepository>,
    }

    impl UserService {
        fn new(repo: std::sync::Arc<UserRepository>) -> Self {
            Self { repo }
        }

        fn get_user(&self, id: i32) -> String {
            self.repo.find_user(id)
        }
    }

    // Use the DI container macro to wire everything together
    #[di_container]
    struct AppContainer {
        database: DatabaseService,
        user_repository: UserRepository,
        user_service: UserService,
    }

    // The macro should generate:
    // 1. An implementation that creates instances with correct dependencies
    // 2. Methods to access each service
    let container = AppContainer::new();

    // Test that we can access services and they work correctly
    let result = container.user_service().get_user(1);
    assert!(result.contains("postgresql://localhost/test"));
    assert!(result.contains("SELECT * FROM users WHERE id = 1"));
}

/// Test dependency injection with trait-based dependencies
#[test]
fn test_di_trait_injection() {
    // Define a trait for the database
    trait Database: Send + Sync {
        fn query(&self, sql: &str) -> String;
    }

    // Implement the trait
    struct PostgresDatabase;

    impl Database for PostgresDatabase {
        fn query(&self, sql: &str) -> String {
            format!("Postgres: {}", sql)
        }
    }

    // Service that depends on the trait
    struct UserRepository {
        db: std::sync::Arc<Box<dyn Database>>,
    }

    impl UserRepository {
        fn new(db: std::sync::Arc<Box<dyn Database>>) -> Self {
            Self { db }
        }

        fn find_user(&self, id: i32) -> String {
            self.db
                .query(&format!("SELECT * FROM users WHERE id = {}", id))
        }
    }

    // Container with trait-based dependency
    #[di_container]
    struct AppContainer {
        #[provide(Box::new(PostgresDatabase) as Box<dyn Database>)]
        database: Box<dyn Database>,
        user_repository: UserRepository,
    }

    let container = AppContainer::new();
    let result = container.user_repository().find_user(1);
    assert!(result.contains("Postgres:"));
    assert!(result.contains("SELECT * FROM users WHERE id = 1"));
}

/// Test that the DI container is thread-safe
#[test]
fn test_di_thread_safety() {
    use std::sync::Arc;
    use std::thread;

    #[derive(Clone)]
    struct SharedService {
        value: Arc<String>,
    }

    impl SharedService {
        fn new() -> Self {
            Self {
                value: Arc::new("shared".to_string()),
            }
        }

        fn get_value(&self) -> String {
            (*self.value).clone()
        }
    }

    #[di_container]
    struct AppContainer {
        shared: SharedService,
    }

    let container = AppContainer::new();
    let service = container.shared();

    // Spawn multiple threads that use the service
    let handles: Vec<_> = (0..10)
        .map(|i| {
            let svc = service.clone();
            thread::spawn(move || {
                let value = svc.get_value();
                assert_eq!(value, "shared");
                i
            })
        })
        .collect();

    for handle in handles {
        handle.join().unwrap();
    }
}

/// Test nested dependencies (3 levels deep)
#[test]
fn test_di_nested_dependencies() {
    // Level 1: Database
    struct Database {
        name: String,
    }

    impl Database {
        fn new() -> Self {
            Self {
                name: "test_db".to_string(),
            }
        }
    }

    // Level 2: Repository (depends on Database)
    struct Repository {
        db: std::sync::Arc<Database>,
    }

    impl Repository {
        fn new(db: std::sync::Arc<Database>) -> Self {
            Self { db }
        }

        fn db_name(&self) -> &str {
            &self.db.name
        }
    }

    // Level 3: Service (depends on Repository)
    struct Service {
        repo: std::sync::Arc<Repository>,
    }

    impl Service {
        fn new(repo: std::sync::Arc<Repository>) -> Self {
            Self { repo }
        }

        fn get_db_name(&self) -> &str {
            self.repo.db_name()
        }
    }

    // Level 4: Controller (depends on Service)
    struct Controller {
        service: std::sync::Arc<Service>,
    }

    impl Controller {
        fn new(service: std::sync::Arc<Service>) -> Self {
            Self { service }
        }

        fn handle_request(&self) -> String {
            format!("Database: {}", self.service.get_db_name())
        }
    }

    #[di_container]
    struct AppContainer {
        database: Database,
        repository: Repository,
        service: Service,
        controller: Controller,
    }

    let container = AppContainer::new();
    let result = container.controller().handle_request();
    assert_eq!(result, "Database: test_db");
}

/// Test dependency injection with multiple services of same type
#[test]
fn test_di_multiple_instances() {
    struct Counter {
        name: String,
        count: u32,
    }

    impl Counter {
        fn new(name: &str) -> Self {
            Self {
                name: name.to_string(),
                count: 0,
            }
        }

        fn name(&self) -> &str {
            &self.name
        }

        fn count(&self) -> u32 {
            self.count
        }
    }

    #[di_container]
    struct AppContainer {
        #[provide(Counter::new("counter_a"))]
        counter_a: Counter,
        #[provide(Counter::new("counter_b"))]
        counter_b: Counter,
    }

    let container = AppContainer::new();
    assert_eq!(container.counter_a().name(), "counter_a");
    assert_eq!(container.counter_a().count(), 0);
    assert_eq!(container.counter_b().name(), "counter_b");
    assert_eq!(container.counter_b().count(), 0);
}
