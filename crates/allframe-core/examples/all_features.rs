//! All features example for binary size measurement
//!
//! Measures binary size with all features enabled

use allframe_core::prelude::*;

#[tokio::main]
async fn main() {
    // Create a router
    let mut router = Router::new();
    router.get("/users", || async { "Users".to_string() });

    // Generate OpenAPI and Scalar docs
    let _openapi = router.to_openapi("API", "1.0.0");
    let _scalar_html = router.scalar("API", "1.0.0");

    // Print info to prevent optimizations
    println!("Router handlers: {}", router.handlers_count());
    println!("All features example complete");
}
