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
//! - `producer`: Event producer service with outbox pattern and transactional
//!   messaging
//! - `bff`: Backend for Frontend API aggregation service
//! - `scheduled`: Scheduled jobs service with cron-based task execution
//! - `websocket-gateway`: WebSocket gateway for real-time bidirectional
//!   communication
//! - `saga-orchestrator`: Saga orchestrator for distributed transaction
//!   coordination
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

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand, ValueEnum};
pub use config::{Archetype, ProjectConfig};

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

#[derive(Subcommand)]
enum SagaCommands {
    /// Create a new saga with specified steps
    New {
        /// Name of the saga to create
        name: String,

        /// Comma-separated list of step names
        #[arg(short, long)]
        steps: String,

        /// Base path for sagas (default: src/application/cqrs/sagas)
        #[arg(long, default_value = "src/application/cqrs/sagas")]
        path: PathBuf,
    },
    /// Add a step to an existing saga
    AddStep {
        /// Name of the saga to modify
        saga: String,

        /// Name of the step to add
        name: String,

        /// Position to insert the step (first, last, after:<step>,
        /// before:<step>)
        #[arg(short, long, default_value = "last")]
        position: String,

        /// Step timeout in seconds
        #[arg(short, long, default_value = "30")]
        timeout: u64,

        /// Whether the step requires compensation
        #[arg(long)]
        requires_compensation: bool,

        /// Base path for sagas (default: src/application/cqrs/sagas)
        #[arg(long, default_value = "src/application/cqrs/sagas")]
        path: PathBuf,
    },
    /// Validate saga implementation
    Validate {
        /// Name of the saga to validate
        name: String,

        /// Base path for sagas (default: src/application/cqrs/sagas)
        #[arg(long, default_value = "src/application/cqrs/sagas")]
        path: PathBuf,
    },
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

        /// Service name (e.g., "kraken" for gateway, "order-processor" for
        /// consumer)
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
    /// Saga generation and management commands
    Saga {
        #[command(subcommand)]
        command: SagaCommands,
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
        Commands::Saga { command } => {
            handle_saga_command(command)?;
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

/// Handle saga-related commands
fn handle_saga_command(command: SagaCommands) -> anyhow::Result<()> {
    match command {
        SagaCommands::New { name, steps, path } => {
            saga_new(&name, &steps, &path)?;
        }
        SagaCommands::AddStep {
            saga,
            name,
            position,
            timeout,
            requires_compensation,
            path,
        } => {
            saga_add_step(
                &saga,
                &name,
                &position,
                timeout,
                requires_compensation,
                &path,
            )?;
        }
        SagaCommands::Validate { name, path } => {
            saga_validate(&name, &path)?;
        }
    }
    Ok(())
}

/// Create a new saga with specified steps
fn saga_new(name: &str, steps: &str, base_path: &Path) -> anyhow::Result<()> {
    println!("Creating saga '{}' with steps: {}", name, steps);
    println!("Base path: {}", base_path.display());

    // Parse steps from comma-separated string
    let step_list: Vec<&str> = steps.split(',').map(|s| s.trim()).collect();

    // Create saga directory if it doesn't exist
    let saga_path = base_path.join(name.to_lowercase());
    std::fs::create_dir_all(&saga_path)?;

    // Generate saga files
    generate_saga_files(&saga_path, name, &step_list)?;

    println!("Saga '{}' created successfully!", name);
    Ok(())
}

/// Add a step to an existing saga
fn saga_add_step(
    saga: &str,
    step_name: &str,
    position: &str,
    timeout: u64,
    requires_compensation: bool,
    base_path: &Path,
) -> anyhow::Result<()> {
    println!(
        "Adding step '{}' to saga '{}' at position '{}'",
        step_name, saga, position
    );
    println!(
        "Timeout: {}s, Requires compensation: {}",
        timeout, requires_compensation
    );

    let saga_path = base_path.join(saga.to_lowercase());
    if !saga_path.exists() {
        anyhow::bail!("Saga '{}' not found at {}", saga, saga_path.display());
    }

    // Generate step file
    generate_step_file(&saga_path, step_name, timeout, requires_compensation)?;

    println!(
        "Step '{}' added to saga '{}' successfully!",
        step_name, saga
    );
    Ok(())
}

/// Validate saga implementation
fn saga_validate(name: &str, base_path: &Path) -> anyhow::Result<()> {
    println!("Validating saga '{}'...", name);

    let saga_path = base_path.join(name.to_lowercase());
    if !saga_path.exists() {
        anyhow::bail!("Saga '{}' not found at {}", name, saga_path.display());
    }

    // Basic validation - check if files exist
    let mod_file = saga_path.join("mod.rs");
    let saga_file = saga_path.join(format!("{}.rs", name.to_lowercase()));

    if !mod_file.exists() {
        println!("⚠️  Missing mod.rs file");
    } else {
        println!("✅ mod.rs found");
    }

    if !saga_file.exists() {
        println!("⚠️  Missing saga implementation file");
    } else {
        println!("✅ Saga implementation file found");
    }

    println!("Saga '{}' validation completed!", name);
    Ok(())
}

/// Generate saga files
fn generate_saga_files(saga_path: &Path, name: &str, steps: &[&str]) -> anyhow::Result<()> {
    // Generate mod.rs
    let step_mods = steps
        .iter()
        .map(|step| format!("pub mod {};", step.to_lowercase()))
        .collect::<Vec<_>>()
        .join("\n");

    let mod_content = format!(
        "//! {} saga implementation
//!
//! This module contains the {} saga and its associated steps.

pub mod {};
{}",
        name,
        name,
        name.to_lowercase(),
        step_mods
    );
    std::fs::write(saga_path.join("mod.rs"), mod_content)?;

    // Generate saga implementation file
    let saga_file_name = format!("{}.rs", name.to_lowercase());
    let saga_content = generate_saga_content(name, steps);
    std::fs::write(saga_path.join(saga_file_name), saga_content)?;

    // Generate step files
    for step in steps {
        generate_step_file(saga_path, step, 30, true)?;
    }

    Ok(())
}

/// Generate saga implementation content
fn generate_saga_content(name: &str, steps: &[&str]) -> String {
    let data_struct_name = format!("{}Data", name);
    let _workflow_enum_name = format!("{}Workflow", name);

    let step_imports = steps
        .iter()
        .map(|step| format!("use super::{};", step.to_lowercase()))
        .collect::<Vec<_>>()
        .join("\n");

    let workflow_variants = steps
        .iter()
        .enumerate()
        .map(|(i, step)| format!("    /// Step {}: {}\n    {},", i + 1, step, step))
        .collect::<Vec<_>>()
        .join("\n");

    format!(
        "//! {} Saga Implementation
//!
//! This file contains the {} saga implementation using AllFrame macros.

use std::sync::Arc;
use serde::{{Deserialize, Serialize}};
use allframe_core::cqrs::{{Saga, MacroSagaStep}};

{}

// Saga data structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {} {{
    pub user_id: String,
    // Add saga-specific data fields here
}}

// Saga implementation
#[allframe_macros::saga(name = \"{}\", data_field = \"data\")]
pub struct {} {{
    #[saga_data]
    data: {},

    // Add dependency injections here
    // #[inject] repository: Arc<dyn SomeRepository>,
}}

// Saga workflow
#[allframe_macros::saga_workflow({})]
pub enum {} {{
{}
}}

// Step constructor implementations
impl {} {{
{}
}}
",
        name,
        name,
        step_imports,
        data_struct_name,
        name,
        name,
        data_struct_name,
        name,
        name,
        workflow_variants,
        name,
        steps
            .iter()
            .map(|step| {
                let step_struct = format!("{}Step", step);
                let constructor = format!("create_{}_step", step.to_lowercase());
                format!(
                    "    pub fn {}(&self) -> Arc<dyn MacroSagaStep> {{
        Arc::new({} {{
            // TODO: Initialize step with saga dependencies
            user_id: self.data.user_id.clone(),
            // Add other dependencies from saga fields
        }})
    }}",
                    constructor, step_struct
                )
            })
            .collect::<Vec<_>>()
            .join("\n\n")
    )
}

/// Generate step file content
fn generate_step_file(
    saga_path: &Path,
    step_name: &str,
    _timeout: u64,
    requires_compensation: bool,
) -> anyhow::Result<()> {
    let file_name = format!("{}.rs", step_name.to_lowercase());
    let struct_name = format!("{}Step", step_name);

    let compensation_attr = if requires_compensation {
        ""
    } else {
        ", requires_compensation = false"
    };

    let content = format!(
        "//! {} Step Implementation

use std::sync::Arc;
use serde::{{Deserialize, Serialize}};
use allframe_core::cqrs::{{MacroSagaStep, SagaContext, StepExecutionResult}};

#[derive(Serialize, Deserialize)]
pub struct {}Output {{
    // Define step output fields here
    pub success: bool,
}}

// Step implementation
#[allframe_macros::saga_step(name = \"{}\"{}))]
pub struct {} {{
    pub user_id: String,
    // Add step-specific fields and injected dependencies here
}}

impl {} {{
    pub fn new(user_id: String) -> Self {{
        Self {{ user_id }}
    }}

    async fn execute(&self, _ctx: &SagaContext) -> StepExecutionResult {{
        // TODO: Implement step execution logic
        println!(\"Executing step: {}\");

        // Return success with output
        {}Output {{
            success: true,
        }}.into()
    }}
}}
",
        step_name,
        step_name,
        step_name,
        compensation_attr,
        struct_name,
        struct_name,
        step_name,
        step_name
    );

    std::fs::write(saga_path.join(file_name), content)?;
    Ok(())
}

/// Generate code from LLM prompts (not yet implemented)
fn forge_code(_prompt: &str) -> anyhow::Result<()> {
    anyhow::bail!("allframe forge is not yet implemented")
}
