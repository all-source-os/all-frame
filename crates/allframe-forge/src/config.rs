//! Project configuration for code generation
//!
//! This module defines the configuration structures used by AllFrame Ignite
//! to generate different types of projects and archetypes.

use serde::{Deserialize, Serialize};
use std::time::Duration;

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
}

impl std::fmt::Display for Archetype {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Basic => write!(f, "basic"),
            Self::Gateway => write!(f, "gateway"),
            Self::EventSourced => write!(f, "event-sourced"),
            Self::Consumer => write!(f, "consumer"),
            Self::Producer => write!(f, "producer"),
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
        if archetype == Archetype::Gateway && self.gateway.is_none() {
            self.gateway = Some(GatewayConfig::default());
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

fn default_grpc_port() -> u16 {
    8080
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
            grpc_port: 8080,
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
        assert_eq!("cqrs".parse::<Archetype>().unwrap(), Archetype::EventSourced);
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
}
