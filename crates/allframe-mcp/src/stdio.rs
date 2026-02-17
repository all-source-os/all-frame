//! STDIO transport for MCP Server with debugging support
//!
//! This module provides a production-ready stdio transport with:
//! - Structured logging via tracing
//! - Request/response tracing for debugging
//! - Graceful shutdown handling
//! - Built-in diagnostic tools
//!
//! # Usage
//!
//! ```rust,no_run
//! use allframe_core::router::Router;
//! use allframe_mcp::{McpServer, StdioTransport, StdioConfig};
//!
//! #[tokio::main]
//! async fn main() {
//!     let router = Router::new();
//!     let mcp = McpServer::new(router);
//!
//!     let config = StdioConfig::default()
//!         .with_debug_tool(true);
//!
//!     StdioTransport::new(mcp, config)
//!         .serve()
//!         .await;
//! }
//! ```

use std::{
    io::{stdin, stdout, BufRead, Write},
    sync::atomic::{AtomicU64, Ordering},
    time::Instant,
};

use serde_json::{json, Value};

use crate::McpServer;

/// Configuration for the STDIO transport
#[derive(Debug, Clone)]
pub struct StdioConfig {
    /// Server name reported in initialize response
    pub server_name: String,
    /// Server version reported in initialize response
    pub server_version: String,
    /// Protocol version to advertise
    pub protocol_version: String,
    /// Whether to include the allframe/debug tool
    pub include_debug_tool: bool,
    /// Log file path (if set, logs go to file instead of stderr)
    pub log_file: Option<String>,
}

impl Default for StdioConfig {
    fn default() -> Self {
        Self {
            server_name: "allframe-mcp".to_string(),
            server_version: env!("CARGO_PKG_VERSION").to_string(),
            protocol_version: "2024-11-05".to_string(),
            include_debug_tool: false,
            log_file: std::env::var("ALLFRAME_MCP_LOG_FILE").ok(),
        }
    }
}

impl StdioConfig {
    /// Enable the built-in debug tool
    pub fn with_debug_tool(mut self, enabled: bool) -> Self {
        self.include_debug_tool = enabled;
        self
    }

    /// Set the server name
    pub fn with_server_name(mut self, name: impl Into<String>) -> Self {
        self.server_name = name.into();
        self
    }

    /// Set a log file path
    pub fn with_log_file(mut self, path: impl Into<String>) -> Self {
        self.log_file = Some(path.into());
        self
    }
}

/// STDIO transport for MCP server with debugging support
pub struct StdioTransport {
    mcp: McpServer,
    config: StdioConfig,
    start_time: Instant,
    request_count: AtomicU64,
}

impl StdioTransport {
    /// Create a new STDIO transport
    pub fn new(mcp: McpServer, config: StdioConfig) -> Self {
        Self {
            mcp,
            config,
            start_time: Instant::now(),
            request_count: AtomicU64::new(0),
        }
    }

    /// Serve MCP protocol over stdio
    pub async fn serve(self) {
        self.log_startup();

        let stdin = stdin();
        let mut stdout = stdout();

        // Set up shutdown signal handling
        let shutdown = async {
            #[cfg(unix)]
            {
                use tokio::signal::unix::{signal, SignalKind};
                let mut sigterm = signal(SignalKind::terminate()).ok();
                let mut sigint = signal(SignalKind::interrupt()).ok();

                tokio::select! {
                    _ = async { if let Some(ref mut s) = sigterm { s.recv().await } else { std::future::pending().await } } => {
                        self.log_info("Received SIGTERM");
                    }
                    _ = async { if let Some(ref mut s) = sigint { s.recv().await } else { std::future::pending().await } } => {
                        self.log_info("Received SIGINT");
                    }
                }
            }
            #[cfg(not(unix))]
            {
                tokio::signal::ctrl_c().await.ok();
                self.log_info("Received shutdown signal");
            }
        };

        // Run the main loop with shutdown handling
        tokio::select! {
            _ = self.run_loop(&stdin, &mut stdout) => {}
            _ = shutdown => {
                self.log_info("Shutting down gracefully");
            }
        }

        self.log_shutdown();
    }

    async fn run_loop(&self, stdin: &std::io::Stdin, stdout: &mut std::io::Stdout) {
        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    self.log_error(&format!("Error reading line: {}", e));
                    continue;
                }
            };

            // Skip empty lines
            if line.trim().is_empty() {
                continue;
            }

            self.request_count.fetch_add(1, Ordering::SeqCst);
            let request_id = self.request_count.load(Ordering::SeqCst);

            self.log_request(request_id, &line);

            // Parse request
            let request: Value = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    self.log_error(&format!("Parse error: {}", e));
                    let error = json!({
                        "jsonrpc": "2.0",
                        "error": {
                            "code": -32700,
                            "message": "Parse error"
                        },
                        "id": null
                    });
                    self.write_response(stdout, &error, request_id);
                    continue;
                }
            };

            // Handle request
            let response = self.handle_request(request).await;

            // Check if this was a notification (no response needed)
            if let Some(resp) = response {
                self.write_response(stdout, &resp, request_id);
            }
        }
    }

    fn write_response(&self, stdout: &mut std::io::Stdout, response: &Value, request_id: u64) {
        match serde_json::to_string(&response) {
            Ok(json_str) => {
                self.log_response(request_id, &json_str);
                if let Err(e) = writeln!(stdout, "{}", json_str) {
                    self.log_error(&format!("Error writing response: {}", e));
                }
                if let Err(e) = stdout.flush() {
                    self.log_error(&format!("Error flushing stdout: {}", e));
                }
            }
            Err(e) => {
                self.log_error(&format!("Error serializing response: {}", e));
            }
        }
    }

    async fn handle_request(&self, request: Value) -> Option<Value> {
        let method = request["method"].as_str().unwrap_or("");
        let id = request.get("id").cloned();

        // Handle notifications (no id = notification, no response needed)
        match method {
            // Notifications that don't require responses
            "initialized" | "notifications/initialized" => {
                self.log_info("Client initialized connection");
                return None;
            }
            "notifications/cancelled" => {
                self.log_info("Request cancelled by client");
                return None;
            }
            _ => {}
        }

        let result = match method {
            // Initialize
            "initialize" => {
                self.log_info("Initializing MCP connection");
                json!({
                    "protocolVersion": self.config.protocol_version,
                    "capabilities": {
                        "tools": {}
                    },
                    "serverInfo": {
                        "name": self.config.server_name,
                        "version": self.config.server_version
                    }
                })
            }

            // List available tools
            "tools/list" => {
                let mut tools: Vec<Value> = self
                    .mcp
                    .list_tools()
                    .await
                    .iter()
                    .map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": serde_json::from_str::<Value>(&t.input_schema)
                                .unwrap_or_else(|_| json!({"type": "object"}))
                        })
                    })
                    .collect();

                // Add debug tool if enabled
                if self.config.include_debug_tool {
                    tools.push(json!({
                        "name": "allframe/debug",
                        "description": "Get AllFrame MCP server diagnostics and status information",
                        "inputSchema": {
                            "type": "object",
                            "properties": {},
                            "additionalProperties": false
                        }
                    }));
                }

                json!({ "tools": tools })
            }

            // Call a tool
            "tools/call" => {
                let params = &request["params"];
                let name = params["name"].as_str().unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                self.log_info(&format!("Calling tool: {}", name));

                // Handle built-in debug tool
                if name == "allframe/debug" && self.config.include_debug_tool {
                    let diagnostics = self.get_diagnostics();
                    return Some(json!({
                        "jsonrpc": "2.0",
                        "result": {
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&diagnostics).unwrap()
                            }]
                        },
                        "id": id
                    }));
                }

                match self.mcp.call_tool(name, arguments).await {
                    Ok(result) => {
                        json!({
                            "content": [{
                                "type": "text",
                                "text": result.to_string()
                            }]
                        })
                    }
                    Err(e) => {
                        self.log_error(&format!("Tool error: {}", e));
                        json!({
                            "isError": true,
                            "content": [{
                                "type": "text",
                                "text": format!("Error: {}", e)
                            }]
                        })
                    }
                }
            }

            // Ping
            "ping" => {
                json!({})
            }

            // Unknown method
            _ => {
                self.log_warn(&format!("Unknown method: {}", method));
                return Some(json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    },
                    "id": id
                }));
            }
        };

        // Return successful response
        Some(json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        }))
    }

    fn get_diagnostics(&self) -> Value {
        json!({
            "server": {
                "name": self.config.server_name,
                "version": self.config.server_version,
                "protocol_version": self.config.protocol_version
            },
            "runtime": {
                "uptime_seconds": self.start_time.elapsed().as_secs(),
                "request_count": self.request_count.load(Ordering::SeqCst),
                "tool_count": self.mcp.tool_count(),
                "pid": std::process::id()
            },
            "build": {
                "pkg_version": env!("CARGO_PKG_VERSION"),
                "debug_tool_enabled": self.config.include_debug_tool
            }
        })
    }

    // Logging methods that work with or without tracing feature

    fn log_startup(&self) {
        let msg = format!(
            "MCP Server starting: name={}, version={}, pid={}, tools={}",
            self.config.server_name,
            self.config.server_version,
            std::process::id(),
            self.mcp.tool_count()
        );

        #[cfg(feature = "tracing")]
        tracing::info!("{}", msg);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[INFO] {}", msg);
    }

    fn log_shutdown(&self) {
        let msg = format!(
            "MCP Server shutting down: uptime={}s, requests={}",
            self.start_time.elapsed().as_secs(),
            self.request_count.load(Ordering::SeqCst)
        );

        #[cfg(feature = "tracing")]
        tracing::info!("{}", msg);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[INFO] {}", msg);
    }

    fn log_request(&self, id: u64, content: &str) {
        // Truncate long requests for logging
        let truncated = if content.len() > 500 {
            format!("{}...(truncated)", &content[..500])
        } else {
            content.to_string()
        };

        #[cfg(feature = "tracing")]
        tracing::debug!(request_id = id, request = %truncated, "Received MCP request");

        #[cfg(not(feature = "tracing"))]
        if std::env::var("ALLFRAME_MCP_DEBUG").is_ok() {
            eprintln!("[DEBUG] req#{}: {}", id, truncated);
        }
    }

    fn log_response(&self, id: u64, content: &str) {
        let truncated = if content.len() > 500 {
            format!("{}...(truncated)", &content[..500])
        } else {
            content.to_string()
        };

        #[cfg(feature = "tracing")]
        tracing::debug!(request_id = id, response = %truncated, "Sending MCP response");

        #[cfg(not(feature = "tracing"))]
        if std::env::var("ALLFRAME_MCP_DEBUG").is_ok() {
            eprintln!("[DEBUG] res#{}: {}", id, truncated);
        }
    }

    fn log_info(&self, msg: &str) {
        #[cfg(feature = "tracing")]
        tracing::info!("{}", msg);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[INFO] {}", msg);
    }

    fn log_warn(&self, msg: &str) {
        #[cfg(feature = "tracing")]
        tracing::warn!("{}", msg);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[WARN] {}", msg);
    }

    fn log_error(&self, msg: &str) {
        #[cfg(feature = "tracing")]
        tracing::error!("{}", msg);

        #[cfg(not(feature = "tracing"))]
        eprintln!("[ERROR] {}", msg);
    }
}

/// Initialize tracing with file output if ALLFRAME_MCP_LOG_FILE is set
#[cfg(feature = "tracing")]
pub fn init_tracing() {
    use tracing_subscriber::EnvFilter;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));

    if let Ok(log_file) = std::env::var("ALLFRAME_MCP_LOG_FILE") {
        // Log to file
        let file = std::fs::File::create(&log_file).expect("Failed to create log file");

        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(file)
            .with_ansi(false)
            .init();
    } else {
        // Log to stderr
        tracing_subscriber::fmt()
            .with_env_filter(filter)
            .with_writer(std::io::stderr)
            .with_ansi(false)
            .init();
    }
}

#[cfg(not(feature = "tracing"))]
pub fn init_tracing() {
    // No-op when tracing feature is disabled
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_config_default() {
        let config = StdioConfig::default();
        assert_eq!(config.server_name, "allframe-mcp");
        assert!(!config.include_debug_tool);
    }

    #[test]
    fn test_config_builder() {
        let config = StdioConfig::default()
            .with_debug_tool(true)
            .with_server_name("my-server")
            .with_log_file("/tmp/mcp.log");

        assert!(config.include_debug_tool);
        assert_eq!(config.server_name, "my-server");
        assert_eq!(config.log_file, Some("/tmp/mcp.log".to_string()));
    }
}
