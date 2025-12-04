//! MCP Tool definitions and generation

use serde::{Deserialize, Serialize};

/// An MCP Tool generated from a Router handler
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct McpTool {
    /// Tool name (matches handler name)
    pub name: String,
    /// Human-readable description
    pub description: String,
    /// JSON Schema for tool inputs (JSON string)
    pub input_schema: String,
}

impl McpTool {
    /// Create a new MCP tool
    pub fn new(name: impl Into<String>, description: impl Into<String>, input_schema: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            description: description.into(),
            input_schema: input_schema.into(),
        }
    }

    /// Create a tool from a handler name with auto-generated fields
    pub fn from_handler_name(name: impl Into<String>) -> Self {
        let name = name.into();
        let description = format!("Tool: {}", name);
        let input_schema = r#"{"type": "object", "properties": {}}"#.to_string();

        Self {
            name,
            description,
            input_schema,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tool_creation() {
        let tool = McpTool::new("test", "Test tool", r#"{"type": "object"}"#);
        assert_eq!(tool.name, "test");
        assert_eq!(tool.description, "Test tool");
        assert_eq!(tool.input_schema, r#"{"type": "object"}"#);
    }

    #[test]
    fn test_tool_from_handler_name() {
        let tool = McpTool::from_handler_name("my_handler");
        assert_eq!(tool.name, "my_handler");
        assert!(!tool.description.is_empty());
        assert!(tool.input_schema.contains("object"));
    }
}
