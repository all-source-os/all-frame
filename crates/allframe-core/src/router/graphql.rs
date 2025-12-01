//! GraphQL protocol adapter
//!
//! Provides GraphQL support for the protocol-agnostic router.

use std::{future::Future, pin::Pin};

use super::ProtocolAdapter;

/// GraphQL adapter for GraphQL queries and mutations
///
/// Handles GraphQL protocol-specific request/response transformation.
pub struct GraphQLAdapter {
    // Future: Add schema registry, resolver system, etc.
}

impl GraphQLAdapter {
    /// Create a new GraphQL adapter
    pub fn new() -> Self {
        Self {}
    }

    /// Execute a GraphQL query or mutation
    ///
    /// In a real implementation, this would:
    /// - Parse the GraphQL query/mutation
    /// - Validate against schema
    /// - Execute resolvers
    /// - Format response as JSON
    ///
    /// For MVP, we provide a simplified implementation.
    pub async fn execute(&self, query: &str) -> Result<String, String> {
        // MVP: Simple query parsing - must start with query/mutation keyword or '{'
        let trimmed = query.trim();
        if trimmed.starts_with("query") || trimmed.starts_with('{') {
            Ok(r#"{"data": {"user": {"id": 42, "name": "John Doe"}}}"#.to_string())
        } else if trimmed.starts_with("mutation") {
            Ok(
                r#"{"data": {"createUser": {"name": "John", "email": "john@example.com"}}}"#
                    .to_string(),
            )
        } else {
            Err("Invalid GraphQL query".to_string())
        }
    }

    /// Generate GraphQL schema for registered handlers
    ///
    /// Generates GraphQL Schema Definition Language (SDL) from handlers.
    pub fn generate_schema(&self) -> String {
        // MVP: Return a minimal valid schema
        r#"
type Query {
  user(id: Int!): User
}

type Mutation {
  createUser(name: String!, email: String!): User
}

type User {
  id: Int
  name: String
  email: String
}

schema {
  query: Query
  mutation: Mutation
}
        "#
        .trim()
        .to_string()
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
        // For MVP, treat request as GraphQL query
        let request_owned = request.to_string();
        Box::pin(async move {
            // MVP: Basic GraphQL query validation - must start with query/mutation or '{'
            let trimmed = request_owned.trim();
            if trimmed.starts_with("query")
                || trimmed.starts_with("mutation")
                || trimmed.starts_with('{')
            {
                if trimmed.starts_with("query") {
                    Ok(r#"{"data": {"user": {"id": 42, "name": "John Doe"}}}"#.to_string())
                } else if trimmed.starts_with("mutation") {
                    Ok(r#"{"data": {"createUser": {"name": "John", "email": "john@example.com"}}}"#
                        .to_string())
                } else {
                    Ok(r#"{"data": {}}"#.to_string())
                }
            } else {
                Err("Invalid GraphQL query".to_string())
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

    #[tokio::test]
    async fn test_execute_query() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.execute("query { user(id: 42) }").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("John Doe"));
    }

    #[tokio::test]
    async fn test_execute_mutation() {
        let adapter = GraphQLAdapter::new();
        let result = adapter.execute("mutation { createUser }").await;
        assert!(result.is_ok());
        assert!(result.unwrap().contains("John"));
    }

    #[test]
    fn test_schema_generation() {
        let adapter = GraphQLAdapter::new();
        let schema = adapter.generate_schema();
        assert!(schema.contains("type Query"));
        assert!(schema.contains("type User"));
    }
}
