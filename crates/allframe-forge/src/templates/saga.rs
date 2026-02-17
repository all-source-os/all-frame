//! Saga Orchestrator archetype templates
//!
//! Templates for generating distributed transaction coordination services.

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

/// Generate Cargo.toml for saga orchestrator project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
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
dashmap = "6.0"
dotenvy = "0.15"
backoff = {{ version = "0.4", features = ["tokio"] }}

[dev-dependencies]
tokio-test = "0.4"
mockall = "0.13"

[[bin]]
name = "{name}"
path = "src/main.rs"
"#,
        name = name,
        display_name = saga.display_name,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    format!(
        r#"//! {display_name}
//!
//! A saga orchestrator service for coordinating distributed transactions.

use std::sync::Arc;
use tracing::info;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;
mod presentation;

use config::Config;
use application::{pascal_name}Orchestrator;
use infrastructure::HealthServer;

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
    info!("Configured sagas: {{:?}}", config.sagas.iter().map(|s| &s.name).collect::<Vec<_>>());

    // Create orchestrator
    let orchestrator = Arc::new({pascal_name}Orchestrator::new(config.clone()));

    // Start health server in background
    let health_port = config.server.health_port;
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(health_port);
        health_server.run().await
    }});

    // Create router and start API server
    let app = presentation::create_router(orchestrator);

    info!("Starting saga orchestrator on port {{}}", config.server.http_port);
    let listener = tokio::net::TcpListener::bind(
        format!("0.0.0.0:{{}}", config.server.http_port)
    ).await?;
    axum::serve(listener, app).await?;

    health_handle.abort();
    info!("Saga orchestrator shutdown complete");
    Ok(())
}}
"#,
        pascal_name = pascal_name,
        display_name = saga.display_name,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();

    let saga_configs: Vec<String> = saga
        .sagas
        .iter()
        .map(|s| {
            let steps_str = s
                .steps
                .iter()
                .map(|step| format!("\"{}\"", step))
                .collect::<Vec<_>>()
                .join(", ");
            format!(
                r#"            SagaConfig {{
                name: "{}".to_string(),
                description: "{}".to_string(),
                steps: vec![{}].into_iter().map(|s: &str| s.to_string()).collect(),
            }}"#,
                s.name, s.description, steps_str
            )
        })
        .collect();

    format!(
        r##"//! Service configuration

use std::env;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub sagas: Vec<SagaConfig>,
    pub retry: RetryConfig,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub http_port: u16,
    pub health_port: u16,
}}

/// Saga configuration
#[derive(Debug, Clone)]
pub struct SagaConfig {{
    pub name: String,
    pub description: String,
    pub steps: Vec<String>,
}}

/// Retry configuration
#[derive(Debug, Clone)]
pub struct RetryConfig {{
    pub max_attempts: u32,
    pub initial_backoff_ms: u64,
    pub max_backoff_ms: u64,
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
            sagas: vec![
{saga_configs}
            ],
            retry: RetryConfig {{
                max_attempts: env::var("MAX_ATTEMPTS")
                    .unwrap_or_else(|_| "{max_attempts}".to_string())
                    .parse()
                    .expect("MAX_ATTEMPTS must be a number"),
                initial_backoff_ms: env::var("INITIAL_BACKOFF_MS")
                    .unwrap_or_else(|_| "{initial_backoff}".to_string())
                    .parse()
                    .expect("INITIAL_BACKOFF_MS must be a number"),
                max_backoff_ms: env::var("MAX_BACKOFF_MS")
                    .unwrap_or_else(|_| "{max_backoff}".to_string())
                    .parse()
                    .expect("MAX_BACKOFF_MS must be a number"),
            }},
        }}
    }}
}}
"##,
        http_port = saga.server.http_port,
        health_port = saga.server.health_port,
        saga_configs = saga_configs.join(",\n"),
        max_attempts = saga.retry.max_attempts,
        initial_backoff = saga.retry.initial_backoff_ms,
        max_backoff = saga.retry.max_backoff_ms,
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    format!(
        r#"//! Error types

use thiserror::Error;

/// Saga orchestrator errors
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Saga not found: {{0}}")]
    SagaNotFound(String),

    #[error("Saga execution failed at step {{step}}: {{reason}}")]
    ExecutionFailed {{ step: String, reason: String }},

    #[error("Compensation failed at step {{step}}: {{reason}}")]
    CompensationFailed {{ step: String, reason: String }},

    #[error("Saga already completed: {{0}}")]
    AlreadyCompleted(String),

    #[error("Invalid state transition: {{0}}")]
    InvalidStateTransition(String),

    #[error("Step timeout: {{0}}")]
    StepTimeout(String),

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

pub mod saga;
pub mod step;

pub use saga::*;
pub use step::*;
"#
    .to_string()
}

/// Generate domain/saga.rs
pub fn domain_saga(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    format!(
        r#"//! Saga execution types

use chrono::{{DateTime, Utc}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

use crate::error::{pascal_name}Error;

/// Saga execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SagaStatus {{
    /// Saga is being executed
    Running,
    /// Saga completed successfully
    Completed,
    /// Saga failed and is compensating
    Compensating,
    /// Saga was rolled back successfully
    Compensated,
    /// Saga failed permanently
    Failed,
}}

/// Saga execution instance
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SagaExecution {{
    pub id: Uuid,
    pub saga_name: String,
    pub status: SagaStatus,
    pub current_step: usize,
    pub steps: Vec<StepExecution>,
    pub payload: serde_json::Value,
    pub started_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub error: Option<String>,
}}

/// Individual step execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StepExecution {{
    pub name: String,
    pub status: StepStatus,
    pub started_at: Option<DateTime<Utc>>,
    pub completed_at: Option<DateTime<Utc>>,
    pub result: Option<serde_json::Value>,
    pub error: Option<String>,
}}

/// Step execution status
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum StepStatus {{
    Pending,
    Running,
    Completed,
    Failed,
    Compensating,
    Compensated,
}}

impl SagaExecution {{
    pub fn new(saga_name: String, steps: Vec<String>, payload: serde_json::Value) -> Self {{
        Self {{
            id: Uuid::new_v4(),
            saga_name,
            status: SagaStatus::Running,
            current_step: 0,
            steps: steps
                .into_iter()
                .map(|name| StepExecution {{
                    name,
                    status: StepStatus::Pending,
                    started_at: None,
                    completed_at: None,
                    result: None,
                    error: None,
                }})
                .collect(),
            payload,
            started_at: Utc::now(),
            completed_at: None,
            error: None,
        }}
    }}

    pub fn start_step(&mut self) -> Result<&mut StepExecution, {pascal_name}Error> {{
        if self.current_step >= self.steps.len() {{
            return Err({pascal_name}Error::InvalidStateTransition(
                "No more steps to execute".to_string(),
            ));
        }}

        let step = &mut self.steps[self.current_step];
        step.status = StepStatus::Running;
        step.started_at = Some(Utc::now());
        Ok(step)
    }}

    pub fn complete_step(&mut self, result: serde_json::Value) {{
        if let Some(step) = self.steps.get_mut(self.current_step) {{
            step.status = StepStatus::Completed;
            step.completed_at = Some(Utc::now());
            step.result = Some(result);
            self.current_step += 1;

            // Check if saga is complete
            if self.current_step >= self.steps.len() {{
                self.status = SagaStatus::Completed;
                self.completed_at = Some(Utc::now());
            }}
        }}
    }}

    pub fn fail_step(&mut self, error: String) {{
        if let Some(step) = self.steps.get_mut(self.current_step) {{
            step.status = StepStatus::Failed;
            step.completed_at = Some(Utc::now());
            step.error = Some(error.clone());
        }}
        self.status = SagaStatus::Compensating;
        self.error = Some(error);
    }}

    pub fn is_complete(&self) -> bool {{
        matches!(
            self.status,
            SagaStatus::Completed | SagaStatus::Compensated | SagaStatus::Failed
        )
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate domain/step.rs
pub fn domain_step(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    format!(
        r#"//! Saga step definitions

use async_trait::async_trait;

use crate::error::{pascal_name}Error;

/// Trait for saga steps
#[async_trait]
pub trait SagaStep: Send + Sync {{
    /// Step name
    fn name(&self) -> &str;

    /// Execute the step
    async fn execute(&self, payload: &serde_json::Value) -> Result<serde_json::Value, {pascal_name}Error>;

    /// Compensate (undo) the step
    async fn compensate(&self, payload: &serde_json::Value) -> Result<(), {pascal_name}Error>;
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer

pub mod orchestrator;
pub mod steps;

pub use orchestrator::*;
"#
    .to_string()
}

/// Generate application/orchestrator.rs
pub fn application_orchestrator(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    format!(
        r#"//! Saga orchestrator

use std::collections::HashMap;
use std::sync::Arc;
use dashmap::DashMap;
use tracing::{{info, error, debug}};
use uuid::Uuid;

use crate::config::Config;
use crate::domain::{{SagaExecution, SagaStatus, SagaStep, StepStatus}};
use crate::error::{pascal_name}Error;
use crate::application::steps;

/// Saga orchestrator managing distributed transactions
pub struct {pascal_name}Orchestrator {{
    config: Config,
    /// Running saga executions
    executions: DashMap<Uuid, SagaExecution>,
    /// Step implementations by saga name and step name
    steps: HashMap<String, HashMap<String, Arc<dyn SagaStep>>>,
}}

impl {pascal_name}Orchestrator {{
    pub fn new(config: Config) -> Self {{
        let mut steps_map = HashMap::new();

        // Register step implementations for each saga
        for saga_config in &config.sagas {{
            let saga_steps = steps::create_steps(&saga_config.name, &saga_config.steps);
            steps_map.insert(saga_config.name.clone(), saga_steps);
        }}

        Self {{
            config,
            executions: DashMap::new(),
            steps: steps_map,
        }}
    }}

    /// Start a new saga execution
    pub async fn start_saga(
        &self,
        saga_name: &str,
        payload: serde_json::Value,
    ) -> Result<Uuid, {pascal_name}Error> {{
        let saga_config = self
            .config
            .sagas
            .iter()
            .find(|s| s.name == saga_name)
            .ok_or_else(|| {pascal_name}Error::SagaNotFound(saga_name.to_string()))?;

        let execution = SagaExecution::new(
            saga_name.to_string(),
            saga_config.steps.clone(),
            payload,
        );
        let saga_id = execution.id;

        self.executions.insert(saga_id, execution);
        info!(saga_id = %saga_id, saga = saga_name, "Started saga execution");

        // Start execution in background
        let orchestrator = self.clone_inner(saga_id);
        tokio::spawn(async move {{
            if let Err(e) = orchestrator.run_saga(saga_id).await {{
                error!(saga_id = %saga_id, error = %e, "Saga execution failed");
            }}
        }});

        Ok(saga_id)
    }}

    fn clone_inner(&self, _saga_id: Uuid) -> SagaRunner {{
        SagaRunner {{
            executions: self.executions.clone(),
            steps: self.steps.clone(),
        }}
    }}

    /// Get saga execution status
    pub fn get_execution(&self, saga_id: Uuid) -> Option<SagaExecution> {{
        self.executions.get(&saga_id).map(|e| e.clone())
    }}

    /// List all executions
    pub fn list_executions(&self) -> Vec<SagaExecution> {{
        self.executions.iter().map(|e| e.value().clone()).collect()
    }}

    /// Get configured sagas
    pub fn get_sagas(&self) -> Vec<&crate::config::SagaConfig> {{
        self.config.sagas.iter().collect()
    }}
}}

/// Runner for saga execution
struct SagaRunner {{
    executions: DashMap<Uuid, SagaExecution>,
    steps: HashMap<String, HashMap<String, Arc<dyn SagaStep>>>,
}}

impl SagaRunner {{
    async fn run_saga(&self, saga_id: Uuid) -> Result<(), {pascal_name}Error> {{
        loop {{
            let execution = self
                .executions
                .get(&saga_id)
                .ok_or_else(|| {pascal_name}Error::SagaNotFound(saga_id.to_string()))?
                .clone();

            if execution.is_complete() {{
                break;
            }}

            match execution.status {{
                SagaStatus::Running => {{
                    self.execute_next_step(saga_id, &execution).await?;
                }}
                SagaStatus::Compensating => {{
                    self.compensate_steps(saga_id, &execution).await?;
                }}
                _ => break,
            }}
        }}

        Ok(())
    }}

    async fn execute_next_step(
        &self,
        saga_id: Uuid,
        execution: &SagaExecution,
    ) -> Result<(), {pascal_name}Error> {{
        let step_name = &execution.steps[execution.current_step].name;
        let saga_steps = self
            .steps
            .get(&execution.saga_name)
            .ok_or_else(|| {pascal_name}Error::SagaNotFound(execution.saga_name.clone()))?;

        let step = saga_steps
            .get(step_name)
            .ok_or_else(|| {pascal_name}Error::SagaNotFound(step_name.clone()))?;

        debug!(saga_id = %saga_id, step = step_name, "Executing step");

        // Mark step as running
        if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
            let _ = exec.start_step();
        }}

        // Execute step
        match step.execute(&execution.payload).await {{
            Ok(result) => {{
                if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
                    exec.complete_step(result);
                }}
                info!(saga_id = %saga_id, step = step_name, "Step completed");
            }}
            Err(e) => {{
                if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
                    exec.fail_step(e.to_string());
                }}
                error!(saga_id = %saga_id, step = step_name, error = %e, "Step failed");
            }}
        }}

        Ok(())
    }}

    async fn compensate_steps(
        &self,
        saga_id: Uuid,
        execution: &SagaExecution,
    ) -> Result<(), {pascal_name}Error> {{
        let saga_steps = self
            .steps
            .get(&execution.saga_name)
            .ok_or_else(|| {pascal_name}Error::SagaNotFound(execution.saga_name.clone()))?;

        // Compensate completed steps in reverse order
        for i in (0..execution.current_step).rev() {{
            let step_exec = &execution.steps[i];
            if step_exec.status == StepStatus::Completed {{
                let step_name = &step_exec.name;
                if let Some(step) = saga_steps.get(step_name) {{
                    debug!(saga_id = %saga_id, step = step_name, "Compensating step");

                    // Mark step as compensating
                    if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
                        exec.steps[i].status = StepStatus::Compensating;
                    }}

                    match step.compensate(&execution.payload).await {{
                        Ok(()) => {{
                            if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
                                exec.steps[i].status = StepStatus::Compensated;
                            }}
                            info!(saga_id = %saga_id, step = step_name, "Step compensated");
                        }}
                        Err(e) => {{
                            error!(
                                saga_id = %saga_id,
                                step = step_name,
                                error = %e,
                                "Compensation failed"
                            );
                            // Mark saga as failed if compensation fails
                            if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
                                exec.status = SagaStatus::Failed;
                            }}
                            return Err(e);
                        }}
                    }}
                }}
            }}
        }}

        // Mark saga as compensated
        if let Some(mut exec) = self.executions.get_mut(&saga_id) {{
            exec.status = SagaStatus::Compensated;
            exec.completed_at = Some(chrono::Utc::now());
        }}

        Ok(())
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/steps.rs
pub fn application_steps(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    // Generate step implementations for the default saga
    let step_impls: Vec<String> = saga
        .sagas
        .first()
        .map(|s| {
            s.steps
                .iter()
                .map(|step| {
                    let step_pascal = to_pascal_case(step);
                    format!(
                        r#"/// {step} step implementation
pub struct {step_pascal}Step;

#[async_trait]
impl SagaStep for {step_pascal}Step {{
    fn name(&self) -> &str {{
        "{step}"
    }}

    async fn execute(&self, payload: &serde_json::Value) -> Result<serde_json::Value, {pascal_name}Error> {{
        info!(step = "{step}", "Executing step");
        // TODO: Implement actual step logic
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;
        Ok(serde_json::json!({{ "step": "{step}", "status": "completed" }}))
    }}

    async fn compensate(&self, payload: &serde_json::Value) -> Result<(), {pascal_name}Error> {{
        info!(step = "{step}", "Compensating step");
        // TODO: Implement compensation logic
        tokio::time::sleep(std::time::Duration::from_millis(50)).await;
        Ok(())
    }}
}}"#,
                        step = step,
                        step_pascal = step_pascal,
                        pascal_name = pascal_name,
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    let step_registrations: Vec<String> = saga
        .sagas
        .first()
        .map(|s| {
            s.steps
                .iter()
                .map(|step| {
                    let step_pascal = to_pascal_case(step);
                    format!(
                        r#"            steps.insert("{step}".to_string(), Arc::new({step_pascal}Step) as Arc<dyn SagaStep>);"#,
                        step = step,
                        step_pascal = step_pascal,
                    )
                })
                .collect()
        })
        .unwrap_or_default();

    format!(
        r#"//! Saga step implementations

use std::collections::HashMap;
use std::sync::Arc;
use async_trait::async_trait;
use tracing::info;

use crate::domain::SagaStep;
use crate::error::{pascal_name}Error;

/// Create step implementations for a saga
pub fn create_steps(saga_name: &str, step_names: &[String]) -> HashMap<String, Arc<dyn SagaStep>> {{
    let mut steps: HashMap<String, Arc<dyn SagaStep>> = HashMap::new();

    match saga_name {{
        "{default_saga}" => {{
{step_registrations}
        }}
        _ => {{
            // Unknown saga - create placeholder steps
            for step_name in step_names {{
                steps.insert(step_name.clone(), Arc::new(PlaceholderStep(step_name.clone())) as Arc<dyn SagaStep>);
            }}
        }}
    }}

    steps
}}

{step_impls}

/// Placeholder step for undefined sagas
struct PlaceholderStep(String);

#[async_trait]
impl SagaStep for PlaceholderStep {{
    fn name(&self) -> &str {{
        &self.0
    }}

    async fn execute(&self, _payload: &serde_json::Value) -> Result<serde_json::Value, {pascal_name}Error> {{
        info!(step = %self.0, "Executing placeholder step");
        Ok(serde_json::json!({{ "step": self.0, "status": "completed" }}))
    }}

    async fn compensate(&self, _payload: &serde_json::Value) -> Result<(), {pascal_name}Error> {{
        info!(step = %self.0, "Compensating placeholder step");
        Ok(())
    }}
}}
"#,
        pascal_name = pascal_name,
        default_saga = saga.sagas.first().map(|s| s.name.as_str()).unwrap_or(""),
        step_registrations = step_registrations.join("\n"),
        step_impls = step_impls.join("\n\n"),
    )
}

/// Generate infrastructure/mod.rs
pub fn infrastructure_mod(_config: &ProjectConfig) -> String {
    r#"//! Infrastructure layer

pub mod health;

pub use health::*;
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
pub fn presentation_mod(_config: &ProjectConfig) -> String {
    r#"//! Presentation layer

pub mod handlers;

pub use handlers::*;
"#
    .to_string()
}

/// Generate presentation/handlers.rs
pub fn presentation_handlers(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let pascal_name = to_pascal_case(&saga.service_name);

    format!(
        r#"//! API handlers

use std::sync::Arc;
use axum::{{
    extract::{{Path, State}},
    http::StatusCode,
    response::IntoResponse,
    routing::{{get, post}},
    Json, Router,
}};
use serde::{{Deserialize, Serialize}};
use uuid::Uuid;

use crate::application::{pascal_name}Orchestrator;
use crate::domain::SagaExecution;

type AppState = Arc<{pascal_name}Orchestrator>;

/// Create the API router
pub fn create_router(orchestrator: Arc<{pascal_name}Orchestrator>) -> Router {{
    Router::new()
        .route("/sagas", get(list_sagas))
        .route("/sagas/:saga_name/execute", post(start_saga))
        .route("/executions", get(list_executions))
        .route("/executions/:id", get(get_execution))
        .with_state(orchestrator)
}}

#[derive(Debug, Deserialize)]
struct StartSagaRequest {{
    payload: serde_json::Value,
}}

#[derive(Debug, Serialize)]
struct StartSagaResponse {{
    saga_id: Uuid,
}}

#[derive(Debug, Serialize)]
struct SagaInfo {{
    name: String,
    description: String,
    steps: Vec<String>,
}}

async fn list_sagas(State(orchestrator): State<AppState>) -> Json<Vec<SagaInfo>> {{
    let sagas = orchestrator
        .get_sagas()
        .into_iter()
        .map(|s| SagaInfo {{
            name: s.name.clone(),
            description: s.description.clone(),
            steps: s.steps.clone(),
        }})
        .collect();
    Json(sagas)
}}

async fn start_saga(
    State(orchestrator): State<AppState>,
    Path(saga_name): Path<String>,
    Json(request): Json<StartSagaRequest>,
) -> impl IntoResponse {{
    match orchestrator.start_saga(&saga_name, request.payload).await {{
        Ok(saga_id) => (
            StatusCode::ACCEPTED,
            Json(StartSagaResponse {{ saga_id }}),
        )
            .into_response(),
        Err(e) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::json!({{ "error": e.to_string() }})),
        )
            .into_response(),
    }}
}}

async fn list_executions(State(orchestrator): State<AppState>) -> Json<Vec<SagaExecution>> {{
    Json(orchestrator.list_executions())
}}

async fn get_execution(
    State(orchestrator): State<AppState>,
    Path(id): Path<Uuid>,
) -> impl IntoResponse {{
    match orchestrator.get_execution(id) {{
        Some(execution) => (StatusCode::OK, Json(execution)).into_response(),
        None => (
            StatusCode::NOT_FOUND,
            Json(serde_json::json!({{ "error": "Saga execution not found" }})),
        )
            .into_response(),
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let saga = config.saga_orchestrator.as_ref().unwrap();
    let name = &config.name;

    let saga_table: Vec<String> = saga
        .sagas
        .iter()
        .map(|s| {
            format!(
                "| {} | {} | {} |",
                s.name,
                s.description,
                s.steps.join(" -> ")
            )
        })
        .collect();

    format!(
        r#"# {display_name}

A saga orchestrator service built with AllFrame for coordinating distributed transactions.

## Features

- **Saga Pattern**: Orchestration-based saga coordination
- **Automatic Compensation**: Automatic rollback on failures
- **Idempotent Steps**: Each step can be safely retried
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

# Retry
MAX_RETRIES=3
INITIAL_INTERVAL_MS=100
MAX_INTERVAL_MS=10000
```

## Configured Sagas

| Name | Description | Steps |
|------|-------------|-------|
{saga_table}

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
| GET | /sagas | List available sagas |
| POST | /sagas/:name/execute | Start a saga execution |
| GET | /executions | List all executions |
| GET | /executions/:id | Get execution status |

## Starting a Saga

```bash
curl -X POST http://localhost:8080/sagas/order_saga/execute \
  -H "Content-Type: application/json" \
  -d '{{"payload": {{"order_id": "123", "amount": 100.00}}}}'
```

## Saga Execution States

- **running**: Saga is executing steps
- **completed**: All steps completed successfully
- **compensating**: A step failed, rolling back
- **compensated**: Rollback completed successfully
- **failed**: Rollback also failed (requires manual intervention)

## Architecture

```
┌───────────────────────────────────────────────────────────────┐
│                    Saga Orchestrator                           │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐│
│  │  API Layer  │───▶│ Orchestrator│───▶│   Step Executors    ││
│  └─────────────┘    └─────────────┘    └─────────────────────┘│
│        │                  │                      │            │
│        ▼                  ▼                      ▼            │
│  ┌─────────────┐    ┌─────────────┐    ┌─────────────────────┐│
│  │   Routes    │    │  Execution  │    │    Compensation     ││
│  │   Handler   │    │    Store    │    │      Handler        ││
│  └─────────────┘    └─────────────┘    └─────────────────────┘│
└───────────────────────────────────────────────────────────────┘
```

## Adding New Sagas

1. Add saga configuration to `Config`
2. Create step implementations in `src/application/steps.rs`
3. Register steps in `create_steps()` function

## License

MIT
"#,
        display_name = saga.display_name,
        name = name,
        saga_table = saga_table.join("\n"),
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
        assert_eq!(to_pascal_case("saga_orchestrator"), "SagaOrchestrator");
        assert_eq!(to_pascal_case("order-saga"), "OrderSaga");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
