//! Forge MCP Server - AI-assisted code generation
//!
//! This MCP server enables AI assistants to generate code in AllFrame projects.
//!
//! # Usage
//!
//! ```bash
//! # Run for current directory
//! cargo run --example forge_mcp_server
//!
//! # Run for specific project
//! cargo run --example forge_mcp_server -- /path/to/project
//! ```
//!
//! # Claude Desktop Configuration
//!
//! Add to `~/Library/Application Support/Claude/claude_desktop_config.json`:
//!
//! ```json
//! {
//!   "mcpServers": {
//!     "allframe-forge": {
//!       "command": "/path/to/target/debug/examples/forge_mcp_server",
//!       "args": ["/path/to/your/allframe/project"]
//!     }
//!   }
//! }
//! ```
//!
//! # Available Tools
//!
//! - `analyze_project` - Analyze project structure
//! - `add_entity` - Add domain entity with repository
//! - `add_service` - Add application service
//! - `add_handler` - Add HTTP handler
//! - `list_entities` - List domain entities
//! - `list_services` - List application services
//! - `list_handlers` - List presentation handlers
//! - `read_file` - Read project file
//! - `create_saga` - Create a new saga with specified steps
//! - `add_saga_step` - Add a step to an existing saga
//! - `analyze_saga` - Analyze a saga for issues and best practices
//!
//! # Available Resources
//!
//! - `allframe://project/structure` - Complete project structure
//! - `allframe://project/entities` - Domain entities
//! - `allframe://project/services` - Application services
//! - `allframe://project/handlers` - Presentation handlers
//! - `sagas://registry` - List of all registered sagas
//! - `saga://{name}` - Saga definition, steps, and implementation
//! - `saga://{name}/steps` - List of steps with their status
//! - `saga://{name}/step/{step_name}` - Step implementation details

use std::env;
use std::path::PathBuf;

use allframe_mcp::forge::ForgeMcpServer;

fn main() {
    env_logger::init();

    // Get project path from args or use current directory
    let project_path = env::args()
        .nth(1)
        .map(PathBuf::from)
        .unwrap_or_else(|| env::current_dir().expect("Failed to get current directory"));

    eprintln!("AllFrame Forge MCP Server");
    eprintln!("========================");
    eprintln!("Project: {}", project_path.display());

    match ForgeMcpServer::new(project_path) {
        Ok(server) => {
            server.serve_stdio();
        }
        Err(e) => {
            eprintln!("Error: {}", e);
            std::process::exit(1);
        }
    }
}
