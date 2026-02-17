//! Producer archetype templates
//!
//! Templates for generating event producer services with outbox pattern,
//! transactional messaging, and reliable event publishing.

use crate::config::{MessageBroker, ProjectConfig};

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

/// Generate Cargo.toml for producer project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let name = &config.name;

    let broker_deps = match producer.broker {
        MessageBroker::Kafka => r#"rdkafka = { version = "0.36", features = ["cmake-build"] }"#,
        MessageBroker::RabbitMq => r#"lapin = "2.3""#,
        MessageBroker::Redis => {
            r#"redis = { version = "0.25", features = ["tokio-comp", "streams"] }"#
        }
        MessageBroker::Sqs => r#"aws-sdk-sqs = "1.0""#,
        MessageBroker::PubSub => r#"google-cloud-pubsub = "0.25""#,
    };

    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2021"
rust-version = "1.75"
description = "{display_name}"

[dependencies]
# AllFrame
allframe-core = {{ version = "0.1", features = ["resilience", "otel"] }}

# Message Broker
{broker_deps}

# Async
tokio = {{ version = "1", features = ["full"] }}
async-trait = "0.1"
futures = "0.3"

# Web Framework (for API)
axum = "0.7"
tower = "0.5"
tower-http = {{ version = "0.6", features = ["trace", "cors"] }}

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Database (for outbox)
sqlx = {{ version = "0.8", features = ["runtime-tokio", "postgres", "uuid", "chrono", "json"] }}

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Tracing & Metrics
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}
opentelemetry = {{ version = "0.27", features = ["metrics"] }}

# Utilities
chrono = {{ version = "0.4", features = ["serde"] }}
uuid = {{ version = "1.0", features = ["v4", "serde"] }}
dotenvy = "0.15"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = name,
        display_name = producer.display_name,
        broker_deps = broker_deps,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! {display_name}
//!
//! An event producer service with outbox pattern for reliable event publishing.

use std::sync::Arc;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use config::Config;
use application::{pascal_name}Service;
use infrastructure::{{
    KafkaEventPublisher,
    PostgresOutbox,
    PostgresRepository,
    OutboxProcessor,
    HealthServer,
}};

#[tokio::main]
async fn main() -> anyhow::Result<()> {{
    // Load environment variables
    dotenvy::dotenv().ok();

    // Initialize tracing
    tracing_subscriber::fmt()
        .with_env_filter(
            tracing_subscriber::EnvFilter::from_default_env()
                .add_directive(tracing::Level::INFO.into()),
        )
        .init();

    // Load configuration
    let config = Config::from_env();
    info!("Starting {display_name}");
    info!("Database: {{}}", config.database.url);
    info!("Broker: {{}}", config.broker.brokers);

    // Create database pool
    let db_pool = sqlx::postgres::PgPoolOptions::new()
        .max_connections(config.database.max_connections)
        .connect(&config.database.url)
        .await?;

    // Create outbox
    let outbox = Arc::new(PostgresOutbox::new(db_pool.clone()));

    // Create event publisher
    let publisher = Arc::new(KafkaEventPublisher::new(&config.broker).await?);

    // Create repository
    let repository = Arc::new(PostgresRepository::new(db_pool.clone()));

    // Create service
    let service = Arc::new({pascal_name}Service::new(
        repository.clone(),
        outbox.clone(),
    ));

    // Start outbox processor in background
    let outbox_processor = OutboxProcessor::new(
        outbox.clone(),
        publisher.clone(),
        config.outbox.clone(),
    );
    let outbox_handle = tokio::spawn(async move {{
        outbox_processor.run().await
    }});

    // Start health server in background
    let health_port = config.server.health_port;
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(health_port);
        health_server.run().await
    }});

    // Start API server
    info!("Starting API server on port {{}}", config.server.port);
    let app = presentation::create_router(service);
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{{}}", config.server.port)
    ).await?;
    axum::serve(listener, app).await?;

    outbox_handle.abort();
    health_handle.abort();
    info!("Service shutdown complete");
    Ok(())
}}
"#,
        pascal_name = pascal_name,
        display_name = producer.display_name,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();

    format!(
        r#"//! Service configuration

use std::env;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub broker: BrokerConfig,
    pub outbox: OutboxConfig,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub port: u16,
    pub health_port: u16,
}}

/// Database configuration
#[derive(Debug, Clone)]
pub struct DatabaseConfig {{
    pub url: String,
    pub max_connections: u32,
}}

/// Message broker configuration
#[derive(Debug, Clone)]
pub struct BrokerConfig {{
    pub brokers: String,
    pub topic: String,
}}

/// Outbox configuration
#[derive(Debug, Clone)]
pub struct OutboxConfig {{
    pub poll_interval_ms: u64,
    pub batch_size: usize,
    pub max_retries: u32,
}}

impl Config {{
    pub fn from_env() -> Self {{
        Self {{
            server: ServerConfig {{
                port: env::var("PORT")
                    .unwrap_or_else(|_| "{port}".to_string())
                    .parse()
                    .expect("PORT must be a number"),
                health_port: env::var("HEALTH_PORT")
                    .unwrap_or_else(|_| "{health_port}".to_string())
                    .parse()
                    .expect("HEALTH_PORT must be a number"),
            }},
            database: DatabaseConfig {{
                url: env::var("DATABASE_URL")
                    .unwrap_or_else(|_| "postgres://localhost/{db_name}".to_string()),
                max_connections: env::var("DATABASE_MAX_CONNECTIONS")
                    .unwrap_or_else(|_| "10".to_string())
                    .parse()
                    .expect("DATABASE_MAX_CONNECTIONS must be a number"),
            }},
            broker: BrokerConfig {{
                brokers: env::var("KAFKA_BROKERS")
                    .unwrap_or_else(|_| "localhost:9092".to_string()),
                topic: env::var("KAFKA_TOPIC")
                    .unwrap_or_else(|_| "{topic}".to_string()),
            }},
            outbox: OutboxConfig {{
                poll_interval_ms: env::var("OUTBOX_POLL_INTERVAL_MS")
                    .unwrap_or_else(|_| "{poll_interval}".to_string())
                    .parse()
                    .expect("OUTBOX_POLL_INTERVAL_MS must be a number"),
                batch_size: env::var("OUTBOX_BATCH_SIZE")
                    .unwrap_or_else(|_| "{batch_size}".to_string())
                    .parse()
                    .expect("OUTBOX_BATCH_SIZE must be a number"),
                max_retries: env::var("OUTBOX_MAX_RETRIES")
                    .unwrap_or_else(|_| "{max_retries}".to_string())
                    .parse()
                    .expect("OUTBOX_MAX_RETRIES must be a number"),
            }},
        }}
    }}
}}
"#,
        port = producer.server.http_port,
        health_port = producer.server.health_port,
        db_name = config.name.replace('-', "_"),
        topic = producer
            .topics
            .first()
            .map(|t| t.name.as_str())
            .unwrap_or("events"),
        poll_interval = producer.outbox.polling_interval_ms,
        batch_size = producer.outbox.batch_size,
        max_retries = producer.outbox.max_retries,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Error types

use thiserror::Error;

/// Service errors
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Entity not found: {{0}}")]
    NotFound(String),

    #[error("Validation error: {{0}}")]
    Validation(String),

    #[error("Database error: {{0}}")]
    Database(#[from] sqlx::Error),

    #[error("Serialization error: {{0}}")]
    Serialization(#[from] serde_json::Error),

    #[error("Broker error: {{0}}")]
    Broker(String),

    #[error("Internal error: {{0}}")]
    Internal(String),
}}

impl {pascal_name}Error {{
    pub fn status_code(&self) -> axum::http::StatusCode {{
        use axum::http::StatusCode;
        match self {{
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::Database(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Serialization(_) => StatusCode::BAD_REQUEST,
            Self::Broker(_) => StatusCode::SERVICE_UNAVAILABLE,
            Self::Internal(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }}
    }}
}}

impl axum::response::IntoResponse for {pascal_name}Error {{
    fn into_response(self) -> axum::response::Response {{
        let status = self.status_code();
        let body = axum::Json(serde_json::json!({{
            "error": self.to_string()
        }}));
        (status, body).into_response()
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/mod.rs
pub fn domain_mod(_config: &ProjectConfig) -> String {
    r#"//! Domain layer

pub mod entities;
pub mod events;
pub mod repository;

pub use entities::*;
pub use events::*;
pub use repository::*;
"#
    .to_string()
}

/// Generate domain/entities.rs
pub fn domain_entities(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Domain entities

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// Main domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal_name}Entity {{
    pub id: Uuid,
    pub name: String,
    pub data: serde_json::Value,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

impl {pascal_name}Entity {{
    pub fn new(name: String, data: serde_json::Value) -> Self {{
        let now = Utc::now();
        Self {{
            id: Uuid::new_v4(),
            name,
            data,
            created_at: now,
            updated_at: now,
        }}
    }}
}}

/// Request to create a new entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Create{pascal_name}Request {{
    pub name: String,
    pub data: serde_json::Value,
}}

/// Request to update an entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Update{pascal_name}Request {{
    pub name: Option<String>,
    pub data: Option<serde_json::Value>,
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/events.rs
pub fn domain_events(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Domain events

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// Event envelope with metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {{
    pub event_id: Uuid,
    pub event_type: String,
    pub aggregate_id: Uuid,
    pub aggregate_type: String,
    pub timestamp: DateTime<Utc>,
    pub version: u32,
    pub payload: T,
}}

impl<T> EventEnvelope<T> {{
    pub fn new(
        event_type: &str,
        aggregate_id: Uuid,
        aggregate_type: &str,
        payload: T,
    ) -> Self {{
        Self {{
            event_id: Uuid::new_v4(),
            event_type: event_type.to_string(),
            aggregate_id,
            aggregate_type: aggregate_type.to_string(),
            timestamp: Utc::now(),
            version: 1,
            payload,
        }}
    }}
}}

/// Entity created event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal_name}Created {{
    pub id: Uuid,
    pub name: String,
    pub data: serde_json::Value,
}}

/// Entity updated event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal_name}Updated {{
    pub id: Uuid,
    pub name: Option<String>,
    pub data: Option<serde_json::Value>,
}}

/// Entity deleted event
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal_name}Deleted {{
    pub id: Uuid,
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/repository.rs
pub fn domain_repository(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Repository trait

use async_trait::async_trait;
use uuid::Uuid;

use super::{pascal_name}Entity;
use crate::error::{pascal_name}Error;

/// Repository trait for entity persistence
#[async_trait]
pub trait {pascal_name}Repository: Send + Sync {{
    async fn create(&self, entity: &{pascal_name}Entity) -> Result<(), {pascal_name}Error>;
    async fn get(&self, id: Uuid) -> Result<Option<{pascal_name}Entity>, {pascal_name}Error>;
    async fn update(&self, entity: &{pascal_name}Entity) -> Result<(), {pascal_name}Error>;
    async fn delete(&self, id: Uuid) -> Result<(), {pascal_name}Error>;
    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<{pascal_name}Entity>, {pascal_name}Error>;
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer

pub mod service;
pub mod outbox;

pub use service::*;
pub use outbox::*;
"#
    .to_string()
}

/// Generate application/service.rs
pub fn application_service(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Application service

use std::sync::Arc;
use tracing::{{info, instrument}};
use uuid::Uuid;

use crate::domain::{{
    {pascal_name}Entity,
    {pascal_name}Repository,
    {pascal_name}Created,
    {pascal_name}Updated,
    {pascal_name}Deleted,
    EventEnvelope,
    Create{pascal_name}Request,
    Update{pascal_name}Request,
}};
use crate::error::{pascal_name}Error;
use super::Outbox;

/// Application service that coordinates domain operations and event publishing
pub struct {pascal_name}Service {{
    repository: Arc<dyn {pascal_name}Repository>,
    outbox: Arc<dyn Outbox>,
}}

impl {pascal_name}Service {{
    pub fn new(
        repository: Arc<dyn {pascal_name}Repository>,
        outbox: Arc<dyn Outbox>,
    ) -> Self {{
        Self {{ repository, outbox }}
    }}

    #[instrument(skip(self, request))]
    pub async fn create(&self, request: Create{pascal_name}Request) -> Result<{pascal_name}Entity, {pascal_name}Error> {{
        let entity = {pascal_name}Entity::new(request.name, request.data);

        // Save entity
        self.repository.create(&entity).await?;

        // Create and store event in outbox
        let event = EventEnvelope::new(
            "{pascal_name}Created",
            entity.id,
            "{pascal_name}",
            {pascal_name}Created {{
                id: entity.id,
                name: entity.name.clone(),
                data: entity.data.clone(),
            }},
        );
        self.outbox.store(event).await?;

        info!("Created entity: {{}}", entity.id);
        Ok(entity)
    }}

    #[instrument(skip(self))]
    pub async fn get(&self, id: Uuid) -> Result<{pascal_name}Entity, {pascal_name}Error> {{
        self.repository
            .get(id)
            .await?
            .ok_or_else(|| {pascal_name}Error::NotFound(format!("Entity {{}} not found", id)))
    }}

    #[instrument(skip(self, request))]
    pub async fn update(
        &self,
        id: Uuid,
        request: Update{pascal_name}Request,
    ) -> Result<{pascal_name}Entity, {pascal_name}Error> {{
        let mut entity = self.get(id).await?;

        if let Some(name) = &request.name {{
            entity.name = name.clone();
        }}
        if let Some(data) = &request.data {{
            entity.data = data.clone();
        }}
        entity.updated_at = chrono::Utc::now();

        // Update entity
        self.repository.update(&entity).await?;

        // Create and store event in outbox
        let event = EventEnvelope::new(
            "{pascal_name}Updated",
            entity.id,
            "{pascal_name}",
            {pascal_name}Updated {{
                id: entity.id,
                name: request.name,
                data: request.data,
            }},
        );
        self.outbox.store(event).await?;

        info!("Updated entity: {{}}", entity.id);
        Ok(entity)
    }}

    #[instrument(skip(self))]
    pub async fn delete(&self, id: Uuid) -> Result<(), {pascal_name}Error> {{
        // Verify entity exists
        let _ = self.get(id).await?;

        // Delete entity
        self.repository.delete(id).await?;

        // Create and store event in outbox
        let event = EventEnvelope::new(
            "{pascal_name}Deleted",
            id,
            "{pascal_name}",
            {pascal_name}Deleted {{ id }},
        );
        self.outbox.store(event).await?;

        info!("Deleted entity: {{}}", id);
        Ok(())
    }}

    #[instrument(skip(self))]
    pub async fn list(&self, limit: i64, offset: i64) -> Result<Vec<{pascal_name}Entity>, {pascal_name}Error> {{
        self.repository.list(limit, offset).await
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/outbox.rs
pub fn application_outbox(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Outbox pattern trait

use async_trait::async_trait;
use serde::Serialize;
use uuid::Uuid;

use crate::error::{pascal_name}Error;

/// Outbox entry
#[derive(Debug, Clone)]
pub struct OutboxEntry {{
    pub id: Uuid,
    pub event_type: String,
    pub payload: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub processed_at: Option<chrono::DateTime<chrono::Utc>>,
    pub retries: i32,
}}

/// Outbox trait for storing events
#[async_trait]
pub trait Outbox: Send + Sync {{
    async fn store<T: Serialize + Send + Sync>(&self, event: T) -> Result<(), {pascal_name}Error>;
    async fn get_pending(&self, batch_size: usize) -> Result<Vec<OutboxEntry>, {pascal_name}Error>;
    async fn mark_processed(&self, id: Uuid) -> Result<(), {pascal_name}Error>;
    async fn mark_failed(&self, id: Uuid) -> Result<(), {pascal_name}Error>;
}}

/// Event publisher trait
#[async_trait]
pub trait EventPublisher: Send + Sync {{
    async fn publish(&self, topic: &str, key: &str, payload: &str) -> Result<(), {pascal_name}Error>;
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(_config: &ProjectConfig) -> String {
    r#"//! Infrastructure layer

pub mod repository;
pub mod outbox;
pub mod publisher;
pub mod outbox_processor;
pub mod health;

pub use repository::*;
pub use outbox::*;
pub use publisher::*;
pub use outbox_processor::*;
pub use health::*;
"#
    .to_string()
}

/// Generate infrastructure/repository.rs
pub fn infrastructure_repository(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);
    let table_name = producer.service_name.replace('-', "_");

    format!(
        r##"//! PostgreSQL repository implementation

use async_trait::async_trait;
use sqlx::PgPool;
use uuid::Uuid;

use crate::domain::{{
    {pascal_name}Entity,
    {pascal_name}Repository,
}};
use crate::error::{pascal_name}Error;

/// PostgreSQL repository implementation
pub struct PostgresRepository {{
    pool: PgPool,
}}

impl PostgresRepository {{
    pub fn new(pool: PgPool) -> Self {{
        Self {{ pool }}
    }}
}}

#[async_trait]
impl {pascal_name}Repository for PostgresRepository {{
    async fn create(&self, entity: &{pascal_name}Entity) -> Result<(), {pascal_name}Error> {{
        sqlx::query(
            r#"
            INSERT INTO {table_name} (id, name, data, created_at, updated_at)
            VALUES ($1, $2, $3, $4, $5)
            "#,
        )
        .bind(entity.id)
        .bind(&entity.name)
        .bind(&entity.data)
        .bind(entity.created_at)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }}

    async fn get(&self, id: Uuid) -> Result<Option<{pascal_name}Entity>, {pascal_name}Error> {{
        let row = sqlx::query_as::<_, (Uuid, String, serde_json::Value, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, name, data, created_at, updated_at
            FROM {table_name}
            WHERE id = $1
            "#,
        )
        .bind(id)
        .fetch_optional(&self.pool)
        .await?;

        Ok(row.map(|(id, name, data, created_at, updated_at)| {pascal_name}Entity {{
            id,
            name,
            data,
            created_at,
            updated_at,
        }}))
    }}

    async fn update(&self, entity: &{pascal_name}Entity) -> Result<(), {pascal_name}Error> {{
        sqlx::query(
            r#"
            UPDATE {table_name}
            SET name = $2, data = $3, updated_at = $4
            WHERE id = $1
            "#,
        )
        .bind(entity.id)
        .bind(&entity.name)
        .bind(&entity.data)
        .bind(entity.updated_at)
        .execute(&self.pool)
        .await?;

        Ok(())
    }}

    async fn delete(&self, id: Uuid) -> Result<(), {pascal_name}Error> {{
        sqlx::query(
            r#"
            DELETE FROM {table_name}
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }}

    async fn list(&self, limit: i64, offset: i64) -> Result<Vec<{pascal_name}Entity>, {pascal_name}Error> {{
        let rows = sqlx::query_as::<_, (Uuid, String, serde_json::Value, chrono::DateTime<chrono::Utc>, chrono::DateTime<chrono::Utc>)>(
            r#"
            SELECT id, name, data, created_at, updated_at
            FROM {table_name}
            ORDER BY created_at DESC
            LIMIT $1 OFFSET $2
            "#,
        )
        .bind(limit)
        .bind(offset)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, name, data, created_at, updated_at)| {pascal_name}Entity {{
            id,
            name,
            data,
            created_at,
            updated_at,
        }}).collect())
    }}
}}
"##,
        pascal_name = pascal_name,
        table_name = table_name,
    )
}

/// Generate infrastructure/outbox.rs
pub fn infrastructure_outbox(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r##"//! PostgreSQL outbox implementation

use async_trait::async_trait;
use serde::Serialize;
use sqlx::PgPool;
use uuid::Uuid;

use crate::application::{{Outbox, OutboxEntry}};
use crate::error::{pascal_name}Error;

/// PostgreSQL outbox implementation
pub struct PostgresOutbox {{
    pool: PgPool,
}}

impl PostgresOutbox {{
    pub fn new(pool: PgPool) -> Self {{
        Self {{ pool }}
    }}
}}

#[async_trait]
impl Outbox for PostgresOutbox {{
    async fn store<T: Serialize + Send + Sync>(&self, event: T) -> Result<(), {pascal_name}Error> {{
        let payload = serde_json::to_string(&event)?;
        let event_type = std::any::type_name::<T>()
            .rsplit("::")
            .next()
            .unwrap_or("Unknown");

        sqlx::query(
            r#"
            INSERT INTO outbox (id, event_type, payload, created_at, retries)
            VALUES ($1, $2, $3, $4, 0)
            "#,
        )
        .bind(Uuid::new_v4())
        .bind(event_type)
        .bind(payload)
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }}

    async fn get_pending(&self, batch_size: usize) -> Result<Vec<OutboxEntry>, {pascal_name}Error> {{
        let rows = sqlx::query_as::<_, (Uuid, String, String, chrono::DateTime<chrono::Utc>, Option<chrono::DateTime<chrono::Utc>>, i32)>(
            r#"
            SELECT id, event_type, payload, created_at, processed_at, retries
            FROM outbox
            WHERE processed_at IS NULL
            ORDER BY created_at ASC
            LIMIT $1
            FOR UPDATE SKIP LOCKED
            "#,
        )
        .bind(batch_size as i64)
        .fetch_all(&self.pool)
        .await?;

        Ok(rows.into_iter().map(|(id, event_type, payload, created_at, processed_at, retries)| OutboxEntry {{
            id,
            event_type,
            payload,
            created_at,
            processed_at,
            retries,
        }}).collect())
    }}

    async fn mark_processed(&self, id: Uuid) -> Result<(), {pascal_name}Error> {{
        sqlx::query(
            r#"
            UPDATE outbox
            SET processed_at = $2
            WHERE id = $1
            "#,
        )
        .bind(id)
        .bind(chrono::Utc::now())
        .execute(&self.pool)
        .await?;

        Ok(())
    }}

    async fn mark_failed(&self, id: Uuid) -> Result<(), {pascal_name}Error> {{
        sqlx::query(
            r#"
            UPDATE outbox
            SET retries = retries + 1
            WHERE id = $1
            "#,
        )
        .bind(id)
        .execute(&self.pool)
        .await?;

        Ok(())
    }}
}}
"##,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/publisher.rs
pub fn infrastructure_publisher(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Kafka event publisher implementation

use async_trait::async_trait;
use rdkafka::config::ClientConfig;
use rdkafka::producer::{{FutureProducer, FutureRecord}};
use std::time::Duration;
use tracing::error;

use crate::application::EventPublisher;
use crate::config::BrokerConfig;
use crate::error::{pascal_name}Error;

/// Kafka event publisher
pub struct KafkaEventPublisher {{
    producer: FutureProducer,
}}

impl KafkaEventPublisher {{
    pub async fn new(config: &BrokerConfig) -> Result<Self, {pascal_name}Error> {{
        let producer: FutureProducer = ClientConfig::new()
            .set("bootstrap.servers", &config.brokers)
            .set("message.timeout.ms", "5000")
            .create()
            .map_err(|e| {pascal_name}Error::Broker(e.to_string()))?;

        Ok(Self {{ producer }})
    }}
}}

#[async_trait]
impl EventPublisher for KafkaEventPublisher {{
    async fn publish(&self, topic: &str, key: &str, payload: &str) -> Result<(), {pascal_name}Error> {{
        let record = FutureRecord::to(topic)
            .key(key)
            .payload(payload);

        self.producer
            .send(record, Duration::from_secs(5))
            .await
            .map_err(|(e, _)| {{
                error!("Failed to publish event: {{}}", e);
                {pascal_name}Error::Broker(e.to_string())
            }})?;

        Ok(())
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/outbox_processor.rs
pub fn infrastructure_outbox_processor(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);

    format!(
        r#"//! Outbox processor for reliable event publishing

use std::sync::Arc;
use std::time::Duration;
use tracing::{{error, info, warn}};

use crate::application::{{EventPublisher, Outbox}};
use crate::config::OutboxConfig;

/// Outbox processor that polls and publishes pending events
pub struct OutboxProcessor {{
    outbox: Arc<dyn Outbox>,
    publisher: Arc<dyn EventPublisher>,
    config: OutboxConfig,
}}

impl OutboxProcessor {{
    pub fn new(
        outbox: Arc<dyn Outbox>,
        publisher: Arc<dyn EventPublisher>,
        config: OutboxConfig,
    ) -> Self {{
        Self {{
            outbox,
            publisher,
            config,
        }}
    }}

    pub async fn run(&self) {{
        info!("Outbox processor started");

        loop {{
            match self.process_batch().await {{
                Ok(count) => {{
                    if count > 0 {{
                        info!("Processed {{}} outbox entries", count);
                    }}
                }}
                Err(e) => {{
                    error!("Outbox processing error: {{}}", e);
                }}
            }}

            tokio::time::sleep(Duration::from_millis(self.config.poll_interval_ms)).await;
        }}
    }}

    async fn process_batch(&self) -> Result<usize, crate::error::{pascal_name}Error> {{
        let entries = self.outbox.get_pending(self.config.batch_size).await?;
        let count = entries.len();

        for entry in entries {{
            if entry.retries >= self.config.max_retries as i32 {{
                warn!("Skipping entry {{}} after {{}} retries", entry.id, entry.retries);
                continue;
            }}

            match self.publisher
                .publish(&entry.event_type, &entry.id.to_string(), &entry.payload)
                .await
            {{
                Ok(_) => {{
                    self.outbox.mark_processed(entry.id).await?;
                }}
                Err(e) => {{
                    error!("Failed to publish entry {{}}: {{}}", entry.id, e);
                    self.outbox.mark_failed(entry.id).await?;
                }}
            }}
        }}

        Ok(count)
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/health.rs
pub fn infrastructure_health(_config: &ProjectConfig) -> String {
    r#"//! Health check server

use axum::{routing::get, Router};
use tracing::info;

/// Health check server for Kubernetes probes
pub struct HealthServer {
    port: u16,
}

impl HealthServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&self) -> Result<(), std::io::Error> {
        let app = Router::new()
            .route("/health", get(health))
            .route("/ready", get(ready));

        let addr = format!("0.0.0.0:{}", self.port);
        info!("Health server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(&addr).await?;
        axum::serve(listener, app).await
    }
}

async fn health() -> &'static str {
    "OK"
}

async fn ready() -> &'static str {
    "OK"
}
"#
    .to_string()
}

/// Generate presentation/mod.rs
pub fn presentation_mod(_config: &ProjectConfig) -> String {
    r#"//! Presentation layer (HTTP API)

pub mod handlers;

pub use handlers::*;
"#
    .to_string()
}

/// Generate presentation/handlers.rs
pub fn presentation_handlers(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&producer.service_name);
    let snake_name = producer.service_name.replace('-', "_");

    format!(
        r#"//! HTTP API handlers

use std::sync::Arc;
use axum::{{
    extract::{{Path, Query, State}},
    routing::{{delete, get, post, put}},
    Json, Router,
}};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::{pascal_name}Service;
use crate::domain::{{
    {pascal_name}Entity,
    Create{pascal_name}Request,
    Update{pascal_name}Request,
}};
use crate::error::{pascal_name}Error;

type AppState = Arc<{pascal_name}Service>;

/// Create the API router
pub fn create_router(service: Arc<{pascal_name}Service>) -> Router {{
    Router::new()
        .route("/{snake_name}s", post(create))
        .route("/{snake_name}s", get(list))
        .route("/{snake_name}s/:id", get(get_by_id))
        .route("/{snake_name}s/:id", put(update))
        .route("/{snake_name}s/:id", delete(delete_by_id))
        .with_state(service)
}}

#[derive(Debug, Deserialize)]
struct ListQuery {{
    limit: Option<i64>,
    offset: Option<i64>,
}}

async fn create(
    State(service): State<AppState>,
    Json(request): Json<Create{pascal_name}Request>,
) -> Result<Json<{pascal_name}Entity>, {pascal_name}Error> {{
    let entity = service.create(request).await?;
    Ok(Json(entity))
}}

async fn get_by_id(
    State(service): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<{pascal_name}Entity>, {pascal_name}Error> {{
    let entity = service.get(id).await?;
    Ok(Json(entity))
}}

async fn update(
    State(service): State<AppState>,
    Path(id): Path<Uuid>,
    Json(request): Json<Update{pascal_name}Request>,
) -> Result<Json<{pascal_name}Entity>, {pascal_name}Error> {{
    let entity = service.update(id, request).await?;
    Ok(Json(entity))
}}

async fn delete_by_id(
    State(service): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<(), {pascal_name}Error> {{
    service.delete(id).await
}}

async fn list(
    State(service): State<AppState>,
    Query(query): Query<ListQuery>,
) -> Result<Json<Vec<{pascal_name}Entity>>, {pascal_name}Error> {{
    let entities = service.list(
        query.limit.unwrap_or(20),
        query.offset.unwrap_or(0),
    ).await?;
    Ok(Json(entities))
}}
"#,
        pascal_name = pascal_name,
        snake_name = snake_name,
    )
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let producer = config.producer.as_ref().unwrap();
    let name = &config.name;

    format!(
        r#"# {display_name}

An event producer service built with AllFrame, featuring the transactional outbox pattern for reliable event publishing.

## Features

- **Transactional Outbox**: Reliable event publishing with exactly-once semantics
- **REST API**: Full CRUD operations for entities
- **PostgreSQL**: Persistent storage for entities and outbox
- **Kafka**: Event streaming to downstream consumers
- **Health Checks**: Kubernetes-ready liveness and readiness probes
- **OpenTelemetry**: Distributed tracing and metrics

## Prerequisites

- Rust 1.75+
- PostgreSQL 14+
- Kafka (or compatible broker)
- Docker (optional, for local development)

## Configuration

Set the following environment variables:

```bash
# Server
PORT=8080
HEALTH_PORT=8081

# Database
DATABASE_URL=postgres://localhost/{db_name}
DATABASE_MAX_CONNECTIONS=10

# Kafka
KAFKA_BROKERS=localhost:9092
KAFKA_TOPIC=events

# Outbox
OUTBOX_POLL_INTERVAL_MS=1000
OUTBOX_BATCH_SIZE=100
OUTBOX_MAX_RETRIES=3
```

## Database Setup

Run the following SQL to create the required tables:

```sql
CREATE TABLE {table_name} (
    id UUID PRIMARY KEY,
    name VARCHAR(255) NOT NULL,
    data JSONB NOT NULL DEFAULT '{{}}',
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

CREATE TABLE outbox (
    id UUID PRIMARY KEY,
    event_type VARCHAR(255) NOT NULL,
    payload TEXT NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    processed_at TIMESTAMPTZ,
    retries INTEGER NOT NULL DEFAULT 0
);

CREATE INDEX idx_outbox_pending ON outbox (created_at) WHERE processed_at IS NULL;
```

## Running

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/{name}
```

## API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| POST | /{route}s | Create entity |
| GET | /{route}s | List entities |
| GET | /{route}s/:id | Get entity by ID |
| PUT | /{route}s/:id | Update entity |
| DELETE | /{route}s/:id | Delete entity |

### Health Endpoints

| Endpoint | Description |
|----------|-------------|
| GET /health | Liveness probe |
| GET /ready | Readiness probe |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         API Server                               │
│  ┌──────────┐    ┌─────────────┐    ┌───────────────────────┐  │
│  │ Handlers │───▶│   Service   │───▶│      Repository       │  │
│  └──────────┘    └─────────────┘    └───────────────────────┘  │
│                         │                       │               │
│                         ▼                       ▼               │
│                  ┌─────────────┐         ┌──────────┐          │
│                  │   Outbox    │         │ Postgres │          │
│                  └─────────────┘         └──────────┘          │
└─────────────────────────────────────────────────────────────────┘
                          │
                          ▼
┌─────────────────────────────────────────────────────────────────┐
│                    Outbox Processor                              │
│  ┌─────────────┐    ┌────────────────┐    ┌─────────────────┐  │
│  │ Poll Outbox │───▶│ Event Publisher│───▶│      Kafka      │  │
│  └─────────────┘    └────────────────┘    └─────────────────┘  │
└─────────────────────────────────────────────────────────────────┘
```

## License

MIT
"#,
        display_name = producer.display_name,
        name = name,
        db_name = name.replace('-', "_"),
        table_name = producer.service_name.replace('-', "_"),
        route = producer.service_name.replace('_', "-"),
    )
}

/// Generate Dockerfile
pub fn dockerfile(config: &ProjectConfig) -> String {
    let name = &config.name;

    format!(
        r#"FROM rust:1.75-slim as builder

WORKDIR /app

# Install dependencies
RUN apt-get update && apt-get install -y \
    cmake \
    libssl-dev \
    pkg-config \
    && rm -rf /var/lib/apt/lists/*

# Copy manifests
COPY Cargo.toml Cargo.lock ./

# Create dummy main to cache dependencies
RUN mkdir src && echo "fn main() {{}}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source
COPY src ./src

# Build
RUN touch src/main.rs && cargo build --release

# Runtime image
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app

COPY --from=builder /app/target/release/{name} /app/

ENV PORT=8080
ENV HEALTH_PORT=8081
EXPOSE 8080 8081

CMD ["/app/{name}"]
"#,
        name = name,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("order_processor"), "OrderProcessor");
        assert_eq!(to_pascal_case("order-processor"), "OrderProcessor");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
