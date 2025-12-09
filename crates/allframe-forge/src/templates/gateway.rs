//! Gateway archetype templates
//!
//! Templates for generating exchange gateway services with gRPC,
//! resilience patterns, caching, and observability.

use crate::config::{AuthMethod, CacheBackend, ProjectConfig};

/// Generate Cargo.toml for gateway project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let cache_deps = match gateway.cache.backend {
        CacheBackend::Redis => r#"redis = { version = "0.27", features = ["tokio-comp"] }"#,
        CacheBackend::Memory => r#"moka = { version = "0.12", features = ["future"] }"#,
        CacheBackend::None => "",
    };

    format!(
        r#"[package]
name = "{name}"
version = "0.1.0"
edition = "2024"
rust-version = "1.86"
description = "{display_name}"

[dependencies]
# AllFrame
allframe-core = {{ version = "0.1", features = ["resilience", "security", "otel"] }}

# gRPC
tonic = "0.12"
prost = "0.13"

# Async
tokio = {{ version = "1", features = ["full"] }}
async-trait = "0.1"

# HTTP Client
reqwest = {{ version = "0.12", features = ["json", "rustls-tls"] }}

# Caching
{cache_deps}

# Crypto (for API authentication)
hmac = "0.12"
sha2 = "0.10"
base64 = "0.22"
hex = "0.4"

# Data
rust_decimal = {{ version = "1.36", features = ["serde"] }}
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Errors
thiserror = "2.0"
anyhow = "1.0"

# Config
dotenvy = "0.15"

# Tracing
tracing = "0.1"
tracing-subscriber = {{ version = "0.3", features = ["env-filter"] }}

# Metrics
opentelemetry = {{ version = "0.27", features = ["metrics"] }}
opentelemetry-otlp = "0.27"

[dev-dependencies]
mockall = "0.13"
tokio-test = "0.4"

[build-dependencies]
tonic-build = "0.12"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = config.name,
        display_name = gateway.display_name,
        cache_deps = cache_deps,
    )
}

/// Generate build.rs for proto compilation
pub fn build_rs(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    format!(
        r#"fn main() -> Result<(), Box<dyn std::error::Error>> {{
    tonic_build::configure()
        .build_server(true)
        .build_client(false)
        .compile_protos(&["proto/{service_name}.proto"], &["proto/"])?;
    Ok(())
}}
"#,
        service_name = gateway.service_name
    )
}

/// Generate the proto file
pub fn proto_file(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let service_name = &gateway.service_name;
    let pascal_name = to_pascal_case(service_name);

    format!(
        r#"syntax = "proto3";
package {service_name};

// Authentication credentials
message Credentials {{
    string api_key = 1;
    string api_secret = 2;
}}

// ============ PUBLIC ENDPOINTS ============

message GetServerTimeRequest {{}}
message GetServerTimeResponse {{
    int64 server_time = 1;
}}

message GetAssetsRequest {{}}
message GetAssetsResponse {{
    map<string, AssetInfo> assets = 1;
}}

message AssetInfo {{
    string symbol = 1;
    string name = 2;
    int32 decimals = 3;
}}

message GetTickerRequest {{
    repeated string pairs = 1;
}}
message GetTickerResponse {{
    map<string, TickerInfo> tickers = 1;
}}

message TickerInfo {{
    string pair = 1;
    string last_price = 2;
    string bid = 3;
    string ask = 4;
    string volume_24h = 5;
}}

// ============ PRIVATE ENDPOINTS ============

message GetAccountBalanceRequest {{
    Credentials credentials = 1;
}}
message GetAccountBalanceResponse {{
    map<string, string> balances = 1;
}}

message GetTradesHistoryRequest {{
    Credentials credentials = 1;
    optional string start_time = 2;
    optional string end_time = 3;
    optional int32 limit = 4;
}}
message GetTradesHistoryResponse {{
    repeated TradeInfo trades = 1;
}}

message TradeInfo {{
    string id = 1;
    string pair = 2;
    string side = 3;
    string price = 4;
    string volume = 5;
    string fee = 6;
    int64 timestamp = 7;
}}

// ============ ORDER MANAGEMENT ============

message AddOrderRequest {{
    Credentials credentials = 1;
    string pair = 2;
    string side = 3;
    string order_type = 4;
    string volume = 5;
    optional string price = 6;
}}
message AddOrderResponse {{
    string order_id = 1;
    string status = 2;
}}

message CancelOrderRequest {{
    Credentials credentials = 1;
    string order_id = 2;
}}
message CancelOrderResponse {{
    bool success = 1;
}}

// ============ HEALTH ============

message HealthCheckRequest {{}}
message HealthCheckResponse {{
    bool healthy = 1;
    string status = 2;
}}

// ============ SERVICE DEFINITION ============

service {pascal_name}Service {{
    // Public
    rpc GetServerTime(GetServerTimeRequest) returns (GetServerTimeResponse);
    rpc GetAssets(GetAssetsRequest) returns (GetAssetsResponse);
    rpc GetTicker(GetTickerRequest) returns (GetTickerResponse);

    // Private
    rpc GetAccountBalance(GetAccountBalanceRequest) returns (GetAccountBalanceResponse);
    rpc GetTradesHistory(GetTradesHistoryRequest) returns (GetTradesHistoryResponse);

    // Orders
    rpc AddOrder(AddOrderRequest) returns (AddOrderResponse);
    rpc CancelOrder(CancelOrderRequest) returns (CancelOrderResponse);

    // Health
    rpc HealthCheck(HealthCheckRequest) returns (HealthCheckResponse);
}}
"#,
        service_name = service_name,
        pascal_name = pascal_name,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let service_name = &gateway.service_name;
    let pascal_name = to_pascal_case(service_name);

    format!(
        r#"//! {display_name}
//!
//! A gRPC gateway service wrapping the {display_name} API with
//! built-in resilience, caching, and observability.

use std::sync::Arc;
use tonic::transport::Server;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

pub mod generated {{
    tonic::include_proto!("{service_name}");
}}

use config::Config;
use application::{pascal_name}Service;
use infrastructure::{{
    {pascal_name}Client,
    GatewayRateLimiter,
    GatewayMetrics,
}};
use presentation::{pascal_name}GrpcService;
use generated::{service_name}_service_server::{pascal_name}ServiceServer;

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
    info!("Starting {display_name} on port {{}}", config.server.grpc_port);

    // Initialize metrics
    let _metrics = Arc::new(GatewayMetrics::new());

    // Initialize rate limiter
    let _rate_limiter = Arc::new(GatewayRateLimiter::new(
        config.rate_limit.public_rps,
        config.rate_limit.private_rps,
        config.rate_limit.burst,
    ));

    // Create HTTP client
    let client = Arc::new({pascal_name}Client::new(
        &config.{service_name}.base_url,
        config.{service_name}.timeout,
    ));

    // Create service
    let service = Arc::new({pascal_name}Service::new(client));

    // Create gRPC service
    let grpc_service = {pascal_name}GrpcService::new(service);

    // Start gRPC server
    let addr = format!("0.0.0.0:{{}}", config.server.grpc_port).parse()?;

    info!("gRPC server listening on {{}}", addr);

    Server::builder()
        .add_service({pascal_name}ServiceServer::new(grpc_service))
        .serve_with_shutdown(addr, shutdown_signal())
        .await?;

    info!("Server shutdown complete");
    Ok(())
}}

async fn shutdown_signal() {{
    tokio::signal::ctrl_c()
        .await
        .expect("Failed to listen for ctrl+c");
    info!("Shutdown signal received");
}}
"#,
        display_name = gateway.display_name,
        service_name = service_name,
        pascal_name = pascal_name,
    )
}

/// Generate lib.rs
pub fn lib_rs() -> String {
    r#"//! Gateway service library
//!
//! This module exports all the components of the gateway service.

pub mod config;
pub mod error;
pub mod domain;
pub mod application;
pub mod infrastructure;
pub mod presentation;
"#
    .to_string()
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let service_name = &gateway.service_name;
    let upper_name = service_name.to_uppercase();

    format!(
        r#"//! Configuration module
//!
//! Loads configuration from environment variables.

use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub {service_name}: {pascal_name}Config,
    pub rate_limit: RateLimitConfig,
    pub cache: CacheConfig,
}}

#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub grpc_port: u16,
    pub health_port: u16,
    pub metrics_port: u16,
}}

#[derive(Debug, Clone)]
pub struct {pascal_name}Config {{
    pub base_url: String,
    pub timeout: Duration,
}}

#[derive(Debug, Clone)]
pub struct RateLimitConfig {{
    pub public_rps: u32,
    pub private_rps: u32,
    pub burst: u32,
}}

#[derive(Debug, Clone)]
pub struct CacheConfig {{
    pub enabled: bool,
    pub public_ttl: Duration,
    pub private_ttl: Duration,
}}

impl Config {{
    pub fn from_env() -> Self {{
        Self {{
            server: ServerConfig {{
                grpc_port: std::env::var("{upper_name}_GATEWAY_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or({grpc_port}),
                health_port: std::env::var("{upper_name}_HEALTH_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or({health_port}),
                metrics_port: std::env::var("{upper_name}_METRICS_PORT")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or({metrics_port}),
            }},
            {service_name}: {pascal_name}Config {{
                base_url: std::env::var("{upper_name}_API_URL")
                    .unwrap_or_else(|_| "{api_base_url}".to_string()),
                timeout: Duration::from_secs(
                    std::env::var("{upper_name}_API_TIMEOUT_SECONDS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or(30),
                ),
            }},
            rate_limit: RateLimitConfig {{
                public_rps: std::env::var("{upper_name}_RATE_LIMIT_PUBLIC_RPS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or({public_rps}),
                private_rps: std::env::var("{upper_name}_RATE_LIMIT_PRIVATE_RPS")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or({private_rps}),
                burst: std::env::var("{upper_name}_RATE_LIMIT_BURST")
                    .ok()
                    .and_then(|s| s.parse().ok())
                    .unwrap_or({burst}),
            }},
            cache: CacheConfig {{
                enabled: std::env::var("CACHE_ENABLED")
                    .map(|s| s.to_lowercase() == "true")
                    .unwrap_or(true),
                public_ttl: Duration::from_secs(
                    std::env::var("CACHE_PUBLIC_TTL_SECONDS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or({public_ttl}),
                ),
                private_ttl: Duration::from_secs(
                    std::env::var("CACHE_PRIVATE_TTL_SECONDS")
                        .ok()
                        .and_then(|s| s.parse().ok())
                        .unwrap_or({private_ttl}),
                ),
            }},
        }}
    }}
}}
"#,
        service_name = service_name,
        pascal_name = to_pascal_case(service_name),
        upper_name = upper_name,
        grpc_port = gateway.server.grpc_port,
        health_port = gateway.server.health_port,
        metrics_port = gateway.server.metrics_port,
        api_base_url = gateway.api_base_url,
        public_rps = gateway.rate_limit.public_rps,
        private_rps = gateway.rate_limit.private_rps,
        burst = gateway.rate_limit.burst,
        public_ttl = gateway.cache.public_ttl_secs,
        private_ttl = gateway.cache.private_ttl_secs,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Error types for the gateway service

use thiserror::Error;
use tonic::Status;

#[derive(Debug, Error)]
pub enum {pascal_name}Error {{
    #[error("HTTP request failed: {{0}}")]
    HttpError(String),

    #[error("Missing credentials")]
    MissingCredentials,

    #[error("Invalid credentials")]
    InvalidCredentials,

    #[error("Rate limit exceeded")]
    RateLimitExceeded,

    #[error("{pascal_name} API error: {{0}}")]
    ApiError(String),

    #[error("Invalid request: {{0}}")]
    InvalidRequest(String),

    #[error("Asset not found: {{0}}")]
    AssetNotFound(String),

    #[error("Insufficient balance")]
    InsufficientBalance,

    #[error("Service unavailable")]
    ServiceUnavailable,

    #[error("Internal error: {{0}}")]
    Internal(String),
}}

impl From<{pascal_name}Error> for Status {{
    fn from(err: {pascal_name}Error) -> Self {{
        match err {{
            {pascal_name}Error::HttpError(msg) => Status::internal(msg),
            {pascal_name}Error::MissingCredentials => Status::unauthenticated("Missing credentials"),
            {pascal_name}Error::InvalidCredentials => Status::unauthenticated("Invalid credentials"),
            {pascal_name}Error::RateLimitExceeded => Status::resource_exhausted("Rate limit exceeded"),
            {pascal_name}Error::ApiError(msg) => Status::internal(msg),
            {pascal_name}Error::InvalidRequest(msg) => Status::invalid_argument(msg),
            {pascal_name}Error::AssetNotFound(asset) => Status::not_found(format!("Asset not found: {{}}", asset)),
            {pascal_name}Error::InsufficientBalance => Status::failed_precondition("Insufficient balance"),
            {pascal_name}Error::ServiceUnavailable => Status::unavailable("Service unavailable"),
            {pascal_name}Error::Internal(msg) => Status::internal(msg),
        }}
    }}
}}

pub type Result<T> = std::result::Result<T, {pascal_name}Error>;
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/mod.rs
pub fn domain_mod(_config: &ProjectConfig) -> String {
    r#"//! Domain layer - Business entities and repository traits

pub mod entities;
pub mod repository;

pub use entities::*;
pub use repository::*;
"#
    .to_string()
}

/// Generate domain/entities.rs
pub fn domain_entities(_config: &ProjectConfig) -> String {
    r#"//! Domain entities

use rust_decimal::Decimal;
use serde::{Deserialize, Serialize};

/// Asset information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssetInfo {
    pub symbol: String,
    pub name: String,
    pub decimals: i32,
}

/// Ticker information for a trading pair
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TickerInfo {
    pub pair: String,
    pub last_price: Decimal,
    pub bid: Decimal,
    pub ask: Decimal,
    pub volume_24h: Decimal,
}

/// Account balance for an asset
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Balance {
    pub asset: String,
    pub free: Decimal,
    pub locked: Decimal,
}

impl Balance {
    pub fn total(&self) -> Decimal {
        self.free + self.locked
    }
}

/// Trade history entry
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TradeInfo {
    pub id: String,
    pub pair: String,
    pub side: OrderSide,
    pub price: Decimal,
    pub volume: Decimal,
    pub fee: Decimal,
    pub timestamp: i64,
}

/// Order information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrderInfo {
    pub id: String,
    pub pair: String,
    pub side: OrderSide,
    pub order_type: OrderType,
    pub price: Option<Decimal>,
    pub volume: Decimal,
    pub status: OrderStatus,
}

/// Order side (buy or sell)
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderSide {
    Buy,
    Sell,
}

impl std::fmt::Display for OrderSide {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Buy => write!(f, "buy"),
            Self::Sell => write!(f, "sell"),
        }
    }
}

impl std::str::FromStr for OrderSide {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "buy" => Ok(Self::Buy),
            "sell" => Ok(Self::Sell),
            _ => Err(format!("Invalid order side: {}", s)),
        }
    }
}

/// Order type
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderType {
    Market,
    Limit,
}

impl std::fmt::Display for OrderType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Market => write!(f, "market"),
            Self::Limit => write!(f, "limit"),
        }
    }
}

impl std::str::FromStr for OrderType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "market" => Ok(Self::Market),
            "limit" => Ok(Self::Limit),
            _ => Err(format!("Invalid order type: {}", s)),
        }
    }
}

/// Order status
#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum OrderStatus {
    Open,
    Filled,
    Cancelled,
    PartiallyFilled,
}

/// Credentials for authenticated requests
#[derive(Debug, Clone)]
pub struct Credentials {
    pub api_key: String,
    pub api_secret: String,
}
"#
    .to_string()
}

/// Generate domain/repository.rs
pub fn domain_repository(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Repository trait definitions

use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::domain::entities::*;
use crate::error::Result;

/// Repository trait for {pascal_name} operations
#[async_trait]
pub trait {pascal_name}Repository: Send + Sync {{
    // ============ PUBLIC ENDPOINTS ============

    /// Get server time
    async fn get_server_time(&self) -> Result<i64>;

    /// Get all available assets
    async fn get_assets(&self) -> Result<Vec<AssetInfo>>;

    /// Get ticker information for pairs
    async fn get_ticker(&self, pairs: &[String]) -> Result<Vec<TickerInfo>>;

    // ============ PRIVATE ENDPOINTS ============

    /// Get account balance
    async fn get_account_balance(&self, creds: &Credentials) -> Result<Vec<Balance>>;

    /// Get trade history
    async fn get_trades_history(
        &self,
        creds: &Credentials,
        start: Option<i64>,
        end: Option<i64>,
        limit: Option<i32>,
    ) -> Result<Vec<TradeInfo>>;

    // ============ ORDER MANAGEMENT ============

    /// Place a new order
    async fn add_order(
        &self,
        creds: &Credentials,
        pair: &str,
        side: OrderSide,
        order_type: OrderType,
        volume: Decimal,
        price: Option<Decimal>,
    ) -> Result<OrderInfo>;

    /// Cancel an order
    async fn cancel_order(&self, creds: &Credentials, order_id: &str) -> Result<bool>;
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer - Business logic orchestration

pub mod service;

pub use service::*;
"#
    .to_string()
}

/// Generate application/service.rs
pub fn application_service(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Application services

use std::sync::Arc;
use async_trait::async_trait;
use rust_decimal::Decimal;

use crate::domain::entities::*;
use crate::infrastructure::{pascal_name}Client;
use crate::error::Result;

/// Service trait for {pascal_name} operations
#[async_trait]
pub trait {pascal_name}ServiceTrait: Send + Sync {{
    async fn get_server_time(&self) -> Result<i64>;
    async fn get_assets(&self) -> Result<Vec<AssetInfo>>;
    async fn get_ticker(&self, pairs: &[String]) -> Result<Vec<TickerInfo>>;
    async fn get_account_balance(&self, creds: &Credentials) -> Result<Vec<Balance>>;
    async fn get_trades_history(
        &self,
        creds: &Credentials,
        start: Option<i64>,
        end: Option<i64>,
        limit: Option<i32>,
    ) -> Result<Vec<TradeInfo>>;
    async fn add_order(
        &self,
        creds: &Credentials,
        pair: &str,
        side: OrderSide,
        order_type: OrderType,
        volume: Decimal,
        price: Option<Decimal>,
    ) -> Result<OrderInfo>;
    async fn cancel_order(&self, creds: &Credentials, order_id: &str) -> Result<bool>;
}}

/// {pascal_name} service implementation
pub struct {pascal_name}Service {{
    client: Arc<{pascal_name}Client>,
}}

impl {pascal_name}Service {{
    /// Create a new service with the given HTTP client
    pub fn new(client: Arc<{pascal_name}Client>) -> Self {{
        Self {{ client }}
    }}
}}

#[async_trait]
impl {pascal_name}ServiceTrait for {pascal_name}Service {{
    async fn get_server_time(&self) -> Result<i64> {{
        // TODO: Implement actual API call
        // let response: ServerTimeResponse = self.client.query_public("/api/time", &[]).await?;
        // Ok(response.server_time)
        Ok(std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs() as i64)
    }}

    async fn get_assets(&self) -> Result<Vec<AssetInfo>> {{
        // TODO: Implement actual API call
        let _ = &self.client;
        Ok(vec![])
    }}

    async fn get_ticker(&self, pairs: &[String]) -> Result<Vec<TickerInfo>> {{
        // TODO: Implement actual API call
        let _ = (&self.client, pairs);
        Ok(vec![])
    }}

    async fn get_account_balance(&self, creds: &Credentials) -> Result<Vec<Balance>> {{
        // TODO: Implement actual API call
        let _ = (&self.client, creds);
        Ok(vec![])
    }}

    async fn get_trades_history(
        &self,
        creds: &Credentials,
        start: Option<i64>,
        end: Option<i64>,
        limit: Option<i32>,
    ) -> Result<Vec<TradeInfo>> {{
        // TODO: Implement actual API call
        let _ = (&self.client, creds, start, end, limit);
        Ok(vec![])
    }}

    async fn add_order(
        &self,
        creds: &Credentials,
        pair: &str,
        side: OrderSide,
        order_type: OrderType,
        volume: Decimal,
        price: Option<Decimal>,
    ) -> Result<OrderInfo> {{
        // TODO: Implement actual API call
        let _ = &self.client;
        Ok(OrderInfo {{
            id: "placeholder".to_string(),
            pair: pair.to_string(),
            side,
            order_type,
            price,
            volume,
            status: OrderStatus::Open,
        }})
    }}

    async fn cancel_order(&self, creds: &Credentials, order_id: &str) -> Result<bool> {{
        // TODO: Implement actual API call
        let _ = (&self.client, creds, order_id);
        Ok(true)
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Infrastructure layer - External implementations

mod http_client;
mod auth;
mod cache;
mod rate_limiter;

pub use http_client::{pascal_name}Client;
pub use auth::*;
pub use cache::CachedRepository;
pub use rate_limiter::{{GatewayRateLimiter, GatewayMetrics}};
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/http_client.rs
pub fn infrastructure_http_client(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    let auth_impl = match gateway.auth_method {
        AuthMethod::HmacSha256 => hmac_sha256_auth(&pascal_name),
        AuthMethod::HmacSha512Base64 => hmac_sha512_base64_auth(&pascal_name),
        AuthMethod::ApiKey => api_key_auth(&pascal_name),
        _ => no_auth(&pascal_name),
    };

    format!(
        r#"//! HTTP client for {pascal_name} API

use reqwest::Client;
use serde::de::DeserializeOwned;
use std::time::Duration;
use tracing::{{debug, instrument}};

use crate::error::{{Result, {pascal_name}Error}};

/// HTTP client for {pascal_name} API
pub struct {pascal_name}Client {{
    client: Client,
    base_url: String,
}}

impl {pascal_name}Client {{
    /// Create a new client
    pub fn new(base_url: &str, timeout: Duration) -> Self {{
        let client = Client::builder()
            .timeout(timeout)
            .build()
            .expect("Failed to create HTTP client");

        Self {{
            client,
            base_url: base_url.to_string(),
        }}
    }}

    /// Make a public API request (no authentication)
    #[instrument(skip(self))]
    pub async fn query_public<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {{
        let url = format!("{{}}{{}}", self.base_url, endpoint);
        debug!("Public request to {{}}", url);

        let response = self.client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::ApiError(format!("{{}} - {{}}", status, text)));
        }}

        response
            .json()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))
    }}

    /// Make a private API request (with authentication)
    #[instrument(skip(self, api_secret))]
    pub async fn query_private<T: DeserializeOwned>(
        &self,
        endpoint: &str,
        api_key: &str,
        api_secret: &str,
        params: &[(&str, &str)],
    ) -> Result<T> {{
        let url = format!("{{}}{{}}", self.base_url, endpoint);
        debug!("Private request to {{}}", url);

        {auth_impl}
    }}
}}
"#,
        pascal_name = pascal_name,
        auth_impl = auth_impl,
    )
}

fn hmac_sha256_auth(pascal_name: &str) -> String {
    format!(
        r#"use hmac::{{Hmac, Mac}};
        use sha2::Sha256;

        let timestamp = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let query_string = params
            .iter()
            .map(|(k, v)| format!("{{}}={{}}", k, v))
            .collect::<Vec<_>>()
            .join("&");

        let sign_payload = format!("{{}}&timestamp={{}}", query_string, timestamp);

        let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
            .expect("HMAC can take key of any size");
        mac.update(sign_payload.as_bytes());
        let signature = hex::encode(mac.finalize().into_bytes());

        let response = self.client
            .get(&url)
            .query(params)
            .query(&[("timestamp", timestamp.as_str()), ("signature", signature.as_str())])
            .header("X-API-KEY", api_key)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::ApiError(format!("{{}} - {{}}", status, text)));
        }}

        response
            .json()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))"#,
        pascal_name = pascal_name
    )
}

fn hmac_sha512_base64_auth(pascal_name: &str) -> String {
    format!(
        r#"use hmac::{{Hmac, Mac}};
        use sha2::{{Sha256, Sha512, Digest}};
        use base64::{{Engine, engine::general_purpose}};

        let nonce = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_millis()
            .to_string();

        let data = params
            .iter()
            .map(|(k, v)| format!("{{}}={{}}", k, v))
            .chain(std::iter::once(format!("nonce={{}}", nonce)))
            .collect::<Vec<_>>()
            .join("&");

        // SHA256 hash of nonce + data
        let sha256_hash = Sha256::digest(format!("{{}}{{}}", nonce, data).as_bytes());

        // Concatenate path + sha256 hash
        let hmac_input = [endpoint.as_bytes(), &sha256_hash[..]].concat();

        // HMAC-SHA512 with base64-decoded secret
        let secret_decoded = general_purpose::STANDARD
            .decode(api_secret)
            .map_err(|_| {pascal_name}Error::InvalidCredentials)?;

        let mut mac = Hmac::<Sha512>::new_from_slice(&secret_decoded)
            .expect("HMAC can take key of any size");
        mac.update(&hmac_input);
        let signature = general_purpose::STANDARD.encode(mac.finalize().into_bytes());

        let response = self.client
            .post(&url)
            .header("API-Key", api_key)
            .header("API-Sign", signature)
            .form(&[("nonce", nonce.as_str())].into_iter().chain(params.iter().copied()).collect::<Vec<_>>())
            .send()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::ApiError(format!("{{}} - {{}}", status, text)));
        }}

        response
            .json()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))"#,
        pascal_name = pascal_name
    )
}

fn api_key_auth(pascal_name: &str) -> String {
    format!(
        r#"let response = self.client
            .get(&url)
            .query(params)
            .header("X-API-KEY", api_key)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::ApiError(format!("{{}} - {{}}", status, text)));
        }}

        response
            .json()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))"#,
        pascal_name = pascal_name
    )
}

fn no_auth(pascal_name: &str) -> String {
    format!(
        r#"let response = self.client
            .get(&url)
            .query(params)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let text = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::ApiError(format!("{{}} - {{}}", status, text)));
        }}

        response
            .json()
            .await
            .map_err(|e| {pascal_name}Error::HttpError(e.to_string()))"#,
        pascal_name = pascal_name
    )
}

/// Generate infrastructure/auth.rs
pub fn infrastructure_auth(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Authentication utilities for {pascal_name} API

use hmac::{{Hmac, Mac}};
use sha2::{{Sha256, Sha512, Digest}};
use base64::{{Engine, engine::general_purpose}};

use crate::error::{pascal_name}Error;

/// Sign a request using HMAC-SHA256
pub fn sign_hmac_sha256(
    api_secret: &str,
    message: &str,
) -> String {{
    let mut mac = Hmac::<Sha256>::new_from_slice(api_secret.as_bytes())
        .expect("HMAC can take key of any size");
    mac.update(message.as_bytes());
    hex::encode(mac.finalize().into_bytes())
}}

/// Sign a request using HMAC-SHA512 with Base64 encoding
pub fn sign_hmac_sha512_base64(
    api_secret: &str,
    path: &str,
    nonce: &str,
    post_data: &str,
) -> Result<String, {pascal_name}Error> {{
    // SHA256 hash of nonce + post_data
    let sha256_hash = Sha256::digest(format!("{{}}{{}}", nonce, post_data).as_bytes());

    // Concatenate path + sha256 hash
    let hmac_input = [path.as_bytes(), &sha256_hash[..]].concat();

    // Decode base64 secret
    let secret_decoded = general_purpose::STANDARD
        .decode(api_secret)
        .map_err(|_| {pascal_name}Error::InvalidCredentials)?;

    // HMAC-SHA512
    let mut mac = Hmac::<Sha512>::new_from_slice(&secret_decoded)
        .expect("HMAC can take key of any size");
    mac.update(&hmac_input);

    Ok(general_purpose::STANDARD.encode(mac.finalize().into_bytes()))
}}

/// Generate a nonce (timestamp in milliseconds)
pub fn generate_nonce() -> String {{
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis()
        .to_string()
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/{service}_repository.rs (not used directly, kept for reference)
fn _infrastructure_repository(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Repository implementation

use std::sync::Arc;
use async_trait::async_trait;
use rust_decimal::Decimal;
use tracing::instrument;

use crate::domain::{{entities::*, repository::{pascal_name}Repository}};
use crate::error::Result;
use crate::infrastructure::{{
    {pascal_name}Client,
    GatewayRateLimiter,
    GatewayMetrics,
}};

/// Repository implementation using HTTP client
pub struct {pascal_name}RestRepository {{
    client: {pascal_name}Client,
    rate_limiter: Arc<GatewayRateLimiter>,
    metrics: Arc<GatewayMetrics>,
}}

impl {pascal_name}RestRepository {{
    pub fn new(
        client: {pascal_name}Client,
        rate_limiter: Arc<GatewayRateLimiter>,
        metrics: Arc<GatewayMetrics>,
    ) -> Self {{
        Self {{
            client,
            rate_limiter,
            metrics,
        }}
    }}
}}

#[async_trait]
impl {pascal_name}Repository for {pascal_name}RestRepository {{
    #[instrument(skip(self))]
    async fn get_server_time(&self) -> Result<i64> {{
        self.rate_limiter.wait_public().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        // let response: ServerTimeResponse = self.client.query_public("/api/time", &[]).await?;
        // Ok(response.server_time)

        Ok(chrono::Utc::now().timestamp())
    }}

    #[instrument(skip(self))]
    async fn get_assets(&self) -> Result<Vec<AssetInfo>> {{
        self.rate_limiter.wait_public().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        Ok(vec![])
    }}

    #[instrument(skip(self))]
    async fn get_ticker(&self, pairs: &[String]) -> Result<Vec<TickerInfo>> {{
        self.rate_limiter.wait_public().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        let _ = pairs;
        Ok(vec![])
    }}

    #[instrument(skip(self, creds))]
    async fn get_account_balance(&self, creds: &Credentials) -> Result<Vec<Balance>> {{
        self.rate_limiter.wait_private().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        let _ = creds;
        Ok(vec![])
    }}

    #[instrument(skip(self, creds))]
    async fn get_trades_history(
        &self,
        creds: &Credentials,
        start: Option<i64>,
        end: Option<i64>,
        limit: Option<i32>,
    ) -> Result<Vec<TradeInfo>> {{
        self.rate_limiter.wait_private().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        let _ = (creds, start, end, limit);
        Ok(vec![])
    }}

    #[instrument(skip(self, creds))]
    async fn add_order(
        &self,
        creds: &Credentials,
        pair: &str,
        side: OrderSide,
        order_type: OrderType,
        volume: Decimal,
        price: Option<Decimal>,
    ) -> Result<OrderInfo> {{
        self.rate_limiter.wait_private().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        Ok(OrderInfo {{
            id: "placeholder".to_string(),
            pair: pair.to_string(),
            side,
            order_type,
            price,
            volume,
            status: OrderStatus::Open,
        }})
    }}

    #[instrument(skip(self, creds))]
    async fn cancel_order(&self, creds: &Credentials, order_id: &str) -> Result<bool> {{
        self.rate_limiter.wait_private().await;
        self.metrics.requests_total.increment(1);

        // TODO: Implement actual API call
        let _ = (creds, order_id);
        Ok(true)
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/rate_limiter.rs (includes metrics)
pub fn infrastructure_rate_limiter(_config: &ProjectConfig) -> String {
    r#"//! Rate limiting and metrics for API calls

use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;

/// Rate limiter for gateway API calls
pub struct GatewayRateLimiter {
    public_semaphore: Semaphore,
    private_semaphore: Semaphore,
    public_interval: Duration,
    private_interval: Duration,
    last_public: AtomicU64,
    last_private: AtomicU64,
}

impl GatewayRateLimiter {
    pub fn new(public_rps: u32, private_rps: u32, burst: u32) -> Self {
        Self {
            public_semaphore: Semaphore::new(burst as usize),
            private_semaphore: Semaphore::new(burst as usize),
            public_interval: Duration::from_millis(1000 / public_rps.max(1) as u64),
            private_interval: Duration::from_millis(1000 / private_rps.max(1) as u64),
            last_public: AtomicU64::new(0),
            last_private: AtomicU64::new(0),
        }
    }

    /// Wait for public rate limit
    pub async fn wait_public(&self) {
        let _permit = self.public_semaphore.acquire().await.unwrap();
        self.wait_interval(&self.last_public, self.public_interval).await;
    }

    /// Wait for private rate limit
    pub async fn wait_private(&self) {
        let _permit = self.private_semaphore.acquire().await.unwrap();
        self.wait_interval(&self.last_private, self.private_interval).await;
    }

    async fn wait_interval(&self, last: &AtomicU64, interval: Duration) {
        let now = Instant::now().elapsed().as_millis() as u64;
        let last_time = last.load(Ordering::Relaxed);
        let elapsed = now.saturating_sub(last_time);

        if elapsed < interval.as_millis() as u64 {
            let wait_time = interval.as_millis() as u64 - elapsed;
            tokio::time::sleep(Duration::from_millis(wait_time)).await;
        }

        last.store(Instant::now().elapsed().as_millis() as u64, Ordering::Relaxed);
    }
}

/// Metrics collector for gateway operations
pub struct GatewayMetrics {
    pub requests_total: Counter,
    pub cache_hits: Counter,
    pub cache_misses: Counter,
    pub errors_total: Counter,
}

impl GatewayMetrics {
    pub fn new() -> Self {
        Self {
            requests_total: Counter::new(),
            cache_hits: Counter::new(),
            cache_misses: Counter::new(),
            errors_total: Counter::new(),
        }
    }
}

impl Default for GatewayMetrics {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple atomic counter
pub struct Counter {
    value: AtomicU64,
}

impl Counter {
    pub fn new() -> Self {
        Self {
            value: AtomicU64::new(0),
        }
    }

    pub fn increment(&self, n: u64) {
        self.value.fetch_add(n, Ordering::Relaxed);
    }

    pub fn get(&self) -> u64 {
        self.value.load(Ordering::Relaxed)
    }
}

impl Default for Counter {
    fn default() -> Self {
        Self::new()
    }
}
"#
    .to_string()
}

/// Generate infrastructure/cache.rs
pub fn infrastructure_cache(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let pascal_name = to_pascal_case(&gateway.service_name);

    format!(
        r#"//! Caching decorator for repository

use std::sync::Arc;
use std::time::Duration;
use async_trait::async_trait;
use moka::future::Cache;
use rust_decimal::Decimal;
use tracing::debug;

use crate::domain::{{entities::*, repository::{pascal_name}Repository}};
use crate::error::Result;

/// Caching decorator for repository
pub struct CachedRepository {{
    inner: Arc<dyn {pascal_name}Repository>,
    asset_cache: Cache<String, Vec<AssetInfo>>,
    ticker_cache: Cache<String, Vec<TickerInfo>>,
    public_ttl: Duration,
    private_ttl: Duration,
}}

impl CachedRepository {{
    pub fn new(
        inner: Arc<dyn {pascal_name}Repository>,
        public_ttl: Duration,
        private_ttl: Duration,
    ) -> Self {{
        Self {{
            inner,
            asset_cache: Cache::builder()
                .time_to_live(public_ttl)
                .max_capacity(100)
                .build(),
            ticker_cache: Cache::builder()
                .time_to_live(public_ttl)
                .max_capacity(1000)
                .build(),
            public_ttl,
            private_ttl,
        }}
    }}
}}

#[async_trait]
impl {pascal_name}Repository for CachedRepository {{
    async fn get_server_time(&self) -> Result<i64> {{
        // Server time should not be cached
        self.inner.get_server_time().await
    }}

    async fn get_assets(&self) -> Result<Vec<AssetInfo>> {{
        let cache_key = "assets".to_string();

        if let Some(cached) = self.asset_cache.get(&cache_key).await {{
            debug!("Cache hit for assets");
            return Ok(cached);
        }}

        debug!("Cache miss for assets");
        let result = self.inner.get_assets().await?;
        self.asset_cache.insert(cache_key, result.clone()).await;
        Ok(result)
    }}

    async fn get_ticker(&self, pairs: &[String]) -> Result<Vec<TickerInfo>> {{
        let cache_key = pairs.join(",");

        if let Some(cached) = self.ticker_cache.get(&cache_key).await {{
            debug!("Cache hit for ticker");
            return Ok(cached);
        }}

        debug!("Cache miss for ticker");
        let result = self.inner.get_ticker(pairs).await?;
        self.ticker_cache.insert(cache_key, result.clone()).await;
        Ok(result)
    }}

    // Private methods are not cached by default
    async fn get_account_balance(&self, creds: &Credentials) -> Result<Vec<Balance>> {{
        self.inner.get_account_balance(creds).await
    }}

    async fn get_trades_history(
        &self,
        creds: &Credentials,
        start: Option<i64>,
        end: Option<i64>,
        limit: Option<i32>,
    ) -> Result<Vec<TradeInfo>> {{
        self.inner.get_trades_history(creds, start, end, limit).await
    }}

    async fn add_order(
        &self,
        creds: &Credentials,
        pair: &str,
        side: OrderSide,
        order_type: OrderType,
        volume: Decimal,
        price: Option<Decimal>,
    ) -> Result<OrderInfo> {{
        self.inner.add_order(creds, pair, side, order_type, volume, price).await
    }}

    async fn cancel_order(&self, creds: &Credentials, order_id: &str) -> Result<bool> {{
        self.inner.cancel_order(creds, order_id).await
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate presentation/mod.rs
pub fn presentation_mod(_config: &ProjectConfig) -> String {
    r#"//! Presentation layer - gRPC service implementation

pub mod grpc;

pub use grpc::*;
"#
    .to_string()
}

/// Generate presentation/grpc.rs
pub fn presentation_grpc(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let service_name = &gateway.service_name;
    let pascal_name = to_pascal_case(service_name);

    format!(
        r#"//! gRPC service implementation

use std::sync::Arc;
use tonic::{{Request, Response, Status}};
use tracing::instrument;

use crate::application::{pascal_name}ServiceTrait;
use crate::domain::entities::{{Credentials, OrderSide, OrderType}};
use crate::generated::{{
    {service_name}_service_server::{pascal_name}Service as GrpcServiceTrait,
    *,
}};

/// gRPC service implementation
pub struct {pascal_name}GrpcService {{
    service: Arc<dyn {pascal_name}ServiceTrait>,
}}

impl {pascal_name}GrpcService {{
    pub fn new(service: Arc<dyn {pascal_name}ServiceTrait>) -> Self {{
        Self {{ service }}
    }}

    fn extract_credentials(creds: Option<crate::generated::Credentials>) -> Result<Credentials, Status> {{
        let c = creds.ok_or_else(|| Status::unauthenticated("Missing credentials"))?;
        Ok(Credentials {{
            api_key: c.api_key,
            api_secret: c.api_secret,
        }})
    }}
}}

#[tonic::async_trait]
impl GrpcServiceTrait for {pascal_name}GrpcService {{
    #[instrument(skip(self))]
    async fn get_server_time(
        &self,
        _request: Request<GetServerTimeRequest>,
    ) -> Result<Response<GetServerTimeResponse>, Status> {{
        let time = self.service.get_server_time().await.map_err(Status::from)?;
        Ok(Response::new(GetServerTimeResponse {{ server_time: time }}))
    }}

    #[instrument(skip(self))]
    async fn get_assets(
        &self,
        _request: Request<GetAssetsRequest>,
    ) -> Result<Response<GetAssetsResponse>, Status> {{
        let assets = self.service.get_assets().await.map_err(Status::from)?;

        let assets_map = assets
            .into_iter()
            .map(|a| {{
                (
                    a.symbol.clone(),
                    AssetInfo {{
                        symbol: a.symbol,
                        name: a.name,
                        decimals: a.decimals,
                    }},
                )
            }})
            .collect();

        Ok(Response::new(GetAssetsResponse {{ assets: assets_map }}))
    }}

    #[instrument(skip(self))]
    async fn get_ticker(
        &self,
        request: Request<GetTickerRequest>,
    ) -> Result<Response<GetTickerResponse>, Status> {{
        let pairs = request.into_inner().pairs;
        let tickers = self.service.get_ticker(&pairs).await.map_err(Status::from)?;

        let tickers_map = tickers
            .into_iter()
            .map(|t| {{
                (
                    t.pair.clone(),
                    crate::generated::TickerInfo {{
                        pair: t.pair,
                        last_price: t.last_price.to_string(),
                        bid: t.bid.to_string(),
                        ask: t.ask.to_string(),
                        volume_24h: t.volume_24h.to_string(),
                    }},
                )
            }})
            .collect();

        Ok(Response::new(GetTickerResponse {{ tickers: tickers_map }}))
    }}

    #[instrument(skip(self))]
    async fn get_account_balance(
        &self,
        request: Request<GetAccountBalanceRequest>,
    ) -> Result<Response<GetAccountBalanceResponse>, Status> {{
        let creds = Self::extract_credentials(request.into_inner().credentials)?;
        let balances = self.service.get_account_balance(&creds).await.map_err(Status::from)?;

        let balances_map = balances
            .into_iter()
            .map(|b| (b.asset, b.free.to_string()))
            .collect();

        Ok(Response::new(GetAccountBalanceResponse {{ balances: balances_map }}))
    }}

    #[instrument(skip(self))]
    async fn get_trades_history(
        &self,
        request: Request<GetTradesHistoryRequest>,
    ) -> Result<Response<GetTradesHistoryResponse>, Status> {{
        let req = request.into_inner();
        let creds = Self::extract_credentials(req.credentials)?;

        let start = req.start_time.and_then(|s| s.parse().ok());
        let end = req.end_time.and_then(|s| s.parse().ok());
        let limit = req.limit;

        let trades = self.service
            .get_trades_history(&creds, start, end, limit)
            .await
            .map_err(Status::from)?;

        let trades_proto = trades
            .into_iter()
            .map(|t| crate::generated::TradeInfo {{
                id: t.id,
                pair: t.pair,
                side: t.side.to_string(),
                price: t.price.to_string(),
                volume: t.volume.to_string(),
                fee: t.fee.to_string(),
                timestamp: t.timestamp,
            }})
            .collect();

        Ok(Response::new(GetTradesHistoryResponse {{ trades: trades_proto }}))
    }}

    #[instrument(skip(self))]
    async fn add_order(
        &self,
        request: Request<AddOrderRequest>,
    ) -> Result<Response<AddOrderResponse>, Status> {{
        let req = request.into_inner();
        let creds = Self::extract_credentials(req.credentials)?;

        let side: OrderSide = req.side.parse()
            .map_err(|_| Status::invalid_argument("Invalid order side"))?;
        let order_type: OrderType = req.order_type.parse()
            .map_err(|_| Status::invalid_argument("Invalid order type"))?;
        let volume = req.volume.parse()
            .map_err(|_| Status::invalid_argument("Invalid volume"))?;
        let price = req.price.map(|p| p.parse()).transpose()
            .map_err(|_| Status::invalid_argument("Invalid price"))?;

        let order = self.service
            .add_order(&creds, &req.pair, side, order_type, volume, price)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(AddOrderResponse {{
            order_id: order.id,
            status: format!("{{:?}}", order.status),
        }}))
    }}

    #[instrument(skip(self))]
    async fn cancel_order(
        &self,
        request: Request<CancelOrderRequest>,
    ) -> Result<Response<CancelOrderResponse>, Status> {{
        let req = request.into_inner();
        let creds = Self::extract_credentials(req.credentials)?;

        let success = self.service
            .cancel_order(&creds, &req.order_id)
            .await
            .map_err(Status::from)?;

        Ok(Response::new(CancelOrderResponse {{ success }}))
    }}

    #[instrument(skip(self))]
    async fn health_check(
        &self,
        _request: Request<HealthCheckRequest>,
    ) -> Result<Response<HealthCheckResponse>, Status> {{
        // Simple health check - try to get server time
        match self.service.get_server_time().await {{
            Ok(_) => Ok(Response::new(HealthCheckResponse {{
                healthy: true,
                status: "healthy".to_string(),
            }})),
            Err(e) => Ok(Response::new(HealthCheckResponse {{
                healthy: false,
                status: format!("unhealthy: {{}}", e),
            }})),
        }}
    }}
}}
"#,
        service_name = service_name,
        pascal_name = pascal_name,
    )
}

/// Generate .env.example
pub fn env_example(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();
    let upper_name = gateway.service_name.to_uppercase();

    format!(
        r#"# Server Configuration
{upper_name}_GATEWAY_PORT={grpc_port}
{upper_name}_HEALTH_PORT={health_port}
{upper_name}_METRICS_PORT={metrics_port}

# API Configuration
{upper_name}_API_URL={api_base_url}
{upper_name}_API_TIMEOUT_SECONDS=30

# Rate Limiting
{upper_name}_RATE_LIMIT_PUBLIC_RPS={public_rps}
{upper_name}_RATE_LIMIT_PRIVATE_RPS={private_rps}
{upper_name}_RATE_LIMIT_BURST={burst}

# Cache Configuration
CACHE_ENABLED=true
CACHE_PUBLIC_TTL_SECONDS={public_ttl}
CACHE_PRIVATE_TTL_SECONDS={private_ttl}

# Observability
RUST_LOG=info
"#,
        upper_name = upper_name,
        grpc_port = gateway.server.grpc_port,
        health_port = gateway.server.health_port,
        metrics_port = gateway.server.metrics_port,
        api_base_url = gateway.api_base_url,
        public_rps = gateway.rate_limit.public_rps,
        private_rps = gateway.rate_limit.private_rps,
        burst = gateway.rate_limit.burst,
        public_ttl = gateway.cache.public_ttl_secs,
        private_ttl = gateway.cache.private_ttl_secs,
    )
}

/// Generate Dockerfile
pub fn dockerfile(config: &ProjectConfig) -> String {
    format!(
        r#"FROM rust:1.86 AS builder
WORKDIR /app
COPY . .
RUN apt-get update && apt-get install -y protobuf-compiler
RUN cargo build --release

FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/{name} /usr/local/bin/
EXPOSE 8080 8081 9090
CMD ["{name}"]
"#,
        name = config.name,
    )
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let gateway = config.gateway.as_ref().unwrap();

    format!(
        r#"# {display_name}

A gRPC gateway service wrapping the {display_name} API with built-in resilience, caching, and observability.

## Features

- **gRPC API**: Full gRPC service with proto definitions
- **Rate Limiting**: Configurable rate limits for public and private endpoints
- **Caching**: In-memory caching with configurable TTLs
- **Resilience**: Built-in retry, circuit breaker patterns
- **Observability**: Tracing and metrics support

## Quick Start

```bash
# Copy environment template
cp .env.example .env

# Build and run
cargo run

# Or with Docker
docker build -t {name} .
docker run -p 8080:8080 -p 8081:8081 -p 9090:9090 {name}
```

## Configuration

See `.env.example` for all available configuration options.

## Ports

| Port | Purpose |
|------|---------|
| {grpc_port} | gRPC server |
| {health_port} | Health check |
| {metrics_port} | Prometheus metrics |

## Generated with AllFrame

This project was generated using [AllFrame](https://github.com/all-source-os/all-frame).

```bash
allframe ignite {name} --archetype gateway
```
"#,
        name = config.name,
        display_name = gateway.display_name,
        grpc_port = gateway.server.grpc_port,
        health_port = gateway.server.health_port,
        metrics_port = gateway.server.metrics_port,
    )
}

/// Generate .gitignore
pub fn gitignore() -> String {
    r#"/target
Cargo.lock
.env
*.log
"#
    .to_string()
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                Some(c) => c.to_uppercase().chain(chars).collect::<String>(),
                None => String::new(),
            }
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("kraken"), "Kraken");
        assert_eq!(to_pascal_case("my_exchange"), "MyExchange");
        assert_eq!(to_pascal_case("api_gateway_service"), "ApiGatewayService");
    }
}
