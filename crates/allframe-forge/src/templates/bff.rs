//! BFF (Backend for Frontend) archetype templates
//!
//! Templates for generating API aggregation services that combine
//! multiple backend services into a unified API for specific frontends.

use crate::config::ProjectConfig;

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

/// Generate Cargo.toml for BFF project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let name = &config.name;

    let graphql_deps = if bff.graphql_enabled {
        r#"
# GraphQL
async-graphql = { version = "7.0", features = ["dataloader", "uuid", "chrono"] }
async-graphql-axum = "7.0""#
    } else {
        ""
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

# Web Framework
axum = "0.7"
tower = "0.5"
tower-http = {{ version = "0.6", features = ["trace", "cors", "compression-gzip"] }}
{graphql_deps}

# HTTP Client
reqwest = {{ version = "0.12", features = ["json", "rustls-tls"] }}

# Async
tokio = {{ version = "1", features = ["full"] }}
async-trait = "0.1"
futures = "0.3"

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# Caching
moka = {{ version = "0.12", features = ["future"] }}

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
wiremock = "0.6"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = name,
        display_name = bff.display_name,
        graphql_deps = graphql_deps,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    let graphql_setup = if bff.graphql_enabled {
        r#"
    // Create GraphQL schema
    let schema = presentation::create_graphql_schema(aggregator.clone());
"#
    } else {
        ""
    };

    let graphql_routes = if bff.graphql_enabled {
        r#"
        .nest("/graphql", presentation::graphql_routes(schema))"#
    } else {
        ""
    };

    format!(
        r#"//! {display_name}
//!
//! A Backend for Frontend service that aggregates multiple backend APIs
//! into a unified interface optimized for {frontend_type:?} clients.

use std::sync::Arc;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use config::Config;
use application::{pascal_name}Aggregator;
use infrastructure::{{
    BackendClients,
    CacheService,
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
    info!("Backend services: {{:?}}", config.backends.keys().collect::<Vec<_>>());

    // Create cache service
    let cache = Arc::new(CacheService::new(&config.cache));

    // Create backend clients
    let clients = Arc::new(BackendClients::new(&config.backends).await?);

    // Create aggregator service
    let aggregator = Arc::new({pascal_name}Aggregator::new(
        clients.clone(),
        cache.clone(),
    ));
{graphql_setup}
    // Start health server in background
    let health_port = config.server.health_port;
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(health_port);
        health_server.run().await
    }});

    // Create router
    let app = presentation::create_router(aggregator.clone()){graphql_routes};

    // Start API server
    info!("Starting API server on port {{}}", config.server.port);
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{{}}", config.server.port)
    ).await?;
    axum::serve(listener, app).await?;

    health_handle.abort();
    info!("BFF shutdown complete");
    Ok(())
}}
"#,
        pascal_name = pascal_name,
        display_name = bff.display_name,
        frontend_type = bff.frontend_type,
        graphql_setup = graphql_setup,
        graphql_routes = graphql_routes,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();

    format!(
        r#"//! Service configuration

use std::collections::HashMap;
use std::env;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub backends: HashMap<String, BackendConfig>,
    pub cache: CacheConfig,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub port: u16,
    pub health_port: u16,
}}

/// Backend service configuration
#[derive(Debug, Clone)]
pub struct BackendConfig {{
    pub base_url: String,
    pub timeout_ms: u64,
    pub circuit_breaker: bool,
}}

/// Cache configuration
#[derive(Debug, Clone)]
pub struct CacheConfig {{
    pub max_capacity: u64,
    pub ttl_secs: u64,
}}

impl Config {{
    pub fn from_env() -> Self {{
        let mut backends = HashMap::new();

        // Default backend configuration
        backends.insert(
            "api".to_string(),
            BackendConfig {{
                base_url: env::var("API_BASE_URL")
                    .unwrap_or_else(|_| "{default_backend_url}".to_string()),
                timeout_ms: env::var("API_TIMEOUT_MS")
                    .unwrap_or_else(|_| "{default_timeout}".to_string())
                    .parse()
                    .expect("API_TIMEOUT_MS must be a number"),
                circuit_breaker: true,
            }},
        );

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
            backends,
            cache: CacheConfig {{
                max_capacity: env::var("CACHE_MAX_CAPACITY")
                    .unwrap_or_else(|_| "10000".to_string())
                    .parse()
                    .expect("CACHE_MAX_CAPACITY must be a number"),
                ttl_secs: env::var("CACHE_TTL_SECS")
                    .unwrap_or_else(|_| "{cache_ttl}".to_string())
                    .parse()
                    .expect("CACHE_TTL_SECS must be a number"),
            }},
        }}
    }}
}}
"#,
        port = bff.server.http_port,
        health_port = bff.server.health_port,
        default_backend_url = bff.backends.first().map(|b| b.base_url.as_str()).unwrap_or("http://localhost:8080"),
        default_timeout = bff.backends.first().map(|b| b.timeout_ms).unwrap_or(5000),
        cache_ttl = bff.cache.public_ttl_secs,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! Error types

use thiserror::Error;

/// BFF errors
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Backend service error: {{0}}")]
    BackendError(String),

    #[error("Aggregation error: {{0}}")]
    AggregationError(String),

    #[error("Validation error: {{0}}")]
    Validation(String),

    #[error("Not found: {{0}}")]
    NotFound(String),

    #[error("HTTP client error: {{0}}")]
    HttpClient(#[from] reqwest::Error),

    #[error("Serialization error: {{0}}")]
    Serialization(#[from] serde_json::Error),

    #[error("Internal error: {{0}}")]
    Internal(String),
}}

impl {pascal_name}Error {{
    pub fn status_code(&self) -> axum::http::StatusCode {{
        use axum::http::StatusCode;
        match self {{
            Self::BackendError(_) => StatusCode::BAD_GATEWAY,
            Self::AggregationError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::Validation(_) => StatusCode::BAD_REQUEST,
            Self::NotFound(_) => StatusCode::NOT_FOUND,
            Self::HttpClient(_) => StatusCode::BAD_GATEWAY,
            Self::Serialization(_) => StatusCode::BAD_REQUEST,
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

pub mod models;
pub mod aggregates;

pub use models::*;
pub use aggregates::*;
"#
    .to_string()
}

/// Generate domain/models.rs
pub fn domain_models(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! Domain models

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// User model (aggregated from user service)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct User {{
    pub id: Uuid,
    pub name: String,
    pub email: String,
    pub created_at: DateTime<Utc>,
}}

/// Resource model (example domain model)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {pascal_name}Resource {{
    pub id: Uuid,
    pub name: String,
    pub data: serde_json::Value,
    pub owner: Option<User>,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}}

/// API response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {{
    pub data: T,
    pub meta: Option<ResponseMeta>,
}}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {{
    pub total: Option<i64>,
    pub page: Option<i32>,
    pub per_page: Option<i32>,
}}

/// Paginated list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {{
    pub items: Vec<T>,
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/aggregates.rs
pub fn domain_aggregates(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! Aggregated views for frontend

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

use super::{{User, {pascal_name}Resource}};

/// Dashboard aggregate - combines multiple data sources
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardAggregate {{
    pub user: User,
    pub recent_resources: Vec<{pascal_name}Resource>,
    pub stats: DashboardStats,
    pub generated_at: DateTime<Utc>,
}}

/// Dashboard statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DashboardStats {{
    pub total_resources: i64,
    pub resources_this_week: i64,
    pub active_users: i64,
}}

/// Detail view aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResourceDetailAggregate {{
    pub resource: {pascal_name}Resource,
    pub owner: Option<User>,
    pub related_resources: Vec<{pascal_name}Resource>,
}}

/// Search results aggregate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SearchResultsAggregate {{
    pub resources: Vec<{pascal_name}Resource>,
    pub users: Vec<User>,
    pub total_results: i64,
    pub query: String,
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer

pub mod aggregator;

pub use aggregator::*;
"#
    .to_string()
}

/// Generate application/aggregator.rs
pub fn application_aggregator(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! Aggregator service

use std::sync::Arc;
use tracing::{{info, instrument}};
use uuid::Uuid;
use chrono::Utc;

use crate::domain::{{
    User,
    {pascal_name}Resource,
    DashboardAggregate,
    DashboardStats,
    ResourceDetailAggregate,
    SearchResultsAggregate,
    PaginatedResponse,
}};
use crate::infrastructure::{{BackendClients, CacheService}};
use crate::error::{pascal_name}Error;

/// Aggregator service that combines data from multiple backends
pub struct {pascal_name}Aggregator {{
    clients: Arc<BackendClients>,
    cache: Arc<CacheService>,
}}

impl {pascal_name}Aggregator {{
    pub fn new(clients: Arc<BackendClients>, cache: Arc<CacheService>) -> Self {{
        Self {{ clients, cache }}
    }}

    /// Get dashboard aggregate for a user
    #[instrument(skip(self))]
    pub async fn get_dashboard(&self, user_id: Uuid) -> Result<DashboardAggregate, {pascal_name}Error> {{
        // Check cache first
        let cache_key = format!("dashboard:{{}}", user_id);
        if let Some(cached) = self.cache.get::<DashboardAggregate>(&cache_key).await {{
            info!("Dashboard cache hit for user {{}}", user_id);
            return Ok(cached);
        }}

        // Fetch user
        let user = self.clients.get_user(user_id).await?;

        // Fetch recent resources in parallel
        let (resources_result, stats_result) = tokio::join!(
            self.clients.get_user_resources(user_id, 10),
            self.clients.get_stats()
        );

        let recent_resources = resources_result?;
        let stats = stats_result?;

        let aggregate = DashboardAggregate {{
            user,
            recent_resources,
            stats,
            generated_at: Utc::now(),
        }};

        // Cache the result
        self.cache.set(&cache_key, &aggregate).await;

        Ok(aggregate)
    }}

    /// Get resource detail with related data
    #[instrument(skip(self))]
    pub async fn get_resource_detail(&self, resource_id: Uuid) -> Result<ResourceDetailAggregate, {pascal_name}Error> {{
        let cache_key = format!("resource:{{}}", resource_id);
        if let Some(cached) = self.cache.get::<ResourceDetailAggregate>(&cache_key).await {{
            return Ok(cached);
        }}

        let resource = self.clients.get_resource(resource_id).await?;

        let (owner_result, related_result) = tokio::join!(
            async {{
                if let Some(ref owner) = resource.owner {{
                    self.clients.get_user(owner.id).await.ok()
                }} else {{
                    None
                }}
            }},
            self.clients.get_related_resources(resource_id, 5)
        );

        let aggregate = ResourceDetailAggregate {{
            resource,
            owner: owner_result,
            related_resources: related_result.unwrap_or_default(),
        }};

        self.cache.set(&cache_key, &aggregate).await;
        Ok(aggregate)
    }}

    /// Search across all resources
    #[instrument(skip(self))]
    pub async fn search(&self, query: &str) -> Result<SearchResultsAggregate, {pascal_name}Error> {{
        let (resources_result, users_result) = tokio::join!(
            self.clients.search_resources(query),
            self.clients.search_users(query)
        );

        let resources = resources_result?;
        let users = users_result?;
        let total_results = (resources.len() + users.len()) as i64;

        Ok(SearchResultsAggregate {{
            resources,
            users,
            total_results,
            query: query.to_string(),
        }})
    }}

    /// List resources with pagination
    #[instrument(skip(self))]
    pub async fn list_resources(
        &self,
        page: i32,
        per_page: i32,
    ) -> Result<PaginatedResponse<{pascal_name}Resource>, {pascal_name}Error> {{
        self.clients.list_resources(page, per_page).await
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(_config: &ProjectConfig) -> String {
    r#"//! Infrastructure layer

pub mod clients;
pub mod cache;
pub mod health;

pub use clients::*;
pub use cache::*;
pub use health::*;
"#
    .to_string()
}

/// Generate infrastructure/clients.rs
pub fn infrastructure_clients(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! Backend HTTP clients

use std::collections::HashMap;
use std::time::Duration;
use reqwest::Client;
use uuid::Uuid;

use crate::config::BackendConfig;
use crate::domain::{{
    User,
    {pascal_name}Resource,
    DashboardStats,
    PaginatedResponse,
}};
use crate::error::{pascal_name}Error;

/// Backend clients for all external services
pub struct BackendClients {{
    clients: HashMap<String, BackendClient>,
}}

/// Individual backend client
struct BackendClient {{
    client: Client,
    base_url: String,
}}

impl BackendClients {{
    pub async fn new(configs: &HashMap<String, BackendConfig>) -> Result<Self, {pascal_name}Error> {{
        let mut clients = HashMap::new();

        for (name, config) in configs {{
            let client = Client::builder()
                .timeout(Duration::from_millis(config.timeout_ms))
                .build()
                .map_err(|e| {pascal_name}Error::Internal(e.to_string()))?;

            clients.insert(name.clone(), BackendClient {{
                client,
                base_url: config.base_url.clone(),
            }});
        }}

        Ok(Self {{ clients }})
    }}

    fn get_client(&self, name: &str) -> Result<&BackendClient, {pascal_name}Error> {{
        self.clients
            .get(name)
            .ok_or_else(|| {pascal_name}Error::BackendError(format!("Backend '{{}}' not configured", name)))
    }}

    pub async fn get_user(&self, user_id: Uuid) -> Result<User, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/users/{{}}", client.base_url, user_id);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn get_user_resources(&self, user_id: Uuid, limit: i32) -> Result<Vec<{pascal_name}Resource>, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/users/{{}}/resources?limit={{}}", client.base_url, user_id, limit);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn get_resource(&self, resource_id: Uuid) -> Result<{pascal_name}Resource, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/resources/{{}}", client.base_url, resource_id);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn get_related_resources(&self, resource_id: Uuid, limit: i32) -> Result<Vec<{pascal_name}Resource>, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/resources/{{}}/related?limit={{}}", client.base_url, resource_id, limit);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn get_stats(&self) -> Result<DashboardStats, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/stats", client.base_url);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn search_resources(&self, query: &str) -> Result<Vec<{pascal_name}Resource>, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/resources/search?q={{}}", client.base_url, query);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn search_users(&self, query: &str) -> Result<Vec<User>, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/users/search?q={{}}", client.base_url, query);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}

    pub async fn list_resources(&self, page: i32, per_page: i32) -> Result<PaginatedResponse<{pascal_name}Resource>, {pascal_name}Error> {{
        let client = self.get_client("api")?;
        let url = format!("{{}}/resources?page={{}}&per_page={{}}", client.base_url, page, per_page);

        let response = client.client
            .get(&url)
            .send()
            .await?
            .error_for_status()
            .map_err(|e| {pascal_name}Error::BackendError(e.to_string()))?;

        response.json().await.map_err(Into::into)
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate infrastructure/cache.rs
pub fn infrastructure_cache(_config: &ProjectConfig) -> String {
    r#"//! Cache service

use moka::future::Cache;
use serde::{de::DeserializeOwned, Serialize};
use std::time::Duration;

use crate::config::CacheConfig;

/// Cache service using moka
pub struct CacheService {
    cache: Cache<String, String>,
}

impl CacheService {
    pub fn new(config: &CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(Duration::from_secs(config.ttl_secs))
            .build();

        Self { cache }
    }

    pub async fn get<T: DeserializeOwned>(&self, key: &str) -> Option<T> {
        let value = self.cache.get(key).await?;
        serde_json::from_str(&value).ok()
    }

    pub async fn set<T: Serialize>(&self, key: &str, value: &T) {
        if let Ok(serialized) = serde_json::to_string(value) {
            self.cache.insert(key.to_string(), serialized).await;
        }
    }

    pub async fn invalidate(&self, key: &str) {
        self.cache.invalidate(key).await;
    }

    pub async fn invalidate_prefix(&self, prefix: &str) {
        // Note: This is a simple implementation. For production,
        // consider using a more efficient invalidation strategy.
        self.cache.invalidate_all();
        let _ = prefix; // Suppress unused warning
    }
}
"#
    .to_string()
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
pub fn presentation_mod(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();

    if bff.graphql_enabled {
        r#"//! Presentation layer (HTTP API + GraphQL)

pub mod handlers;
pub mod graphql;

pub use handlers::*;
pub use graphql::*;
"#
        .to_string()
    } else {
        r#"//! Presentation layer (HTTP API)

pub mod handlers;

pub use handlers::*;
"#
        .to_string()
    }
}

/// Generate presentation/handlers.rs
pub fn presentation_handlers(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! HTTP API handlers

use std::sync::Arc;
use axum::{{
    extract::{{Path, Query, State}},
    routing::get,
    Json, Router,
}};
use serde::Deserialize;
use uuid::Uuid;

use crate::application::{pascal_name}Aggregator;
use crate::domain::{{
    DashboardAggregate,
    ResourceDetailAggregate,
    SearchResultsAggregate,
    PaginatedResponse,
    {pascal_name}Resource,
}};
use crate::error::{pascal_name}Error;

type AppState = Arc<{pascal_name}Aggregator>;

/// Create the REST API router
pub fn create_router(aggregator: Arc<{pascal_name}Aggregator>) -> Router {{
    Router::new()
        .route("/api/dashboard/:user_id", get(get_dashboard))
        .route("/api/resources", get(list_resources))
        .route("/api/resources/:id", get(get_resource_detail))
        .route("/api/search", get(search))
        .with_state(aggregator)
}}

#[derive(Debug, Deserialize)]
struct PaginationQuery {{
    page: Option<i32>,
    per_page: Option<i32>,
}}

#[derive(Debug, Deserialize)]
struct SearchQuery {{
    q: String,
}}

async fn get_dashboard(
    State(aggregator): State<AppState>,
    Path(user_id): Path<Uuid>,
) -> Result<Json<DashboardAggregate>, {pascal_name}Error> {{
    let dashboard = aggregator.get_dashboard(user_id).await?;
    Ok(Json(dashboard))
}}

async fn list_resources(
    State(aggregator): State<AppState>,
    Query(query): Query<PaginationQuery>,
) -> Result<Json<PaginatedResponse<{pascal_name}Resource>>, {pascal_name}Error> {{
    let page = query.page.unwrap_or(1);
    let per_page = query.per_page.unwrap_or(20);
    let resources = aggregator.list_resources(page, per_page).await?;
    Ok(Json(resources))
}}

async fn get_resource_detail(
    State(aggregator): State<AppState>,
    Path(id): Path<Uuid>,
) -> Result<Json<ResourceDetailAggregate>, {pascal_name}Error> {{
    let detail = aggregator.get_resource_detail(id).await?;
    Ok(Json(detail))
}}

async fn search(
    State(aggregator): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> Result<Json<SearchResultsAggregate>, {pascal_name}Error> {{
    let results = aggregator.search(&query.q).await?;
    Ok(Json(results))
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate presentation/graphql.rs (if GraphQL is enabled)
pub fn presentation_graphql(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let pascal_name = to_pascal_case(&bff.service_name);

    format!(
        r#"//! GraphQL API

use std::sync::Arc;
use async_graphql::{{Context, EmptySubscription, Object, Schema, SimpleObject}};
use async_graphql::http::GraphiQLSource;
use async_graphql_axum::GraphQL;
use axum::{{
    routing::get,
    response::{{Html, IntoResponse}},
    Router,
}};
use uuid::Uuid;

use crate::application::{pascal_name}Aggregator;
use crate::domain::{{
    User as DomainUser,
    {pascal_name}Resource as DomainResource,
    DashboardAggregate as DomainDashboard,
}};

/// GraphQL User type
#[derive(SimpleObject)]
struct User {{
    id: Uuid,
    name: String,
    email: String,
}}

impl From<DomainUser> for User {{
    fn from(u: DomainUser) -> Self {{
        Self {{
            id: u.id,
            name: u.name,
            email: u.email,
        }}
    }}
}}

/// GraphQL Resource type
#[derive(SimpleObject)]
struct Resource {{
    id: Uuid,
    name: String,
}}

impl From<DomainResource> for Resource {{
    fn from(r: DomainResource) -> Self {{
        Self {{
            id: r.id,
            name: r.name,
        }}
    }}
}}

/// GraphQL Dashboard type
#[derive(SimpleObject)]
struct Dashboard {{
    user: User,
    recent_resources: Vec<Resource>,
    total_resources: i64,
}}

impl From<DomainDashboard> for Dashboard {{
    fn from(d: DomainDashboard) -> Self {{
        Self {{
            user: d.user.into(),
            recent_resources: d.recent_resources.into_iter().map(Into::into).collect(),
            total_resources: d.stats.total_resources,
        }}
    }}
}}

/// GraphQL Query root
pub struct QueryRoot;

#[Object]
impl QueryRoot {{
    async fn dashboard(&self, ctx: &Context<'_>, user_id: Uuid) -> async_graphql::Result<Dashboard> {{
        let aggregator = ctx.data::<Arc<{pascal_name}Aggregator>>()?;
        let dashboard = aggregator.get_dashboard(user_id).await?;
        Ok(dashboard.into())
    }}

    async fn resource(&self, ctx: &Context<'_>, id: Uuid) -> async_graphql::Result<Resource> {{
        let aggregator = ctx.data::<Arc<{pascal_name}Aggregator>>()?;
        let detail = aggregator.get_resource_detail(id).await?;
        Ok(detail.resource.into())
    }}
}}

/// GraphQL Mutation root
pub struct MutationRoot;

#[Object]
impl MutationRoot {{
    async fn invalidate_cache(&self, _key: String) -> bool {{
        // Placeholder for cache invalidation
        true
    }}
}}

pub type {pascal_name}Schema = Schema<QueryRoot, MutationRoot, EmptySubscription>;

/// Create the GraphQL schema
pub fn create_graphql_schema(aggregator: Arc<{pascal_name}Aggregator>) -> {pascal_name}Schema {{
    Schema::build(QueryRoot, MutationRoot, EmptySubscription)
        .data(aggregator)
        .finish()
}}

/// Create GraphQL routes
pub fn graphql_routes(schema: {pascal_name}Schema) -> Router {{
    Router::new()
        .route("/", get(graphiql).post_service(GraphQL::new(schema)))
}}

async fn graphiql() -> impl IntoResponse {{
    Html(GraphiQLSource::build().endpoint("/graphql").finish())
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let bff = config.bff.as_ref().unwrap();
    let name = &config.name;

    let graphql_section = if bff.graphql_enabled {
        r#"
## GraphQL API

Access the GraphQL playground at `http://localhost:8080/graphql`

### Example Query

```graphql
query {
  dashboard(userId: "550e8400-e29b-41d4-a716-446655440000") {
    user {
      name
      email
    }
    recentResources {
      id
      name
    }
    totalResources
  }
}
```
"#
    } else {
        ""
    };

    format!(
        r#"# {display_name}

A Backend for Frontend (BFF) service built with AllFrame that aggregates multiple backend APIs into a unified interface optimized for {frontend_type:?} clients.

## Features

- **API Aggregation**: Combines multiple backend services into unified endpoints
- **Caching**: In-memory caching with Moka for improved performance
- **Circuit Breaker**: Resilient backend communication
- **REST API**: Clean REST endpoints for frontend consumption{graphql_feature}
- **Health Checks**: Kubernetes-ready liveness and readiness probes
- **OpenTelemetry**: Distributed tracing and metrics

## Prerequisites

- Rust 1.75+
- Backend services running

## Configuration

Set the following environment variables:

```bash
# Server
PORT=8080
HEALTH_PORT=8081

# Backend Services
API_BASE_URL=http://localhost:8000
API_TIMEOUT_MS=5000

# Cache
CACHE_MAX_CAPACITY=10000
CACHE_TTL_SECS=300
```

## Running

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/{name}
```

## REST API Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | /api/dashboard/:user_id | Get aggregated dashboard data |
| GET | /api/resources | List resources with pagination |
| GET | /api/resources/:id | Get resource detail with related data |
| GET | /api/search?q=query | Search across resources and users |
{graphql_section}
## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        BFF Service                            │
│  ┌──────────┐    ┌─────────────┐    ┌────────────────────┐  │
│  │ Handlers │───▶│  Aggregator │───▶│   Backend Clients  │  │
│  └──────────┘    └─────────────┘    └────────────────────┘  │
│       │                │                      │              │
│       │                ▼                      │              │
│       │         ┌─────────────┐              │              │
│       │         │    Cache    │              │              │
│       │         └─────────────┘              │              │
└───────│──────────────────────────────────────│──────────────┘
        │                                      │
        ▼                                      ▼
   ┌─────────┐                          ┌─────────────┐
   │ Frontend│                          │   Backend   │
   │  Client │                          │  Services   │
   └─────────┘                          └─────────────┘
```

## License

MIT
"#,
        display_name = bff.display_name,
        name = name,
        frontend_type = bff.frontend_type,
        graphql_feature = if bff.graphql_enabled { "\n- **GraphQL**: Full GraphQL API with GraphiQL playground" } else { "" },
        graphql_section = graphql_section,
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
    pkg-config \
    libssl-dev \
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
        assert_eq!(to_pascal_case("web_bff"), "WebBff");
        assert_eq!(to_pascal_case("mobile-bff"), "MobileBff");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
