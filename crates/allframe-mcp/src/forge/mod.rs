//! Forge MCP - AI-assisted code generation for AllFrame projects
//!
//! This module provides MCP tools and resources for generating code
//! within existing AllFrame projects using Clean Architecture patterns.
//!
//! # Features
//!
//! - **Project Analysis**: Understand existing project structure
//! - **Code Generation**: Add entities, services, handlers following patterns
//! - **Architecture Validation**: Ensure Clean Architecture rules are followed
//!
//! # Example
//!
//! ```rust,no_run
//! use allframe_mcp::forge::ForgeMcpServer;
//! use std::path::PathBuf;
//!
//! # async fn example() -> Result<(), Box<dyn std::error::Error>> {
//! let server = ForgeMcpServer::new(PathBuf::from("./my-project"))?;
//! // server.serve_stdio().await?;
//! # Ok(())
//! # }
//! ```

mod analyzer;
mod generator;
mod server;

pub use analyzer::{ProjectAnalyzer, ProjectStructure, Layer, Entity, Service, Handler};
pub use generator::CodeGenerator;
pub use server::ForgeMcpServer;
