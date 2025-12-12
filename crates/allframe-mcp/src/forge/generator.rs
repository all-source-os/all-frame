//! Code generator for AllFrame projects

use std::fs;
use std::path::PathBuf;

use serde::{Deserialize, Serialize};

use super::analyzer::Layer;

/// Field definition for entity generation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDef {
    /// Field name
    pub name: String,
    /// Rust type
    pub field_type: String,
    /// Is optional (Option<T>)
    #[serde(default)]
    pub optional: bool,
}

/// Entity generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityRequest {
    /// Entity name (PascalCase)
    pub name: String,
    /// Fields
    pub fields: Vec<FieldDef>,
    /// Generate repository trait
    #[serde(default = "default_true")]
    pub with_repository: bool,
}

fn default_true() -> bool {
    true
}

/// Service generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServiceRequest {
    /// Service name (PascalCase, without "Service" suffix)
    pub name: String,
    /// Dependencies to inject
    #[serde(default)]
    pub dependencies: Vec<String>,
    /// Methods to generate
    #[serde(default)]
    pub methods: Vec<MethodDef>,
}

/// Method definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MethodDef {
    /// Method name
    pub name: String,
    /// Parameters (name: type)
    #[serde(default)]
    pub params: Vec<(String, String)>,
    /// Return type
    pub return_type: String,
    /// Is async
    #[serde(default = "default_true")]
    pub is_async: bool,
}

/// Handler generation request
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HandlerRequest {
    /// Handler function name
    pub name: String,
    /// HTTP method
    pub method: String,
    /// Route path
    pub path: String,
    /// Service to call
    pub service: Option<String>,
}

/// Generation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationResult {
    /// Success status
    pub success: bool,
    /// Files created
    pub files_created: Vec<String>,
    /// Files modified
    pub files_modified: Vec<String>,
    /// Error message if any
    pub error: Option<String>,
}

impl GenerationResult {
    fn success(files_created: Vec<String>, files_modified: Vec<String>) -> Self {
        Self {
            success: true,
            files_created,
            files_modified,
            error: None,
        }
    }

    fn error(message: impl Into<String>) -> Self {
        Self {
            success: false,
            files_created: vec![],
            files_modified: vec![],
            error: Some(message.into()),
        }
    }
}

/// Code generator for AllFrame projects
pub struct CodeGenerator {
    project_path: PathBuf,
}

impl CodeGenerator {
    /// Create a new code generator
    pub fn new(project_path: impl Into<PathBuf>) -> Self {
        Self {
            project_path: project_path.into(),
        }
    }

    /// Generate a new entity
    pub fn generate_entity(&self, request: &EntityRequest) -> GenerationResult {
        let mut files_created = Vec::new();
        let mut files_modified = Vec::new();

        let entity_name = &request.name;
        let snake_name = to_snake_case(entity_name);

        // Generate entity file
        let entity_content = self.generate_entity_content(request);
        let entity_path = self.project_path
            .join("src/domain")
            .join(format!("{}.rs", snake_name));

        if let Err(e) = fs::write(&entity_path, entity_content) {
            return GenerationResult::error(format!("Failed to write entity file: {}", e));
        }
        files_created.push(format!("src/domain/{}.rs", snake_name));

        // Generate repository trait if requested
        if request.with_repository {
            let repo_content = self.generate_repository_content(entity_name);
            let repo_path = self.project_path
                .join("src/domain")
                .join(format!("{}_repository.rs", snake_name));

            if let Err(e) = fs::write(&repo_path, repo_content) {
                return GenerationResult::error(format!("Failed to write repository file: {}", e));
            }
            files_created.push(format!("src/domain/{}_repository.rs", snake_name));
        }

        // Update mod.rs
        if let Err(e) = self.update_mod_file(Layer::Domain, &snake_name, request.with_repository) {
            return GenerationResult::error(format!("Failed to update mod.rs: {}", e));
        }
        files_modified.push("src/domain/mod.rs".to_string());

        GenerationResult::success(files_created, files_modified)
    }

    /// Generate entity file content
    fn generate_entity_content(&self, request: &EntityRequest) -> String {
        let entity_name = &request.name;

        let mut fields = String::new();
        let mut has_uuid = false;
        let mut has_chrono = false;

        for field in &request.fields {
            let field_type = if field.optional {
                format!("Option<{}>", field.field_type)
            } else {
                field.field_type.clone()
            };

            if field.field_type.contains("Uuid") {
                has_uuid = true;
            }
            if field.field_type.contains("DateTime") || field.field_type.contains("NaiveDate") {
                has_chrono = true;
            }

            fields.push_str(&format!("    pub {}: {},\n", field.name, field_type));
        }

        let mut imports = String::new();
        if has_uuid {
            imports.push_str("use uuid::Uuid;\n");
        }
        if has_chrono {
            imports.push_str("use chrono::{DateTime, Utc};\n");
        }
        imports.push_str("use serde::{Deserialize, Serialize};\n");

        format!(
            r#"//! {entity_name} entity

{imports}
/// {entity_name} domain entity
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct {entity_name} {{
{fields}}}

impl {entity_name} {{
    /// Create a new {entity_name}
    pub fn new({params}) -> Self {{
        Self {{
{field_assignments}        }}
    }}
}}
"#,
            entity_name = entity_name,
            imports = imports,
            fields = fields.trim_end(),
            params = request.fields.iter()
                .map(|f| format!("{}: {}", f.name, if f.optional { format!("Option<{}>", f.field_type) } else { f.field_type.clone() }))
                .collect::<Vec<_>>()
                .join(", "),
            field_assignments = request.fields.iter()
                .map(|f| format!("            {},\n", f.name))
                .collect::<String>()
        )
    }

    /// Generate repository trait content
    fn generate_repository_content(&self, entity_name: &str) -> String {
        let snake_name = to_snake_case(entity_name);

        format!(
            r#"//! {entity_name} repository trait

use async_trait::async_trait;
use uuid::Uuid;

use super::{snake_name}::{entity_name};

/// Repository error
#[derive(Debug, thiserror::Error)]
pub enum RepositoryError {{
    #[error("Entity not found: {{0}}")]
    NotFound(String),
    #[error("Database error: {{0}}")]
    Database(String),
}}

/// {entity_name} repository trait
#[async_trait]
pub trait {entity_name}Repository: Send + Sync {{
    /// Find by ID
    async fn find_by_id(&self, id: Uuid) -> Result<Option<{entity_name}>, RepositoryError>;

    /// Find all
    async fn find_all(&self) -> Result<Vec<{entity_name}>, RepositoryError>;

    /// Save entity
    async fn save(&self, entity: &{entity_name}) -> Result<(), RepositoryError>;

    /// Delete by ID
    async fn delete(&self, id: Uuid) -> Result<(), RepositoryError>;
}}
"#,
            entity_name = entity_name,
            snake_name = snake_name,
        )
    }

    /// Generate a new service
    pub fn generate_service(&self, request: &ServiceRequest) -> GenerationResult {
        let mut files_created = Vec::new();
        let mut files_modified = Vec::new();

        let snake_name = to_snake_case(&request.name);

        let content = self.generate_service_content(request);
        let service_path = self.project_path
            .join("src/application")
            .join(format!("{}_service.rs", snake_name));

        if let Err(e) = fs::write(&service_path, content) {
            return GenerationResult::error(format!("Failed to write service file: {}", e));
        }
        files_created.push(format!("src/application/{}_service.rs", snake_name));

        // Update mod.rs
        if let Err(e) = self.update_service_mod_file(&snake_name) {
            return GenerationResult::error(format!("Failed to update mod.rs: {}", e));
        }
        files_modified.push("src/application/mod.rs".to_string());

        GenerationResult::success(files_created, files_modified)
    }

    /// Generate service file content
    fn generate_service_content(&self, request: &ServiceRequest) -> String {
        let service_name = format!("{}Service", request.name);

        let mut deps_fields = String::new();
        let mut deps_params = String::new();
        let mut deps_init = String::new();

        for (i, dep) in request.dependencies.iter().enumerate() {
            let field_name = to_snake_case(dep);
            deps_fields.push_str(&format!("    {}: Arc<dyn {}>,\n", field_name, dep));
            deps_params.push_str(&format!("{}: Arc<dyn {}>", field_name, dep));
            deps_init.push_str(&format!("            {},\n", field_name));
            if i < request.dependencies.len() - 1 {
                deps_params.push_str(", ");
            }
        }

        let mut methods = String::new();
        for method in &request.methods {
            let async_kw = if method.is_async { "async " } else { "" };
            let params: String = method.params.iter()
                .map(|(name, ty)| format!("{}: {}", name, ty))
                .collect::<Vec<_>>()
                .join(", ");

            methods.push_str(&format!(
                r#"
    /// {}
    pub {async_kw}fn {name}(&self{params_sep}{params}) -> Result<{return_type}, {service_name}Error> {{
        todo!("Implement {name}")
    }}
"#,
                method.name,
                async_kw = async_kw,
                name = method.name,
                params_sep = if params.is_empty() { "" } else { ", " },
                params = params,
                return_type = method.return_type,
                service_name = service_name,
            ));
        }

        format!(
            r#"//! {service_name}

use std::sync::Arc;

use thiserror::Error;

/// {service_name} error
#[derive(Debug, Error)]
pub enum {service_name}Error {{
    #[error("Not found: {{0}}")]
    NotFound(String),
    #[error("Validation error: {{0}}")]
    Validation(String),
    #[error("Internal error: {{0}}")]
    Internal(String),
}}

/// {service_name}
pub struct {service_name} {{
{deps_fields}}}

impl {service_name} {{
    /// Create a new {service_name}
    pub fn new({deps_params}) -> Self {{
        Self {{
{deps_init}        }}
    }}
{methods}}}
"#,
            service_name = service_name,
            deps_fields = deps_fields,
            deps_params = deps_params,
            deps_init = deps_init,
            methods = methods,
        )
    }

    /// Generate a new handler
    pub fn generate_handler(&self, request: &HandlerRequest) -> GenerationResult {
        let mut files_created = Vec::new();
        let mut files_modified = Vec::new();

        let handler_path = self.project_path
            .join("src/presentation")
            .join("handlers.rs");

        // Read existing content or create new
        let existing = fs::read_to_string(&handler_path).unwrap_or_default();

        let new_handler = self.generate_handler_content(request);
        let content = if existing.is_empty() {
            self.generate_handlers_file(&[new_handler])
        } else {
            // Append new handler before the last closing brace or at end
            format!("{}\n\n{}", existing.trim_end(), new_handler)
        };

        if let Err(e) = fs::write(&handler_path, content) {
            return GenerationResult::error(format!("Failed to write handler file: {}", e));
        }

        if existing.is_empty() {
            files_created.push("src/presentation/handlers.rs".to_string());
        } else {
            files_modified.push("src/presentation/handlers.rs".to_string());
        }

        GenerationResult::success(files_created, files_modified)
    }

    /// Generate handler function content
    fn generate_handler_content(&self, request: &HandlerRequest) -> String {
        let method = request.method.to_uppercase();

        format!(
            r#"/// {method} {path}
pub async fn {name}(
    // State(service): State<Arc<{service}>>,
    // Path(id): Path<Uuid>,
) -> impl IntoResponse {{
    // TODO: Implement handler
    Json(serde_json::json!({{
        "status": "ok",
        "handler": "{name}"
    }}))
}}"#,
            method = method,
            path = request.path,
            name = request.name,
            service = request.service.as_deref().unwrap_or("AppService"),
        )
    }

    /// Generate a complete handlers file
    fn generate_handlers_file(&self, handlers: &[String]) -> String {
        format!(
            r#"//! HTTP handlers

use axum::{{
    extract::{{Path, State}},
    response::IntoResponse,
    Json,
}};
use serde_json;
use std::sync::Arc;
use uuid::Uuid;

{handlers}
"#,
            handlers = handlers.join("\n\n")
        )
    }

    /// Update domain mod.rs with new entity
    fn update_mod_file(&self, layer: Layer, module_name: &str, with_repository: bool) -> Result<(), String> {
        let mod_path = self.project_path
            .join("src")
            .join(layer.dir_name())
            .join("mod.rs");

        let existing = fs::read_to_string(&mod_path).unwrap_or_default();

        let mut content = existing.clone();

        // Add module declaration if not present
        let mod_decl = format!("pub mod {};", module_name);
        if !content.contains(&mod_decl) {
            content = format!("{}\n{}", mod_decl, content);
        }

        // Add re-export if not present
        let re_export = format!("pub use {}::*;", module_name);
        if !content.contains(&re_export) && !content.contains(&format!("pub use {}::", module_name)) {
            content.push_str(&format!("\n{}", re_export));
        }

        // Add repository if requested
        if with_repository {
            let repo_mod = format!("pub mod {}_repository;", module_name);
            if !content.contains(&repo_mod) {
                content = format!("{}\n{}", repo_mod, content);
            }
            let repo_export = format!("pub use {}_repository::*;", module_name);
            if !content.contains(&repo_export) {
                content.push_str(&format!("\n{}", repo_export));
            }
        }

        fs::write(&mod_path, content)
            .map_err(|e| format!("Failed to write mod.rs: {}", e))
    }

    /// Update application mod.rs with new service
    fn update_service_mod_file(&self, service_name: &str) -> Result<(), String> {
        let mod_path = self.project_path
            .join("src/application/mod.rs");

        let existing = fs::read_to_string(&mod_path).unwrap_or_default();
        let mut content = existing.clone();

        let mod_decl = format!("pub mod {}_service;", service_name);
        if !content.contains(&mod_decl) {
            content = format!("{}\n{}", mod_decl, content);
        }

        let re_export = format!("pub use {}_service::*;", service_name);
        if !content.contains(&re_export) {
            content.push_str(&format!("\n{}", re_export));
        }

        fs::write(&mod_path, content)
            .map_err(|e| format!("Failed to write mod.rs: {}", e))
    }
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, c) in s.chars().enumerate() {
        if c.is_uppercase() {
            if i > 0 {
                result.push('_');
            }
            result.push(c.to_lowercase().next().unwrap());
        } else {
            result.push(c);
        }
    }
    result
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_snake_case() {
        assert_eq!(to_snake_case("UserProfile"), "user_profile");
        assert_eq!(to_snake_case("User"), "user");
        assert_eq!(to_snake_case("HTTPClient"), "h_t_t_p_client");
    }

    #[test]
    fn test_entity_request_serialization() {
        let request = EntityRequest {
            name: "User".to_string(),
            fields: vec![
                FieldDef {
                    name: "id".to_string(),
                    field_type: "Uuid".to_string(),
                    optional: false,
                },
                FieldDef {
                    name: "email".to_string(),
                    field_type: "String".to_string(),
                    optional: false,
                },
            ],
            with_repository: true,
        };

        let json = serde_json::to_string(&request).unwrap();
        assert!(json.contains("User"));
        assert!(json.contains("email"));
    }
}
