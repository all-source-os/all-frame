//! AllFrame Forge - Project scaffolding library
//!
//! This library provides utilities for creating new AllFrame projects.
//! It is used by the `allframe` CLI binary.

#![deny(missing_docs)]

pub mod scaffolding;
pub mod templates;
pub mod validation;

use std::path::{Path, PathBuf};

use clap::{Parser, Subcommand};

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
        Commands::Ignite { name, all_features } => {
            ignite_project(&name, all_features)?;
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
fn ignite_project(project_path: &Path, _all_features: bool) -> anyhow::Result<()> {
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project path"))?;

    validation::validate_project_name(project_name)?;

    if project_path.exists() {
        anyhow::bail!("Directory already exists: {}", project_path.display());
    }

    std::fs::create_dir_all(project_path)?;

    scaffolding::create_directory_structure(project_path)?;
    scaffolding::generate_files(project_path, project_name)?;

    println!("AllFrame project created successfully: {}", project_name);
    println!("\nNext steps:");
    println!("  cd {}", project_name);
    println!("  cargo test");
    println!("  cargo run");

    Ok(())
}

/// Generate code from LLM prompts (not yet implemented)
fn forge_code(_prompt: &str) -> anyhow::Result<()> {
    anyhow::bail!("allframe forge is not yet implemented")
}
