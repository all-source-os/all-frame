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
//!
//! # Usage
//!
//! ```bash
//! # Create a basic project
//! allframe ignite my-service
//!
//! # Create a gateway project
//! allframe ignite my-gateway --archetype gateway
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
}

impl From<CliArchetype> for Archetype {
    fn from(cli: CliArchetype) -> Self {
        match cli {
            CliArchetype::Basic => Archetype::Basic,
            CliArchetype::Gateway => Archetype::Gateway,
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

        /// Project archetype (basic, gateway)
        #[arg(short, long, value_enum, default_value_t = CliArchetype::Basic)]
        archetype: CliArchetype,

        /// Service name for gateway archetype (e.g., "kraken", "binance")
        #[arg(long)]
        service_name: Option<String>,

        /// Base URL for gateway's external API
        #[arg(long)]
        api_base_url: Option<String>,

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
            all_features: _,
        } => {
            ignite_project(&name, archetype, service_name, api_base_url)?;
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
                if let Some(svc_name) = service_name {
                    gateway.service_name = svc_name.clone();
                    gateway.display_name = format!("{} Gateway", to_title_case(&svc_name));
                } else {
                    // Default to project name
                    gateway.service_name = project_name.replace('-', "_");
                    gateway.display_name = format!("{} Gateway", to_title_case(project_name));
                }

                if let Some(url) = api_base_url {
                    gateway.api_base_url = url;
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
