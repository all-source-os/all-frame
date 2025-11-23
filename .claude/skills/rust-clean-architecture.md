# Rust Clean Architecture Skill

## Purpose
This skill enforces Clean Architecture patterns, SOLID principles, and Rust best practices for the AllFrame framework development.

## When to Use
Activate this skill when:
- Writing Rust code for AllFrame framework
- Implementing proc macros, core abstractions, or framework features
- Working with async/await, trait-based design, or compile-time DI
- Refactoring existing framework code
- Developing framework examples or tests

---

## 🏛️ Clean Architecture Layers (ENFORCED)

### Dependency Rule: Dependencies Point INWARD ONLY

```
Presentation → Application → Domain ← Infrastructure
   (API)       (Use Cases)   (Entities)   (I/O, External)
```

### Layer Responsibilities

#### 1. **Domain Layer** (Core Business Logic)
- **Contains**: Entities, value objects, domain errors, repository traits
- **Dependencies**: NONE (pure Rust, no external crates except basic ones like serde, thiserror)
- **Location**: `src/domain/`

```rust
// ✅ CORRECT - Domain entity
#[derive(Debug, Clone)]
pub struct User {
    id: UserId,
    email: Email,
    created_at: DateTime<Utc>,
}

impl User {
    /// Business logic lives here
    pub fn validate_email(&self) -> Result<(), DomainError> {
        self.email.validate()
    }
}

// ✅ CORRECT - Domain repository trait
#[async_trait]
pub trait UserRepository: Send + Sync {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;
    async fn save(&self, user: &User) -> Result<(), DomainError>;
}
```

#### 2. **Application Layer** (Use Cases & Orchestration)
- **Contains**: Services that orchestrate domain objects and repositories
- **Dependencies**: Domain layer only
- **Location**: `src/application/`

```rust
// ✅ CORRECT - Application service orchestrates
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
}

impl UserService {
    pub async fn create_user(&self, email: String) -> Result<User, ApplicationError> {
        // Validate (domain logic)
        let email = Email::parse(email)?;

        // Create entity
        let user = User::new(UserId::generate(), email);

        // Persist
        self.user_repo.save(&user).await?;

        Ok(user)
    }
}
```

#### 3. **Infrastructure Layer** (External Concerns)
- **Contains**: Database implementations, HTTP clients, external service adapters
- **Dependencies**: Domain layer (implements traits)
- **Location**: `src/infrastructure/`

```rust
// ✅ CORRECT - Infrastructure implements domain trait
pub struct PostgresUserRepository {
    pool: Arc<PgPool>,
}

#[async_trait]
impl UserRepository for PostgresUserRepository {
    async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError> {
        let uuid = Uuid::parse_str(id.as_str())
            .map_err(|e| DomainError::InvalidId(e.to_string()))?;

        let row = sqlx::query!(
            "SELECT id, email, created_at FROM users WHERE id = $1",
            uuid
        )
        .fetch_optional(&*self.pool)
        .await
        .map_err(|e| DomainError::Infrastructure(e.into()))?;

        // Map database row to domain entity
        row.map(|r| User::from_row(r)).transpose()
    }

    async fn save(&self, user: &User) -> Result<(), DomainError> {
        // Save logic
        Ok(())
    }
}
```

#### 4. **Presentation Layer** (HTTP/gRPC/GraphQL Handlers)
- **Contains**: Protocol handlers, DTOs, request/response mapping
- **Dependencies**: Application and Domain layers
- **Location**: `src/presentation/`

```rust
// ✅ CORRECT - Thin handler delegates to service
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    // Delegate to application layer
    let user = state
        .user_service
        .create_user(req.email)
        .await
        .map_err(ApiError::from)?;

    // Convert domain entity to DTO
    Ok(Json(UserResponse::from(user)))
}
```

---

## 🎯 SOLID Principles (ENFORCED)

### 1. Single Responsibility Principle (SRP)

**Each struct/function has ONE reason to change**

```rust
// ❌ WRONG - Multiple responsibilities
pub struct ApplicationService {
    pub fn process_request(&self) -> Result<()> { ... }    // HTTP concern
    pub fn validate_data(&self) -> Result<()> { ... }      // Business logic
    pub fn save_to_database(&self) -> Result<()> { ... }   // Persistence
    pub fn send_notification(&self) -> Result<()> { ... }  // External service
}

// ✅ CORRECT - Separate concerns
pub struct RequestHandler {
    pub fn process_request(&self) -> Result<()> { ... }
}

pub struct DataValidator {
    pub fn validate(&self, data: &Data) -> Result<()> { ... }
}

pub struct Repository {
    pub fn save(&self, entity: &Entity) -> Result<()> { ... }
}

pub struct NotificationService {
    pub fn send(&self, message: &Message) -> Result<()> { ... }
}
```

### 2. Open/Closed Principle (OCP)

**Open for extension, closed for modification**

```rust
// ✅ CORRECT - Trait allows extension without modification
#[async_trait]
pub trait CacheStrategy: Send + Sync {
    async fn get(&self, key: &str) -> Result<Option<Vec<u8>>>;
    async fn set(&self, key: &str, value: Vec<u8>) -> Result<()>;
}

// Redis implementation
pub struct RedisCache { ... }
impl CacheStrategy for RedisCache { ... }

// In-memory implementation (new, no modification to existing code)
pub struct InMemoryCache { ... }
impl CacheStrategy for InMemoryCache { ... }

// Service uses trait, doesn't care about implementation
pub struct CachedService {
    cache: Arc<dyn CacheStrategy>,
}
```

**Event-Driven Extensions**:
```rust
// ✅ Event system allows new handlers without modifying existing code
pub trait EventHandler: Send + Sync {
    async fn handle(&self, event: &DomainEvent) -> Result<()>;
}

pub struct EventBus {
    handlers: Vec<Arc<dyn EventHandler>>,
}

impl EventBus {
    pub async fn publish(&self, event: DomainEvent) -> Result<()> {
        for handler in &self.handlers {
            handler.handle(&event).await?;
        }
        Ok(())
    }

    pub fn subscribe(&mut self, handler: Arc<dyn EventHandler>) {
        self.handlers.push(handler);
    }
}

// New handlers can be added without changing the bus
pub struct LoggingHandler;

#[async_trait]
impl EventHandler for LoggingHandler {
    async fn handle(&self, event: &DomainEvent) -> Result<()> {
        info!("Event received: {:?}", event);
        Ok(())
    }
}
```

### 3. Liskov Substitution Principle (LSP)

**All trait implementations must honor contracts**

```rust
// ✅ CORRECT - All implementations return same contract
#[async_trait]
pub trait Repository<T>: Send + Sync {
    /// Returns None if not found, never errors on not found
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, RepositoryError>;
}

// ✅ CORRECT - Honors contract (returns None, not error)
impl Repository<User> for PostgresUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        sqlx::query_as("SELECT * FROM users WHERE id = $1", id)
            .fetch_optional(&self.pool)
            .await
            .map_err(RepositoryError::from)
    }
}

// ✅ CORRECT - Also honors contract
impl Repository<User> for InMemoryUserRepository {
    async fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        Ok(self.users.read().await.get(id).cloned())
    }
}
```

### 4. Interface Segregation Principle (ISP)

**Create specific traits, not fat interfaces**

```rust
// ❌ WRONG - Fat interface
pub trait DataStore {
    async fn read(&self, key: &str) -> Result<Vec<u8>>;
    async fn write(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
    async fn list_keys(&self) -> Result<Vec<String>>;
    async fn backup(&self, path: &str) -> Result<()>;
    async fn restore(&self, path: &str) -> Result<()>;
    async fn export_csv(&self) -> Result<String>;
    // ... 15 more methods
}

// ✅ CORRECT - Segregated interfaces
pub trait ReadableStore: Send + Sync {
    async fn read(&self, key: &str) -> Result<Vec<u8>>;
}

pub trait WritableStore: Send + Sync {
    async fn write(&self, key: &str, value: Vec<u8>) -> Result<()>;
    async fn delete(&self, key: &str) -> Result<()>;
}

pub trait QueryableStore: Send + Sync {
    async fn list_keys(&self) -> Result<Vec<String>>;
}

pub trait BackupableStore: Send + Sync {
    async fn backup(&self, path: &str) -> Result<()>;
    async fn restore(&self, path: &str) -> Result<()>;
}
```

### 5. Dependency Inversion Principle (DIP)

**Depend on traits, not implementations**

```rust
// ✅ CORRECT - Service depends on traits
pub struct ApplicationService {
    repository: Arc<dyn Repository<User>>,     // Trait, not concrete type
    cache: Arc<dyn CacheStrategy>,              // Trait, not concrete type
    event_bus: Arc<dyn EventBus>,               // Trait, not concrete type
}

impl ApplicationService {
    pub fn new(
        repository: Arc<dyn Repository<User>>,
        cache: Arc<dyn CacheStrategy>,
        event_bus: Arc<dyn EventBus>,
    ) -> Self {
        Self { repository, cache, event_bus }
    }
}

// ❌ WRONG - Service depends on concrete implementations
pub struct ApplicationService {
    repository: PostgresRepository,     // Concrete!
    cache: RedisCache,                  // Concrete!
    event_bus: KafkaEventBus,           // Concrete!
}
```

**Factory Pattern with Dependency Injection**:
```rust
// ✅ CORRECT - Factory returns traits
pub struct ServiceFactory {
    db_pool: Arc<PgPool>,
    redis_client: Arc<redis::Client>,
}

impl ServiceFactory {
    pub fn create_repository(&self) -> Arc<dyn Repository<User>> {
        Arc::new(PostgresRepository::new(Arc::clone(&self.db_pool)))
    }

    pub fn create_cache(&self) -> Arc<dyn CacheStrategy> {
        Arc::new(RedisCache::new(Arc::clone(&self.redis_client)))
    }
}
```

---

## 🦀 Rust-Specific Best Practices

### Error Handling (MANDATORY)

#### Layer-Specific Error Types

```rust
// ✅ CORRECT - Domain errors (business failures)
#[derive(Debug, thiserror::Error)]
pub enum DomainError {
    #[error("Entity not found: {id}")]
    NotFound { id: String },

    #[error("Invalid input: {reason}")]
    InvalidInput { reason: String },

    #[error("Business rule violated: {rule}")]
    BusinessRuleViolation { rule: String },

    #[error("Infrastructure error: {0}")]
    Infrastructure(#[from] InfrastructureError),
}

// ✅ CORRECT - Infrastructure errors (external system failures)
#[derive(Debug, thiserror::Error)]
pub enum InfrastructureError {
    #[error("Database error: {0}")]
    Database(#[from] sqlx::Error),

    #[error("HTTP client error: {0}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

// ✅ CORRECT - API errors (presentation-layer responses)
#[derive(Debug, thiserror::Error)]
pub enum ApiError {
    #[error("Bad request: {0}")]
    BadRequest(String),

    #[error("Not found: {0}")]
    NotFound(String),

    #[error("Internal server error")]
    InternalServerError,
}

// Error conversion between layers
impl From<DomainError> for ApiError {
    fn from(err: DomainError) -> Self {
        match err {
            DomainError::NotFound { id } => ApiError::NotFound(id),
            DomainError::InvalidInput { reason } => ApiError::BadRequest(reason),
            _ => ApiError::InternalServerError,
        }
    }
}
```

#### NEVER use unwrap() or expect() in production

```rust
// ❌ WRONG - Will panic!
let value = repository.find_by_id(id).await.unwrap();
let parsed = serde_json::from_str(&data).expect("valid JSON");

// ✅ CORRECT - Proper error handling
let value = repository.find_by_id(id).await?
    .ok_or(DomainError::NotFound { id: id.to_string() })?;

let parsed = serde_json::from_str(&data)
    .map_err(|e| InfrastructureError::Serialization(e))?;
```

### Async/Await Patterns

```rust
// ✅ CORRECT - Async trait with async_trait
#[async_trait]
pub trait Repository<T>: Send + Sync {
    async fn find_by_id(&self, id: &str) -> Result<Option<T>, DomainError>;
    async fn save(&self, entity: &T) -> Result<(), DomainError>;
}

// ✅ CORRECT - Async service method
impl UserService {
    pub async fn get_user(&self, id: &str) -> Result<User, ApplicationError> {
        let user = self.repository.find_by_id(id).await?
            .ok_or(ApplicationError::UserNotFound)?;
        Ok(user)
    }
}

// ❌ WRONG - Blocking operation in async context
async fn load_config() -> Result<Config, Error> {
    let data = std::fs::read_to_string("config.toml")?;  // Blocking!
    Ok(toml::from_str(&data)?)
}

// ✅ CORRECT - Async file operation
async fn load_config() -> Result<Config, Error> {
    let data = tokio::fs::read_to_string("config.toml").await?;  // Async!
    Ok(toml::from_str(&data)?)
}
```

### Type Safety with Newtypes

```rust
// ✅ CORRECT - Newtype pattern for domain values
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct UserId(Uuid);

impl UserId {
    pub fn generate() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn parse(s: &str) -> Result<Self, ValidationError> {
        let uuid = Uuid::parse_str(s)
            .map_err(|_| ValidationError::InvalidUserId)?;
        Ok(Self(uuid))
    }

    pub fn as_str(&self) -> &str {
        self.0.as_hyphenated().encode_lower(&mut Uuid::encode_buffer())
    }
}

// ❌ WRONG - Primitive obsession
pub async fn transfer(from: String, to: String, amount: f64) -> Result<()> {
    // Easy to swap parameters by mistake!
}

// ✅ CORRECT - Type-safe domain values
pub async fn transfer(
    from: UserId,
    to: UserId,
    amount: Money,
) -> Result<(), DomainError> {
    // Compiler prevents mistakes!
}
```

### Dependency Injection with Arc

```rust
// ✅ CORRECT - Factory pattern with Arc for shared ownership
pub struct DependencyContainer {
    db_pool: Arc<PgPool>,
}

impl DependencyContainer {
    pub fn new(db_pool: PgPool) -> Self {
        Self {
            db_pool: Arc::new(db_pool),
        }
    }

    pub fn user_repository(&self) -> Arc<dyn UserRepository> {
        Arc::new(PostgresUserRepository::new(Arc::clone(&self.db_pool)))
    }

    pub fn post_repository(&self) -> Arc<dyn PostRepository> {
        Arc::new(PostgresPostRepository::new(Arc::clone(&self.db_pool)))
    }
}

// ✅ CORRECT - Service uses Arc for shared ownership
pub struct UserService {
    user_repo: Arc<dyn UserRepository>,
    post_repo: Arc<dyn PostRepository>,
}

impl UserService {
    pub fn new(
        user_repo: Arc<dyn UserRepository>,
        post_repo: Arc<dyn PostRepository>,
    ) -> Self {
        Self { user_repo, post_repo }
    }
}
```

---

## 🚫 Rust Anti-Patterns (FORBIDDEN)

### ❌ NEVER DO:

1. **Use `unwrap()` or `expect()` in production**
```rust
// ❌ WRONG
let value = some_option.unwrap();
let result = operation().expect("this should work");

// ✅ CORRECT
let value = some_option.ok_or(Error::ValueNotFound)?;
let result = operation().map_err(Error::from)?;
```

2. **Put business logic in handlers**
```rust
// ❌ WRONG - Business logic in presentation layer
pub async fn create_user_handler(
    Json(req): Json<CreateUserRequest>
) -> Result<Json<User>, ApiError> {
    // Validation (business logic!)
    if req.email.is_empty() {
        return Err(ApiError::BadRequest("Email required".into()));
    }

    // Business logic!
    let user = User {
        id: Uuid::new_v4(),
        email: req.email,
    };

    // Database access!
    sqlx::query!("INSERT INTO users...").execute(&pool).await?;

    Ok(Json(user))
}

// ✅ CORRECT - Delegate to service
pub async fn create_user_handler(
    State(state): State<AppState>,
    Json(req): Json<CreateUserRequest>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.user_service.create_user(req.email).await?;
    Ok(Json(UserResponse::from(user)))
}
```

3. **Use `panic!` for recoverable errors**
```rust
// ❌ WRONG
fn divide(a: i32, b: i32) -> i32 {
    if b == 0 {
        panic!("Division by zero!");
    }
    a / b
}

// ✅ CORRECT
fn divide(a: i32, b: i32) -> Result<i32, MathError> {
    if b == 0 {
        return Err(MathError::DivisionByZero);
    }
    Ok(a / b)
}
```

4. **Access infrastructure directly from handlers**
```rust
// ❌ WRONG
pub async fn get_user(
    Path(id): Path<String>,
    State(pool): State<PgPool>,
) -> Result<Json<User>, ApiError> {
    let user = sqlx::query_as("SELECT * FROM users WHERE id = $1")
        .bind(id)
        .fetch_one(&pool)
        .await?;
    Ok(Json(user))
}

// ✅ CORRECT
pub async fn get_user(
    Path(id): Path<String>,
    State(state): State<AppState>,
) -> Result<Json<UserResponse>, ApiError> {
    let user = state.user_service.get_user(&id).await?;
    Ok(Json(UserResponse::from(user)))
}
```

5. **Ignore compiler warnings**
```rust
// ❌ WRONG
#![allow(warnings)]

// ✅ CORRECT
#![deny(warnings)]  // Fail build on warnings

// Or for specific cases with documentation:
#[allow(dead_code)]  // Used in integration tests
fn helper_function() { ... }
```

---

## 🧪 Testing Requirements

### Unit Tests (Domain Layer)

```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn user_email_validation_succeeds_for_valid_email() {
        let email = Email::parse("user@example.com").unwrap();
        assert_eq!(email.as_str(), "user@example.com");
    }

    #[test]
    fn user_email_validation_fails_for_invalid_email() {
        let result = Email::parse("not-an-email");
        assert!(result.is_err());
    }
}
```

### Service Tests with Mocks

```rust
#[cfg(test)]
mod tests {
    use super::*;
    use mockall::predicate::*;
    use mockall::mock;

    mock! {
        pub UserRepo {}

        #[async_trait]
        impl UserRepository for UserRepo {
            async fn find_by_id(&self, id: &UserId) -> Result<Option<User>, DomainError>;
            async fn save(&self, user: &User) -> Result<(), DomainError>;
        }
    }

    #[tokio::test]
    async fn create_user_success() {
        let mut mock_repo = MockUserRepo::new();

        mock_repo
            .expect_save()
            .times(1)
            .returning(|_| Ok(()));

        let service = UserService::new(Arc::new(mock_repo));
        let result = service.create_user("test@example.com".to_string()).await;

        assert!(result.is_ok());
    }
}
```

### Integration Tests (Repository Layer)

```rust
#[cfg(test)]
mod integration_tests {
    use super::*;

    #[sqlx::test]
    async fn save_and_find_user(pool: PgPool) {
        let repo = PostgresUserRepository::new(Arc::new(pool));

        // Save user
        let user = User::new(UserId::generate(), Email::parse("test@example.com").unwrap());
        repo.save(&user).await.unwrap();

        // Retrieve user
        let found = repo.find_by_id(&user.id).await.unwrap();

        assert!(found.is_some());
        assert_eq!(found.unwrap().email, user.email);
    }
}
```

---

## 📋 Code Templates for Common Patterns

### Domain Entity Template

```rust
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    id: EntityId,
    name: String,
    created_at: DateTime<Utc>,
}

impl Entity {
    /// Constructor with validation
    pub fn new(name: String) -> Result<Self, DomainError> {
        if name.trim().is_empty() {
            return Err(DomainError::InvalidInput {
                reason: "Name cannot be empty".to_string()
            });
        }

        Ok(Self {
            id: EntityId::generate(),
            name: name.trim().to_string(),
            created_at: Utc::now(),
        })
    }

    /// Business logic methods
    pub fn rename(&mut self, new_name: String) -> Result<(), DomainError> {
        if new_name.trim().is_empty() {
            return Err(DomainError::InvalidInput {
                reason: "Name cannot be empty".to_string()
            });
        }
        self.name = new_name.trim().to_string();
        Ok(())
    }
}
```

### Repository Interface Template

```rust
use async_trait::async_trait;

#[async_trait]
pub trait EntityRepository: Send + Sync {
    async fn find_by_id(&self, id: &EntityId) -> Result<Option<Entity>, DomainError>;
    async fn find_all(&self, limit: usize, offset: usize) -> Result<Vec<Entity>, DomainError>;
    async fn save(&self, entity: &Entity) -> Result<(), DomainError>;
    async fn delete(&self, id: &EntityId) -> Result<bool, DomainError>;
}
```

### Service Template

```rust
use std::sync::Arc;
use tracing::{info, instrument};

pub struct EntityService {
    repository: Arc<dyn EntityRepository>,
}

impl EntityService {
    pub fn new(repository: Arc<dyn EntityRepository>) -> Self {
        Self { repository }
    }

    #[instrument(skip(self))]
    pub async fn create_entity(&self, name: String) -> Result<Entity, ApplicationError> {
        info!("Creating entity with name: {}", name);

        let entity = Entity::new(name)?;
        self.repository.save(&entity).await?;

        info!("Entity created: {:?}", entity.id);
        Ok(entity)
    }

    #[instrument(skip(self))]
    pub async fn get_entity(&self, id: &EntityId) -> Result<Entity, ApplicationError> {
        self.repository
            .find_by_id(id)
            .await?
            .ok_or(ApplicationError::NotFound)
    }
}
```

---

## 📋 Feature Implementation Checklist

When implementing a new feature, follow this EXACT order:

- [ ] **1. Domain Layer**
  - [ ] Define entities and value objects
  - [ ] Create repository trait
  - [ ] Implement business logic in entities
  - [ ] Write unit tests for entities

- [ ] **2. Application Layer**
  - [ ] Create service struct
  - [ ] Implement use case methods
  - [ ] Write unit tests with mocked repositories

- [ ] **3. Infrastructure Layer**
  - [ ] Implement repository trait
  - [ ] Add to dependency container/factory
  - [ ] Write integration tests for repository

- [ ] **4. Presentation Layer**
  - [ ] Create DTOs
  - [ ] Implement handlers
  - [ ] Add API documentation
  - [ ] Write API integration tests

- [ ] **5. Quality Checks**
  - [ ] Run `cargo clippy` and fix all warnings
  - [ ] Run `cargo fmt`
  - [ ] All tests passing (`cargo test`)
  - [ ] Update documentation if needed

---

## Quick Reference

### Before Committing Code:

- [ ] All layers respect boundaries (no circular dependencies)
- [ ] Business logic in domain entities (not services or handlers)
- [ ] No `unwrap()` or `panic!` in production code
- [ ] All errors properly handled with `Result<T, E>`
- [ ] Repository traits in domain, implementations in infrastructure
- [ ] Services use dependency injection via `Arc<dyn Trait>`
- [ ] Handlers are thin (delegate to services)
- [ ] All tests passing (unit + integration)
- [ ] `cargo clippy` has no warnings
- [ ] `cargo fmt` applied
- [ ] Documentation updated if needed

---

**Remember**: Clean Architecture → Maintainable Code → Happy Developers
