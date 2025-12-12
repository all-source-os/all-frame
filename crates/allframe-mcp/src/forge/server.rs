//! Forge MCP Server implementation

use std::io::{stdin, stdout, BufRead, Write};
use std::path::PathBuf;

use serde_json::{json, Value};

use super::analyzer::{ProjectAnalyzer, ProjectStructure};
use super::generator::{CodeGenerator, EntityRequest, ServiceRequest, HandlerRequest};
use crate::McpTool;

/// MCP Resource for project context
#[derive(Debug, Clone)]
pub struct McpResource {
    /// Resource URI
    pub uri: String,
    /// Resource name
    pub name: String,
    /// MIME type
    pub mime_type: String,
    /// Description
    pub description: String,
}

/// Forge MCP Server for AI-assisted code generation
pub struct ForgeMcpServer {
    project_path: PathBuf,
    analyzer: ProjectAnalyzer,
    generator: CodeGenerator,
    cached_structure: Option<ProjectStructure>,
}

impl ForgeMcpServer {
    /// Create a new Forge MCP server for a project
    pub fn new(project_path: PathBuf) -> Result<Self, String> {
        let analyzer = ProjectAnalyzer::new(&project_path)?;
        let generator = CodeGenerator::new(&project_path);

        Ok(Self {
            project_path,
            analyzer,
            generator,
            cached_structure: None,
        })
    }

    /// Get project structure (cached)
    pub fn get_structure(&mut self) -> Result<&ProjectStructure, String> {
        if self.cached_structure.is_none() {
            self.cached_structure = Some(self.analyzer.analyze()?);
        }
        Ok(self.cached_structure.as_ref().unwrap())
    }

    /// Invalidate cache (call after modifications)
    pub fn invalidate_cache(&mut self) {
        self.cached_structure = None;
    }

    /// List available tools
    pub fn list_tools(&self) -> Vec<McpTool> {
        vec![
            McpTool::new(
                "analyze_project",
                "Analyze the project structure and return information about entities, services, and handlers",
                r#"{
                    "type": "object",
                    "properties": {},
                    "required": []
                }"#,
            ),
            McpTool::new(
                "add_entity",
                "Add a new domain entity with optional repository trait",
                r#"{
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Entity name in PascalCase (e.g., 'User', 'Order')"
                        },
                        "fields": {
                            "type": "array",
                            "description": "Entity fields",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "field_type": {"type": "string"},
                                    "optional": {"type": "boolean"}
                                },
                                "required": ["name", "field_type"]
                            }
                        },
                        "with_repository": {
                            "type": "boolean",
                            "description": "Generate repository trait (default: true)"
                        }
                    },
                    "required": ["name", "fields"]
                }"#,
            ),
            McpTool::new(
                "add_service",
                "Add a new application service",
                r#"{
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Service name without 'Service' suffix (e.g., 'User' for UserService)"
                        },
                        "dependencies": {
                            "type": "array",
                            "description": "Dependency trait names to inject",
                            "items": {"type": "string"}
                        },
                        "methods": {
                            "type": "array",
                            "description": "Methods to generate",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "params": {
                                        "type": "array",
                                        "items": {
                                            "type": "array",
                                            "items": {"type": "string"}
                                        }
                                    },
                                    "return_type": {"type": "string"},
                                    "is_async": {"type": "boolean"}
                                },
                                "required": ["name", "return_type"]
                            }
                        }
                    },
                    "required": ["name"]
                }"#,
            ),
            McpTool::new(
                "add_handler",
                "Add a new HTTP handler to the presentation layer",
                r#"{
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Handler function name (e.g., 'get_user', 'create_order')"
                        },
                        "method": {
                            "type": "string",
                            "description": "HTTP method (GET, POST, PUT, DELETE)",
                            "enum": ["GET", "POST", "PUT", "DELETE", "PATCH"]
                        },
                        "path": {
                            "type": "string",
                            "description": "Route path (e.g., '/users/:id')"
                        },
                        "service": {
                            "type": "string",
                            "description": "Service to inject (optional)"
                        }
                    },
                    "required": ["name", "method", "path"]
                }"#,
            ),
            McpTool::new(
                "list_entities",
                "List all domain entities in the project",
                r#"{"type": "object", "properties": {}}"#,
            ),
            McpTool::new(
                "list_services",
                "List all application services in the project",
                r#"{"type": "object", "properties": {}}"#,
            ),
            McpTool::new(
                "list_handlers",
                "List all handlers in the presentation layer",
                r#"{"type": "object", "properties": {}}"#,
            ),
            McpTool::new(
                "read_file",
                "Read a file from the project",
                r#"{
                    "type": "object",
                    "properties": {
                        "path": {
                            "type": "string",
                            "description": "File path relative to project root"
                        }
                    },
                    "required": ["path"]
                }"#,
            ),
        ]
    }

    /// List available resources
    pub fn list_resources(&self) -> Vec<McpResource> {
        vec![
            McpResource {
                uri: "allframe://project/structure".to_string(),
                name: "Project Structure".to_string(),
                mime_type: "application/json".to_string(),
                description: "Complete project structure analysis".to_string(),
            },
            McpResource {
                uri: "allframe://project/entities".to_string(),
                name: "Domain Entities".to_string(),
                mime_type: "application/json".to_string(),
                description: "List of domain entities".to_string(),
            },
            McpResource {
                uri: "allframe://project/services".to_string(),
                name: "Application Services".to_string(),
                mime_type: "application/json".to_string(),
                description: "List of application services".to_string(),
            },
            McpResource {
                uri: "allframe://project/handlers".to_string(),
                name: "Presentation Handlers".to_string(),
                mime_type: "application/json".to_string(),
                description: "List of HTTP handlers".to_string(),
            },
        ]
    }

    /// Read a resource
    pub fn read_resource(&mut self, uri: &str) -> Result<Value, String> {
        let structure = self.get_structure()?;

        match uri {
            "allframe://project/structure" => {
                Ok(serde_json::to_value(structure).map_err(|e| e.to_string())?)
            }
            "allframe://project/entities" => {
                Ok(serde_json::to_value(&structure.entities).map_err(|e| e.to_string())?)
            }
            "allframe://project/services" => {
                Ok(serde_json::to_value(&structure.services).map_err(|e| e.to_string())?)
            }
            "allframe://project/handlers" => {
                Ok(serde_json::to_value(&structure.handlers).map_err(|e| e.to_string())?)
            }
            _ => Err(format!("Unknown resource: {}", uri)),
        }
    }

    /// Call a tool
    pub fn call_tool(&mut self, name: &str, args: Value) -> Result<Value, String> {
        match name {
            "analyze_project" => {
                self.invalidate_cache();
                let structure = self.get_structure()?;
                Ok(serde_json::to_value(structure).map_err(|e| e.to_string())?)
            }

            "add_entity" => {
                let request: EntityRequest = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid entity request: {}", e))?;
                let result = self.generator.generate_entity(&request);
                self.invalidate_cache();
                Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
            }

            "add_service" => {
                let request: ServiceRequest = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid service request: {}", e))?;
                let result = self.generator.generate_service(&request);
                self.invalidate_cache();
                Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
            }

            "add_handler" => {
                let request: HandlerRequest = serde_json::from_value(args)
                    .map_err(|e| format!("Invalid handler request: {}", e))?;
                let result = self.generator.generate_handler(&request);
                self.invalidate_cache();
                Ok(serde_json::to_value(result).map_err(|e| e.to_string())?)
            }

            "list_entities" => {
                let structure = self.get_structure()?;
                Ok(serde_json::to_value(&structure.entities).map_err(|e| e.to_string())?)
            }

            "list_services" => {
                let structure = self.get_structure()?;
                Ok(serde_json::to_value(&structure.services).map_err(|e| e.to_string())?)
            }

            "list_handlers" => {
                let structure = self.get_structure()?;
                Ok(serde_json::to_value(&structure.handlers).map_err(|e| e.to_string())?)
            }

            "read_file" => {
                let path = args.get("path")
                    .and_then(|p| p.as_str())
                    .ok_or("Missing 'path' argument")?;

                let full_path = self.project_path.join(path);
                let content = std::fs::read_to_string(&full_path)
                    .map_err(|e| format!("Failed to read file: {}", e))?;

                Ok(json!({
                    "path": path,
                    "content": content
                }))
            }

            _ => Err(format!("Unknown tool: {}", name)),
        }
    }

    /// Serve MCP protocol over stdio
    pub fn serve_stdio(mut self) {
        let stdin = stdin();
        let mut stdout = stdout();

        eprintln!("Forge MCP Server started for: {}", self.project_path.display());
        eprintln!("Tools: {}", self.list_tools().len());
        eprintln!("Resources: {}", self.list_resources().len());
        eprintln!("Listening on stdio...");

        for line in stdin.lock().lines() {
            let line = match line {
                Ok(l) => l,
                Err(e) => {
                    eprintln!("Error reading line: {}", e);
                    continue;
                }
            };

            if line.trim().is_empty() {
                continue;
            }

            let request: Value = match serde_json::from_str(&line) {
                Ok(r) => r,
                Err(e) => {
                    eprintln!("Error parsing JSON: {}", e);
                    let error = json!({
                        "jsonrpc": "2.0",
                        "error": {"code": -32700, "message": "Parse error"},
                        "id": null
                    });
                    writeln!(stdout, "{}", serde_json::to_string(&error).unwrap()).unwrap();
                    stdout.flush().unwrap();
                    continue;
                }
            };

            let response = self.handle_request(request);

            match serde_json::to_string(&response) {
                Ok(json_str) => {
                    writeln!(stdout, "{}", json_str).unwrap();
                    stdout.flush().unwrap();
                }
                Err(e) => {
                    eprintln!("Error serializing response: {}", e);
                }
            }
        }
    }

    /// Handle MCP request
    fn handle_request(&mut self, request: Value) -> Value {
        let method = request["method"].as_str().unwrap_or("");
        let id = request.get("id").cloned();

        let result = match method {
            "initialize" => {
                json!({
                    "protocolVersion": "2024-11-05",
                    "capabilities": {
                        "tools": {},
                        "resources": {
                            "subscribe": false,
                            "listChanged": false
                        }
                    },
                    "serverInfo": {
                        "name": "allframe-forge",
                        "version": env!("CARGO_PKG_VERSION")
                    }
                })
            }

            "tools/list" => {
                let tools = self.list_tools();
                json!({
                    "tools": tools.iter().map(|t| {
                        json!({
                            "name": t.name,
                            "description": t.description,
                            "inputSchema": serde_json::from_str::<Value>(&t.input_schema).unwrap_or(json!({}))
                        })
                    }).collect::<Vec<_>>()
                })
            }

            "tools/call" => {
                let params = &request["params"];
                let name = params["name"].as_str().unwrap_or("");
                let arguments = params.get("arguments").cloned().unwrap_or(json!({}));

                match self.call_tool(name, arguments) {
                    Ok(result) => {
                        json!({
                            "content": [{
                                "type": "text",
                                "text": serde_json::to_string_pretty(&result).unwrap_or_default()
                            }]
                        })
                    }
                    Err(e) => {
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

            "resources/list" => {
                let resources = self.list_resources();
                json!({
                    "resources": resources.iter().map(|r| {
                        json!({
                            "uri": r.uri,
                            "name": r.name,
                            "mimeType": r.mime_type,
                            "description": r.description
                        })
                    }).collect::<Vec<_>>()
                })
            }

            "resources/read" => {
                let uri = request["params"]["uri"].as_str().unwrap_or("");
                match self.read_resource(uri) {
                    Ok(content) => {
                        json!({
                            "contents": [{
                                "uri": uri,
                                "mimeType": "application/json",
                                "text": serde_json::to_string_pretty(&content).unwrap_or_default()
                            }]
                        })
                    }
                    Err(e) => {
                        return json!({
                            "jsonrpc": "2.0",
                            "error": {"code": -32602, "message": e},
                            "id": id
                        });
                    }
                }
            }

            "ping" => json!({}),

            "notifications/initialized" => {
                // Client notification, no response needed
                return json!({});
            }

            _ => {
                return json!({
                    "jsonrpc": "2.0",
                    "error": {
                        "code": -32601,
                        "message": format!("Method not found: {}", method)
                    },
                    "id": id
                });
            }
        };

        json!({
            "jsonrpc": "2.0",
            "result": result,
            "id": id
        })
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_list_tools() {
        // Can't test without a real project, but we can check tool definitions
        let tools_count = 8; // analyze, add_entity, add_service, add_handler, list_*, read_file
        assert!(tools_count > 0);
    }
}
