//! TypeScript client code generation from Router handler metadata
//!
//! Generates typed async functions that call AllFrame handlers via IPC,
//! eliminating stringly-typed `allframe_call` invocations.
//!
//! # Example
//!
//! ```rust
//! use allframe_core::router::{Router, TsField, TsType};
//!
//! let mut router = Router::new();
//! router.register("get_user", || async { r#"{"id":1}"#.to_string() });
//! router.describe_handler("get_user", vec![], TsType::Object(vec![
//!     TsField::new("id", TsType::Number),
//!     TsField::new("name", TsType::String),
//! ]));
//!
//! let ts_code = router.generate_ts_client();
//! assert!(ts_code.contains("export async function getUser()"));
//! ```

use std::collections::HashMap;
use std::fmt::Write;

/// TypeScript type representation
#[derive(Debug, Clone, PartialEq)]
pub enum TsType {
    /// `string`
    String,
    /// `number`
    Number,
    /// `boolean`
    Boolean,
    /// `null`
    Null,
    /// `T | null`
    Optional(Box<TsType>),
    /// `T[]`
    Array(Box<TsType>),
    /// `{ field: Type, ... }`
    Object(Vec<TsField>),
    /// Named interface reference (e.g., `UserResponse`)
    Named(String),
    /// `void`
    Void,
    /// Raw TS type string (escape hatch)
    Raw(String),
}

/// A field in a TS object/interface
#[derive(Debug, Clone, PartialEq)]
pub struct TsField {
    /// Field name
    pub name: String,
    /// Field type
    pub ty: TsType,
    /// Whether the field is optional (renders as `name?: Type`)
    pub optional: bool,
}

impl TsField {
    /// Create a required field
    pub fn new(name: &str, ty: TsType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            optional: false,
        }
    }

    /// Create an optional field
    pub fn optional(name: &str, ty: TsType) -> Self {
        Self {
            name: name.to_string(),
            ty,
            optional: true,
        }
    }
}

/// Metadata describing a handler's argument and return types
#[derive(Debug, Clone)]
pub struct HandlerMeta {
    /// Argument fields (empty = no args)
    pub args: Vec<TsField>,
    /// Return type
    pub returns: TsType,
}

impl TsType {
    /// Render this type as a TypeScript type string
    pub fn render(&self) -> String {
        match self {
            TsType::String => "string".to_string(),
            TsType::Number => "number".to_string(),
            TsType::Boolean => "boolean".to_string(),
            TsType::Null => "null".to_string(),
            TsType::Void => "void".to_string(),
            TsType::Optional(inner) => format!("{} | null", inner.render()),
            TsType::Array(inner) => {
                let inner_str = inner.render();
                if inner_str.contains('|') {
                    format!("({inner_str})[]")
                } else {
                    format!("{inner_str}[]")
                }
            }
            TsType::Object(fields) => {
                if fields.is_empty() {
                    return "Record<string, never>".to_string();
                }
                let mut s = "{ ".to_string();
                for (i, field) in fields.iter().enumerate() {
                    if i > 0 {
                        s.push_str("; ");
                    }
                    if field.optional {
                        write!(s, "{}?: {}", field.name, field.ty.render()).unwrap();
                    } else {
                        write!(s, "{}: {}", field.name, field.ty.render()).unwrap();
                    }
                }
                s.push_str(" }");
                s
            }
            TsType::Named(name) => name.clone(),
            TsType::Raw(raw) => raw.clone(),
        }
    }
}

/// Convert a snake_case handler name to camelCase for TS function name.
///
/// Lowercases the entire input first, then capitalizes the first char after
/// each underscore. Handles ALL_CAPS input correctly.
fn to_camel_case(s: &str) -> String {
    let lower = s.to_lowercase();
    let mut result = String::new();
    let mut capitalize_next = false;

    for c in lower.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_uppercase().next().unwrap_or(c));
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Convert a snake_case name to PascalCase for TS interface name
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => {
                    let upper: String = c.to_uppercase().collect();
                    let rest: String = chars.flat_map(|ch| ch.to_lowercase()).collect();
                    format!("{upper}{rest}")
                }
            }
        })
        .collect()
}

/// Generate a complete TypeScript client module from handler metadata
pub fn generate_ts_client(handler_metas: &HashMap<String, HandlerMeta>) -> String {
    let mut output = String::new();

    // Header
    output.push_str("// Auto-generated by AllFrame. Do not edit manually.\n");
    output.push_str("// Regenerate with: allframe generate-ts-client\n\n");
    output.push_str("import { invoke } from \"@tauri-apps/api/core\";\n\n");

    // Internal helper that unwraps CallResponse and parses JSON
    output.push_str("/** @internal Unwrap CallResponse and parse the JSON result. */\n");
    output.push_str("async function callHandler<T>(handler: string, args: Record<string, unknown> = {}): Promise<T> {\n");
    output.push_str("  const response = await invoke<{ result: string }>(\"plugin:allframe|allframe_call\", { handler, args });\n");
    output.push_str("  return JSON.parse(response.result) as T;\n");
    output.push_str("}\n\n");

    // Collect interfaces to generate
    let mut interfaces: Vec<(String, &[TsField])> = Vec::new();

    // Sort handlers by name for deterministic output
    let mut sorted_handlers: Vec<_> = handler_metas.iter().collect();
    sorted_handlers.sort_by_key(|(name, _)| (*name).clone());

    // First pass: collect interfaces from Object types
    for (handler_name, meta) in &sorted_handlers {
        let pascal = to_pascal_case(handler_name);

        // Args interface
        if !meta.args.is_empty() {
            interfaces.push((format!("{pascal}Args"), &meta.args));
        }

        // Return interface (if Object type)
        if let TsType::Object(fields) = &meta.returns {
            interfaces.push((format!("{pascal}Response"), fields));
        }
    }

    // Generate interfaces
    for (name, fields) in &interfaces {
        writeln!(output, "export interface {name} {{").unwrap();
        for field in *fields {
            if field.optional {
                writeln!(output, "  {}?: {};", field.name, field.ty.render()).unwrap();
            } else {
                writeln!(output, "  {}: {};", field.name, field.ty.render()).unwrap();
            }
        }
        output.push_str("}\n\n");
    }

    // Generate functions
    for (handler_name, meta) in &sorted_handlers {
        let fn_name = to_camel_case(handler_name);
        let pascal = to_pascal_case(handler_name);

        // Determine return type string
        let return_type = if let TsType::Object(_) = &meta.returns {
            format!("{pascal}Response")
        } else {
            meta.returns.render()
        };

        if meta.args.is_empty() {
            writeln!(
                output,
                "export async function {fn_name}(): Promise<{return_type}> {{",
            )
            .unwrap();
            writeln!(
                output,
                "  return callHandler<{return_type}>(\"{handler_name}\");",
            )
            .unwrap();
        } else {
            let args_type = format!("{pascal}Args");
            writeln!(
                output,
                "export async function {fn_name}(args: {args_type}): Promise<{return_type}> {{",
            )
            .unwrap();
            writeln!(
                output,
                "  return callHandler<{return_type}>(\"{handler_name}\", args);",
            )
            .unwrap();
        }

        output.push_str("}\n\n");
    }

    output.trim_end().to_string()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_camel_case() {
        assert_eq!(to_camel_case("get_user"), "getUser");
        assert_eq!(to_camel_case("create_new_item"), "createNewItem");
        assert_eq!(to_camel_case("hello"), "hello");
        assert_eq!(to_camel_case("GET_USER"), "getUser");
    }

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("get_user"), "GetUser");
        assert_eq!(to_pascal_case("create_new_item"), "CreateNewItem");
        assert_eq!(to_pascal_case("hello"), "Hello");
        assert_eq!(to_pascal_case("GET_USER"), "GetUser");
    }

    #[test]
    fn test_ts_type_render_primitives() {
        assert_eq!(TsType::String.render(), "string");
        assert_eq!(TsType::Number.render(), "number");
        assert_eq!(TsType::Boolean.render(), "boolean");
        assert_eq!(TsType::Null.render(), "null");
        assert_eq!(TsType::Void.render(), "void");
    }

    #[test]
    fn test_ts_type_render_optional() {
        let opt = TsType::Optional(Box::new(TsType::String));
        assert_eq!(opt.render(), "string | null");
    }

    #[test]
    fn test_ts_type_render_array() {
        let arr = TsType::Array(Box::new(TsType::Number));
        assert_eq!(arr.render(), "number[]");

        // Union in array gets parens
        let arr_opt = TsType::Array(Box::new(TsType::Optional(Box::new(TsType::String))));
        assert_eq!(arr_opt.render(), "(string | null)[]");
    }

    #[test]
    fn test_ts_type_render_object() {
        let obj = TsType::Object(vec![
            TsField::new("id", TsType::Number),
            TsField::new("name", TsType::String),
        ]);
        assert_eq!(obj.render(), "{ id: number; name: string }");
    }

    #[test]
    fn test_ts_type_render_named() {
        assert_eq!(TsType::Named("UserResponse".to_string()).render(), "UserResponse");
    }

    #[test]
    fn test_generate_no_args_handler() {
        let mut metas = HashMap::new();
        metas.insert(
            "get_status".to_string(),
            HandlerMeta {
                args: vec![],
                returns: TsType::String,
            },
        );

        let ts = generate_ts_client(&metas);
        assert!(ts.contains("export async function getStatus(): Promise<string>"));
        assert!(ts.contains("callHandler<string>(\"get_status\")"));
    }

    #[test]
    fn test_generate_with_args_handler() {
        let mut metas = HashMap::new();
        metas.insert(
            "greet".to_string(),
            HandlerMeta {
                args: vec![
                    TsField::new("name", TsType::String),
                    TsField::new("age", TsType::Number),
                ],
                returns: TsType::Object(vec![TsField::new("greeting", TsType::String)]),
            },
        );

        let ts = generate_ts_client(&metas);

        // Should generate args interface
        assert!(ts.contains("export interface GreetArgs {"));
        assert!(ts.contains("  name: string;"));
        assert!(ts.contains("  age: number;"));

        // Should generate response interface
        assert!(ts.contains("export interface GreetResponse {"));
        assert!(ts.contains("  greeting: string;"));

        // Should generate function using callHandler
        assert!(ts.contains("export async function greet(args: GreetArgs): Promise<GreetResponse>"));
        assert!(ts.contains("callHandler<GreetResponse>(\"greet\", args)"));
    }

    #[test]
    fn test_generate_optional_field() {
        let mut metas = HashMap::new();
        metas.insert(
            "search".to_string(),
            HandlerMeta {
                args: vec![
                    TsField::new("query", TsType::String),
                    TsField::optional("limit", TsType::Number),
                ],
                returns: TsType::Array(Box::new(TsType::String)),
            },
        );

        let ts = generate_ts_client(&metas);
        assert!(ts.contains("  query: string;"));
        assert!(ts.contains("  limit?: number;"));
        assert!(ts.contains("Promise<string[]>"));
    }

    #[test]
    fn test_generate_multiple_handlers_sorted() {
        let mut metas = HashMap::new();
        metas.insert(
            "delete_user".to_string(),
            HandlerMeta {
                args: vec![TsField::new("id", TsType::Number)],
                returns: TsType::Void,
            },
        );
        metas.insert(
            "create_user".to_string(),
            HandlerMeta {
                args: vec![TsField::new("name", TsType::String)],
                returns: TsType::Object(vec![TsField::new("id", TsType::Number)]),
            },
        );

        let ts = generate_ts_client(&metas);

        // create_user should come before delete_user (sorted)
        let create_pos = ts.find("createUser").unwrap();
        let delete_pos = ts.find("deleteUser").unwrap();
        assert!(create_pos < delete_pos);
    }

    #[test]
    fn test_generate_named_return_type() {
        let mut metas = HashMap::new();
        metas.insert(
            "get_user".to_string(),
            HandlerMeta {
                args: vec![TsField::new("id", TsType::Number)],
                returns: TsType::Named("User".to_string()),
            },
        );

        let ts = generate_ts_client(&metas);
        assert!(ts.contains("Promise<User>"));
        // Named types don't generate interfaces
        assert!(!ts.contains("export interface GetUserResponse"));
    }

    #[test]
    fn test_generate_header_and_helper() {
        let metas = HashMap::new();
        let ts = generate_ts_client(&metas);
        assert!(ts.contains("Auto-generated by AllFrame"));
        assert!(ts.contains("import { invoke }"));
        assert!(ts.contains("async function callHandler<T>"));
        assert!(ts.contains("JSON.parse(response.result)"));
    }

    #[test]
    fn test_generate_idempotent() {
        let mut metas = HashMap::new();
        metas.insert(
            "greet".to_string(),
            HandlerMeta {
                args: vec![TsField::new("name", TsType::String)],
                returns: TsType::String,
            },
        );

        let ts1 = generate_ts_client(&metas);
        let ts2 = generate_ts_client(&metas);
        assert_eq!(ts1, ts2);
    }

    #[test]
    fn test_full_example_output() {
        let mut metas = HashMap::new();
        metas.insert(
            "get_user".to_string(),
            HandlerMeta {
                args: vec![TsField::new("id", TsType::Number)],
                returns: TsType::Object(vec![
                    TsField::new("id", TsType::Number),
                    TsField::new("name", TsType::String),
                    TsField::optional("email", TsType::String),
                ]),
            },
        );

        let ts = generate_ts_client(&metas);

        // Verify complete structure
        assert!(ts.contains("export interface GetUserArgs {\n  id: number;\n}"));
        assert!(ts.contains("export interface GetUserResponse {\n  id: number;\n  name: string;\n  email?: string;\n}"));
        assert!(ts.contains(
            "export async function getUser(args: GetUserArgs): Promise<GetUserResponse>"
        ));
        assert!(ts.contains("callHandler<GetUserResponse>(\"get_user\", args)"));
    }
}
