//! Security Patterns Example
//!
//! This example demonstrates AllFrame's security utilities for safe logging:
//! - URL obfuscation (hide credentials, paths, query strings)
//! - API key obfuscation (show prefix/suffix only)
//! - Header obfuscation (smart handling of Authorization, Cookie, etc.)
//! - Sensitive<T> wrapper (always shows "***")
//! - #[derive(Obfuscate)] macro (auto-generate safe Debug/Display)
//!
//! Run with: cargo run --example security --features security

use allframe_core::{
    security::{
        obfuscate_api_key, obfuscate_header, obfuscate_redis_url, obfuscate_url, Obfuscate,
        Sensitive,
    },
    Obfuscate as ObfuscateMacro,
};

fn main() {
    println!("=== AllFrame Security Patterns Demo ===\n");

    // 1. URL Obfuscation
    demo_url_obfuscation();

    // 2. Redis URL Obfuscation
    demo_redis_url_obfuscation();

    // 3. API Key Obfuscation
    demo_api_key_obfuscation();

    // 4. Header Obfuscation
    demo_header_obfuscation();

    // 5. Sensitive Wrapper
    demo_sensitive_wrapper();

    // 6. Derive Obfuscate Macro
    demo_derive_obfuscate();

    println!("\n=== Demo Complete ===");
}

fn demo_url_obfuscation() {
    println!("--- 1. URL Obfuscation ---");

    let examples = [
        "https://user:password@api.example.com:8080/v1/users?token=secret",
        "http://admin:hunter2@localhost:3000/api/data",
        "https://api.example.com/public/info",
        "postgresql://db_user:db_pass@db.host.com:5432/mydb",
    ];

    for url in examples {
        println!("  Original: {}", url);
        println!("  Obfuscated: {}", obfuscate_url(url));
        println!();
    }
}

fn demo_redis_url_obfuscation() {
    println!("--- 2. Redis URL Obfuscation ---");

    let examples = [
        "redis://:secretpassword@redis.example.com:6379/0",
        "redis://default:mypass@localhost:6379",
        "rediss://user:pass@redis-cluster.aws.com:6380/1",
        "redis://localhost:6379",
    ];

    for url in examples {
        println!("  Original: {}", url);
        println!("  Obfuscated: {}", obfuscate_redis_url(url));
        println!();
    }
}

fn demo_api_key_obfuscation() {
    println!("--- 3. API Key Obfuscation ---");

    let examples = [
        "sk_live_1234567890abcdefghij",
        "pk_test_abcdefghijklmnopqrst",
        "api_key_very_long_secret_value_here",
        "short", // Short keys handled gracefully
        "",      // Empty keys handled gracefully
    ];

    for key in examples {
        println!("  Original: \"{}\"", key);
        println!("  Obfuscated: {}", obfuscate_api_key(key));
        println!();
    }
}

fn demo_header_obfuscation() {
    println!("--- 4. Header Obfuscation ---");

    let headers = [
        (
            "Authorization",
            "Bearer eyJhbGciOiJIUzI1NiIsInR5cCI6IkpXVCJ9.secret",
        ),
        ("Cookie", "session=abc123; csrf=xyz789"),
        ("X-API-Key", "sk_live_1234567890abcdef"),
        ("X-Request-ID", "req-12345"),        // Non-sensitive header
        ("Content-Type", "application/json"), // Non-sensitive header
        ("Set-Cookie", "session=newsession; HttpOnly; Secure"),
    ];

    for (name, value) in headers {
        println!("  {}: {}", name, value);
        println!("  Obfuscated: {}: {}", name, obfuscate_header(name, value));
        println!();
    }
}

fn demo_sensitive_wrapper() {
    println!("--- 5. Sensitive<T> Wrapper ---");

    let password = Sensitive::new("super_secret_password".to_string());
    let api_key = Sensitive::new("sk_live_abcdef123456");
    let token = Sensitive::new(vec![0x12, 0x34, 0x56, 0x78]);

    println!("  Password (Debug): {:?}", password);
    println!("  Password (Display): {}", password);
    println!();
    println!("  API Key (Debug): {:?}", api_key);
    println!("  API Key (Display): {}", api_key);
    println!();
    println!("  Token bytes (Debug): {:?}", token);
    println!("  Token bytes (Display): {}", token);
    println!();

    // Access the inner value when needed
    println!("  Actual password (inner): {}", password.as_inner());
    println!();
}

fn demo_derive_obfuscate() {
    println!("--- 6. #[derive(Obfuscate)] Macro ---");

    // Example struct with sensitive fields
    #[derive(ObfuscateMacro)]
    struct UserCredentials {
        username: String,
        #[sensitive]
        password: String,
        #[sensitive]
        api_key: String,
        role: String,
    }

    let creds = UserCredentials {
        username: "alice".to_string(),
        password: "super_secret_123".to_string(),
        api_key: "sk_live_abcdef123456".to_string(),
        role: "admin".to_string(),
    };

    println!("  UserCredentials obfuscated:");
    println!("  {}", creds.obfuscate());
    println!();

    // Example with nested sensitive data
    #[derive(ObfuscateMacro)]
    struct DatabaseConfig {
        host: String,
        port: u16,
        database: String,
        #[sensitive]
        username: String,
        #[sensitive]
        password: String,
    }

    let db_config = DatabaseConfig {
        host: "db.example.com".to_string(),
        port: 5432,
        database: "production".to_string(),
        username: "db_admin".to_string(),
        password: "very_secret_db_pass".to_string(),
    };

    println!("  DatabaseConfig obfuscated:");
    println!("  {}", db_config.obfuscate());
    println!();

    // Show that sensitive fields are properly hidden in logs
    println!("  Safe for logging:");
    println!("  - Connecting to database: {}", db_config.obfuscate());
}
