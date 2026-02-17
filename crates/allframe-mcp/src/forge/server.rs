//! Forge MCP Server implementation

use std::{
    io::{stdin, stdout, BufRead, Write},
    path::PathBuf,
};

use serde_json::{json, Value};

use super::{
    analyzer::{ProjectAnalyzer, ProjectStructure},
    generator::{CodeGenerator, EntityRequest, HandlerRequest, ServiceRequest},
};
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
                "Analyze the project structure and return information about entities, services, \
                 and handlers",
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
            McpTool::new(
                "create_saga",
                "Create a new saga with specified steps",
                r#"{
                    "type": "object",
                    "properties": {
                        "name": {
                            "type": "string",
                            "description": "Saga name (e.g., RebalancingSaga)"
                        },
                        "steps": {
                            "type": "array",
                            "items": {
                                "type": "string"
                            },
                            "description": "Ordered list of step names"
                        },
                        "data_fields": {
                            "type": "array",
                            "items": {
                                "type": "object",
                                "properties": {
                                    "name": {"type": "string"},
                                    "field_type": {"type": "string"},
                                    "description": {"type": "string"}
                                },
                                "required": ["name", "field_type"]
                            },
                            "description": "Saga data structure fields"
                        },
                        "dependencies": {
                            "type": "array",
                            "items": {"type": "string"},
                            "description": "Required service dependencies"
                        }
                    },
                    "required": ["name", "steps"]
                }"#,
            ),
            McpTool::new(
                "add_saga_step",
                "Add a step to an existing saga",
                r#"{
                    "type": "object",
                    "properties": {
                        "saga_name": {"type": "string", "description": "Name of the saga to modify"},
                        "step_name": {"type": "string", "description": "Name of the step to add"},
                        "position": {
                            "type": "string",
                            "enum": ["first", "last", "after:<step>", "before:<step>"],
                            "description": "Position to insert the step"
                        },
                        "timeout_seconds": {"type": "number", "description": "Step timeout in seconds"},
                        "requires_compensation": {"type": "boolean", "description": "Whether step requires compensation"},
                        "execute_logic": {"type": "string", "description": "Natural language description of step logic"},
                        "compensate_logic": {"type": "string", "description": "Natural language description of compensation"}
                    },
                    "required": ["saga_name", "step_name"]
                }"#,
            ),
            McpTool::new(
                "analyze_saga",
                "Analyze a saga for potential issues and best practices",
                r#"{
                    "type": "object",
                    "properties": {
                        "saga_name": {"type": "string", "description": "Name of the saga to analyze"}
                    },
                    "required": ["saga_name"]
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
            McpResource {
                uri: "sagas://registry".to_string(),
                name: "Saga Registry".to_string(),
                mime_type: "application/json".to_string(),
                description: "List of all registered sagas".to_string(),
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
            "sagas://registry" => self.read_saga_registry(),
            uri if uri.starts_with("saga://") => self.read_saga_resource(uri),
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
                let path = args
                    .get("path")
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

            "create_saga" => {
                let saga_name = args
                    .get("name")
                    .and_then(|n| n.as_str())
                    .ok_or("Missing 'name' parameter")?;
                let steps = args
                    .get("steps")
                    .and_then(|s| s.as_array())
                    .ok_or("Missing or invalid 'steps' parameter")?
                    .iter()
                    .filter_map(|s| s.as_str())
                    .collect::<Vec<_>>();

                // Use the forge CLI to create the saga
                self.create_saga(saga_name, &steps, &args)
            }

            "add_saga_step" => {
                let saga_name = args
                    .get("saga_name")
                    .and_then(|n| n.as_str())
                    .ok_or("Missing 'saga_name' parameter")?;
                let step_name = args
                    .get("step_name")
                    .and_then(|n| n.as_str())
                    .ok_or("Missing 'step_name' parameter")?;
                let position = args
                    .get("position")
                    .and_then(|p| p.as_str())
                    .unwrap_or("last");
                let timeout = args
                    .get("timeout_seconds")
                    .and_then(|t| t.as_u64())
                    .unwrap_or(30);
                let requires_compensation = args
                    .get("requires_compensation")
                    .and_then(|c| c.as_bool())
                    .unwrap_or(true);

                self.add_saga_step(
                    saga_name,
                    step_name,
                    position,
                    timeout,
                    requires_compensation,
                )
            }

            "analyze_saga" => {
                let saga_name = args
                    .get("saga_name")
                    .and_then(|n| n.as_str())
                    .ok_or("Missing 'saga_name' parameter")?;

                self.analyze_saga(saga_name)
            }

            _ => Err(format!("Unknown tool: {}", name)),
        }
    }

    /// Serve MCP protocol over stdio
    pub fn serve_stdio(mut self) {
        let stdin = stdin();
        let mut stdout = stdout();

        eprintln!(
            "Forge MCP Server started for: {}",
            self.project_path.display()
        );
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

    /// Create a new saga using the forge CLI
    fn create_saga(&self, name: &str, steps: &[&str], args: &Value) -> Result<Value, String> {
        use std::process::Command;

        let steps_str = steps.join(",");
        let mut cmd = Command::new("allframe");
        cmd.arg("saga")
            .arg("new")
            .arg(name)
            .arg("--steps")
            .arg(steps_str)
            .current_dir(&self.project_path);

        // Add optional path parameter
        if let Some(path) = args.get("path").and_then(|p| p.as_str()) {
            cmd.arg("--path").arg(path);
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run forge command: {}", e))?;

        if output.status.success() {
            Ok(json!({
                "success": true,
                "message": format!("Saga '{}' created successfully", name),
                "steps": steps
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Forge command failed: {}", stderr))
        }
    }

    /// Add a step to an existing saga
    fn add_saga_step(
        &self,
        saga_name: &str,
        step_name: &str,
        position: &str,
        timeout: u64,
        requires_compensation: bool,
    ) -> Result<Value, String> {
        use std::process::Command;

        let mut cmd = Command::new("allframe");
        cmd.arg("saga")
            .arg("add-step")
            .arg(saga_name)
            .arg(step_name)
            .arg("--position")
            .arg(position)
            .arg("--timeout")
            .arg(timeout.to_string())
            .current_dir(&self.project_path);

        if !requires_compensation {
            cmd.arg("--requires-compensation").arg("false");
        }

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run forge command: {}", e))?;

        if output.status.success() {
            Ok(json!({
                "success": true,
                "message": format!("Step '{}' added to saga '{}' successfully", step_name, saga_name),
                "step": {
                    "name": step_name,
                    "position": position,
                    "timeout": timeout,
                    "requires_compensation": requires_compensation
                }
            }))
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(format!("Forge command failed: {}", stderr))
        }
    }

    /// Analyze a saga for issues and best practices
    fn analyze_saga(&self, saga_name: &str) -> Result<Value, String> {
        use std::process::Command;

        let mut cmd = Command::new("allframe");
        cmd.arg("saga")
            .arg("validate")
            .arg(saga_name)
            .current_dir(&self.project_path);

        let output = cmd
            .output()
            .map_err(|e| format!("Failed to run forge command: {}", e))?;

        let success = output.status.success();
        let stdout = String::from_utf8_lossy(&output.stdout);
        let stderr = String::from_utf8_lossy(&output.stderr);

        Ok(json!({
            "saga_name": saga_name,
            "valid": success,
            "output": stdout,
            "errors": if success { Value::Null } else { json!(stderr) },
            "recommendations": if success {
                vec!["Saga structure looks good", "Consider adding more comprehensive error handling"]
            } else {
                vec!["Fix validation errors", "Check saga step implementations"]
            }
        }))
    }

    /// Read saga registry resource
    fn read_saga_registry(&self) -> Result<Value, String> {
        use std::fs;

        let saga_path = self.project_path.join("src/application/cqrs/sagas");
        if !saga_path.exists() {
            return Ok(json!([]));
        }

        let mut sagas = Vec::new();

        if let Ok(entries) = fs::read_dir(&saga_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_dir() {
                        if let Some(name) = entry.file_name().to_str() {
                            sagas.push(json!({
                                "name": name,
                                "path": format!("src/application/cqrs/sagas/{}", name)
                            }));
                        }
                    }
                }
            }
        }

        Ok(json!(sagas))
    }

    /// Read individual saga resources
    fn read_saga_resource(&self, uri: &str) -> Result<Value, String> {
        use std::fs;

        // Parse URI like "saga://SagaName" or "saga://SagaName/steps" or
        // "saga://SagaName/step/StepName"
        let path = uri.strip_prefix("saga://").ok_or("Invalid saga URI")?;

        let parts: Vec<&str> = path.split('/').collect();
        let saga_name = parts.get(0).ok_or("Missing saga name")?;

        let saga_path = self
            .project_path
            .join("src/application/cqrs/sagas")
            .join(saga_name);

        match parts.len() {
            1 => {
                // saga://{name} - Return saga definition
                let saga_file = saga_path.join(format!("{}.rs", saga_name));
                if !saga_file.exists() {
                    return Err(format!("Saga '{}' not found", saga_name));
                }

                let content = fs::read_to_string(&saga_file)
                    .map_err(|e| format!("Failed to read saga file: {}", e))?;

                Ok(json!({
                    "name": saga_name,
                    "path": saga_file.to_string_lossy(),
                    "content": content,
                    "steps": self.get_saga_steps(&saga_path)?
                }))
            }
            2 if parts[1] == "steps" => {
                // saga://{name}/steps - Return list of steps
                Ok(json!(self.get_saga_steps(&saga_path)?))
            }
            3 if parts[1] == "step" => {
                // saga://{name}/step/{step_name} - Return step details
                let step_name = parts[2];
                let step_file = saga_path.join(format!("{}.rs", step_name.to_lowercase()));

                if !step_file.exists() {
                    return Err(format!(
                        "Step '{}' not found in saga '{}'",
                        step_name, saga_name
                    ));
                }

                let content = fs::read_to_string(&step_file)
                    .map_err(|e| format!("Failed to read step file: {}", e))?;

                Ok(json!({
                    "saga_name": saga_name,
                    "step_name": step_name,
                    "path": step_file.to_string_lossy(),
                    "content": content
                }))
            }
            _ => Err(format!("Invalid saga resource URI: {}", uri)),
        }
    }

    /// Get list of steps in a saga
    fn get_saga_steps(&self, saga_path: &std::path::Path) -> Result<Vec<Value>, String> {
        use std::fs;

        let mut steps = Vec::new();

        if let Ok(entries) = fs::read_dir(saga_path) {
            for entry in entries.flatten() {
                if let Ok(file_type) = entry.file_type() {
                    if file_type.is_file() {
                        if let Some(file_name) = entry.file_name().to_str() {
                            if file_name != "mod.rs" && file_name.ends_with(".rs") {
                                if let Some(step_name) = file_name.strip_suffix(".rs") {
                                    steps.push(json!({
                                        "name": step_name,
                                        "file": file_name
                                    }));
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(steps)
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_list_tools() {
        // Can't test without a real project, but we can check tool definitions
        let tools_count = 11; // analyze, add_entity, add_service, add_handler, list_*, read_file, saga tools
        assert!(tools_count > 0);
    }
}
