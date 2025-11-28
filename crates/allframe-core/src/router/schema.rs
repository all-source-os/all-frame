//! JSON Schema generation for OpenAPI documentation
//!
//! This module provides types and traits for generating JSON Schema
//! from Rust types. This is used to automatically document request
//! and response types in OpenAPI specifications.

use serde_json::{json, Value};

/// Trait for types that can be converted to JSON Schema
///
/// This trait enables automatic schema generation for OpenAPI documentation.
/// Implement this trait for your request/response types to enable
/// automatic API documentation.
pub trait ToJsonSchema {
    /// Generate a JSON Schema for this type
    fn schema() -> Value;

    /// Get the schema type name (for referencing in OpenAPI)
    fn schema_name() -> Option<String> {
        None
    }
}

// Implement ToJsonSchema for primitive types

impl ToJsonSchema for String {
    fn schema() -> Value {
        json!({
            "type": "string"
        })
    }
}

impl ToJsonSchema for &str {
    fn schema() -> Value {
        json!({
            "type": "string"
        })
    }
}

impl ToJsonSchema for i32 {
    fn schema() -> Value {
        json!({
            "type": "integer",
            "format": "int32"
        })
    }
}

impl ToJsonSchema for i64 {
    fn schema() -> Value {
        json!({
            "type": "integer",
            "format": "int64"
        })
    }
}

impl ToJsonSchema for u32 {
    fn schema() -> Value {
        json!({
            "type": "integer",
            "format": "uint32",
            "minimum": 0
        })
    }
}

impl ToJsonSchema for u64 {
    fn schema() -> Value {
        json!({
            "type": "integer",
            "format": "uint64",
            "minimum": 0
        })
    }
}

impl ToJsonSchema for f32 {
    fn schema() -> Value {
        json!({
            "type": "number",
            "format": "float"
        })
    }
}

impl ToJsonSchema for f64 {
    fn schema() -> Value {
        json!({
            "type": "number",
            "format": "double"
        })
    }
}

impl ToJsonSchema for bool {
    fn schema() -> Value {
        json!({
            "type": "boolean"
        })
    }
}

// Implement for Option<T>
impl<T: ToJsonSchema> ToJsonSchema for Option<T> {
    fn schema() -> Value {
        let mut schema = T::schema();
        if let Value::Object(ref mut map) = schema {
            map.insert("nullable".to_string(), Value::Bool(true));
        }
        schema
    }
}

// Implement for Vec<T>
impl<T: ToJsonSchema> ToJsonSchema for Vec<T> {
    fn schema() -> Value {
        json!({
            "type": "array",
            "items": T::schema()
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_string_schema() {
        let schema = String::schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn test_str_schema() {
        let schema = <&str>::schema();
        assert_eq!(schema["type"], "string");
    }

    #[test]
    fn test_i32_schema() {
        let schema = i32::schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["format"], "int32");
    }

    #[test]
    fn test_i64_schema() {
        let schema = i64::schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["format"], "int64");
    }

    #[test]
    fn test_u32_schema() {
        let schema = u32::schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["format"], "uint32");
        assert_eq!(schema["minimum"], 0);
    }

    #[test]
    fn test_u64_schema() {
        let schema = u64::schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["format"], "uint64");
        assert_eq!(schema["minimum"], 0);
    }

    #[test]
    fn test_f32_schema() {
        let schema = f32::schema();
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["format"], "float");
    }

    #[test]
    fn test_f64_schema() {
        let schema = f64::schema();
        assert_eq!(schema["type"], "number");
        assert_eq!(schema["format"], "double");
    }

    #[test]
    fn test_bool_schema() {
        let schema = bool::schema();
        assert_eq!(schema["type"], "boolean");
    }

    #[test]
    fn test_option_string_schema() {
        let schema = Option::<String>::schema();
        assert_eq!(schema["type"], "string");
        assert_eq!(schema["nullable"], true);
    }

    #[test]
    fn test_option_i32_schema() {
        let schema = Option::<i32>::schema();
        assert_eq!(schema["type"], "integer");
        assert_eq!(schema["nullable"], true);
    }

    #[test]
    fn test_vec_string_schema() {
        let schema = Vec::<String>::schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
    }

    #[test]
    fn test_vec_i32_schema() {
        let schema = Vec::<i32>::schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "integer");
        assert_eq!(schema["items"]["format"], "int32");
    }

    #[test]
    fn test_nested_vec_schema() {
        let schema = Vec::<Vec<String>>::schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "array");
        assert_eq!(schema["items"]["items"]["type"], "string");
    }

    #[test]
    fn test_option_vec_schema() {
        let schema = Option::<Vec<String>>::schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["nullable"], true);
        assert_eq!(schema["items"]["type"], "string");
    }

    #[test]
    fn test_vec_option_schema() {
        let schema = Vec::<Option<String>>::schema();
        assert_eq!(schema["type"], "array");
        assert_eq!(schema["items"]["type"], "string");
        assert_eq!(schema["items"]["nullable"], true);
    }
}
