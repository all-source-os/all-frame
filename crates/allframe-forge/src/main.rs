//! AllFrame CLI - Project scaffolding and code generation
//!
//! This binary provides the `allframe` command-line tool for:
//! - Creating new AllFrame projects (`allframe ignite`)
//! - Generating code from LLM prompts (`allframe forge`)
//! - Running migrations and deployments

mod scaffolding;
mod templates;
mod validation;

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

fn main() -> anyhow::Result<()> {
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
///
/// # Arguments
/// * `project_path` - Path where the project will be created
/// * `_all_features` - Whether to enable all features (currently unused)
///
/// # Errors
/// Returns an error if:
/// - Project name is invalid
/// - Directory already exists
/// - File system operations fail
fn ignite_project(project_path: &Path, _all_features: bool) -> anyhow::Result<()> {
    // Extract and validate project name
    let project_name = project_path
        .file_name()
        .and_then(|n| n.to_str())
        .ok_or_else(|| anyhow::anyhow!("Invalid project path"))?;

    validation::validate_project_name(project_name)?;

    // Check if directory already exists
    if project_path.exists() {
        anyhow::bail!("Directory already exists: {}", project_path.display());
    }

    // Create project directory
    std::fs::create_dir_all(project_path)?;

    // Generate project structure
    scaffolding::create_directory_structure(project_path)?;
    scaffolding::generate_files(project_path, project_name)?;

    // Success message
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parses() {
        let cli = Cli::parse_from(["allframe", "ignite", "test-project"]);
        match cli.command {
            Commands::Ignite { name, .. } => {
                assert_eq!(name, PathBuf::from("test-project"));
            }
            _ => panic!("Expected Ignite command"),
        }
    }
}
