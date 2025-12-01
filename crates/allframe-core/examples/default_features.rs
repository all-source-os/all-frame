//! Default features example for binary size measurement
//!
//! Measures binary size with default features: di, openapi, router

use allframe_core::prelude::*;

fn main() {
    // Create a router with routes
    let mut router = Router::new();
    router.get("/users", || async { "Users".to_string() });
    router.post("/users", || async { "Created".to_string() });

    // Generate OpenAPI spec
    let _spec = router.to_openapi("API", "1.0.0");

    // Print info to prevent optimizations
    println!("Router handlers: {}", router.handlers_count());
}
