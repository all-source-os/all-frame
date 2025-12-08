//! Obfuscation utilities for safe logging of sensitive data.
//!
//! These functions help prevent accidental exposure of sensitive information
//! in logs, error messages, and debug output.

use url::Url;

/// Obfuscate a URL by removing credentials, path, and query parameters.
///
/// # Example
///
/// ```
/// use allframe_core::security::obfuscate_url;
///
/// let url = "https://user:pass@api.example.com/v1/data?key=secret";
/// assert_eq!(obfuscate_url(url), "https://api.example.com/***");
///
/// let simple = "https://example.com";
/// assert_eq!(obfuscate_url(simple), "https://example.com/***");
/// ```
pub fn obfuscate_url(url: &str) -> String {
    match Url::parse(url) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            let host = parsed.host_str().unwrap_or("unknown");
            let port = parsed.port().map(|p| format!(":{}", p)).unwrap_or_default();

            format!("{}://{}{}/***", scheme, host, port)
        }
        Err(_) => {
            // If parsing fails, just show a generic placeholder
            "***invalid-url***".to_string()
        }
    }
}

/// Obfuscate a Redis URL, preserving host and port but hiding authentication.
///
/// # Example
///
/// ```
/// use allframe_core::security::obfuscate_redis_url;
///
/// let url = "redis://:password@localhost:6379/0";
/// assert_eq!(obfuscate_redis_url(url), "redis://***@localhost:6379/***");
///
/// let simple = "redis://localhost:6379";
/// assert_eq!(obfuscate_redis_url(simple), "redis://localhost:6379/***");
/// ```
pub fn obfuscate_redis_url(url: &str) -> String {
    match Url::parse(url) {
        Ok(parsed) => {
            let scheme = parsed.scheme();
            let host = parsed.host_str().unwrap_or("unknown");
            let port = parsed.port().map(|p| format!(":{}", p)).unwrap_or_default();

            // Check if there's authentication
            let has_auth = !parsed.username().is_empty() || parsed.password().is_some();
            let auth_part = if has_auth { "***@" } else { "" };

            format!("{}://{}{}{}/***", scheme, auth_part, host, port)
        }
        Err(_) => "***invalid-redis-url***".to_string(),
    }
}

/// Obfuscate an API key, showing only prefix and suffix.
///
/// # Example
///
/// ```
/// use allframe_core::security::obfuscate_api_key;
///
/// let key = "sk_live_abcdefghijklmnop";
/// assert_eq!(obfuscate_api_key(key), "sk_l***mnop");
///
/// let short = "abc";
/// assert_eq!(obfuscate_api_key(short), "***");
/// ```
pub fn obfuscate_api_key(key: &str) -> String {
    let len = key.len();

    if len <= 8 {
        // Too short to safely reveal any part
        "***".to_string()
    } else {
        // Show first 4 and last 4 characters
        let prefix = &key[..4];
        let suffix = &key[len - 4..];
        format!("{}***{}", prefix, suffix)
    }
}

/// Obfuscate a header value based on the header name.
///
/// Sensitive headers (Authorization, Cookie, etc.) are fully obfuscated.
/// Other headers are returned as-is.
///
/// # Example
///
/// ```
/// use allframe_core::security::obfuscate_header;
///
/// // Sensitive headers are obfuscated
/// assert_eq!(
///     obfuscate_header("Authorization", "Bearer sk_live_secret"),
///     "Bearer ***"
/// );
/// assert_eq!(
///     obfuscate_header("Cookie", "session=abc123"),
///     "***"
/// );
///
/// // Non-sensitive headers are passed through
/// assert_eq!(
///     obfuscate_header("Content-Type", "application/json"),
///     "application/json"
/// );
/// ```
pub fn obfuscate_header(name: &str, value: &str) -> String {
    let name_lower = name.to_lowercase();

    match name_lower.as_str() {
        "authorization" => {
            // Preserve the auth scheme but hide the token
            if let Some(space_idx) = value.find(' ') {
                let scheme = &value[..space_idx];
                format!("{} ***", scheme)
            } else {
                "***".to_string()
            }
        }
        "cookie" | "set-cookie" => "***".to_string(),
        "x-api-key" | "api-key" | "apikey" => obfuscate_api_key(value),
        "x-auth-token" | "x-access-token" | "x-refresh-token" => "***".to_string(),
        "proxy-authorization" => "***".to_string(),
        _ => value.to_string(),
    }
}

/// Trait for types that can be obfuscated for safe logging.
///
/// Implement this trait for custom types that contain sensitive data.
///
/// # Example
///
/// ```
/// use allframe_core::security::Obfuscate;
///
/// struct DatabaseConfig {
///     host: String,
///     password: String,
/// }
///
/// impl Obfuscate for DatabaseConfig {
///     fn obfuscate(&self) -> String {
///         format!("DatabaseConfig {{ host: {}, password: *** }}", self.host)
///     }
/// }
///
/// let config = DatabaseConfig {
///     host: "localhost".to_string(),
///     password: "secret".to_string(),
/// };
///
/// assert_eq!(config.obfuscate(), "DatabaseConfig { host: localhost, password: *** }");
/// ```
pub trait Obfuscate {
    /// Return an obfuscated representation suitable for logging.
    fn obfuscate(&self) -> String;
}

// Implement Obfuscate for common types

impl Obfuscate for String {
    fn obfuscate(&self) -> String {
        if self.len() <= 8 {
            "***".to_string()
        } else {
            obfuscate_api_key(self)
        }
    }
}

impl Obfuscate for &str {
    fn obfuscate(&self) -> String {
        if self.len() <= 8 {
            "***".to_string()
        } else {
            obfuscate_api_key(self)
        }
    }
}

impl<T: Obfuscate> Obfuscate for Option<T> {
    fn obfuscate(&self) -> String {
        match self {
            Some(v) => format!("Some({})", v.obfuscate()),
            None => "None".to_string(),
        }
    }
}

/// Helper struct for wrapping sensitive values.
///
/// When formatted with `Debug` or `Display`, shows obfuscated value.
///
/// # Example
///
/// ```
/// use allframe_core::security::Sensitive;
///
/// let password = Sensitive::new("super_secret_password");
/// assert_eq!(format!("{}", password), "***");
/// assert_eq!(format!("{:?}", password), "Sensitive(***)");
/// ```
#[derive(Clone)]
#[allow(dead_code)]
pub struct Sensitive<T>(T);

#[allow(dead_code)]
impl<T> Sensitive<T> {
    /// Create a new sensitive wrapper.
    pub fn new(value: T) -> Self {
        Self(value)
    }

    /// Get the inner value (use with caution).
    pub fn into_inner(self) -> T {
        self.0
    }

    /// Get a reference to the inner value (use with caution).
    pub fn as_inner(&self) -> &T {
        &self.0
    }
}

impl<T> std::fmt::Display for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "***")
    }
}

impl<T> std::fmt::Debug for Sensitive<T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Sensitive(***)")
    }
}

impl<T: Default> Default for Sensitive<T> {
    fn default() -> Self {
        Self(T::default())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_url_with_credentials() {
        let url = "https://user:password@api.example.com/v1/users?key=secret";
        assert_eq!(obfuscate_url(url), "https://api.example.com/***");
    }

    #[test]
    fn test_obfuscate_url_simple() {
        let url = "https://example.com";
        assert_eq!(obfuscate_url(url), "https://example.com/***");
    }

    #[test]
    fn test_obfuscate_url_with_port() {
        let url = "http://localhost:8080/api";
        assert_eq!(obfuscate_url(url), "http://localhost:8080/***");
    }

    #[test]
    fn test_obfuscate_url_invalid() {
        let url = "not-a-url";
        assert_eq!(obfuscate_url(url), "***invalid-url***");
    }

    #[test]
    fn test_obfuscate_redis_url_with_auth() {
        let url = "redis://:password@localhost:6379/0";
        assert_eq!(obfuscate_redis_url(url), "redis://***@localhost:6379/***");
    }

    #[test]
    fn test_obfuscate_redis_url_with_user() {
        let url = "redis://user:password@localhost:6379/0";
        assert_eq!(obfuscate_redis_url(url), "redis://***@localhost:6379/***");
    }

    #[test]
    fn test_obfuscate_redis_url_no_auth() {
        let url = "redis://localhost:6379";
        assert_eq!(obfuscate_redis_url(url), "redis://localhost:6379/***");
    }

    #[test]
    fn test_obfuscate_api_key_long() {
        let key = "sk_live_abcdefghijklmnop";
        assert_eq!(obfuscate_api_key(key), "sk_l***mnop");
    }

    #[test]
    fn test_obfuscate_api_key_short() {
        let key = "short";
        assert_eq!(obfuscate_api_key(key), "***");
    }

    #[test]
    fn test_obfuscate_api_key_exactly_8() {
        let key = "12345678";
        assert_eq!(obfuscate_api_key(key), "***");
    }

    #[test]
    fn test_obfuscate_api_key_9_chars() {
        let key = "123456789";
        assert_eq!(obfuscate_api_key(key), "1234***6789");
    }

    #[test]
    fn test_obfuscate_header_authorization_bearer() {
        assert_eq!(
            obfuscate_header("Authorization", "Bearer token123"),
            "Bearer ***"
        );
    }

    #[test]
    fn test_obfuscate_header_authorization_basic() {
        assert_eq!(
            obfuscate_header("Authorization", "Basic dXNlcjpwYXNz"),
            "Basic ***"
        );
    }

    #[test]
    fn test_obfuscate_header_cookie() {
        assert_eq!(obfuscate_header("Cookie", "session=abc123"), "***");
    }

    #[test]
    fn test_obfuscate_header_api_key() {
        assert_eq!(
            obfuscate_header("X-API-Key", "sk_live_abcdefghij"),
            "sk_l***ghij"
        );
    }

    #[test]
    fn test_obfuscate_header_content_type() {
        assert_eq!(
            obfuscate_header("Content-Type", "application/json"),
            "application/json"
        );
    }

    #[test]
    fn test_obfuscate_header_case_insensitive() {
        assert_eq!(
            obfuscate_header("AUTHORIZATION", "Bearer token"),
            "Bearer ***"
        );
        assert_eq!(
            obfuscate_header("authorization", "Bearer token"),
            "Bearer ***"
        );
    }

    #[test]
    fn test_obfuscate_trait_string() {
        let s = "a_long_secret_string".to_string();
        assert_eq!(s.obfuscate(), "a_lo***ring");
    }

    #[test]
    fn test_obfuscate_trait_short_string() {
        let s = "short".to_string();
        assert_eq!(s.obfuscate(), "***");
    }

    #[test]
    fn test_obfuscate_trait_option_some() {
        let opt: Option<String> = Some("long_secret_value".to_string());
        assert_eq!(opt.obfuscate(), "Some(long***alue)");
    }

    #[test]
    fn test_obfuscate_trait_option_none() {
        let opt: Option<String> = None;
        assert_eq!(opt.obfuscate(), "None");
    }

    #[test]
    fn test_sensitive_display() {
        let s = Sensitive::new("secret");
        assert_eq!(format!("{}", s), "***");
    }

    #[test]
    fn test_sensitive_debug() {
        let s = Sensitive::new("secret");
        assert_eq!(format!("{:?}", s), "Sensitive(***)");
    }

    #[test]
    fn test_sensitive_into_inner() {
        let s = Sensitive::new("secret");
        assert_eq!(s.into_inner(), "secret");
    }

    #[test]
    fn test_sensitive_as_inner() {
        let s = Sensitive::new("secret");
        assert_eq!(s.as_inner(), &"secret");
    }
}
