//! AllFrame Forge - Project scaffolding library
//!
//! This library provides utilities for creating new AllFrame projects.
//! It is used by the `allframe` CLI binary.
//!
//! # Archetypes
//!
//! AllFrame supports different project archetypes:
//! - `basic` (default): Simple Clean Architecture project with greeter example
//! - `gateway`: API Gateway service with gRPC, resilience, and caching
//! - `consumer`: Event consumer service with Kafka, idempotency, and DLQ
//! - `producer`: Event producer service with outbox pattern and transactional messaging
//! - `bff`: Backend for Frontend API aggregation service
//! - `scheduled`: Scheduled jobs service with cron-based task execution
//! - `websocket-gateway`: WebSocket gateway for real-time bidirectional communication
//! - `saga-orchestrator`: Saga orchestrator for distributed transaction coordination
//! - `legacy-adapter`: Legacy system adapter (anti-corruption layer)
//!
//! # Usage
//!
//! ```bash
//! # Create a basic project
//! allframe ignite my-service
//!
//! # Create a gateway project
//! allframe ignite my-gateway --archetype gateway
//!
//! # Create a consumer project
//! allframe ignite my-consumer --archetype consumer
//!
//! # Create a producer project
//! allframe ignite my-producer --archetype producer
//!
//! # Create a BFF project
//! allframe ignite my-bff --archetype bff
//!
//! # Create a scheduled jobs project
//! allframe ignite my-scheduler --archetype scheduled
//!
//! # Create a WebSocket gateway project
//! allframe ignite my-ws --archetype websocket-gateway
//!
//! # Create a saga orchestrator project
//! allframe ignite my-saga --archetype saga-orchestrator
//!
//! # Create a legacy adapter project
//! allframe ignite my-adapter --archetype legacy-adapter
//! ```

#![deny(missing_docs)]

pub mod config;
pub mod scaffolding;
pub mod templates;
pub mod validation;

pub use config::{Archetype, ProjectConfig};

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};

/// CLI archetype selection (maps to config::Archetype)
#[derive(Debug, Clone, Copy, Default, ValueEnum)]
pub enum CliArchetype {
    /// Basic Clean Architecture project (default)
    #[default]
    Basic,
    /// API Gateway service with gRPC, resilience, and caching
    Gateway,
    /// Event consumer service with Kafka, idempotency, and DLQ
    Consumer,
    /// Event producer service with outbox pattern and transactional messaging
    Producer,
    /// Backend for Frontend API aggregation service
    Bff,
    /// Scheduled jobs service with cron-based task execution
    Scheduled,
    /// WebSocket gateway for real-time bidirectional communication
    WebsocketGateway,
    /// Saga orchestrator for distributed transaction coordination
    SagaOrchestrator,
    /// Legacy system adapter (anti-corruption layer)
    LegacyAdapter,
}

impl From<CliArchetype> for Archetype {
    fn from(cli: CliArchetype) -> Self {
        match cli {
            CliArchetype::Basic => Archetype::Basic,
            CliArchetype::Gateway => Archetype::Gateway,
            CliArchetype::Consumer => Archetype::Consumer,
            CliArchetype::Producer => Archetype::Producer,
            CliArchetype::Bff => Archetype::Bff,
            CliArchetype::Scheduled => Archetype::Scheduled,
            CliArchetype::WebsocketGateway => Archetype::WebSocketGateway,
            CliArchetype::SagaOrchestrator => Archetype::SagaOrchestrator,
            CliArchetype::LegacyAdapter => Archetype::AntiCorruptionLayer,
        }
    }
}

#[derive(Parser)]
#[command(name = "allframe")]
#[command(about = "AllFrame CLI - The composable Rust API framework", long_about = None)]
#[command(version)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Create a new AllFrame project
    Ignite {
        /// Name of the project to create
        name: PathBuf,

        /// Project archetype (basic, gateway, consumer)
        #[arg(short, long, value_enum, default_value_t = CliArchetype::Basic)]
        archetype: CliArchetype,

        /// Service name (e.g., "kraken" for gateway, "order-processor" for consumer)
        #[arg(long)]
        service_name: Option<String>,

        /// Base URL for gateway's external API
        #[arg(long)]
        api_base_url: Option<String>,

        /// Consumer group ID (for consumer archetype)
        #[arg(long)]
        group_id: Option<String>,

        /// Kafka broker addresses (for consumer archetype)
        #[arg(long)]
        brokers: Option<String>,

        /// Enable all features
        #[arg(long)]
        all_features: bool,
    },
    /// Generate code from LLM prompts (coming soon)
    Forge {
        /// The prompt for code generation
        prompt: String,
    },
}

/// Run the AllFrame CLI with command-line arguments.
///
/// This is the main entry point for the CLI, designed to be called from
/// both the `allframe-forge` binary and the `allframe` binary wrapper.
///
/// # Errors
/// Returns an error if command parsing fails or if the executed command fails.
pub fn run() -> anyhow::Result<()> {
    let cli = Cli::parse();

    match cli.command {
        Commands::Ignite {
            name,
            archetype,
            service_name,
            api_base_url,
            group_id,
            brokers: _,
            all_features: _,
        } => {
            ignite_project(&name, archetype, service_name, api_base_url, group_id)?;
        }
        Commands::Forge { prompt } => {
            forge_code(&prompt)?;
        }
    }

    Ok(())
}

/// Create a new AllFrame project
///
/// This function orchestrates the creation of a new AllFrame project with
/// Clean Architecture structure.
fn ignite_project(
    project_path: &Path,
    archetype: CliArchetype,
    service_name: Option<String>,
    api_base_url: Option<String>,
    group_id: Option<String>,
) -> anyhow::Result<()> {
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project path"))?;

    validation::validate_project_name(project_name)?;

    if project_path.exists() {
        anyhow::bail!("Directory already exists: {}", project_path.display());
    }

    std::fs::create_dir_all(project_path)?;

    // Build project configuration based on archetype
    let config = match archetype {
        CliArchetype::Basic => ProjectConfig::new(project_name),
        CliArchetype::Gateway => {
            let mut config = ProjectConfig::new(project_name).with_archetype(Archetype::Gateway);

            // Configure gateway-specific settings
            if let Some(gateway) = config.gateway.as_mut() {
                if let Some(svc_name) = service_name.clone() {
                    gateway.service_name = svc_name.clone();
                    gateway.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    gateway.service_name = project_name.replace('-', "_");
                    gateway.display_name = to_title_case(project_name);
                }

                if let Some(url) = api_base_url {
                    gateway.api_base_url = url;
                }
            }

            config
        }
        CliArchetype::Consumer => {
            let mut config = ProjectConfig::new(project_name).with_archetype(Archetype::Consumer);

            // Configure consumer-specific settings
            if let Some(consumer) = config.consumer.as_mut() {
                if let Some(svc_name) = service_name {
                    consumer.service_name = svc_name.clone();
                    consumer.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    consumer.service_name = project_name.replace('-', "_");
                    consumer.display_name = to_title_case(project_name);
                }

                if let Some(gid) = group_id {
                    consumer.group_id = gid;
                } else {
                    consumer.group_id = format!("{}-group", consumer.service_name);
                }
            }

            config
        }
        CliArchetype::Producer => {
            let mut config = ProjectConfig::new(project_name).with_archetype(Archetype::Producer);

            // Configure producer-specific settings
            if let Some(producer) = config.producer.as_mut() {
                if let Some(svc_name) = service_name.clone() {
                    producer.service_name = svc_name.clone();
                    producer.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    producer.service_name = project_name.replace('-', "_");
                    producer.display_name = to_title_case(project_name);
                }
            }

            config
        }
        CliArchetype::Bff => {
            let mut config = ProjectConfig::new(project_name).with_archetype(Archetype::Bff);

            // Configure BFF-specific settings
            if let Some(bff) = config.bff.as_mut() {
                if let Some(svc_name) = service_name.clone() {
                    bff.service_name = svc_name.clone();
                    bff.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    bff.service_name = project_name.replace('-', "_");
                    bff.display_name = to_title_case(project_name);
                }

                if let Some(url) = api_base_url {
                    if let Some(backend) = bff.backends.first_mut() {
                        backend.base_url = url;
                    }
                }
            }

            config
        }
        CliArchetype::Scheduled => {
            let mut config = ProjectConfig::new(project_name).with_archetype(Archetype::Scheduled);

            // Configure scheduled-specific settings
            if let Some(scheduled) = config.scheduled.as_mut() {
                if let Some(svc_name) = service_name.clone() {
                    scheduled.service_name = svc_name.clone();
                    scheduled.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    scheduled.service_name = project_name.replace('-', "_");
                    scheduled.display_name = to_title_case(project_name);
                }
            }

            config
        }
        CliArchetype::WebsocketGateway => {
            let mut config =
                ProjectConfig::new(project_name).with_archetype(Archetype::WebSocketGateway);

            // Configure websocket-specific settings
            if let Some(ws) = config.websocket_gateway.as_mut() {
                if let Some(svc_name) = service_name.clone() {
                    ws.service_name = svc_name.clone();
                    ws.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    ws.service_name = project_name.replace('-', "_");
                    ws.display_name = to_title_case(project_name);
                }
            }

            config
        }
        CliArchetype::SagaOrchestrator => {
            let mut config =
                ProjectConfig::new(project_name).with_archetype(Archetype::SagaOrchestrator);

            // Configure saga-specific settings
            if let Some(saga) = config.saga_orchestrator.as_mut() {
                if let Some(svc_name) = service_name.clone() {
                    saga.service_name = svc_name.clone();
                    saga.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    saga.service_name = project_name.replace('-', "_");
                    saga.display_name = to_title_case(project_name);
                }
            }

            config
        }
        CliArchetype::LegacyAdapter => {
            let mut config =
                ProjectConfig::new(project_name).with_archetype(Archetype::AntiCorruptionLayer);

            // Configure legacy adapter settings
            if let Some(acl) = config.acl.as_mut() {
                if let Some(svc_name) = service_name {
                    acl.service_name = svc_name.clone();
                    acl.display_name = to_title_case(&svc_name);
                } else {
                    // Default to project name
                    acl.service_name = project_name.replace('-', "_");
                    acl.display_name = to_title_case(project_name);
                }

                if let Some(url) = api_base_url {
                    acl.legacy_system.connection_string = url;
                }
            }

            config
        }
    };

    // Create directory structure and generate files based on archetype
    match config.archetype {
        Archetype::Basic => {
            scaffolding::create_directory_structure(project_path)?;
            scaffolding::generate_files(project_path, project_name)?;
        }
        Archetype::Gateway => {
            scaffolding::create_gateway_structure(project_path)?;
            scaffolding::generate_gateway_files(project_path, &config)?;
        }
        Archetype::Consumer => {
            scaffolding::create_consumer_structure(project_path)?;
            scaffolding::generate_consumer_files(project_path, &config)?;
        }
        Archetype::Producer => {
            scaffolding::create_producer_structure(project_path)?;
            scaffolding::generate_producer_files(project_path, &config)?;
        }
        Archetype::Bff => {
            scaffolding::create_bff_structure(project_path)?;
            scaffolding::generate_bff_files(project_path, &config)?;
        }
        Archetype::Scheduled => {
            scaffolding::create_scheduled_structure(project_path)?;
            scaffolding::generate_scheduled_files(project_path, &config)?;
        }
        Archetype::WebSocketGateway => {
            scaffolding::create_websocket_structure(project_path)?;
            scaffolding::generate_websocket_files(project_path, &config)?;
        }
        Archetype::SagaOrchestrator => {
            scaffolding::create_saga_structure(project_path)?;
            scaffolding::generate_saga_files(project_path, &config)?;
        }
        Archetype::AntiCorruptionLayer => {
            scaffolding::create_acl_structure(project_path)?;
            scaffolding::generate_acl_files(project_path, &config)?;
        }
        _ => {
            anyhow::bail!("Archetype {:?} is not yet implemented", config.archetype);
        }
    }

    println!(
        "AllFrame {} project created successfully: {}",
        config.archetype, project_name
    );
    println!("\nNext steps:");
    println!("  cd {}", project_name);

    match config.archetype {
        Archetype::Gateway => {
            println!("  # Edit src/config.rs to set your API credentials");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::Consumer => {
            println!("  # Set environment variables for broker connection");
            println!("  # See README.md for configuration options");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::Producer => {
            println!("  # Set DATABASE_URL and KAFKA_BROKERS environment variables");
            println!("  # Run database migrations (see README.md)");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::Bff => {
            println!("  # Set API_BASE_URL for backend service connection");
            println!("  # See README.md for configuration options");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::Scheduled => {
            println!("  # Configure jobs in src/config.rs");
            println!("  # See README.md for cron expression reference");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::WebSocketGateway => {
            println!("  # Configure channels in src/config.rs");
            println!("  # See README.md for WebSocket API documentation");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::SagaOrchestrator => {
            println!("  # Configure sagas and steps in src/config.rs");
            println!("  # See README.md for saga pattern documentation");
            println!("  cargo build");
            println!("  cargo run");
        }
        Archetype::AntiCorruptionLayer => {
            println!("  # Configure legacy system connection in src/config.rs");
            println!("  # See README.md for transformer implementation guide");
            println!("  cargo build");
            println!("  cargo run");
        }
        _ => {
            println!("  cargo test");
            println!("  cargo run");
        }
    }

    Ok(())
}

/// Convert a string to title case (e.g., "kraken" -> "Kraken")
fn to_title_case(s: &str) -> String {
    s.split(|c| c == '-' || c == '_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => first.to_uppercase().chain(chars).collect(),
            }
        })
        .collect::<Vec<_>>()
        .join(" ")
}

/// Generate code from LLM prompts (not yet implemented)
fn forge_code(_prompt: &str) -> anyhow::Result<()> {
    anyhow::bail!("allframe forge is not yet implemented")
}
