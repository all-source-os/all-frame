//! API Handler Macro Implementation
//!
//! This module implements the `#[api_handler]` procedural macro for automatic
//! OpenAPI 3.1 schema generation.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, Result};

/// Implementation of the #[api_handler] macro
///
/// Generates:
/// - Keeps the original function intact
/// - Generates a `{function_name}_openapi_schema()` function that returns JSON schema
///
/// For MVP (v0.2), we generate a minimal valid OpenAPI schema.
/// Full type introspection will be added in v0.3.
pub fn api_handler_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    // Parse the function
    let func: ItemFn = parse2(item.clone())?;
    let func_name = &func.sig.ident;

    // Parse attributes to extract path, method, description
    let attr_str = attr.to_string();

    // Simple parsing for MVP - just extract key values
    let path = extract_attr_value(&attr_str, "path").unwrap_or("/".to_string());
    let method = extract_attr_value(&attr_str, "method").unwrap_or("GET".to_string());
    let description = extract_attr_value(&attr_str, "description")
        .unwrap_or_else(|| format!("{} endpoint", func_name));

    // Generate the schema function name
    let schema_fn_name =
        syn::Ident::new(&format!("{}_openapi_schema", func_name), func_name.span());

    // Generate minimal OpenAPI schema as JSON string
    let schema = format!(
        r#"{{
  "openapi": "3.1.0",
  "info": {{
    "title": "API",
    "version": "1.0.0"
  }},
  "paths": {{
    "{}": {{
      "{}": {{
        "description": "{}",
        "responses": {{
          "200": {{
            "description": "Successful response"
          }}
        }}
      }}
    }}
  }}
}}"#,
        path,
        method.to_lowercase(),
        description
    );

    // Generate the output
    let expanded = quote! {
        // Keep the original function
        #func

        // Generate schema function
        fn #schema_fn_name() -> String {
            #schema.to_string()
        }
    };

    Ok(expanded)
}

/// Extract value from attribute string
/// Format: key = "value" or key = 'value'
fn extract_attr_value(attr_str: &str, key: &str) -> Option<String> {
    // Look for key = "value" or key = 'value'
    let key_pattern = format!("{} =", key);

    if let Some(start_pos) = attr_str.find(&key_pattern) {
        let after_key = &attr_str[start_pos + key_pattern.len()..];

        // Skip whitespace
        let trimmed = after_key.trim_start();

        // Check for quotes
        if let Some(quote_char) = trimmed.chars().next() {
            if quote_char == '"' || quote_char == '\'' {
                // Find the closing quote
                if let Some(end_pos) = trimmed[1..].find(quote_char) {
                    return Some(trimmed[1..=end_pos].to_string());
                }
            }
        }
    }

    None
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_attr_value() {
        assert_eq!(
            extract_attr_value(r#"path = "/users", method = "POST""#, "path"),
            Some("/users".to_string())
        );
        assert_eq!(
            extract_attr_value(r#"path = "/users", method = "POST""#, "method"),
            Some("POST".to_string())
        );
        assert_eq!(
            extract_attr_value(r#"description = "Create user""#, "description"),
            Some("Create user".to_string())
        );
    }
}
