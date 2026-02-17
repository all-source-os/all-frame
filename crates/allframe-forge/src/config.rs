//! Project configuration for code generation
//!
//! This module defines the configuration structures used by AllFrame Ignite
//! to generate different types of projects and archetypes.

use std::time::Duration;

use serde::{Deserialize, Serialize};

/// Project archetype - defines the type of service to generate
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Archetype {
    /// Basic Clean Architecture project (default)
    #[default]
    Basic,
    /// API Gateway service with gRPC, resilience, and caching
    Gateway,
    /// Event-sourced CQRS service
    EventSourced,
    /// Consumer service (event handler)
    Consumer,
    /// Producer service (event publisher)
    Producer,
    /// Backend for Frontend - API aggregation layer
    Bff,
    /// Scheduled job service (cron jobs, periodic tasks)
    Scheduled,
    /// WebSocket gateway for real-time streaming
    WebSocketGateway,
    /// Saga orchestrator for distributed transactions
    SagaOrchestrator,
    /// Anti-corruption layer for legacy system integration
    AntiCorruptionLayer,
}

impl std::fmt::Display for Archetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic => write!(f, "basic"),
            Self::Gateway => write!(f, "gateway"),
            Self::EventSourced => write!(f, "event-sourced"),
            Self::Consumer => write!(f, "consumer"),
            Self::Producer => write!(f, "producer"),
            Self::Bff => write!(f, "bff"),
            Self::Scheduled => write!(f, "scheduled"),
            Self::WebSocketGateway => write!(f, "websocket-gateway"),
            Self::SagaOrchestrator => write!(f, "saga-orchestrator"),
            Self::AntiCorruptionLayer => write!(f, "anti-corruption-layer"),
        }
    }
}

impl std::str::FromStr for Archetype {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "basic" => Ok(Self::Basic),
            "gateway" => Ok(Self::Gateway),
            "event-sourced" | "eventsourced" | "cqrs" => Ok(Self::EventSourced),
            "consumer" => Ok(Self::Consumer),
            "producer" => Ok(Self::Producer),
            "bff" | "backend-for-frontend" => Ok(Self::Bff),
            "scheduled" | "cron" | "jobs" => Ok(Self::Scheduled),
            "websocket-gateway" | "websocket" | "ws" => Ok(Self::WebSocketGateway),
            "saga-orchestrator" | "saga" => Ok(Self::SagaOrchestrator),
            "anti-corruption-layer" | "acl" | "legacy-adapter" => Ok(Self::AntiCorruptionLayer),
            _ => Err(format!("Unknown archetype: {}", s)),
        }
    }
}

/// Protocol support for the service
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Protocol {
    /// REST API
    Rest,
    /// GraphQL API
    Graphql,
    /// gRPC API
    Grpc,
}

/// Authentication method for external APIs
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum AuthMethod {
    /// No authentication
    #[default]
    None,
    /// API Key in header
    ApiKey,
    /// HMAC-SHA256 signature
    HmacSha256,
    /// HMAC-SHA512 with Base64 encoding
    HmacSha512Base64,
    /// OAuth2
    OAuth2,
    /// JWT Bearer token
    Jwt,
}

/// Cache backend selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum CacheBackend {
    /// In-memory cache (moka)
    #[default]
    Memory,
    /// Redis cache
    Redis,
    /// No caching
    None,
}

/// Message broker type for event-driven services
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum MessageBroker {
    /// Apache Kafka
    #[default]
    Kafka,
    /// RabbitMQ
    RabbitMq,
    /// Redis Streams
    Redis,
    /// AWS SQS
    Sqs,
    /// Google Cloud Pub/Sub
    PubSub,
}

impl std::fmt::Display for MessageBroker {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Kafka => write!(f, "kafka"),
            Self::RabbitMq => write!(f, "rabbitmq"),
            Self::Redis => write!(f, "redis"),
            Self::Sqs => write!(f, "sqs"),
            Self::PubSub => write!(f, "pubsub"),
        }
    }
}

/// Main project configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectConfig {
    /// Project name (used for crate name, directories)
    pub name: String,
    /// Project archetype
    #[serde(default)]
    pub archetype: Archetype,
    /// Protocols to expose
    #[serde(default)]
    pub protocols: Vec<Protocol>,
    /// Enable OpenTelemetry tracing
    #[serde(default = "default_true")]
    pub tracing: bool,
    /// Enable Prometheus metrics
    #[serde(default = "default_true")]
    pub metrics: bool,
    /// Gateway-specific configuration
    #[serde(default)]
    pub gateway: Option<GatewayConfig>,
    /// Consumer-specific configuration
    #[serde(default)]
    pub consumer: Option<ConsumerConfig>,
    /// Producer-specific configuration
    #[serde(default)]
    pub producer: Option<ProducerConfig>,
    /// BFF-specific configuration
    #[serde(default)]
    pub bff: Option<BffConfig>,
    /// Scheduled jobs configuration
    #[serde(default)]
    pub scheduled: Option<ScheduledConfig>,
    /// WebSocket gateway configuration
    #[serde(default)]
    pub websocket_gateway: Option<WebSocketGatewayConfig>,
    /// Saga orchestrator configuration
    #[serde(default)]
    pub saga_orchestrator: Option<SagaOrchestratorConfig>,
    /// Anti-corruption layer configuration
    #[serde(default)]
    pub acl: Option<AntiCorruptionLayerConfig>,
}

fn default_true() -> bool {
    true
}

impl Default for ProjectConfig {
    fn default() -> Self {
        Self {
            name: String::new(),
            archetype: Archetype::default(),
            protocols: vec![Protocol::Grpc],
            tracing: true,
            metrics: true,
            gateway: None,
            consumer: None,
            producer: None,
            bff: None,
            scheduled: None,
            websocket_gateway: None,
            saga_orchestrator: None,
            acl: None,
        }
    }
}

impl ProjectConfig {
    /// Create a new project configuration
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            ..Default::default()
        }
    }

    /// Set the archetype
    pub fn with_archetype(mut self, archetype: Archetype) -> Self {
        self.archetype = archetype;
        match archetype {
            Archetype::Gateway if self.gateway.is_none() => {
                self.gateway = Some(GatewayConfig::default());
            }
            Archetype::Consumer if self.consumer.is_none() => {
                self.consumer = Some(ConsumerConfig::default());
            }
            Archetype::Producer if self.producer.is_none() => {
                self.producer = Some(ProducerConfig::default());
            }
            Archetype::Bff if self.bff.is_none() => {
                self.bff = Some(BffConfig::default());
            }
            Archetype::Scheduled if self.scheduled.is_none() => {
                self.scheduled = Some(ScheduledConfig::default());
            }
            Archetype::WebSocketGateway if self.websocket_gateway.is_none() => {
                self.websocket_gateway = Some(WebSocketGatewayConfig::default());
            }
            Archetype::SagaOrchestrator if self.saga_orchestrator.is_none() => {
                self.saga_orchestrator = Some(SagaOrchestratorConfig::default());
            }
            Archetype::AntiCorruptionLayer if self.acl.is_none() => {
                self.acl = Some(AntiCorruptionLayerConfig::default());
            }
            _ => {}
        }
        self
    }

    /// Set the protocols
    pub fn with_protocols(mut self, protocols: Vec<Protocol>) -> Self {
        self.protocols = protocols;
        self
    }

    /// Set gateway configuration
    pub fn with_gateway(mut self, gateway: GatewayConfig) -> Self {
        self.gateway = Some(gateway);
        self
    }

    /// Set consumer configuration
    pub fn with_consumer(mut self, consumer: ConsumerConfig) -> Self {
        self.consumer = Some(consumer);
        self
    }

    /// Set producer configuration
    pub fn with_producer(mut self, producer: ProducerConfig) -> Self {
        self.producer = Some(producer);
        self
    }

    /// Set BFF configuration
    pub fn with_bff(mut self, bff: BffConfig) -> Self {
        self.bff = Some(bff);
        self
    }

    /// Set scheduled jobs configuration
    pub fn with_scheduled(mut self, scheduled: ScheduledConfig) -> Self {
        self.scheduled = Some(scheduled);
        self
    }

    /// Set WebSocket gateway configuration
    pub fn with_websocket_gateway(mut self, ws: WebSocketGatewayConfig) -> Self {
        self.websocket_gateway = Some(ws);
        self
    }

    /// Set saga orchestrator configuration
    pub fn with_saga_orchestrator(mut self, saga: SagaOrchestratorConfig) -> Self {
        self.saga_orchestrator = Some(saga);
        self
    }

    /// Set anti-corruption layer configuration
    pub fn with_acl(mut self, acl: AntiCorruptionLayerConfig) -> Self {
        self.acl = Some(acl);
        self
    }
}

/// Gateway-specific configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GatewayConfig {
    /// Service name (e.g., "kraken", "binance")
    pub service_name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Base URL for the external API
    pub api_base_url: String,
    /// Authentication method
    #[serde(default)]
    pub auth_method: AuthMethod,
    /// Rate limiting configuration
    #[serde(default)]
    pub rate_limit: RateLimitConfig,
    /// Cache configuration
    #[serde(default)]
    pub cache: CacheConfig,
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
    /// Entity definitions for the domain
    #[serde(default)]
    pub entities: Vec<EntityConfig>,
    /// API endpoints to wrap
    #[serde(default)]
    pub endpoints: Vec<EndpointConfig>,
}

impl Default for GatewayConfig {
    fn default() -> Self {
        Self {
            service_name: "exchange".to_string(),
            display_name: "Exchange Gateway".to_string(),
            api_base_url: "https://api.example.com".to_string(),
            auth_method: AuthMethod::default(),
            rate_limit: RateLimitConfig::default(),
            cache: CacheConfig::default(),
            server: ServerConfig::default(),
            entities: vec![],
            endpoints: vec![],
        }
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Requests per second for public endpoints
    pub public_rps: u32,
    /// Requests per second for private endpoints
    pub private_rps: u32,
    /// Burst size
    pub burst: u32,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            public_rps: 10,
            private_rps: 2,
            burst: 5,
        }
    }
}

/// Cache configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    /// Enable caching
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Cache backend
    #[serde(default)]
    pub backend: CacheBackend,
    /// TTL for public data in seconds
    #[serde(default = "default_public_ttl")]
    pub public_ttl_secs: u64,
    /// TTL for private data in seconds
    #[serde(default = "default_private_ttl")]
    pub private_ttl_secs: u64,
}

fn default_public_ttl() -> u64 {
    300
}

fn default_private_ttl() -> u64 {
    60
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            backend: CacheBackend::Memory,
            public_ttl_secs: 300,
            private_ttl_secs: 60,
        }
    }
}

impl CacheConfig {
    /// Get public TTL as Duration
    pub fn public_ttl(&self) -> Duration {
        Duration::from_secs(self.public_ttl_secs)
    }

    /// Get private TTL as Duration
    pub fn private_ttl(&self) -> Duration {
        Duration::from_secs(self.private_ttl_secs)
    }
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// HTTP server port (for REST APIs)
    #[serde(default = "default_http_port")]
    pub http_port: u16,
    /// gRPC server port
    #[serde(default = "default_grpc_port")]
    pub grpc_port: u16,
    /// Health check port
    #[serde(default = "default_health_port")]
    pub health_port: u16,
    /// Metrics port
    #[serde(default = "default_metrics_port")]
    pub metrics_port: u16,
}

fn default_http_port() -> u16 {
    8080
}

fn default_grpc_port() -> u16 {
    50051
}

fn default_health_port() -> u16 {
    8081
}

fn default_metrics_port() -> u16 {
    9090
}

impl Default for ServerConfig {
    fn default() -> Self {
        Self {
            http_port: 8080,
            grpc_port: 50051,
            health_port: 8081,
            metrics_port: 9090,
        }
    }
}

/// Entity definition for domain model generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConfig {
    /// Entity name (e.g., "Asset", "Balance", "Trade")
    pub name: String,
    /// Fields for the entity
    pub fields: Vec<FieldConfig>,
}

/// Field definition for entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConfig {
    /// Field name
    pub name: String,
    /// Field type (Rust type)
    pub field_type: String,
    /// Is this field optional?
    #[serde(default)]
    pub optional: bool,
}

/// API endpoint configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EndpointConfig {
    /// Endpoint name (used for method names)
    pub name: String,
    /// HTTP method (GET, POST, etc.)
    pub method: String,
    /// Endpoint path
    pub path: String,
    /// Is this a private endpoint requiring auth?
    #[serde(default)]
    pub private: bool,
    /// Request parameters
    #[serde(default)]
    pub params: Vec<ParamConfig>,
    /// Response type
    pub response_type: String,
    /// Should responses be cached?
    #[serde(default)]
    pub cacheable: bool,
}

/// Parameter configuration for endpoints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParamConfig {
    /// Parameter name
    pub name: String,
    /// Parameter type
    pub param_type: String,
    /// Is this parameter required?
    #[serde(default = "default_true")]
    pub required: bool,
}

/// Consumer-specific configuration for event handler services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsumerConfig {
    /// Service name (e.g., "order-processor", "notification-handler")
    pub service_name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Message broker type
    #[serde(default)]
    pub broker: MessageBroker,
    /// Topics/queues to consume from
    pub topics: Vec<TopicConfig>,
    /// Consumer group ID
    pub group_id: String,
    /// Dead Letter Queue configuration
    #[serde(default)]
    pub dlq: DlqConfig,
    /// Retry configuration
    #[serde(default)]
    pub retry: RetryConfig,
    /// Idempotency configuration
    #[serde(default)]
    pub idempotency: IdempotencyConfig,
    /// Server configuration (health, metrics)
    #[serde(default)]
    pub server: ServerConfig,
}

impl Default for ConsumerConfig {
    fn default() -> Self {
        Self {
            service_name: "consumer".to_string(),
            display_name: "Event Consumer".to_string(),
            broker: MessageBroker::default(),
            topics: vec![TopicConfig::default()],
            group_id: "consumer-group".to_string(),
            dlq: DlqConfig::default(),
            retry: RetryConfig::default(),
            idempotency: IdempotencyConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

/// Topic/queue configuration for consumers
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopicConfig {
    /// Topic/queue name
    pub name: String,
    /// Event type expected on this topic
    pub event_type: String,
    /// Number of partitions (for Kafka)
    #[serde(default = "default_partitions")]
    pub partitions: u32,
    /// Whether to auto-commit offsets
    #[serde(default)]
    pub auto_commit: bool,
}

fn default_partitions() -> u32 {
    1
}

impl Default for TopicConfig {
    fn default() -> Self {
        Self {
            name: "events".to_string(),
            event_type: "Event".to_string(),
            partitions: 1,
            auto_commit: false,
        }
    }
}

/// Dead Letter Queue configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DlqConfig {
    /// Enable DLQ
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// DLQ topic/queue name suffix
    #[serde(default = "default_dlq_suffix")]
    pub suffix: String,
    /// Max retries before sending to DLQ
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_dlq_suffix() -> String {
    ".dlq".to_string()
}

fn default_max_retries() -> u32 {
    3
}

impl Default for DlqConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            suffix: ".dlq".to_string(),
            max_retries: 3,
        }
    }
}

/// Retry configuration for message processing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    #[serde(default = "default_max_retries")]
    pub max_attempts: u32,
    /// Initial backoff delay in milliseconds
    #[serde(default = "default_initial_backoff")]
    pub initial_backoff_ms: u64,
    /// Maximum backoff delay in milliseconds
    #[serde(default = "default_max_backoff")]
    pub max_backoff_ms: u64,
    /// Backoff multiplier
    #[serde(default = "default_backoff_multiplier")]
    pub multiplier: f64,
}

fn default_initial_backoff() -> u64 {
    100
}

fn default_max_backoff() -> u64 {
    10000
}

fn default_backoff_multiplier() -> f64 {
    2.0
}

impl Default for RetryConfig {
    fn default() -> Self {
        Self {
            max_attempts: 3,
            initial_backoff_ms: 100,
            max_backoff_ms: 10000,
            multiplier: 2.0,
        }
    }
}

/// Idempotency configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdempotencyConfig {
    /// Enable idempotency checking
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Storage backend for idempotency keys
    #[serde(default)]
    pub storage: IdempotencyStorage,
    /// TTL for idempotency keys in seconds
    #[serde(default = "default_idempotency_ttl")]
    pub ttl_secs: u64,
}

fn default_idempotency_ttl() -> u64 {
    86400 // 24 hours
}

impl Default for IdempotencyConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            storage: IdempotencyStorage::default(),
            ttl_secs: 86400,
        }
    }
}

/// Storage backend for idempotency keys
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum IdempotencyStorage {
    /// In-memory storage (for development)
    #[default]
    Memory,
    /// Redis
    Redis,
    /// PostgreSQL
    Postgres,
}

/// Producer-specific configuration for event publishing services
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProducerConfig {
    /// Service name
    pub service_name: String,
    /// Human-readable display name
    pub display_name: String,
    /// Message broker type
    #[serde(default)]
    pub broker: MessageBroker,
    /// Topics to publish to
    pub topics: Vec<TopicConfig>,
    /// Outbox pattern configuration
    #[serde(default)]
    pub outbox: OutboxConfig,
    /// Server configuration
    #[serde(default)]
    pub server: ServerConfig,
}

impl Default for ProducerConfig {
    fn default() -> Self {
        Self {
            service_name: "producer".to_string(),
            display_name: "Event Producer".to_string(),
            broker: MessageBroker::default(),
            topics: vec![TopicConfig::default()],
            outbox: OutboxConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

/// Outbox pattern configuration for reliable event publishing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OutboxConfig {
    /// Enable outbox pattern
    #[serde(default = "default_true")]
    pub enabled: bool,
    /// Outbox table name
    #[serde(default = "default_outbox_table")]
    pub table_name: String,
    /// Polling interval in milliseconds
    #[serde(default = "default_polling_interval")]
    pub polling_interval_ms: u64,
    /// Batch size for outbox processing
    #[serde(default = "default_batch_size")]
    pub batch_size: u32,
    /// Maximum retries before giving up on an outbox entry
    #[serde(default = "default_max_retries")]
    pub max_retries: u32,
}

fn default_outbox_table() -> String {
    "outbox".to_string()
}

fn default_polling_interval() -> u64 {
    1000
}

fn default_batch_size() -> u32 {
    100
}

impl Default for OutboxConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            table_name: "outbox".to_string(),
            polling_interval_ms: 1000,
            batch_size: 100,
            max_retries: 3,
        }
    }
}

/// BFF (Backend for Frontend) configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BffConfig {
    /// Service name (e.g., "web-bff", "mobile-bff")
    pub service_name: String,
    /// Display name for documentation
    pub display_name: String,
    /// Target frontend type
    pub frontend_type: FrontendType,
    /// Backend services to aggregate
    pub backends: Vec<BackendServiceConfig>,
    /// Enable GraphQL support
    pub graphql_enabled: bool,
    /// Enable REST API
    pub rest_enabled: bool,
    /// Cache configuration
    pub cache: CacheConfig,
    /// Server configuration
    pub server: ServerConfig,
}

/// Target frontend type for BFF
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FrontendType {
    /// Web browser frontend
    #[default]
    Web,
    /// Mobile app frontend (iOS/Android)
    Mobile,
    /// Desktop application frontend
    Desktop,
    /// Command-line interface
    Cli,
}

/// Backend service configuration for BFF
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackendServiceConfig {
    /// Service name
    pub name: String,
    /// Base URL of the backend service
    pub base_url: String,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
    /// Enable circuit breaker
    pub circuit_breaker: bool,
}

impl Default for BffConfig {
    fn default() -> Self {
        Self {
            service_name: "bff".to_string(),
            display_name: "Backend for Frontend".to_string(),
            frontend_type: FrontendType::Web,
            backends: vec![BackendServiceConfig {
                name: "api".to_string(),
                base_url: "http://localhost:8080".to_string(),
                timeout_ms: 5000,
                circuit_breaker: true,
            }],
            graphql_enabled: true,
            rest_enabled: true,
            cache: CacheConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

/// Scheduled job service configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScheduledConfig {
    /// Service name
    pub service_name: String,
    /// Display name for documentation
    pub display_name: String,
    /// Jobs to schedule
    pub jobs: Vec<JobConfig>,
    /// Server configuration (for health checks)
    pub server: ServerConfig,
}

/// Individual job configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JobConfig {
    /// Job name
    pub name: String,
    /// Cron expression (e.g., "0 0 * * *" for daily at midnight)
    pub cron: String,
    /// Job description
    pub description: String,
    /// Enable job by default
    pub enabled: bool,
    /// Timeout in seconds
    pub timeout_secs: u64,
}

impl Default for ScheduledConfig {
    fn default() -> Self {
        Self {
            service_name: "scheduler".to_string(),
            display_name: "Scheduled Jobs".to_string(),
            jobs: vec![JobConfig {
                name: "cleanup".to_string(),
                cron: "0 0 * * *".to_string(),
                description: "Daily cleanup job".to_string(),
                enabled: true,
                timeout_secs: 300,
            }],
            server: ServerConfig::default(),
        }
    }
}

/// WebSocket gateway configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WebSocketGatewayConfig {
    /// Service name
    pub service_name: String,
    /// Display name for documentation
    pub display_name: String,
    /// Channels/rooms configuration
    pub channels: Vec<ChannelConfig>,
    /// Maximum connections per client
    pub max_connections_per_client: u32,
    /// Heartbeat interval in seconds
    pub heartbeat_interval_secs: u64,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Server configuration
    pub server: ServerConfig,
}

/// Channel configuration for WebSocket
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChannelConfig {
    /// Channel name
    pub name: String,
    /// Channel description
    pub description: String,
    /// Require authentication
    pub authenticated: bool,
}

impl Default for WebSocketGatewayConfig {
    fn default() -> Self {
        Self {
            service_name: "ws_gateway".to_string(),
            display_name: "WebSocket Gateway".to_string(),
            channels: vec![ChannelConfig {
                name: "events".to_string(),
                description: "Real-time event stream".to_string(),
                authenticated: true,
            }],
            max_connections_per_client: 5,
            heartbeat_interval_secs: 30,
            connection_timeout_secs: 60,
            server: ServerConfig::default(),
        }
    }
}

/// Saga orchestrator configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaOrchestratorConfig {
    /// Service name
    pub service_name: String,
    /// Display name for documentation
    pub display_name: String,
    /// Saga definitions
    pub sagas: Vec<SagaDefinitionConfig>,
    /// Retry configuration
    pub retry: RetryConfig,
    /// Server configuration
    pub server: ServerConfig,
}

/// Saga definition configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaDefinitionConfig {
    /// Saga name
    pub name: String,
    /// Saga description
    pub description: String,
    /// Steps in the saga
    pub steps: Vec<String>,
}

impl Default for SagaOrchestratorConfig {
    fn default() -> Self {
        Self {
            service_name: "saga_orchestrator".to_string(),
            display_name: "Saga Orchestrator".to_string(),
            sagas: vec![SagaDefinitionConfig {
                name: "order_saga".to_string(),
                description: "Order processing saga".to_string(),
                steps: vec![
                    "validate_order".to_string(),
                    "reserve_inventory".to_string(),
                    "process_payment".to_string(),
                    "send_notification".to_string(),
                ],
            }],
            retry: RetryConfig::default(),
            server: ServerConfig::default(),
        }
    }
}

/// Anti-corruption layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntiCorruptionLayerConfig {
    /// Service name
    pub service_name: String,
    /// Display name for documentation
    pub display_name: String,
    /// Legacy system configuration
    pub legacy_system: LegacySystemConfig,
    /// Transformation rules
    pub transformations: Vec<TransformationConfig>,
    /// Server configuration
    pub server: ServerConfig,
}

/// Legacy system connection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacySystemConfig {
    /// System name
    pub name: String,
    /// Connection type (REST, SOAP, Database, etc.)
    pub connection_type: LegacyConnectionType,
    /// Base URL or connection string
    pub connection_string: String,
    /// Timeout in milliseconds
    pub timeout_ms: u64,
}

/// Legacy system connection type
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum LegacyConnectionType {
    /// REST API connection
    #[default]
    Rest,
    /// SOAP/XML web service
    Soap,
    /// Direct database connection
    Database,
    /// File-based integration
    File,
    /// Message queue integration
    Mq,
}

/// Transformation rule configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransformationConfig {
    /// Source entity name (legacy)
    pub source: String,
    /// Target entity name (modern)
    pub target: String,
    /// Description of the transformation
    pub description: String,
}

impl Default for AntiCorruptionLayerConfig {
    fn default() -> Self {
        Self {
            service_name: "acl".to_string(),
            display_name: "Anti-Corruption Layer".to_string(),
            legacy_system: LegacySystemConfig {
                name: "legacy_api".to_string(),
                connection_type: LegacyConnectionType::Rest,
                connection_string: "http://legacy-system:8080".to_string(),
                timeout_ms: 10000,
            },
            transformations: vec![TransformationConfig {
                source: "LegacyEntity".to_string(),
                target: "ModernEntity".to_string(),
                description: "Transform legacy entity to modern format".to_string(),
            }],
            server: ServerConfig::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_archetype_from_str() {
        assert_eq!("basic".parse::<Archetype>().unwrap(), Archetype::Basic);
        assert_eq!("gateway".parse::<Archetype>().unwrap(), Archetype::Gateway);
        assert_eq!(
            "event-sourced".parse::<Archetype>().unwrap(),
            Archetype::EventSourced
        );
        assert_eq!(
            "cqrs".parse::<Archetype>().unwrap(),
            Archetype::EventSourced
        );
        assert_eq!(
            "consumer".parse::<Archetype>().unwrap(),
            Archetype::Consumer
        );
        assert_eq!(
            "producer".parse::<Archetype>().unwrap(),
            Archetype::Producer
        );
    }

    #[test]
    fn test_project_config_default() {
        let config = ProjectConfig::new("test-project");
        assert_eq!(config.name, "test-project");
        assert_eq!(config.archetype, Archetype::Basic);
        assert!(config.tracing);
        assert!(config.metrics);
    }

    #[test]
    fn test_gateway_config_default() {
        let config = GatewayConfig::default();
        assert_eq!(config.rate_limit.public_rps, 10);
        assert_eq!(config.cache.public_ttl_secs, 300);
    }

    #[test]
    fn test_consumer_config_default() {
        let config = ConsumerConfig::default();
        assert_eq!(config.broker, MessageBroker::Kafka);
        assert!(config.dlq.enabled);
        assert!(config.idempotency.enabled);
        assert_eq!(config.retry.max_attempts, 3);
    }

    #[test]
    fn test_producer_config_default() {
        let config = ProducerConfig::default();
        assert_eq!(config.broker, MessageBroker::Kafka);
        assert!(config.outbox.enabled);
        assert_eq!(config.outbox.batch_size, 100);
    }

    #[test]
    fn test_with_archetype_consumer() {
        let config = ProjectConfig::new("test").with_archetype(Archetype::Consumer);
        assert!(config.consumer.is_some());
    }

    #[test]
    fn test_with_archetype_producer() {
        let config = ProjectConfig::new("test").with_archetype(Archetype::Producer);
        assert!(config.producer.is_some());
    }
}
