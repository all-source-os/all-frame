//! Anti-Corruption Layer archetype templates
//!
//! Templates for generating services that translate between legacy and modern
//! systems.

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

/// Generate Cargo.toml for ACL project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let name = &config.name;

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
tower-http = {{ version = "0.6", features = ["trace", "cors"] }}

# HTTP Client
reqwest = {{ version = "0.12", features = ["json", "rustls-tls"] }}

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

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = name,
        display_name = acl.display_name,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let pascal_name = to_pascal_case(&acl.service_name);

    format!(
        r#"//! {display_name}
//!
//! An anti-corruption layer service for translating between legacy and modern systems.

use std::sync::Arc;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use config::Config;
use application::{pascal_name}Translator;
use infrastructure::{{LegacyClient, HealthServer}};

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
    info!("Legacy system: {{}} ({{:?}})", config.legacy.name, config.legacy.connection_type);

    // Create legacy client
    let legacy_client = Arc::new(LegacyClient::new(&config.legacy));

    // Create translator
    let translator = Arc::new({pascal_name}Translator::new(config.clone(), legacy_client));

    // Start health server in background
    let health_port = config.server.health_port;
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(health_port);
        health_server.run().await
    }});

    // Create router and start API server
    let app = presentation::create_router(translator);

    info!("Starting ACL server on port {{}}", config.server.http_port);
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{{}}", config.server.http_port)
    ).await?;
    axum::serve(listener, app).await?;

    health_handle.abort();
    info!("ACL shutdown complete");
    Ok(())
}}
"#,
        pascal_name = pascal_name,
        display_name = acl.display_name,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();

    format!(
        r#"//! Service configuration

use std::env;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub legacy: LegacyConfig,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub http_port: u16,
    pub health_port: u16,
}}

/// Legacy system configuration
#[derive(Debug, Clone)]
pub struct LegacyConfig {{
    pub name: String,
    pub connection_type: ConnectionType,
    pub connection_string: String,
    pub timeout_ms: u64,
}}

/// Connection type for legacy system
#[derive(Debug, Clone, Copy)]
pub enum ConnectionType {{
    Rest,
    Soap,
    Database,
    File,
    Mq,
}}

impl Config {{
    pub fn from_env() -> Self {{
        Self {{
            server: ServerConfig {{
                http_port: env::var("PORT")
                    .unwrap_or_else(|_| "{http_port}".to_string())
                    .parse()
                    .expect("PORT must be a number"),
                health_port: env::var("HEALTH_PORT")
                    .unwrap_or_else(|_| "{health_port}".to_string())
                    .parse()
                    .expect("HEALTH_PORT must be a number"),
            }},
            legacy: LegacyConfig {{
                name: env::var("LEGACY_NAME")
                    .unwrap_or_else(|_| "{legacy_name}".to_string()),
                connection_type: ConnectionType::Rest,
                connection_string: env::var("LEGACY_URL")
                    .unwrap_or_else(|_| "{legacy_url}".to_string()),
                timeout_ms: env::var("LEGACY_TIMEOUT_MS")
                    .unwrap_or_else(|_| "{timeout_ms}".to_string())
                    .parse()
                    .expect("LEGACY_TIMEOUT_MS must be a number"),
            }},
        }}
    }}
}}
"#,
        http_port = acl.server.http_port,
        health_port = acl.server.health_port,
        legacy_name = acl.legacy_system.name,
        legacy_url = acl.legacy_system.connection_string,
        timeout_ms = acl.legacy_system.timeout_ms,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let pascal_name = to_pascal_case(&acl.service_name);

    format!(
        r#"//! Error types

use thiserror::Error;

/// ACL errors
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Legacy system error: {{0}}")]
    LegacySystem(String),

    #[error("Transformation error: {{0}}")]
    Transformation(String),

    #[error("Connection error: {{0}}")]
    Connection(String),

    #[error("Timeout error: {{0}}")]
    Timeout(String),

    #[error("Entity not found: {{0}}")]
    NotFound(String),

    #[error("Validation error: {{0}}")]
    Validation(String),

    #[error("Internal error: {{0}}")]
    Internal(String),
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/mod.rs
pub fn domain_mod(_config: &ProjectConfig) -> String {
    r#"//! Domain layer

pub mod legacy;
pub mod modern;
pub mod transformer;

pub use legacy::*;
pub use modern::*;
pub use transformer::*;
"#
    .to_string()
}

/// Generate domain/legacy.rs
pub fn domain_legacy(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let source = acl
        .transformations
        .first()
        .map(|t| &t.source)
        .cloned()
        .unwrap_or_else(|| "LegacyEntity".to_string());

    format!(
        r#"//! Legacy system domain models
//!
//! These models represent the data structures from the legacy system.
//! They should match the legacy system's schema exactly.

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};

/// Legacy entity from the old system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {source} {{
    /// Legacy ID (often not a UUID)
    pub id: String,
    /// Name field (legacy format)
    pub name: String,
    /// Status code (legacy uses numeric codes)
    pub status_code: i32,
    /// Creation date (legacy format)
    pub created_date: String,
    /// Additional data (legacy uses generic map)
    #[serde(default)]
    pub extra_data: serde_json::Value,
}}

/// Legacy response wrapper
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyResponse<T> {{
    pub success: bool,
    pub data: Option<T>,
    pub error_code: Option<i32>,
    pub error_message: Option<String>,
}}

/// Legacy list response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LegacyListResponse<T> {{
    pub items: Vec<T>,
    pub total_count: i64,
    pub page: i32,
    pub page_size: i32,
}}
"#,
        source = source,
    )
}

/// Generate domain/modern.rs
pub fn domain_modern(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let target = acl
        .transformations
        .first()
        .map(|t| &t.target)
        .cloned()
        .unwrap_or_else(|| "ModernEntity".to_string());

    format!(
        r#"//! Modern domain models
//!
//! These models represent the canonical domain models for the new system.
//! They follow modern best practices and use proper types.

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

/// Modern entity in canonical format
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {target} {{
    /// UUID identifier
    pub id: Uuid,
    /// Name
    pub name: String,
    /// Status (enum-like)
    pub status: EntityStatus,
    /// Creation timestamp
    pub created_at: DateTime<Utc>,
    /// Last update timestamp
    pub updated_at: Option<DateTime<Utc>>,
    /// Metadata
    #[serde(default)]
    pub metadata: std::collections::HashMap<String, serde_json::Value>,
}}

/// Entity status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum EntityStatus {{
    Active,
    Inactive,
    Pending,
    Archived,
    Unknown,
}}

impl Default for EntityStatus {{
    fn default() -> Self {{
        Self::Unknown
    }}
}}

/// Modern API response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ApiResponse<T> {{
    pub data: T,
    pub meta: ResponseMeta,
}}

/// Response metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ResponseMeta {{
    pub request_id: Uuid,
    pub timestamp: DateTime<Utc>,
}}

/// Paginated response
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PaginatedResponse<T> {{
    pub data: Vec<T>,
    pub pagination: Pagination,
}}

/// Pagination info
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Pagination {{
    pub total: i64,
    pub page: i32,
    pub per_page: i32,
    pub total_pages: i32,
}}
"#,
        target = target,
    )
}

/// Generate domain/transformer.rs
pub fn domain_transformer(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let pascal_name = to_pascal_case(&acl.service_name);
    // Source and target entity names (available for future customization)
    let _source = acl
        .transformations
        .first()
        .map(|t| &t.source)
        .cloned()
        .unwrap_or_else(|| "LegacyEntity".to_string());
    let _target = acl
        .transformations
        .first()
        .map(|t| &t.target)
        .cloned()
        .unwrap_or_else(|| "ModernEntity".to_string());

    format!(
        r#"//! Transformation traits

use async_trait::async_trait;
use crate::error::{pascal_name}Error;

/// Trait for transforming legacy entities to modern format
#[async_trait]
pub trait LegacyToModern<L, M> {{
    /// Transform a legacy entity to modern format
    fn transform(&self, legacy: L) -> Result<M, {pascal_name}Error>;
}}

/// Trait for transforming modern entities to legacy format
#[async_trait]
pub trait ModernToLegacy<M, L> {{
    /// Transform a modern entity to legacy format
    fn transform(&self, modern: M) -> Result<L, {pascal_name}Error>;
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer

pub mod translator;

pub use translator::*;
"#
    .to_string()
}

/// Generate application/translator.rs
pub fn application_translator(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let pascal_name = to_pascal_case(&acl.service_name);
    let source = acl
        .transformations
        .first()
        .map(|t| &t.source)
        .cloned()
        .unwrap_or_else(|| "LegacyEntity".to_string());
    let target = acl
        .transformations
        .first()
        .map(|t| &t.target)
        .cloned()
        .unwrap_or_else(|| "ModernEntity".to_string());

    format!(
        r#"//! Entity translator

use std::sync::Arc;
use chrono::{{DateTime, Utc, NaiveDateTime}};
use tracing::{{info, warn}};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::{{
    {source}, {target}, EntityStatus,
    LegacyToModern, ModernToLegacy,
}};
use crate::error::{pascal_name}Error;
use crate::infrastructure::LegacyClient;

/// Main translator service
pub struct {pascal_name}Translator {{
    config: Config,
    legacy_client: Arc<LegacyClient>,
}}

impl {pascal_name}Translator {{
    pub fn new(config: Config, legacy_client: Arc<LegacyClient>) -> Self {{
        Self {{
            config,
            legacy_client,
        }}
    }}

    /// Get an entity from the legacy system and return in modern format
    pub async fn get_entity(&self, id: &str) -> Result<{target}, {pascal_name}Error> {{
        let legacy_entity = self.legacy_client.get_entity(id).await?;
        self.transform(legacy_entity)
    }}

    /// List entities from the legacy system in modern format
    pub async fn list_entities(&self, page: i32, per_page: i32) -> Result<Vec<{target}>, {pascal_name}Error> {{
        let legacy_entities = self.legacy_client.list_entities(page, per_page).await?;
        legacy_entities
            .into_iter()
            .map(|e| self.transform(e))
            .collect()
    }}

    /// Create an entity in the legacy system from modern format
    pub async fn create_entity(&self, modern: {target}) -> Result<{target}, {pascal_name}Error> {{
        let legacy = self.to_legacy(modern)?;
        let created = self.legacy_client.create_entity(legacy).await?;
        self.transform(created)
    }}

    fn status_from_code(code: i32) -> EntityStatus {{
        match code {{
            1 => EntityStatus::Active,
            2 => EntityStatus::Inactive,
            3 => EntityStatus::Pending,
            4 => EntityStatus::Archived,
            _ => EntityStatus::Unknown,
        }}
    }}

    fn status_to_code(status: EntityStatus) -> i32 {{
        match status {{
            EntityStatus::Active => 1,
            EntityStatus::Inactive => 2,
            EntityStatus::Pending => 3,
            EntityStatus::Archived => 4,
            EntityStatus::Unknown => 0,
        }}
    }}

    fn to_legacy(&self, modern: {target}) -> Result<{source}, {pascal_name}Error> {{
        Ok({source} {{
            id: modern.id.to_string(),
            name: modern.name,
            status_code: Self::status_to_code(modern.status),
            created_date: modern.created_at.format("%Y-%m-%d %H:%M:%S").to_string(),
            extra_data: serde_json::to_value(modern.metadata)
                .unwrap_or(serde_json::Value::Null),
        }})
    }}
}}

impl LegacyToModern<{source}, {target}> for {pascal_name}Translator {{
    fn transform(&self, legacy: {source}) -> Result<{target}, {pascal_name}Error> {{
        // Parse legacy ID to UUID (or generate new one if invalid)
        let id = Uuid::parse_str(&legacy.id).unwrap_or_else(|_| {{
            warn!(legacy_id = %legacy.id, "Invalid legacy ID, generating new UUID");
            Uuid::new_v4()
        }});

        // Parse legacy date format
        let created_at = NaiveDateTime::parse_from_str(&legacy.created_date, "%Y-%m-%d %H:%M:%S")
            .map(|dt| dt.and_utc())
            .unwrap_or_else(|_| {{
                warn!(date = %legacy.created_date, "Invalid legacy date format");
                Utc::now()
            }});

        // Convert status code to enum
        let status = Self::status_from_code(legacy.status_code);

        // Convert extra_data to metadata
        let metadata = if let serde_json::Value::Object(map) = legacy.extra_data {{
            map.into_iter().collect()
        }} else {{
            std::collections::HashMap::new()
        }};

        Ok({target} {{
            id,
            name: legacy.name,
            status,
            created_at,
            updated_at: None,
            metadata,
        }})
    }}
}}
"#,
        pascal_name = pascal_name,
        source = source,
        target = target,
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(_config: &ProjectConfig) -> String {
    r#"//! Infrastructure layer

pub mod legacy_client;
pub mod health;

pub use legacy_client::*;
pub use health::*;
"#
    .to_string()
}

/// Generate infrastructure/legacy_client.rs
pub fn infrastructure_legacy_client(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let pascal_name = to_pascal_case(&acl.service_name);
    let source = acl
        .transformations
        .first()
        .map(|t| &t.source)
        .cloned()
        .unwrap_or_else(|| "LegacyEntity".to_string());

    format!(
        r#"//! Legacy system client

use std::time::Duration;
use reqwest::Client;
use tracing::{{info, error}};

use crate::config::LegacyConfig;
use crate::domain::{source};
use crate::error::{pascal_name}Error;

/// Client for communicating with the legacy system
pub struct LegacyClient {{
    client: Client,
    base_url: String,
}}

impl LegacyClient {{
    pub fn new(config: &LegacyConfig) -> Self {{
        let client = Client::builder()
            .timeout(Duration::from_millis(config.timeout_ms))
            .build()
            .expect("Failed to create HTTP client");

        Self {{
            client,
            base_url: config.connection_string.clone(),
        }}
    }}

    pub async fn get_entity(&self, id: &str) -> Result<{source}, {pascal_name}Error> {{
        let url = format!("{{}}/entities/{{}}", self.base_url, id);
        info!(url = %url, "Fetching entity from legacy system");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::Connection(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            error!(status = %status, body = %body, "Legacy system error");
            return Err({pascal_name}Error::LegacySystem(format!(
                "HTTP {{}}: {{}}",
                status, body
            )));
        }}

        response
            .json::<{source}>()
            .await
            .map_err(|e| {pascal_name}Error::Transformation(e.to_string()))
    }}

    pub async fn list_entities(&self, page: i32, per_page: i32) -> Result<Vec<{source}>, {pascal_name}Error> {{
        let url = format!(
            "{{}}/entities?page={{}}&page_size={{}}",
            self.base_url, page, per_page
        );
        info!(url = %url, "Listing entities from legacy system");

        let response = self
            .client
            .get(&url)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::Connection(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::LegacySystem(format!(
                "HTTP {{}}: {{}}",
                status, body
            )));
        }}

        response
            .json::<Vec<{source}>>()
            .await
            .map_err(|e| {pascal_name}Error::Transformation(e.to_string()))
    }}

    pub async fn create_entity(&self, entity: {source}) -> Result<{source}, {pascal_name}Error> {{
        let url = format!("{{}}/entities", self.base_url);
        info!(url = %url, "Creating entity in legacy system");

        let response = self
            .client
            .post(&url)
            .json(&entity)
            .send()
            .await
            .map_err(|e| {pascal_name}Error::Connection(e.to_string()))?;

        if !response.status().is_success() {{
            let status = response.status();
            let body = response.text().await.unwrap_or_default();
            return Err({pascal_name}Error::LegacySystem(format!(
                "HTTP {{}}: {{}}",
                status, body
            )));
        }}

        response
            .json::<{source}>()
            .await
            .map_err(|e| {pascal_name}Error::Transformation(e.to_string()))
    }}
}}
"#,
        pascal_name = pascal_name,
        source = source,
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
    r#"//! Presentation layer

pub mod handlers;

pub use handlers::*;
"#
    .to_string()
}

/// Generate presentation/handlers.rs
pub fn presentation_handlers(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let pascal_name = to_pascal_case(&acl.service_name);
    let target = acl
        .transformations
        .first()
        .map(|t| &t.target)
        .cloned()
        .unwrap_or_else(|| "ModernEntity".to_string());

    format!(
        r#"//! API handlers

use std::sync::Arc;
use axum::{{
    extract::{{Path, Query, State}},
    http::StatusCode,
    response::IntoResponse,
    routing::{{get, post}},
    Json, Router,
}};
use serde::Deserialize;
use chrono::Utc;
use uuid::Uuid;

use crate::application::{pascal_name}Translator;
use crate::domain::{{ApiResponse, ResponseMeta, PaginatedResponse, Pagination, {target}}};

type AppState = Arc<{pascal_name}Translator>;

/// Create the API router
pub fn create_router(translator: Arc<{pascal_name}Translator>) -> Router {{
    Router::new()
        .route("/api/v1/entities", get(list_entities).post(create_entity))
        .route("/api/v1/entities/:id", get(get_entity))
        .with_state(translator)
}}

#[derive(Debug, Deserialize)]
struct ListParams {{
    #[serde(default = "default_page")]
    page: i32,
    #[serde(default = "default_per_page")]
    per_page: i32,
}}

fn default_page() -> i32 {{ 1 }}
fn default_per_page() -> i32 {{ 20 }}

async fn get_entity(
    State(translator): State<AppState>,
    Path(id): Path<String>,
) -> impl IntoResponse {{
    match translator.get_entity(&id).await {{
        Ok(entity) => (
            StatusCode::OK,
            Json(ApiResponse {{
                data: entity,
                meta: ResponseMeta {{
                    request_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                }},
            }}),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({{ "error": e.to_string() }})),
        )
            .into_response(),
    }}
}}

async fn list_entities(
    State(translator): State<AppState>,
    Query(params): Query<ListParams>,
) -> impl IntoResponse {{
    match translator.list_entities(params.page, params.per_page).await {{
        Ok(entities) => {{
            let total = entities.len() as i64;
            let total_pages = ((total as f64) / (params.per_page as f64)).ceil() as i32;
            (
                StatusCode::OK,
                Json(PaginatedResponse {{
                    data: entities,
                    pagination: Pagination {{
                        total,
                        page: params.page,
                        per_page: params.per_page,
                        total_pages,
                    }},
                }}),
            )
                .into_response()
        }}
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({{ "error": e.to_string() }})),
        )
            .into_response(),
    }}
}}

async fn create_entity(
    State(translator): State<AppState>,
    Json(entity): Json<{target}>,
) -> impl IntoResponse {{
    match translator.create_entity(entity).await {{
        Ok(created) => (
            StatusCode::CREATED,
            Json(ApiResponse {{
                data: created,
                meta: ResponseMeta {{
                    request_id: Uuid::new_v4(),
                    timestamp: Utc::now(),
                }},
            }}),
        )
            .into_response(),
        Err(e) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::json!({{ "error": e.to_string() }})),
        )
            .into_response(),
    }}
}}
"#,
        pascal_name = pascal_name,
        target = target,
    )
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let acl = config.acl.as_ref().unwrap();
    let name = &config.name;

    let transform_table: Vec<String> = acl
        .transformations
        .iter()
        .map(|t| format!("| {} | {} | {} |", t.source, t.target, t.description))
        .collect();

    format!(
        r#"# {display_name}

An anti-corruption layer service built with AllFrame for translating between legacy and modern systems.

## Features

- **Bidirectional Translation**: Convert between legacy and modern formats
- **Clean Separation**: Isolate legacy system complexity from modern services
- **Type Safety**: Strong typing for both legacy and modern models
- **OpenTelemetry**: Distributed tracing and metrics
- **Health Checks**: Kubernetes-ready liveness and readiness probes

## Prerequisites

- Rust 1.75+

## Configuration

Set the following environment variables:

```bash
# Server
PORT=8080
HEALTH_PORT=8081

# Legacy System
LEGACY_NAME=legacy_api
LEGACY_URL=http://legacy-system:8080
LEGACY_TIMEOUT_MS=10000
```

## Transformations

| Source (Legacy) | Target (Modern) | Description |
|-----------------|-----------------|-------------|
{transform_table}

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
| GET | /api/v1/entities | List entities (paginated) |
| GET | /api/v1/entities/:id | Get entity by ID |
| POST | /api/v1/entities | Create entity |

## Architecture

```
┌─────────────────────────────────────────────────────────────┐
│                  Anti-Corruption Layer                       │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐  │
│  │  Modern API │───▶│  Translator │───▶│  Legacy Client  │  │
│  └─────────────┘    └─────────────┘    └─────────────────┘  │
│        │                  │                    │            │
│        ▼                  ▼                    ▼            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐  │
│  │   Modern    │    │   Domain    │    │     Legacy      │  │
│  │   Models    │◀──▶│  Transform  │◀──▶│     Models      │  │
│  └─────────────┘    └─────────────┘    └─────────────────┘  │
└─────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌───────────────────┐
                    │   Legacy System   │
                    │   (External)      │
                    └───────────────────┘
```

## Adding New Transformations

1. Define the legacy model in `src/domain/legacy.rs`
2. Define the modern model in `src/domain/modern.rs`
3. Implement the `LegacyToModern` trait in the translator
4. Add API endpoints as needed

## License

MIT
"#,
        display_name = acl.display_name,
        name = name,
        transform_table = transform_table.join("\n"),
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
        assert_eq!(to_pascal_case("acl"), "Acl");
        assert_eq!(
            to_pascal_case("anti_corruption_layer"),
            "AntiCorruptionLayer"
        );
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
