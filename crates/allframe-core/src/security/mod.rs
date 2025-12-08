//! Security utilities for safe logging and data handling.
//!
//! This module provides utilities for:
//! - **Obfuscation**: Safe logging of sensitive data (URLs, API keys, headers)
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::security::{obfuscate_url, obfuscate_api_key};
//!
//! let url = "https://user:password@api.example.com/v1/data?key=secret";
//! println!("Connecting to: {}", obfuscate_url(url));
//! // Output: "Connecting to: https://api.example.com/***"
//!
//! let key = "sk_live_abcdefghijklmnop";
//! println!("Using key: {}", obfuscate_api_key(key));
//! // Output: "Using key: sk_l***mnop"
//! ```

mod obfuscation;

pub use obfuscation::{
    obfuscate_api_key, obfuscate_header, obfuscate_redis_url, obfuscate_url, Obfuscate, Sensitive,
};
