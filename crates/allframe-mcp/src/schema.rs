//! JSON Schema generation from OpenAPI specifications

use serde_json::{json, Value};

/// Convert OpenAPI schema to JSON Schema (draft 2020-12)
pub fn openapi_to_json_schema(openapi_schema: &Value) -> Result<String, String> {
    // Extract schema object from OpenAPI
    let schema_obj = openapi_schema
        .as_object()
        .ok_or("Invalid OpenAPI schema: not an object")?;

    // Convert to JSON Schema format
    let mut json_schema = serde_json::Map::new();
    json_schema.insert(
        "$schema".to_string(),
        json!("https://json-schema.org/draft/2020-12/schema"),
    );

    // Copy type information
    if let Some(type_val) = schema_obj.get("type") {
        json_schema.insert("type".to_string(), type_val.clone());
    }

    // Copy properties if present
    if let Some(properties) = schema_obj.get("properties") {
        json_schema.insert("properties".to_string(), properties.clone());
    }

    // Copy required fields
    if let Some(required) = schema_obj.get("required") {
        json_schema.insert("required".to_string(), required.clone());
    }

    // Convert to string
    serde_json::to_string_pretty(&json_schema)
        .map_err(|e| format!("Failed to serialize JSON Schema: {}", e))
}

/// Validate input against JSON Schema
pub fn validate_input(input: &Value, schema: &str) -> Result<(), Vec<String>> {
    // Parse schema
    let schema_val: Value =
        serde_json::from_str(schema).map_err(|e| vec![format!("Invalid schema: {}", e)])?;

    // Basic validation - check required fields
    let mut errors = Vec::new();

    if let Some(required) = schema_val.get("required").and_then(|r| r.as_array()) {
        let input_obj = input
            .as_object()
            .ok_or(vec!["Input must be an object".to_string()])?;

        for req_field in required {
            if let Some(field_name) = req_field.as_str() {
                if !input_obj.contains_key(field_name) {
                    errors.push(format!("Missing required field: {}", field_name));
                }
            }
        }
    }

    // Check property types if specified
    if let Some(properties) = schema_val.get("properties").and_then(|p| p.as_object()) {
        if let Some(input_obj) = input.as_object() {
            for (prop_name, prop_schema) in properties {
                if let Some(input_value) = input_obj.get(prop_name) {
                    if let Some(expected_type) = prop_schema.get("type").and_then(|t| t.as_str()) {
                        let actual_type = match input_value {
                            Value::Null => "null",
                            Value::Bool(_) => "boolean",
                            Value::Number(_) => "number",
                            Value::String(_) => "string",
                            Value::Array(_) => "array",
                            Value::Object(_) => "object",
                        };

                        if expected_type != actual_type {
                            errors.push(format!(
                                "Field '{}' has wrong type: expected {}, got {}",
                                prop_name, expected_type, actual_type
                            ));
                        }
                    }
                }
            }
        }
    }

    if errors.is_empty() {
        Ok(())
    } else {
        Err(errors)
    }
}

/// Convert string to number if schema expects number
pub fn coerce_type(value: &Value, expected_type: &str) -> Result<Value, String> {
    match (value, expected_type) {
        (Value::String(s), "number") | (Value::String(s), "integer") => s
            .parse::<i64>()
            .map(|n| json!(n))
            .or_else(|_| s.parse::<f64>().map(|n| json!(n)))
            .map_err(|e| format!("Cannot convert '{}' to number: {}", s, e)),
        (Value::String(s), "boolean") => match s.to_lowercase().as_str() {
            "true" | "1" | "yes" => Ok(json!(true)),
            "false" | "0" | "no" => Ok(json!(false)),
            _ => Err(format!("Cannot convert '{}' to boolean", s)),
        },
        (v, _) => Ok(v.clone()),
    }
}

/// Extract enum values from schema
pub fn extract_enum_values(schema: &Value) -> Option<Vec<String>> {
    schema.get("enum").and_then(|e| e.as_array()).map(|arr| {
        arr.iter()
            .filter_map(|v| v.as_str())
            .map(|s| s.to_string())
            .collect()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_openapi_to_json_schema_basic() {
        let openapi = json!({
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "integer"}
            },
            "required": ["name"]
        });

        let result = openapi_to_json_schema(&openapi).unwrap();
        let parsed: Value = serde_json::from_str(&result).unwrap();

        assert_eq!(
            parsed["$schema"],
            "https://json-schema.org/draft/2020-12/schema"
        );
        assert_eq!(parsed["type"], "object");
        assert!(parsed["properties"].is_object());
        assert_eq!(parsed["required"], json!(["name"]));
    }

    #[test]
    fn test_validate_input_success() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"},
                "age": {"type": "number"}
            },
            "required": ["name"]
        }"#;

        let input = json!({"name": "Alice", "age": 30});
        assert!(validate_input(&input, schema).is_ok());
    }

    #[test]
    fn test_validate_input_missing_required() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "name": {"type": "string"}
            },
            "required": ["name"]
        }"#;

        let input = json!({"age": 30});
        let result = validate_input(&input, schema);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert_eq!(errors.len(), 1);
        assert!(errors[0].contains("Missing required field: name"));
    }

    #[test]
    fn test_validate_input_wrong_type() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "age": {"type": "number"}
            }
        }"#;

        let input = json!({"age": "not a number"});
        let result = validate_input(&input, schema);

        assert!(result.is_err());
        let errors = result.unwrap_err();
        assert!(errors[0].contains("wrong type"));
    }

    #[test]
    fn test_coerce_string_to_number() {
        let value = json!("42");
        let result = coerce_type(&value, "number").unwrap();
        assert_eq!(result, json!(42));
    }

    #[test]
    fn test_coerce_string_to_float() {
        let value = json!("3.14");
        let result = coerce_type(&value, "number").unwrap();
        assert_eq!(result, json!(3.14));
    }

    #[test]
    fn test_coerce_string_to_boolean() {
        assert_eq!(coerce_type(&json!("true"), "boolean").unwrap(), json!(true));
        assert_eq!(
            coerce_type(&json!("false"), "boolean").unwrap(),
            json!(false)
        );
        assert_eq!(coerce_type(&json!("1"), "boolean").unwrap(), json!(true));
        assert_eq!(coerce_type(&json!("0"), "boolean").unwrap(), json!(false));
    }

    #[test]
    fn test_coerce_invalid_number() {
        let value = json!("not-a-number");
        let result = coerce_type(&value, "number");
        assert!(result.is_err());
    }

    #[test]
    fn test_extract_enum_values() {
        let schema = json!({
            "type": "string",
            "enum": ["red", "green", "blue"]
        });

        let values = extract_enum_values(&schema).unwrap();
        assert_eq!(values, vec!["red", "green", "blue"]);
    }

    #[test]
    fn test_extract_enum_none() {
        let schema = json!({"type": "string"});
        assert!(extract_enum_values(&schema).is_none());
    }

    #[test]
    fn test_validate_nested_object() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "user": {
                    "type": "object",
                    "properties": {
                        "name": {"type": "string"}
                    }
                }
            }
        }"#;

        let input = json!({
            "user": {
                "name": "Alice"
            }
        });

        assert!(validate_input(&input, schema).is_ok());
    }

    #[test]
    fn test_validate_array_type() {
        let schema = r#"{
            "type": "object",
            "properties": {
                "tags": {"type": "array"}
            }
        }"#;

        let input = json!({"tags": ["rust", "web"]});
        assert!(validate_input(&input, schema).is_ok());
    }
}
