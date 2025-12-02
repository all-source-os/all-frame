//! GraphQL protocol adapter
//!
//! Provides GraphQL support for the protocol-agnostic router.

use std::{future::Future, pin::Pin};

use super::ProtocolAdapter;

/// GraphQL operation type
#[derive(Debug, Clone, PartialEq)]
pub enum OperationType {
    /// Query operation (read)
    Query,
    /// Mutation operation (write)
    Mutation,
}

/// GraphQL operation definition
#[derive(Debug, Clone)]
pub struct GraphQLOperation {
    /// Operation type (query or mutation)
    pub operation_type: OperationType,
    /// Operation name (e.g., "user", "createUser")
    pub name: String,
    /// Handler name to call
    pub handler: String,
}

impl GraphQLOperation {
    /// Create a new GraphQL operation
    pub fn new(
        operation_type: OperationType,
        name: impl Into<String>,
        handler: impl Into<String>,
    ) -> Self {
        Self {
            operation_type,
            name: name.into(),
            handler: handler.into(),
        }
    }
}

/// GraphQL adapter for GraphQL queries and mutations
///
/// Handles GraphQL protocol-specific request/response transformation.
pub struct GraphQLAdapter {
    operations: Vec<GraphQLOperation>,
}

impl GraphQLAdapter {
    /// Create a new GraphQL adapter
    pub fn new() -> Self {
        Self {
            operations: Vec::new(),
        }
    }

    /// Register a GraphQL query
    pub fn query(&mut self, name: &str, handler: &str) -> &mut Self {
        self.operations.push(GraphQLOperation::new(
            OperationType::Query,
            name,
            handler,
        ));
        self
    }

    /// Register a GraphQL mutation
    pub fn mutation(&mut self, name: &str, handler: &str) -> &mut Self {
        self.operations.push(GraphQLOperation::new(
            OperationType::Mutation,
            name,
            handler,
        ));
        self
    }

    /// Find a matching operation by name and type
    pub fn match_operation(
        &self,
        operation_type: OperationType,
        name: &str,
    ) -> Option<&GraphQLOperation> {
        self.operations
            .iter()
            .find(|op| op.operation_type == operation_type && op.name == name)
    }

    /// Parse a GraphQL query string
    ///
    /// Extracts operation type and operation name from a GraphQL query.
    /// Supports:
    /// - Named queries: "query GetUser { user }"
    /// - Shorthand queries: "{ user }"
    /// - Named mutations: "mutation CreateUser { createUser }"
    pub fn parse_query(&self, query: &str) -> Result<(OperationType, String), String> {
        let trimmed = query.trim();

        if trimmed.is_empty() {
            return Err("Empty GraphQL query".to_string());
        }

        // Check for explicit operation type
        if let Some(stripped) = trimmed.strip_prefix("query") {
            // Extract operation name from query body
            let name = self.extract_operation_name(stripped)?;
            Ok((OperationType::Query, name))
        } else if let Some(stripped) = trimmed.strip_prefix("mutation") {
            // Extract operation name from mutation body
            let name = self.extract_operation_name(stripped)?;
            Ok((OperationType::Mutation, name))
        } else if trimmed.starts_with('{') {
            // Shorthand query syntax: { user }
            let name = self.extract_operation_name(trimmed)?;
            Ok((OperationType::Query, name))
        } else {
            Err("Invalid GraphQL query format".to_string())
        }
    }

    /// Extract operation name from query body
    ///
    /// Handles: "{ user }", "GetUser { user }", etc.
    fn extract_operation_name(&self, query_body: &str) -> Result<String, String> {
        let trimmed = query_body.trim();

        // Find opening brace
        if let Some(brace_pos) = trimmed.find('{') {
            // Extract content between braces
            let content = &trimmed[brace_pos + 1..];
            if let Some(close_pos) = content.find('}') {
                let inner = content[..close_pos].trim();
                // Take first word (operation name)
                if let Some(name) = inner.split_whitespace().next() {
                    // Remove parentheses if present (e.g., "user(id: 42)" -> "user")
                    let name = if let Some(paren_pos) = name.find('(') {
                        &name[..paren_pos]
                    } else {
                        name
                    };
                    return Ok(name.to_string());
                }
            }
        }

        Err("Could not extract operation name from query".to_string())
    }

    /// Generate GraphQL schema for registered operations
    ///
    /// Generates GraphQL Schema Definition Language (SDL) from registered operations.
    pub fn generate_schema(&self) -> String {
        let mut schema = String::new();

        // Generate Query type
        let queries: Vec<&GraphQLOperation> = self
            .operations
            .iter()
            .filter(|op| op.operation_type == OperationType::Query)
            .collect();

        if !queries.is_empty() {
            schema.push_str("type Query {\n");
            for query in &queries {
                schema.push_str(&format!("  {}: String\n", query.name));
            }
            schema.push_str("}\n\n");
        }

        // Generate Mutation type
        let mutations: Vec<&GraphQLOperation> = self
            .operations
            .iter()
            .filter(|op| op.operation_type == OperationType::Mutation)
            .collect();

        if !mutations.is_empty() {
            schema.push_str("type Mutation {\n");
            for mutation in &mutations {
                schema.push_str(&format!("  {}: String\n", mutation.name));
            }
            schema.push_str("}\n\n");
        }

        // Generate schema definition
        if !queries.is_empty() || !mutations.is_empty() {
            schema.push_str("schema {\n");
            if !queries.is_empty() {
                schema.push_str("  query: Query\n");
            }
            if !mutations.is_empty() {
                schema.push_str("  mutation: Mutation\n");
            }
            schema.push_str("}\n");
        }

        schema.trim().to_string()
    }
}

impl Default for GraphQLAdapter {
    fn default() -> Self {
        Self::new()
    }
}

impl ProtocolAdapter for GraphQLAdapter {
    fn name(&self) -> &str {
        "graphql"
    }

    fn handle(
        &self,
        request: &str,
    ) -> Pin<Box<dyn Future<Output = Result<String, String>> + Send + '_>> {
        // Parse query before async block
        let parse_result = self.parse_query(request);
        let operations = self.operations.clone();

        Box::pin(async move {
            // Handle parse error
            let (operation_type, operation_name) = match parse_result {
                Ok(parsed) => parsed,
                Err(e) => {
                    let response = format!(r#"{{"errors":[{{"message":"{}"}}]}}"#, e);
                    return Ok(response);
                }
            };

            // Find matching operation
            let matched_operation = operations
                .iter()
                .find(|op| op.operation_type == operation_type && op.name == operation_name);

            match matched_operation {
                Some(operation) => {
                    // In full implementation, would call handler here
                    // For now, return success with handler name
                    let response = format!(
                        r#"{{"data":{{"{}":"{}"}},"extensions":{{"handler":"{}"}}}}"#,
                        operation.name, operation.name, operation.handler
                    );
                    Ok(response)
                }
                None => {
                    // Operation not found
                    let error = format!(
                        r#"{{"errors":[{{"message":"Operation not found: {}"}}]}}"#,
                        operation_name
                    );
                    Ok(error)
                }
            }
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphql_adapter_creation() {
        let adapter = GraphQLAdapter::new();
        assert_eq!(adapter.name(), "graphql");
    }

    #[test]
    fn test_operation_registration_query() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");

        assert_eq!(adapter.operations.len(), 1);
        assert_eq!(adapter.operations[0].operation_type, OperationType::Query);
        assert_eq!(adapter.operations[0].name, "user");
        assert_eq!(adapter.operations[0].handler, "get_user");
    }

    #[test]
    fn test_operation_registration_mutation() {
        let mut adapter = GraphQLAdapter::new();
        adapter.mutation("createUser", "create_user_handler");

        assert_eq!(adapter.operations.len(), 1);
        assert_eq!(adapter.operations[0].operation_type, OperationType::Mutation);
        assert_eq!(adapter.operations[0].name, "createUser");
        assert_eq!(adapter.operations[0].handler, "create_user_handler");
    }

    #[test]
    fn test_operation_registration_multiple() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");
        adapter.query("users", "list_users");
        adapter.mutation("createUser", "create_user");

        assert_eq!(adapter.operations.len(), 3);
    }

    #[test]
    fn test_match_operation_query() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");

        let matched = adapter.match_operation(OperationType::Query, "user");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().handler, "get_user");
    }

    #[test]
    fn test_match_operation_mutation() {
        let mut adapter = GraphQLAdapter::new();
        adapter.mutation("createUser", "create_user");

        let matched = adapter.match_operation(OperationType::Mutation, "createUser");
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().handler, "create_user");
    }

    #[test]
    fn test_match_operation_not_found() {
        let adapter = GraphQLAdapter::new();
        let matched = adapter.match_operation(OperationType::Query, "nonexistent");
        assert!(matched.is_none());
    }

    #[test]
    fn test_match_operation_wrong_type() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");

        // Try to find as mutation (should fail)
        let matched = adapter.match_operation(OperationType::Mutation, "user");
        assert!(matched.is_none());
    }

    #[test]
    fn test_parse_query_named() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("query GetUser { user }");

        assert!(result.is_ok());
        let (op_type, name) = result.unwrap();
        assert_eq!(op_type, OperationType::Query);
        assert_eq!(name, "user");
    }

    #[test]
    fn test_parse_query_shorthand() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("{ user }");

        assert!(result.is_ok());
        let (op_type, name) = result.unwrap();
        assert_eq!(op_type, OperationType::Query);
        assert_eq!(name, "user");
    }

    #[test]
    fn test_parse_query_with_args() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("query { user(id: 42) }");

        assert!(result.is_ok());
        let (op_type, name) = result.unwrap();
        assert_eq!(op_type, OperationType::Query);
        assert_eq!(name, "user");
    }

    #[test]
    fn test_parse_mutation_named() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("mutation CreateUser { createUser }");

        assert!(result.is_ok());
        let (op_type, name) = result.unwrap();
        assert_eq!(op_type, OperationType::Mutation);
        assert_eq!(name, "createUser");
    }

    #[test]
    fn test_parse_mutation_with_args() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("mutation { createUser(name: \"John\") }");

        assert!(result.is_ok());
        let (op_type, name) = result.unwrap();
        assert_eq!(op_type, OperationType::Mutation);
        assert_eq!(name, "createUser");
    }

    #[test]
    fn test_parse_query_empty() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Empty"));
    }

    #[test]
    fn test_parse_query_invalid() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.parse_query("invalid query");

        assert!(result.is_err());
        assert!(result.unwrap_err().contains("Invalid GraphQL query format"));
    }

    #[test]
    fn test_schema_generation_empty() {
        let adapter = GraphQLAdapter::new();
        let schema = adapter.generate_schema();
        assert_eq!(schema, "");
    }

    #[test]
    fn test_schema_generation_with_queries() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");
        adapter.query("users", "list_users");

        let schema = adapter.generate_schema();
        assert!(schema.contains("type Query {"));
        assert!(schema.contains("user: String"));
        assert!(schema.contains("users: String"));
        assert!(schema.contains("schema {"));
        assert!(schema.contains("query: Query"));
    }

    #[test]
    fn test_schema_generation_with_mutations() {
        let mut adapter = GraphQLAdapter::new();
        adapter.mutation("createUser", "create_user");
        adapter.mutation("deleteUser", "delete_user");

        let schema = adapter.generate_schema();
        assert!(schema.contains("type Mutation {"));
        assert!(schema.contains("createUser: String"));
        assert!(schema.contains("deleteUser: String"));
        assert!(schema.contains("schema {"));
        assert!(schema.contains("mutation: Mutation"));
    }

    #[test]
    fn test_schema_generation_with_both() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");
        adapter.mutation("createUser", "create_user");

        let schema = adapter.generate_schema();
        assert!(schema.contains("type Query {"));
        assert!(schema.contains("type Mutation {"));
        assert!(schema.contains("query: Query"));
        assert!(schema.contains("mutation: Mutation"));
    }

    #[tokio::test]
    async fn test_handle_query_success() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");

        let result = adapter.handle("query { user }").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains(r#""data""#));
        assert!(response.contains("user"));
        assert!(response.contains("get_user"));
    }

    #[tokio::test]
    async fn test_handle_mutation_success() {
        let mut adapter = GraphQLAdapter::new();
        adapter.mutation("createUser", "create_user_handler");

        let result = adapter.handle("mutation { createUser }").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains(r#""data""#));
        assert!(response.contains("createUser"));
        assert!(response.contains("create_user_handler"));
    }

    #[tokio::test]
    async fn test_handle_operation_not_found() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.handle("query { user }").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains(r#""errors""#));
        assert!(response.contains("Operation not found"));
    }

    #[tokio::test]
    async fn test_handle_invalid_query() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.handle("invalid query").await;

        assert!(result.is_ok());
        let response = result.unwrap();
        assert!(response.contains(r#""errors""#));
    }

    #[tokio::test]
    async fn test_handle_shorthand_query() {
        let mut adapter = GraphQLAdapter::new();
        adapter.query("user", "get_user");

        let result = adapter.handle("{ user }").await;
        assert!(result.is_ok());

        let response = result.unwrap();
        assert!(response.contains(r#""data""#));
        assert!(response.contains("user"));
    }

    #[test]
    fn test_graphql_operation_new() {
        let op = GraphQLOperation::new(OperationType::Query, "user", "get_user");
        assert_eq!(op.operation_type, OperationType::Query);
        assert_eq!(op.name, "user");
        assert_eq!(op.handler, "get_user");
    }
}
