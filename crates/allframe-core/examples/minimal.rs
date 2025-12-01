//! Minimal example for binary size measurement
//!
//! This example is used to measure the baseline binary size
//! when using allframe-core with no features enabled.

use allframe_core::router::Router;

fn main() {
    // Create a minimal router
    let router = Router::new();

    // Print some basic info to prevent compiler optimizations
    println!("Handlers registered: {}", router.handlers_count());
}
