//! Consumer archetype templates
//!
//! Templates for generating event consumer services with Kafka,
//! idempotency, dead letter queues, and resilience patterns.

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

/// Generate Cargo.toml for consumer project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let name = &config.name;

    let broker_deps = match consumer.broker {
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

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

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

# Health checks
axum = "0.7"

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = name,
        display_name = consumer.display_name,
        broker_deps = broker_deps,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&consumer.service_name);

    format!(
        r#"//! {display_name}
//!
//! An event consumer service with idempotency, retry, and dead letter queue handling.

use std::sync::Arc;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;

use config::Config;
use application::{pascal_name}Consumer;
use infrastructure::{{
    KafkaMessageBroker,
    InMemoryIdempotencyStore,
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
    info!("Broker: {{}}", config.broker.brokers);
    info!("Topics: {{:?}}", config.topics);

    // Create idempotency store
    let idempotency_store = Arc::new(InMemoryIdempotencyStore::new());

    // Create message broker
    let broker = KafkaMessageBroker::new(&config.broker).await?;

    // Create consumer
    let consumer = {pascal_name}Consumer::new(
        broker,
        idempotency_store,
        config.retry.clone(),
        config.dlq.clone(),
    );

    // Start health server in background
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(config.server.health_port);
        health_server.run().await
    }});

    // Run consumer
    info!("Consumer started, waiting for messages...");
    consumer.run(&config.topics).await?;

    health_handle.abort();
    info!("Consumer shutdown complete");
    Ok(())
}}
"#,
        display_name = consumer.display_name,
        pascal_name = pascal_name,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let service_name = &consumer.service_name;
    let upper_name = service_name.to_uppercase().replace('-', "_");

    format!(
        r#"//! Configuration module
//!
//! Loads configuration from environment variables.

use std::time::Duration;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub broker: BrokerConfig,
    pub topics: Vec<String>,
    pub group_id: String,
    pub retry: RetryConfig,
    pub dlq: DlqConfig,
    pub server: ServerConfig,
}}

/// Message broker configuration
#[derive(Debug, Clone)]
pub struct BrokerConfig {{
    pub brokers: String,
    pub security_protocol: String,
    pub sasl_mechanism: Option<String>,
    pub sasl_username: Option<String>,
    pub sasl_password: Option<String>,
}}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {{
    pub max_attempts: u32,
    pub initial_backoff: Duration,
    pub max_backoff: Duration,
    pub multiplier: f64,
}}

/// Dead Letter Queue configuration
#[derive(Debug, Clone)]
pub struct DlqConfig {{
    pub enabled: bool,
    pub suffix: String,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub health_port: u16,
    pub metrics_port: u16,
}}

impl Config {{
    pub fn from_env() -> Self {{
        Self {{
            broker: BrokerConfig {{
                brokers: std::env::var("{upper_name}_BROKERS")
                    .unwrap_or_else(|_| "localhost:9092".to_string()),
                security_protocol: std::env::var("{upper_name}_SECURITY_PROTOCOL")
                    .unwrap_or_else(|_| "PLAINTEXT".to_string()),
                sasl_mechanism: std::env::var("{upper_name}_SASL_MECHANISM").ok(),
                sasl_username: std::env::var("{upper_name}_SASL_USERNAME").ok(),
                sasl_password: std::env::var("{upper_name}_SASL_PASSWORD").ok(),
            }},
            topics: std::env::var("{upper_name}_TOPICS")
                .unwrap_or_else(|_| "events".to_string())
                .split(',')
                .map(|s| s.trim().to_string())
                .collect(),
            group_id: std::env::var("{upper_name}_GROUP_ID")
                .unwrap_or_else(|_| "{service_name}-group".to_string()),
            retry: RetryConfig {{
                max_attempts: std::env::var("{upper_name}_RETRY_MAX_ATTEMPTS")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or({max_attempts}),
                initial_backoff: Duration::from_millis(
                    std::env::var("{upper_name}_RETRY_INITIAL_BACKOFF_MS")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or({initial_backoff_ms}),
                ),
                max_backoff: Duration::from_millis(
                    std::env::var("{upper_name}_RETRY_MAX_BACKOFF_MS")
                        .ok()
                        .and_then(|v| v.parse().ok())
                        .unwrap_or({max_backoff_ms}),
                ),
                multiplier: std::env::var("{upper_name}_RETRY_MULTIPLIER")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or({multiplier}),
            }},
            dlq: DlqConfig {{
                enabled: std::env::var("{upper_name}_DLQ_ENABLED")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or(true),
                suffix: std::env::var("{upper_name}_DLQ_SUFFIX")
                    .unwrap_or_else(|_| ".dlq".to_string()),
            }},
            server: ServerConfig {{
                health_port: std::env::var("{upper_name}_HEALTH_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or({health_port}),
                metrics_port: std::env::var("{upper_name}_METRICS_PORT")
                    .ok()
                    .and_then(|v| v.parse().ok())
                    .unwrap_or({metrics_port}),
            }},
        }}
    }}
}}
"#,
        upper_name = upper_name,
        service_name = service_name,
        max_attempts = consumer.retry.max_attempts,
        initial_backoff_ms = consumer.retry.initial_backoff_ms,
        max_backoff_ms = consumer.retry.max_backoff_ms,
        multiplier = consumer.retry.multiplier,
        health_port = consumer.server.health_port,
        metrics_port = consumer.server.metrics_port,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&consumer.service_name);

    format!(
        r#"//! Error types for the consumer service

use thiserror::Error;

/// Consumer error type
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Broker error: {{0}}")]
    BrokerError(String),

    #[error("Deserialization error: {{0}}")]
    DeserializationError(String),

    #[error("Handler error: {{0}}")]
    HandlerError(String),

    #[error("Idempotency error: {{0}}")]
    IdempotencyError(String),

    #[error("DLQ error: {{0}}")]
    DlqError(String),

    #[error("Configuration error: {{0}}")]
    ConfigError(String),
}}

/// Result type alias
pub type Result<T> = std::result::Result<T, {pascal_name}Error>;

impl From<{pascal_name}Error> for tonic::Status {{
    fn from(e: {pascal_name}Error) -> Self {{
        tonic::Status::internal(e.to_string())
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/mod.rs
pub fn domain_mod(_config: &ProjectConfig) -> String {
    r#"//! Domain layer - Events and handlers

pub mod events;
pub mod handlers;

pub use events::*;
pub use handlers::*;
"#
    .to_string()
}

/// Generate domain/events.rs
pub fn domain_events(_config: &ProjectConfig) -> String {
    r#"//! Domain events

use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc};
use uuid::Uuid;

/// Base event envelope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EventEnvelope<T> {
    /// Unique event ID (used for idempotency)
    pub id: Uuid,
    /// Event type name
    pub event_type: String,
    /// Event timestamp
    pub timestamp: DateTime<Utc>,
    /// Correlation ID for tracing
    pub correlation_id: Option<Uuid>,
    /// Event payload
    pub payload: T,
    /// Event metadata
    pub metadata: EventMetadata,
}

/// Event metadata
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct EventMetadata {
    /// Source service
    pub source: Option<String>,
    /// Event version
    pub version: Option<String>,
    /// Retry count
    pub retry_count: u32,
}

impl<T> EventEnvelope<T> {
    /// Create a new event envelope
    pub fn new(event_type: impl Into<String>, payload: T) -> Self {
        Self {
            id: Uuid::new_v4(),
            event_type: event_type.into(),
            timestamp: Utc::now(),
            correlation_id: None,
            payload,
            metadata: EventMetadata::default(),
        }
    }

    /// Set correlation ID
    pub fn with_correlation_id(mut self, id: Uuid) -> Self {
        self.correlation_id = Some(id);
        self
    }
}

/// Example domain event: User Created
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserCreated {
    pub user_id: Uuid,
    pub email: String,
    pub name: String,
}

/// Example domain event: Order Placed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderPlaced {
    pub order_id: Uuid,
    pub user_id: Uuid,
    pub total: f64,
    pub items: Vec<OrderItem>,
}

/// Order item
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderItem {
    pub product_id: Uuid,
    pub quantity: u32,
    pub price: f64,
}
"#
    .to_string()
}

/// Generate domain/handlers.rs
pub fn domain_handlers(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&consumer.service_name);

    format!(
        r#"//! Event handlers

use async_trait::async_trait;
use tracing::{{info, warn}};

use crate::domain::events::*;
use crate::error::Result;

/// Event handler trait
#[async_trait]
pub trait EventHandler<E>: Send + Sync {{
    /// Handle an event
    async fn handle(&self, event: EventEnvelope<E>) -> Result<()>;
}}

/// User created event handler
pub struct UserCreatedHandler;

#[async_trait]
impl EventHandler<UserCreated> for UserCreatedHandler {{
    async fn handle(&self, event: EventEnvelope<UserCreated>) -> Result<()> {{
        info!(
            user_id = %event.payload.user_id,
            email = %event.payload.email,
            "Processing UserCreated event"
        );

        // TODO: Implement your business logic here
        // Examples:
        // - Send welcome email
        // - Create user profile
        // - Initialize user preferences

        Ok(())
    }}
}}

/// Order placed event handler
pub struct OrderPlacedHandler;

#[async_trait]
impl EventHandler<OrderPlaced> for OrderPlacedHandler {{
    async fn handle(&self, event: EventEnvelope<OrderPlaced>) -> Result<()> {{
        info!(
            order_id = %event.payload.order_id,
            user_id = %event.payload.user_id,
            total = %event.payload.total,
            "Processing OrderPlaced event"
        );

        // TODO: Implement your business logic here
        // Examples:
        // - Reserve inventory
        // - Process payment
        // - Send confirmation email

        Ok(())
    }}
}}

/// Handler registry for routing events to handlers
pub struct {pascal_name}HandlerRegistry {{
    user_created_handler: UserCreatedHandler,
    order_placed_handler: OrderPlacedHandler,
}}

impl Default for {pascal_name}HandlerRegistry {{
    fn default() -> Self {{
        Self::new()
    }}
}}

impl {pascal_name}HandlerRegistry {{
    pub fn new() -> Self {{
        Self {{
            user_created_handler: UserCreatedHandler,
            order_placed_handler: OrderPlacedHandler,
        }}
    }}

    /// Route and handle an event based on its type
    pub async fn handle_event(&self, event_type: &str, payload: &[u8]) -> Result<()> {{
        match event_type {{
            "UserCreated" => {{
                let event: EventEnvelope<UserCreated> = serde_json::from_slice(payload)
                    .map_err(|e| crate::error::{pascal_name}Error::DeserializationError(e.to_string()))?;
                self.user_created_handler.handle(event).await
            }}
            "OrderPlaced" => {{
                let event: EventEnvelope<OrderPlaced> = serde_json::from_slice(payload)
                    .map_err(|e| crate::error::{pascal_name}Error::DeserializationError(e.to_string()))?;
                self.order_placed_handler.handle(event).await
            }}
            _ => {{
                warn!(event_type = %event_type, "Unknown event type, skipping");
                Ok(())
            }}
        }}
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer - Consumer orchestration

pub mod consumer;

pub use consumer::*;
"#
    .to_string()
}

/// Generate application/consumer.rs
pub fn application_consumer(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&consumer.service_name);

    format!(
        r#"//! Main consumer implementation

use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use tracing::{{info, warn, error, instrument}};

use crate::config::{{RetryConfig, DlqConfig}};
use crate::domain::{pascal_name}HandlerRegistry;
use crate::error::{{Result, {pascal_name}Error}};
use crate::infrastructure::{{MessageBroker, IdempotencyStore}};

/// Message from the broker
#[derive(Debug, Clone)]
pub struct Message {{
    /// Message key
    pub key: Option<String>,
    /// Message payload
    pub payload: Vec<u8>,
    /// Message headers
    pub headers: Vec<(String, String)>,
    /// Topic
    pub topic: String,
    /// Partition
    pub partition: i32,
    /// Offset
    pub offset: i64,
}}

impl Message {{
    /// Get the event ID from headers
    pub fn event_id(&self) -> Option<String> {{
        self.headers
            .iter()
            .find(|(k, _)| k == "event_id")
            .map(|(_, v)| v.clone())
    }}

    /// Get the event type from headers
    pub fn event_type(&self) -> Option<String> {{
        self.headers
            .iter()
            .find(|(k, _)| k == "event_type")
            .map(|(_, v)| v.clone())
    }}
}}

/// Main consumer service
pub struct {pascal_name}Consumer<B: MessageBroker, I: IdempotencyStore> {{
    broker: B,
    idempotency_store: Arc<I>,
    handler_registry: {pascal_name}HandlerRegistry,
    retry_config: RetryConfig,
    dlq_config: DlqConfig,
}}

impl<B: MessageBroker, I: IdempotencyStore> {pascal_name}Consumer<B, I> {{
    pub fn new(
        broker: B,
        idempotency_store: Arc<I>,
        retry_config: RetryConfig,
        dlq_config: DlqConfig,
    ) -> Self {{
        Self {{
            broker,
            idempotency_store,
            handler_registry: {pascal_name}HandlerRegistry::new(),
            retry_config,
            dlq_config,
        }}
    }}

    /// Run the consumer loop
    pub async fn run(&self, topics: &[String]) -> Result<()> {{
        info!("Subscribing to topics: {{:?}}", topics);
        self.broker.subscribe(topics).await?;

        loop {{
            match self.broker.poll(Duration::from_secs(1)).await {{
                Ok(Some(message)) => {{
                    if let Err(e) = self.process_message(message).await {{
                        error!("Error processing message: {{}}", e);
                    }}
                }}
                Ok(None) => {{
                    // No message available, continue polling
                }}
                Err(e) => {{
                    error!("Error polling for messages: {{}}", e);
                    tokio::time::sleep(Duration::from_secs(1)).await;
                }}
            }}
        }}
    }}

    #[instrument(skip(self, message), fields(topic = %message.topic, partition = %message.partition, offset = %message.offset))]
    async fn process_message(&self, message: Message) -> Result<()> {{
        let event_id = message.event_id().unwrap_or_else(|| {{
            format!("{{}}:{{}}:{{}}", message.topic, message.partition, message.offset)
        }});

        // Check idempotency
        if self.idempotency_store.exists(&event_id).await? {{
            info!(event_id = %event_id, "Event already processed, skipping");
            self.broker.commit(&message).await?;
            return Ok(());
        }}

        let event_type = message.event_type().unwrap_or_else(|| "Unknown".to_string());

        // Process with retry
        let result = self.process_with_retry(&event_type, &message.payload).await;

        match result {{
            Ok(()) => {{
                // Mark as processed
                self.idempotency_store.mark_processed(&event_id).await?;
                self.broker.commit(&message).await?;
                info!(event_id = %event_id, "Event processed successfully");
            }}
            Err(e) => {{
                error!(event_id = %event_id, error = %e, "Failed to process event");

                if self.dlq_config.enabled {{
                    // Send to DLQ
                    let dlq_topic = format!("{{}}{{}}", message.topic, self.dlq_config.suffix);
                    self.broker.send_to_dlq(&dlq_topic, &message).await?;
                    self.broker.commit(&message).await?;
                    warn!(event_id = %event_id, dlq_topic = %dlq_topic, "Event sent to DLQ");
                }} else {{
                    return Err(e);
                }}
            }}
        }}

        Ok(())
    }}

    async fn process_with_retry(&self, event_type: &str, payload: &[u8]) -> Result<()> {{
        let mut attempts = 0;
        let mut backoff = self.retry_config.initial_backoff;

        loop {{
            attempts += 1;

            match self.handler_registry.handle_event(event_type, payload).await {{
                Ok(()) => return Ok(()),
                Err(e) if attempts >= self.retry_config.max_attempts => {{
                    return Err(e);
                }}
                Err(e) => {{
                    warn!(
                        attempt = attempts,
                        max_attempts = self.retry_config.max_attempts,
                        error = %e,
                        "Handler failed, retrying"
                    );

                    tokio::time::sleep(backoff).await;
                    backoff = std::cmp::min(
                        Duration::from_secs_f64(backoff.as_secs_f64() * self.retry_config.multiplier),
                        self.retry_config.max_backoff,
                    );
                }}
            }}
        }}
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(_config: &ProjectConfig) -> String {
    r#"//! Infrastructure layer - Message brokers and stores

mod broker;
mod idempotency;
mod health;

pub use broker::*;
pub use idempotency::*;
pub use health::*;
"#
    .to_string()
}

/// Generate infrastructure/broker.rs
pub fn infrastructure_broker(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&consumer.service_name);

    format!(
        r#"//! Message broker implementations

use std::time::Duration;
use async_trait::async_trait;
use tracing::{{info, debug}};

use crate::config::BrokerConfig;
use crate::application::Message;
use crate::error::{{Result, {pascal_name}Error}};

/// Message broker trait
#[async_trait]
pub trait MessageBroker: Send + Sync {{
    /// Subscribe to topics
    async fn subscribe(&self, topics: &[String]) -> Result<()>;

    /// Poll for messages
    async fn poll(&self, timeout: Duration) -> Result<Option<Message>>;

    /// Commit message offset
    async fn commit(&self, message: &Message) -> Result<()>;

    /// Send message to DLQ
    async fn send_to_dlq(&self, dlq_topic: &str, message: &Message) -> Result<()>;
}}

/// Kafka message broker implementation
pub struct KafkaMessageBroker {{
    // In a real implementation, this would hold the rdkafka consumer
    _config: BrokerConfig,
}}

impl KafkaMessageBroker {{
    pub async fn new(config: &BrokerConfig) -> Result<Self> {{
        info!("Connecting to Kafka brokers: {{}}", config.brokers);

        // In a real implementation:
        // let consumer: StreamConsumer = ClientConfig::new()
        //     .set("bootstrap.servers", &config.brokers)
        //     .set("group.id", &group_id)
        //     .set("enable.partition.eof", "false")
        //     .set("session.timeout.ms", "6000")
        //     .set("enable.auto.commit", "false")
        //     .create()?;

        Ok(Self {{
            _config: config.clone(),
        }})
    }}
}}

#[async_trait]
impl MessageBroker for KafkaMessageBroker {{
    async fn subscribe(&self, topics: &[String]) -> Result<()> {{
        info!("Subscribing to topics: {{:?}}", topics);
        // In a real implementation:
        // self.consumer.subscribe(&topics.iter().map(|s| s.as_str()).collect::<Vec<_>>())?;
        Ok(())
    }}

    async fn poll(&self, timeout: Duration) -> Result<Option<Message>> {{
        debug!("Polling for messages with timeout: {{:?}}", timeout);

        // In a real implementation:
        // match self.consumer.poll(timeout) {{
        //     Some(Ok(msg)) => {{
        //         let headers = msg.headers()
        //             .map(|h| {{
        //                 (0..h.count())
        //                     .filter_map(|i| {{
        //                         h.get(i).map(|header| {{
        //                             (header.key.to_string(), String::from_utf8_lossy(header.value.unwrap_or_default()).to_string())
        //                         }})
        //                     }})
        //                     .collect()
        //             }})
        //             .unwrap_or_default();
        //
        //         Ok(Some(Message {{
        //             key: msg.key().map(|k| String::from_utf8_lossy(k).to_string()),
        //             payload: msg.payload().unwrap_or_default().to_vec(),
        //             headers,
        //             topic: msg.topic().to_string(),
        //             partition: msg.partition(),
        //             offset: msg.offset(),
        //         }}))
        //     }}
        //     Some(Err(e)) => Err({pascal_name}Error::BrokerError(e.to_string())),
        //     None => Ok(None),
        // }}

        // Placeholder: simulate no messages
        tokio::time::sleep(timeout).await;
        Ok(None)
    }}

    async fn commit(&self, message: &Message) -> Result<()> {{
        debug!(
            topic = %message.topic,
            partition = %message.partition,
            offset = %message.offset,
            "Committing offset"
        );
        // In a real implementation:
        // self.consumer.commit_message(&message, CommitMode::Async)?;
        Ok(())
    }}

    async fn send_to_dlq(&self, dlq_topic: &str, message: &Message) -> Result<()> {{
        info!(
            dlq_topic = %dlq_topic,
            original_topic = %message.topic,
            "Sending message to DLQ"
        );
        // In a real implementation, use a producer to send to DLQ
        Ok(())
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/idempotency.rs
pub fn infrastructure_idempotency(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();
    let pascal_name = to_pascal_case(&consumer.service_name);

    format!(
        r#"//! Idempotency store implementations

use std::collections::HashSet;
use std::sync::RwLock;
use async_trait::async_trait;

use crate::error::{{Result, {pascal_name}Error}};

/// Idempotency store trait
#[async_trait]
pub trait IdempotencyStore: Send + Sync {{
    /// Check if an event has already been processed
    async fn exists(&self, event_id: &str) -> Result<bool>;

    /// Mark an event as processed
    async fn mark_processed(&self, event_id: &str) -> Result<()>;
}}

/// In-memory idempotency store (for development/testing)
pub struct InMemoryIdempotencyStore {{
    processed: RwLock<HashSet<String>>,
}}

impl InMemoryIdempotencyStore {{
    pub fn new() -> Self {{
        Self {{
            processed: RwLock::new(HashSet::new()),
        }}
    }}
}}

impl Default for InMemoryIdempotencyStore {{
    fn default() -> Self {{
        Self::new()
    }}
}}

#[async_trait]
impl IdempotencyStore for InMemoryIdempotencyStore {{
    async fn exists(&self, event_id: &str) -> Result<bool> {{
        let guard = self.processed.read()
            .map_err(|e| {pascal_name}Error::IdempotencyError(e.to_string()))?;
        Ok(guard.contains(event_id))
    }}

    async fn mark_processed(&self, event_id: &str) -> Result<()> {{
        let mut guard = self.processed.write()
            .map_err(|e| {pascal_name}Error::IdempotencyError(e.to_string()))?;
        guard.insert(event_id.to_string());
        Ok(())
    }}
}}

// TODO: Add Redis implementation
// pub struct RedisIdempotencyStore {{ ... }}

// TODO: Add PostgreSQL implementation
// pub struct PostgresIdempotencyStore {{ ... }}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/health.rs
pub fn infrastructure_health(_config: &ProjectConfig) -> String {
    r#"//! Health check server

use std::net::SocketAddr;
use axum::{routing::get, Router, Json};
use serde::Serialize;
use tracing::info;

/// Health response
#[derive(Serialize)]
pub struct HealthResponse {
    pub status: String,
    pub service: String,
}

/// Health check server
pub struct HealthServer {
    port: u16,
}

impl HealthServer {
    pub fn new(port: u16) -> Self {
        Self { port }
    }

    pub async fn run(&self) -> anyhow::Result<()> {
        let app = Router::new()
            .route("/health", get(health_handler))
            .route("/ready", get(ready_handler));

        let addr: SocketAddr = ([0, 0, 0, 0], self.port).into();
        info!("Health server listening on {}", addr);

        let listener = tokio::net::TcpListener::bind(addr).await?;
        axum::serve(listener, app).await?;

        Ok(())
    }
}

async fn health_handler() -> Json<HealthResponse> {
    Json(HealthResponse {
        status: "healthy".to_string(),
        service: env!("CARGO_PKG_NAME").to_string(),
    })
}

async fn ready_handler() -> Json<HealthResponse> {
    // TODO: Check actual readiness (broker connection, etc.)
    Json(HealthResponse {
        status: "ready".to_string(),
        service: env!("CARGO_PKG_NAME").to_string(),
    })
}
"#
    .to_string()
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let consumer = config.consumer.as_ref().unwrap();

    format!(
        r#"# {display_name}

An event consumer service built with AllFrame.

## Features

- **{broker} Consumer**: Event-driven message processing
- **Idempotency**: Duplicate event detection
- **Dead Letter Queue**: Failed message handling
- **Retry with Backoff**: Configurable retry strategy
- **Health Checks**: Kubernetes-ready endpoints

## Configuration

Set the following environment variables:

```bash
# Broker configuration
{upper_name}_BROKERS=localhost:9092
{upper_name}_GROUP_ID={service_name}-group
{upper_name}_TOPICS=events

# Retry configuration
{upper_name}_RETRY_MAX_ATTEMPTS=3
{upper_name}_RETRY_INITIAL_BACKOFF_MS=100
{upper_name}_RETRY_MAX_BACKOFF_MS=10000

# DLQ configuration
{upper_name}_DLQ_ENABLED=true
{upper_name}_DLQ_SUFFIX=.dlq

# Server configuration
{upper_name}_HEALTH_PORT=8081
{upper_name}_METRICS_PORT=9090
```

## Running

```bash
cargo run
```

## Health Endpoints

- `GET /health` - Liveness check
- `GET /ready` - Readiness check

## Adding Event Handlers

1. Define your event type in `src/domain/events.rs`
2. Create a handler in `src/domain/handlers.rs`
3. Register it in the `HandlerRegistry`

## Generated by AllFrame Ignite
"#,
        display_name = consumer.display_name,
        broker = consumer.broker,
        service_name = consumer.service_name,
        upper_name = consumer.service_name.to_uppercase().replace('-', "_"),
    )
}

/// Generate Dockerfile
pub fn dockerfile(config: &ProjectConfig) -> String {
    let name = &config.name;

    format!(
        r#"# Build stage
FROM rust:1.86-slim as builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Runtime stage
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/{name} /app/{name}

ENV RUST_LOG=info

EXPOSE 8081

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
        assert_eq!(to_pascal_case("order-processor"), "OrderProcessor");
        assert_eq!(to_pascal_case("user_handler"), "UserHandler");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
