//! AllFrame CLI - Project scaffolding and code generation
//!
//! This binary provides the `allframe` command-line tool for:
//! - Creating new AllFrame projects (`allframe ignite`)
//! - Generating code from LLM prompts (`allframe forge`)
//! - Running migrations and deployments

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
        name: String,

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
fn ignite_project(name: &str, _all_features: bool) -> anyhow::Result<()> {
    // RED PHASE: This implementation is intentionally incomplete
    // to make the test fail.
    //
    // TODO (GREEN PHASE):
    // 1. Validate project name
    // 2. Create project directory
    // 3. Generate Cargo.toml
    // 4. Generate src/main.rs
    // 5. Generate Clean Architecture structure
    // 6. Generate .gitignore
    // 7. Generate README.md

    anyhow::bail!(
        "allframe ignite is not yet implemented. \
         This is the RED phase of TDD - the test should fail here. \
         Project name was: {}",
        name
    );
}

/// Generate code from LLM prompts
fn forge_code(_prompt: &str) -> anyhow::Result<()> {
    anyhow::bail!("allframe forge is not yet implemented")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parses() {
        // Verify CLI structure compiles and can be parsed
        let cli = Cli::parse_from(["allframe", "ignite", "test-project"]);
        match cli.command {
            Commands::Ignite { name, .. } => {
                assert_eq!(name, "test-project");
            }
            _ => panic!("Expected Ignite command"),
        }
    }
}
