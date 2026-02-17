//! Project analyzer for understanding AllFrame project structure

use std::{
    collections::HashMap,
    fs,
    path::{Path, PathBuf},
};

use serde::{Deserialize, Serialize};

/// Clean Architecture layer
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Layer {
    /// Domain layer - business entities and rules
    Domain,
    /// Application layer - use cases and services
    Application,
    /// Infrastructure layer - external implementations
    Infrastructure,
    /// Presentation layer - API handlers
    Presentation,
}

impl Layer {
    /// Get the directory name for this layer
    pub fn dir_name(&self) -> &'static str {
        match self {
            Layer::Domain => "domain",
            Layer::Application => "application",
            Layer::Infrastructure => "infrastructure",
            Layer::Presentation => "presentation",
        }
    }
}

/// Detected entity in the domain layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Entity {
    /// Entity name (e.g., "User", "Order")
    pub name: String,
    /// File path relative to project root
    pub file_path: String,
    /// Detected fields (name -> type)
    pub fields: HashMap<String, String>,
}

/// Detected service in the application layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Service {
    /// Service name (e.g., "UserService", "OrderService")
    pub name: String,
    /// File path relative to project root
    pub file_path: String,
    /// Dependencies (injected services/repositories)
    pub dependencies: Vec<String>,
}

/// Detected handler in the presentation layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Handler {
    /// Handler name (e.g., "get_user", "create_order")
    pub name: String,
    /// File path relative to project root
    pub file_path: String,
    /// HTTP method if applicable
    pub method: Option<String>,
    /// Route path if applicable
    pub path: Option<String>,
}

/// Repository trait detected in domain layer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Repository {
    /// Repository name (e.g., "UserRepository")
    pub name: String,
    /// File path relative to project root
    pub file_path: String,
    /// Associated entity
    pub entity: Option<String>,
}

/// Complete project structure analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProjectStructure {
    /// Project name from Cargo.toml
    pub name: String,
    /// Project root path
    pub root_path: String,
    /// Detected archetype (if recognizable)
    pub archetype: Option<String>,
    /// Entities in domain layer
    pub entities: Vec<Entity>,
    /// Repository traits in domain layer
    pub repositories: Vec<Repository>,
    /// Services in application layer
    pub services: Vec<Service>,
    /// Handlers in presentation layer
    pub handlers: Vec<Handler>,
    /// All source files organized by layer
    pub files_by_layer: HashMap<String, Vec<String>>,
    /// Dependencies from Cargo.toml
    pub dependencies: Vec<String>,
}

/// Project analyzer that parses existing AllFrame projects
pub struct ProjectAnalyzer {
    project_path: PathBuf,
}

impl ProjectAnalyzer {
    /// Create a new analyzer for a project
    pub fn new(project_path: impl Into<PathBuf>) -> Result<Self, String> {
        let project_path = project_path.into();

        // Verify it's a valid Rust project
        if !project_path.join("Cargo.toml").exists() {
            return Err(format!(
                "Not a Rust project: Cargo.toml not found in {}",
                project_path.display()
            ));
        }

        // Verify it has src directory
        if !project_path.join("src").exists() {
            return Err(format!(
                "No src directory found in {}",
                project_path.display()
            ));
        }

        Ok(Self { project_path })
    }

    /// Get the project path
    pub fn project_path(&self) -> &Path {
        &self.project_path
    }

    /// Analyze the project structure
    pub fn analyze(&self) -> Result<ProjectStructure, String> {
        let name = self.parse_project_name()?;
        let archetype = self.detect_archetype();
        let dependencies = self.parse_dependencies()?;
        let files_by_layer = self.scan_layers()?;
        let entities = self.find_entities(&files_by_layer)?;
        let repositories = self.find_repositories(&files_by_layer)?;
        let services = self.find_services(&files_by_layer)?;
        let handlers = self.find_handlers(&files_by_layer)?;

        Ok(ProjectStructure {
            name,
            root_path: self.project_path.display().to_string(),
            archetype,
            entities,
            repositories,
            services,
            handlers,
            files_by_layer,
            dependencies,
        })
    }

    /// Parse project name from Cargo.toml
    fn parse_project_name(&self) -> Result<String, String> {
        let cargo_toml = self.project_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml)
            .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

        // Simple parsing - look for name = "..."
        for line in content.lines() {
            let line = line.trim();
            if line.starts_with("name") && line.contains('=') {
                if let Some(name) = line.split('=').nth(1) {
                    let name = name.trim().trim_matches('"').trim_matches('\'');
                    return Ok(name.to_string());
                }
            }
        }

        Err("Could not find project name in Cargo.toml".to_string())
    }

    /// Parse dependencies from Cargo.toml
    fn parse_dependencies(&self) -> Result<Vec<String>, String> {
        let cargo_toml = self.project_path.join("Cargo.toml");
        let content = fs::read_to_string(&cargo_toml)
            .map_err(|e| format!("Failed to read Cargo.toml: {}", e))?;

        let mut deps = Vec::new();
        let mut in_deps = false;

        for line in content.lines() {
            let line = line.trim();

            if line.starts_with("[dependencies]") || line.starts_with("[dev-dependencies]") {
                in_deps = true;
                continue;
            }

            if line.starts_with('[') {
                in_deps = false;
                continue;
            }

            if in_deps && !line.is_empty() && !line.starts_with('#') {
                if let Some(dep_name) = line.split(['=', ' ', '.']).next() {
                    let dep_name = dep_name.trim();
                    if !dep_name.is_empty() {
                        deps.push(dep_name.to_string());
                    }
                }
            }
        }

        Ok(deps)
    }

    /// Detect the archetype based on project structure and dependencies
    fn detect_archetype(&self) -> Option<String> {
        let src = self.project_path.join("src");
        let deps = self.parse_dependencies().unwrap_or_default();

        // Check for gateway indicators
        if self.project_path.join("proto").exists() {
            return Some("gateway".to_string());
        }

        // Check for consumer indicators
        if deps.iter().any(|d| d.contains("rdkafka")) {
            if src.join("application/consumer.rs").exists() {
                return Some("consumer".to_string());
            }
            if src.join("infrastructure/outbox").exists() {
                return Some("producer".to_string());
            }
        }

        // Check for BFF indicators
        if deps.iter().any(|d| d.contains("async-graphql")) {
            return Some("bff".to_string());
        }

        // Check for scheduled indicators
        if deps.iter().any(|d| d.contains("tokio-cron-scheduler")) {
            return Some("scheduled".to_string());
        }

        // Check for websocket indicators
        if src.join("application/hub.rs").exists() {
            return Some("websocket-gateway".to_string());
        }

        // Check for saga indicators
        if src.join("application/orchestrator.rs").exists() {
            return Some("saga-orchestrator".to_string());
        }

        // Check for legacy adapter indicators
        if src.join("domain/legacy.rs").exists() {
            return Some("legacy-adapter".to_string());
        }

        // Default to basic if has Clean Architecture structure
        if src.join("domain").exists() && src.join("application").exists() {
            return Some("basic".to_string());
        }

        None
    }

    /// Scan all layers and collect file paths
    fn scan_layers(&self) -> Result<HashMap<String, Vec<String>>, String> {
        let mut files_by_layer = HashMap::new();
        let src = self.project_path.join("src");

        for layer in [
            Layer::Domain,
            Layer::Application,
            Layer::Infrastructure,
            Layer::Presentation,
        ] {
            let layer_path = src.join(layer.dir_name());
            let mut files = Vec::new();

            if layer_path.exists() {
                self.collect_rs_files(&layer_path, &mut files)?;
            }

            files_by_layer.insert(layer.dir_name().to_string(), files);
        }

        Ok(files_by_layer)
    }

    /// Recursively collect .rs files
    fn collect_rs_files(&self, dir: &Path, files: &mut Vec<String>) -> Result<(), String> {
        if !dir.is_dir() {
            return Ok(());
        }

        let entries = fs::read_dir(dir)
            .map_err(|e| format!("Failed to read directory {}: {}", dir.display(), e))?;

        for entry in entries {
            let entry = entry.map_err(|e| format!("Failed to read entry: {}", e))?;
            let path = entry.path();

            if path.is_dir() {
                self.collect_rs_files(&path, files)?;
            } else if path.extension().map_or(false, |e| e == "rs") {
                if let Ok(relative) = path.strip_prefix(&self.project_path) {
                    files.push(relative.display().to_string());
                }
            }
        }

        Ok(())
    }

    /// Find entities in domain layer by parsing struct definitions
    fn find_entities(
        &self,
        files_by_layer: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<Entity>, String> {
        let mut entities = Vec::new();

        let domain_files = files_by_layer.get("domain").cloned().unwrap_or_default();

        for file_path in domain_files {
            // Skip mod.rs files
            if file_path.ends_with("mod.rs") {
                continue;
            }

            let full_path = self.project_path.join(&file_path);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Simple parsing: find pub struct definitions
            let mut lines = content.lines().peekable();
            while let Some(line) = lines.next() {
                let line = line.trim();

                // Look for pub struct (excluding trait impl blocks)
                if line.starts_with("pub struct ") && !line.contains("impl") {
                    // Extract struct name
                    let name = line
                        .trim_start_matches("pub struct ")
                        .split([' ', '{', '<', '('])
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if name.is_empty() || name.ends_with("Error") || name.ends_with("Config") {
                        continue;
                    }

                    // Try to extract fields
                    let mut fields = HashMap::new();
                    if line.contains('{') {
                        // Multi-line struct
                        for field_line in lines.by_ref() {
                            let field_line = field_line.trim();
                            if field_line.starts_with('}') {
                                break;
                            }
                            if field_line.starts_with("pub ") && field_line.contains(':') {
                                let parts: Vec<&str> = field_line
                                    .trim_start_matches("pub ")
                                    .splitn(2, ':')
                                    .collect();
                                if parts.len() == 2 {
                                    let field_name = parts[0].trim().to_string();
                                    let field_type =
                                        parts[1].trim().trim_end_matches(',').to_string();
                                    fields.insert(field_name, field_type);
                                }
                            }
                        }
                    }

                    entities.push(Entity {
                        name,
                        file_path: file_path.clone(),
                        fields,
                    });
                }
            }
        }

        Ok(entities)
    }

    /// Find repository traits in domain layer
    fn find_repositories(
        &self,
        files_by_layer: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<Repository>, String> {
        let mut repositories = Vec::new();

        let domain_files = files_by_layer.get("domain").cloned().unwrap_or_default();

        for file_path in domain_files {
            let full_path = self.project_path.join(&file_path);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Find pub trait ... Repository patterns
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("pub trait ") && line.contains("Repository") {
                    let name = line
                        .trim_start_matches("pub trait ")
                        .split([' ', '{', '<', ':'])
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if !name.is_empty() {
                        // Try to guess associated entity
                        let entity = name.trim_end_matches("Repository").to_string();

                        repositories.push(Repository {
                            name,
                            file_path: file_path.clone(),
                            entity: if entity.is_empty() {
                                None
                            } else {
                                Some(entity)
                            },
                        });
                    }
                }
            }
        }

        Ok(repositories)
    }

    /// Find services in application layer
    fn find_services(
        &self,
        files_by_layer: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<Service>, String> {
        let mut services = Vec::new();

        let app_files = files_by_layer
            .get("application")
            .cloned()
            .unwrap_or_default();

        for file_path in app_files {
            if file_path.ends_with("mod.rs") {
                continue;
            }

            let full_path = self.project_path.join(&file_path);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Find pub struct ...Service patterns
            let mut lines = content.lines().peekable();
            while let Some(line) = lines.next() {
                let line = line.trim();
                if line.starts_with("pub struct ")
                    && (line.contains("Service")
                        || line.contains("Orchestrator")
                        || line.contains("Handler"))
                {
                    let name = line
                        .trim_start_matches("pub struct ")
                        .split([' ', '{', '<'])
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if name.is_empty() {
                        continue;
                    }

                    // Extract dependencies from struct fields
                    let mut dependencies = Vec::new();
                    if line.contains('{') {
                        for field_line in lines.by_ref() {
                            let field_line = field_line.trim();
                            if field_line.starts_with('}') {
                                break;
                            }
                            if field_line.contains(':') {
                                let parts: Vec<&str> = field_line.splitn(2, ':').collect();
                                if parts.len() == 2 {
                                    let field_type = parts[1].trim().trim_end_matches(',');
                                    // Check if it looks like a dependency (Arc<dyn ...> or similar)
                                    if field_type.contains("Arc<") || field_type.contains("Box<") {
                                        dependencies.push(field_type.to_string());
                                    }
                                }
                            }
                        }
                    }

                    services.push(Service {
                        name,
                        file_path: file_path.clone(),
                        dependencies,
                    });
                }
            }
        }

        Ok(services)
    }

    /// Find handlers in presentation layer
    fn find_handlers(
        &self,
        files_by_layer: &HashMap<String, Vec<String>>,
    ) -> Result<Vec<Handler>, String> {
        let mut handlers = Vec::new();

        let pres_files = files_by_layer
            .get("presentation")
            .cloned()
            .unwrap_or_default();

        for file_path in pres_files {
            if file_path.ends_with("mod.rs") {
                continue;
            }

            let full_path = self.project_path.join(&file_path);
            let content = match fs::read_to_string(&full_path) {
                Ok(c) => c,
                Err(_) => continue,
            };

            // Find async fn patterns that look like handlers
            for line in content.lines() {
                let line = line.trim();
                if line.starts_with("pub async fn ") || line.starts_with("async fn ") {
                    let name = line
                        .trim_start_matches("pub ")
                        .trim_start_matches("async fn ")
                        .split(['(', '<'])
                        .next()
                        .unwrap_or("")
                        .to_string();

                    if name.is_empty() || name == "main" {
                        continue;
                    }

                    // Try to detect HTTP method from name
                    let method = if name.starts_with("get_")
                        || name.starts_with("list_")
                        || name.starts_with("fetch_")
                    {
                        Some("GET".to_string())
                    } else if name.starts_with("create_") || name.starts_with("add_") {
                        Some("POST".to_string())
                    } else if name.starts_with("update_") || name.starts_with("modify_") {
                        Some("PUT".to_string())
                    } else if name.starts_with("delete_") || name.starts_with("remove_") {
                        Some("DELETE".to_string())
                    } else {
                        None
                    };

                    handlers.push(Handler {
                        name,
                        file_path: file_path.clone(),
                        method,
                        path: None,
                    });
                }
            }
        }

        Ok(handlers)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_dir_names() {
        assert_eq!(Layer::Domain.dir_name(), "domain");
        assert_eq!(Layer::Application.dir_name(), "application");
        assert_eq!(Layer::Infrastructure.dir_name(), "infrastructure");
        assert_eq!(Layer::Presentation.dir_name(), "presentation");
    }
}
