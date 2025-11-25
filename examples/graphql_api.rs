//! GraphQL API Example
//!
//! This example demonstrates how to use AllFrame's GraphQL adapter to build a GraphQL API.
//!
//! Key concepts:
//! - Router setup with GraphQL adapter
//! - Query and mutation handlers
//! - GraphQL schema generation
//! - Query execution and validation
//! - Nested type support
//!
//! Run this example:
//! ```bash
//! cargo run --example graphql_api
//! ```

use allframe_core::prelude::*;

#[tokio::main]
async fn main() {
    println!("=== AllFrame GraphQL API Example ===\n");

    // Create a new router
    let mut router = Router::new();

    // Register handlers for GraphQL operations
    // Each handler is a simple async function that returns JSON

    // Query: user(id: Int!) - Get a user by ID
    router.register("user", || async move {
        r#"{"id": 42, "name": "John Doe", "email": "john@example.com"}"#.to_string()
    });

    // Query: users - List all users
    router.register("users", || async move {
        r#"[{"id": 1, "name": "Alice"}, {"id": 2, "name": "Bob"}]"#.to_string()
    });

    // Mutation: createUser - Create a new user
    router.register("createUser", || async move {
        r#"{"id": 3, "name": "Charlie", "email": "charlie@example.com"}"#.to_string()
    });

    // Mutation: updateUser - Update a user
    router.register("updateUser", || async move {
        r#"{"id": 42, "name": "John Updated", "email": "john.new@example.com"}"#.to_string()
    });

    // Create and register the GraphQL adapter
    let adapter = GraphQLAdapter::new();
    router.add_adapter(Box::new(adapter));

    println!("✓ Router initialized with {} handlers", router.handlers_count());
    println!("✓ GraphQL adapter registered\n");

    // Demonstrate the GraphQL adapter capabilities
    let graphql_adapter = GraphQLAdapter::new();

    println!("--- Example 1: GraphQL Schema Generation ---");
    // Generate the GraphQL schema
    let schema = graphql_adapter.generate_schema();
    println!("Generated GraphQL Schema:\n{}\n", schema);

    println!("--- Example 2: Simple Query ---");
    // Execute a simple GraphQL query
    let simple_query = r#"
        query {
            user(id: 42) {
                id
                name
            }
        }
    "#;
    println!("Query:\n{}", simple_query);

    let response = graphql_adapter.execute(simple_query).await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 3: Query with All Fields ---");
    // Execute a query requesting all fields
    let full_query = r#"
        query {
            user(id: 42) {
                id
                name
                email
            }
        }
    "#;
    println!("Query:\n{}", full_query);

    let response = graphql_adapter.execute(full_query).await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 4: Shorthand Query Syntax ---");
    // GraphQL allows shorthand query syntax without the 'query' keyword
    let shorthand_query = r#"
        {
            user(id: 42) {
                name
            }
        }
    "#;
    println!("Query:\n{}", shorthand_query);

    let response = graphql_adapter.execute(shorthand_query).await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 5: GraphQL Mutation ---");
    // Execute a GraphQL mutation
    let mutation = r#"
        mutation {
            createUser(name: "Charlie", email: "charlie@example.com") {
                id
                name
                email
            }
        }
    "#;
    println!("Mutation:\n{}", mutation);

    let response = graphql_adapter.execute(mutation).await.unwrap();
    println!("Response: {}\n", response);

    println!("--- Example 6: Update Mutation ---");
    // Execute an update mutation
    let update_mutation = r#"
        mutation {
            updateUser(id: 42, name: "John Updated") {
                id
                name
            }
        }
    "#;
    println!("Mutation:\n{}", update_mutation);

    // For MVP, we use the registered handler
    let response = router.execute("updateUser").await.unwrap();
    println!("Handler Response: {}\n", response);

    println!("--- Example 7: Error Handling ---");
    // Try to execute an invalid query
    let invalid_query = "this is not a valid graphql query";
    println!("Invalid Query: {}", invalid_query);

    match graphql_adapter.execute(invalid_query).await {
        Ok(response) => println!("Response: {}", response),
        Err(error) => println!("Error: {}\n", error),
    }

    println!("--- Example 8: Nested Types (MVP) ---");
    // For MVP, nested types are supported in the response format
    let nested_query = r#"
        query {
            user(id: 42) {
                name
                address {
                    city
                    street
                }
            }
        }
    "#;
    println!("Query with nested types:\n{}", nested_query);

    let response = graphql_adapter.execute(nested_query).await.unwrap();
    println!("Response: {}\n", response);
    println!("Note: Full nested type support with field selection coming in future phases\n");

    println!("--- Example 9: Using ProtocolAdapter Trait ---");
    // The GraphQL adapter also implements the ProtocolAdapter trait
    println!("Adapter name: {}", graphql_adapter.name());

    // Use the handle method (same as execute for GraphQL)
    let query = "query { user(id: 42) }";
    let response = graphql_adapter.handle(query).await.unwrap();
    println!("Via handle(): {}\n", response);

    println!("=== Key Takeaways ===");
    println!("1. GraphQL adapter supports both query and mutation operations");
    println!("2. Schema generation creates GraphQL SDL from handlers");
    println!("3. Both explicit 'query' and shorthand '{{}}' syntax supported");
    println!("4. Mutations follow the same pattern as queries");
    println!("5. Error handling validates GraphQL query syntax");
    println!("6. MVP provides basic functionality; full parsing coming later");
    println!("\n✓ GraphQL API example complete!");
    println!("\nNext steps:");
    println!("- See examples/rest_api.rs for REST support");
    println!("- See examples/multi_protocol.rs for multi-protocol routing");
    println!("- Full GraphQL AST parsing and resolver system coming in future phases");
}
