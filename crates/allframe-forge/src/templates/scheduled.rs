//! Scheduled Jobs archetype templates
//!
//! Templates for generating cron job services with recurring task execution.

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

/// Generate Cargo.toml for scheduled jobs project
pub fn cargo_toml(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();
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

# Scheduling
tokio-cron-scheduler = "0.13"
cron = "0.15"

# Async
tokio = {{ version = "1", features = ["full"] }}
async-trait = "0.1"

# Serialization
serde = {{ version = "1.0", features = ["derive"] }}
serde_json = "1.0"

# HTTP (for health checks)
axum = "0.7"

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
        display_name = scheduled.display_name,
    )
}

/// Generate main.rs
pub fn main_rs(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();
    let pascal_name = to_pascal_case(&scheduled.service_name);

    format!(
        r#"//! {display_name}
//!
//! A scheduled jobs service that runs cron-based tasks.

use std::sync::Arc;
use tracing::info;
use tokio_cron_scheduler::JobScheduler;

mod config;
mod error;
mod domain;
mod application;
mod infrastructure;

use config::Config;
use application::{pascal_name}Scheduler;
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
    info!("Configured jobs: {{:?}}", config.jobs.iter().map(|j| &j.name).collect::<Vec<_>>());

    // Create job scheduler
    let mut scheduler = JobScheduler::new().await?;

    // Create and register jobs
    let app_scheduler = Arc::new({pascal_name}Scheduler::new(config.clone()));
    app_scheduler.register_jobs(&scheduler).await?;

    // Start health server in background
    let health_port = config.server.health_port;
    let health_handle = tokio::spawn(async move {{
        let health_server = HealthServer::new(health_port);
        health_server.run().await
    }});

    // Start the scheduler
    info!("Starting job scheduler");
    scheduler.start().await?;

    // Wait for shutdown signal
    tokio::signal::ctrl_c().await?;
    info!("Shutdown signal received");

    // Graceful shutdown
    scheduler.shutdown().await?;
    health_handle.abort();

    info!("Scheduler shutdown complete");
    Ok(())
}}
"#,
        pascal_name = pascal_name,
        display_name = scheduled.display_name,
    )
}

/// Generate config.rs
pub fn config_rs(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();

    let job_configs: Vec<String> = scheduled
        .jobs
        .iter()
        .map(|job| {
            format!(
                r#"            JobConfig {{
                name: "{}".to_string(),
                cron: "{}".to_string(),
                description: "{}".to_string(),
                enabled: {},
                timeout_secs: {},
            }}"#,
                job.name, job.cron, job.description, job.enabled, job.timeout_secs
            )
        })
        .collect();

    format!(
        r#"//! Service configuration

use std::env;

/// Main configuration
#[derive(Debug, Clone)]
pub struct Config {{
    pub server: ServerConfig,
    pub jobs: Vec<JobConfig>,
}}

/// Server configuration
#[derive(Debug, Clone)]
pub struct ServerConfig {{
    pub health_port: u16,
}}

/// Job configuration
#[derive(Debug, Clone)]
pub struct JobConfig {{
    pub name: String,
    pub cron: String,
    pub description: String,
    pub enabled: bool,
    pub timeout_secs: u64,
}}

impl Config {{
    pub fn from_env() -> Self {{
        Self {{
            server: ServerConfig {{
                health_port: env::var("HEALTH_PORT")
                    .unwrap_or_else(|_| "{health_port}".to_string())
                    .parse()
                    .expect("HEALTH_PORT must be a number"),
            }},
            jobs: vec![
{job_configs}
            ],
        }}
    }}
}}
"#,
        health_port = scheduled.server.health_port,
        job_configs = job_configs.join(",\n"),
    )
}

/// Generate error.rs
pub fn error_rs(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();
    let pascal_name = to_pascal_case(&scheduled.service_name);

    format!(
        r#"//! Error types

use thiserror::Error;

/// Scheduler errors
#[derive(Error, Debug)]
pub enum {pascal_name}Error {{
    #[error("Job execution error: {{0}}")]
    JobExecution(String),

    #[error("Job timeout: {{0}}")]
    Timeout(String),

    #[error("Job not found: {{0}}")]
    NotFound(String),

    #[error("Scheduler error: {{0}}")]
    Scheduler(String),

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

pub mod jobs;

pub use jobs::*;
"#
    .to_string()
}

/// Generate domain/jobs.rs
pub fn domain_jobs(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();
    let pascal_name = to_pascal_case(&scheduled.service_name);

    format!(
        r#"//! Job definitions

use async_trait::async_trait;
use chrono::{{DateTime, Utc}};
use uuid::Uuid;

use crate::error::{pascal_name}Error;

/// Job trait that all scheduled jobs must implement
#[async_trait]
pub trait Job: Send + Sync {{
    /// Unique job name
    fn name(&self) -> &str;

    /// Job description
    fn description(&self) -> &str;

    /// Execute the job
    async fn execute(&self, context: JobContext) -> Result<JobResult, {pascal_name}Error>;
}}

/// Context passed to job execution
#[derive(Debug, Clone)]
pub struct JobContext {{
    /// Unique execution ID
    pub execution_id: Uuid,
    /// Scheduled time for this execution
    pub scheduled_at: DateTime<Utc>,
    /// Actual start time
    pub started_at: DateTime<Utc>,
}}

impl JobContext {{
    pub fn new(scheduled_at: DateTime<Utc>) -> Self {{
        Self {{
            execution_id: Uuid::new_v4(),
            scheduled_at,
            started_at: Utc::now(),
        }}
    }}
}}

/// Result of job execution
#[derive(Debug, Clone)]
pub struct JobResult {{
    /// Execution ID
    pub execution_id: Uuid,
    /// Whether the job succeeded
    pub success: bool,
    /// Optional message
    pub message: Option<String>,
    /// Number of items processed (if applicable)
    pub items_processed: Option<u64>,
    /// Duration in milliseconds
    pub duration_ms: u64,
}}

impl JobResult {{
    pub fn success(execution_id: Uuid, duration_ms: u64) -> Self {{
        Self {{
            execution_id,
            success: true,
            message: None,
            items_processed: None,
            duration_ms,
        }}
    }}

    pub fn success_with_message(execution_id: Uuid, message: String, duration_ms: u64) -> Self {{
        Self {{
            execution_id,
            success: true,
            message: Some(message),
            items_processed: None,
            duration_ms,
        }}
    }}

    pub fn failure(execution_id: Uuid, message: String, duration_ms: u64) -> Self {{
        Self {{
            execution_id,
            success: false,
            message: Some(message),
            items_processed: None,
            duration_ms,
        }}
    }}
}}
"#,
        pascal_name = pascal_name,
    )
}

/// Generate application/mod.rs
pub fn application_mod(_config: &ProjectConfig) -> String {
    r#"//! Application layer

pub mod scheduler;

pub use scheduler::*;
"#
    .to_string()
}

/// Generate application/scheduler.rs
pub fn application_scheduler(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();
    let pascal_name = to_pascal_case(&scheduled.service_name);

    // Generate job registrations
    let job_registrations: Vec<String> = scheduled
        .jobs
        .iter()
        .map(|job| {
            let job_pascal = to_pascal_case(&job.name);
            format!(
                r#"        // Register {} job
        if let Some(job_config) = self.config.jobs.iter().find(|j| j.name == "{}") {{
            if job_config.enabled {{
                let job = {}Job::new(job_config.timeout_secs);
                self.register_job(scheduler, &job_config.cron, job).await?;
            }}
        }}"#,
                job.name, job.name, job_pascal
            )
        })
        .collect();

    // Generate job struct definitions
    let job_structs: Vec<String> = scheduled
        .jobs
        .iter()
        .map(|job| {
            let job_pascal = to_pascal_case(&job.name);
            format!(
                r#"/// {} job implementation
pub struct {}Job {{
    timeout_secs: u64,
}}

impl {}Job {{
    pub fn new(timeout_secs: u64) -> Self {{
        Self {{ timeout_secs }}
    }}
}}

#[async_trait]
impl Job for {}Job {{
    fn name(&self) -> &str {{
        "{}"
    }}

    fn description(&self) -> &str {{
        "{}"
    }}

    async fn execute(&self, context: JobContext) -> Result<JobResult, {}Error> {{
        let start = std::time::Instant::now();
        info!(
            execution_id = %context.execution_id,
            job = self.name(),
            "Starting {} job"
        );

        // TODO: Implement actual job logic here
        // For now, just simulate some work
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let duration_ms = start.elapsed().as_millis() as u64;
        info!(
            execution_id = %context.execution_id,
            job = self.name(),
            duration_ms = duration_ms,
            "Completed {} job"
        );

        Ok(JobResult::success_with_message(
            context.execution_id,
            "{} completed successfully".to_string(),
            duration_ms,
        ))
    }}
}}"#,
                job.description,
                job_pascal,
                job_pascal,
                job_pascal,
                job.name,
                job.description,
                pascal_name,
                job.name,
                job.name,
                job.name
            )
        })
        .collect();

    format!(
        r#"//! Job scheduler

use std::sync::Arc;
use async_trait::async_trait;
use tokio_cron_scheduler::{{Job as CronJob, JobScheduler}};
use tracing::{{info, error}};
use chrono::Utc;

use crate::config::Config;
use crate::domain::{{Job, JobContext, JobResult}};
use crate::error::{pascal_name}Error;

/// Main scheduler service
pub struct {pascal_name}Scheduler {{
    config: Config,
}}

impl {pascal_name}Scheduler {{
    pub fn new(config: Config) -> Self {{
        Self {{ config }}
    }}

    /// Register all configured jobs with the scheduler
    pub async fn register_jobs(&self, scheduler: &JobScheduler) -> Result<(), {pascal_name}Error> {{
{job_registrations}

        Ok(())
    }}

    async fn register_job<J: Job + 'static>(
        &self,
        scheduler: &JobScheduler,
        cron_expr: &str,
        job: J,
    ) -> Result<(), {pascal_name}Error> {{
        let job = Arc::new(job);
        let job_name = job.name().to_string();

        let cron_job = CronJob::new_async(cron_expr, move |_uuid, _lock| {{
            let job = job.clone();
            Box::pin(async move {{
                let context = JobContext::new(Utc::now());
                match job.execute(context).await {{
                    Ok(result) => {{
                        if result.success {{
                            info!(
                                job = job.name(),
                                duration_ms = result.duration_ms,
                                "Job completed successfully"
                            );
                        }} else {{
                            error!(
                                job = job.name(),
                                message = ?result.message,
                                "Job completed with failure"
                            );
                        }}
                    }}
                    Err(e) => {{
                        error!(job = job.name(), error = %e, "Job execution failed");
                    }}
                }}
            }})
        }})
        .map_err(|e| {pascal_name}Error::Scheduler(e.to_string()))?;

        scheduler
            .add(cron_job)
            .await
            .map_err(|e| {pascal_name}Error::Scheduler(e.to_string()))?;

        info!(job = job_name, cron = cron_expr, "Registered job");
        Ok(())
    }}
}}

{job_structs}
"#,
        pascal_name = pascal_name,
        job_registrations = job_registrations.join("\n\n"),
        job_structs = job_structs.join("\n\n"),
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

/// Generate README.md
pub fn readme(config: &ProjectConfig) -> String {
    let scheduled = config.scheduled.as_ref().unwrap();
    let name = &config.name;

    let job_table: Vec<String> = scheduled
        .jobs
        .iter()
        .map(|job| format!("| {} | `{}` | {} |", job.name, job.cron, job.description))
        .collect();

    format!(
        r#"# {display_name}

A scheduled jobs service built with AllFrame that runs cron-based recurring tasks.

## Features

- **Cron Scheduling**: Standard cron expressions for flexible scheduling
- **Job Isolation**: Each job runs independently with its own context
- **Timeout Support**: Configurable timeouts per job
- **Tracing**: OpenTelemetry integration for monitoring
- **Health Checks**: Kubernetes-ready liveness and readiness probes

## Prerequisites

- Rust 1.75+

## Configuration

Set the following environment variables:

```bash
# Health Server
HEALTH_PORT=8081
```

## Configured Jobs

| Name | Cron | Description |
|------|------|-------------|
{job_table}

### Cron Expression Reference

```
*    *    *    *    *
┬    ┬    ┬    ┬    ┬
│    │    │    │    │
│    │    │    │    └── Day of Week (0-6, Sun=0)
│    │    │    └─────── Month (1-12)
│    │    └──────────── Day of Month (1-31)
│    └───────────────── Hour (0-23)
└────────────────────── Minute (0-59)
```

## Running

```bash
# Development
cargo run

# Production
cargo build --release
./target/release/{name}
```

## Adding New Jobs

1. Create a new job struct implementing the `Job` trait in `src/application/scheduler.rs`
2. Add the job configuration to `Config`
3. Register the job in `register_jobs()`

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                    Scheduler Service                        │
│  ┌──────────────┐    ┌───────────────┐    ┌─────────────┐  │
│  │  Cron Jobs   │───▶│   Scheduler   │───▶│  Job Exec   │  │
│  └──────────────┘    └───────────────┘    └─────────────┘  │
│         │                                       │          │
│         ▼                                       ▼          │
│  ┌──────────────┐                        ┌─────────────┐   │
│  │   Config     │                        │   Tracing   │   │
│  └──────────────┘                        └─────────────┘   │
└────────────────────────────────────────────────────────────┘
```

## License

MIT
"#,
        display_name = scheduled.display_name,
        name = name,
        job_table = job_table.join("\n"),
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

ENV HEALTH_PORT=8081
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
        assert_eq!(to_pascal_case("cleanup_job"), "CleanupJob");
        assert_eq!(to_pascal_case("daily-report"), "DailyReport");
        assert_eq!(to_pascal_case("simple"), "Simple");
    }
}
