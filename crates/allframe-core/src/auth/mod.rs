//! Authentication primitives for AllFrame.
//!
//! This module provides a layered authentication system:
//!
//! - **`auth`** (this module): Core traits with zero dependencies
//! - **`auth-jwt`**: JWT validation using `jsonwebtoken`
//! - **`auth-axum`**: Axum extractors and middleware
//! - **`auth-tonic`**: gRPC interceptors
//!
//! # Core Concepts
//!
//! The authentication system is built around a few key traits:
//!
//! - [`Authenticator`]: Validates tokens and returns claims
//! - [`Claims`]: Marker trait for claim types
//! - [`AuthContext`]: Holds authenticated user information
//!
//! # Example: Using Core Traits
//!
//! ```rust
//! use allframe_core::auth::{Authenticator, AuthError, AuthContext};
//!
//! // Define your claims type
//! #[derive(Clone, Debug)]
//! struct MyClaims {
//!     sub: String,
//!     email: Option<String>,
//! }
//!
//! // Implement your authenticator
//! struct MyAuthenticator;
//!
//! #[async_trait::async_trait]
//! impl Authenticator for MyAuthenticator {
//!     type Claims = MyClaims;
//!
//!     async fn authenticate(&self, token: &str) -> Result<Self::Claims, AuthError> {
//!         // Your validation logic here
//!         Ok(MyClaims {
//!             sub: "user123".to_string(),
//!             email: Some("user@example.com".to_string()),
//!         })
//!     }
//! }
//! ```
//!
//! # Feature Flags
//!
//! | Feature | Description |
//! |---------|-------------|
//! | `auth` | Core traits (this module) |
//! | `auth-jwt` | JWT validation with HS256/RS256 support |
//! | `auth-axum` | Axum extractors and middleware |
//! | `auth-tonic` | gRPC interceptors |

use std::fmt;

#[cfg(feature = "auth-jwt")]
pub mod jwt;

#[cfg(feature = "auth-axum")]
pub mod axum;

#[cfg(feature = "auth-tonic")]
pub mod tonic;

// Re-exports
#[cfg(feature = "auth-jwt")]
pub use jwt::{JwtAlgorithm, JwtConfig, JwtValidator};

#[cfg(feature = "auth-axum")]
pub use self::axum::{AuthLayer, AuthenticatedUser};
#[cfg(feature = "auth-tonic")]
pub use self::tonic::AuthInterceptor;

/// Error type for authentication failures.
#[derive(Debug, Clone)]
pub enum AuthError {
    /// No token was provided.
    MissingToken,
    /// Token format is invalid.
    InvalidToken(String),
    /// Token has expired.
    TokenExpired,
    /// Token signature is invalid.
    InvalidSignature,
    /// Token issuer doesn't match.
    InvalidIssuer,
    /// Token audience doesn't match.
    InvalidAudience,
    /// Custom validation error.
    ValidationFailed(String),
    /// Internal error during authentication.
    Internal(String),
}

impl fmt::Display for AuthError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            AuthError::MissingToken => write!(f, "missing authentication token"),
            AuthError::InvalidToken(msg) => write!(f, "invalid token: {}", msg),
            AuthError::TokenExpired => write!(f, "token has expired"),
            AuthError::InvalidSignature => write!(f, "invalid token signature"),
            AuthError::InvalidIssuer => write!(f, "invalid token issuer"),
            AuthError::InvalidAudience => write!(f, "invalid token audience"),
            AuthError::ValidationFailed(msg) => write!(f, "validation failed: {}", msg),
            AuthError::Internal(msg) => write!(f, "internal auth error: {}", msg),
        }
    }
}

impl std::error::Error for AuthError {}

impl AuthError {
    /// Check if this is a "missing token" error (vs invalid token).
    pub fn is_missing(&self) -> bool {
        matches!(self, AuthError::MissingToken)
    }

    /// Check if this is an expiration error.
    pub fn is_expired(&self) -> bool {
        matches!(self, AuthError::TokenExpired)
    }

    /// Get an appropriate HTTP status code for this error.
    pub fn status_code(&self) -> u16 {
        match self {
            AuthError::MissingToken => 401,
            AuthError::InvalidToken(_) => 401,
            AuthError::TokenExpired => 401,
            AuthError::InvalidSignature => 401,
            AuthError::InvalidIssuer => 401,
            AuthError::InvalidAudience => 401,
            AuthError::ValidationFailed(_) => 403,
            AuthError::Internal(_) => 500,
        }
    }
}

/// Trait for types that can validate authentication tokens.
///
/// Implement this trait to create custom authenticators for different
/// token types (JWT, API keys, session tokens, etc.).
///
/// # Example
///
/// ```rust
/// use allframe_core::auth::{Authenticator, AuthError};
///
/// struct ApiKeyAuthenticator {
///     valid_keys: Vec<String>,
/// }
///
/// #[async_trait::async_trait]
/// impl Authenticator for ApiKeyAuthenticator {
///     type Claims = String; // Just the key itself
///
///     async fn authenticate(&self, token: &str) -> Result<Self::Claims, AuthError> {
///         if self.valid_keys.contains(&token.to_string()) {
///             Ok(token.to_string())
///         } else {
///             Err(AuthError::InvalidToken("unknown API key".into()))
///         }
///     }
/// }
/// ```
#[async_trait::async_trait]
pub trait Authenticator: Send + Sync {
    /// The claims type returned on successful authentication.
    type Claims: Clone + Send + Sync + 'static;

    /// Validate a token and extract claims.
    ///
    /// # Arguments
    /// * `token` - The raw token string (without "Bearer " prefix)
    ///
    /// # Returns
    /// * `Ok(Claims)` - Authentication successful
    /// * `Err(AuthError)` - Authentication failed
    async fn authenticate(&self, token: &str) -> Result<Self::Claims, AuthError>;
}

/// Context holding authenticated user information.
///
/// This is the result of successful authentication and contains
/// the validated claims.
#[derive(Clone, Debug)]
pub struct AuthContext<C> {
    /// The validated claims.
    pub claims: C,
    /// The original token (for forwarding to downstream services).
    pub token: String,
}

impl<C: Clone> AuthContext<C> {
    /// Create a new auth context.
    pub fn new(claims: C, token: impl Into<String>) -> Self {
        Self {
            claims,
            token: token.into(),
        }
    }

    /// Get the claims.
    pub fn claims(&self) -> &C {
        &self.claims
    }

    /// Get the original token.
    pub fn token(&self) -> &str {
        &self.token
    }

    /// Extract a value from claims using a closure.
    pub fn get<T>(&self, f: impl FnOnce(&C) -> T) -> T {
        f(&self.claims)
    }
}

/// Extract bearer token from an authorization header value.
///
/// # Example
///
/// ```rust
/// use allframe_core::auth::extract_bearer_token;
///
/// assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
/// assert_eq!(extract_bearer_token("bearer ABC"), Some("ABC"));
/// assert_eq!(extract_bearer_token("Basic xyz"), None);
/// assert_eq!(extract_bearer_token("abc123"), None);
/// ```
pub fn extract_bearer_token(header_value: &str) -> Option<&str> {
    let header = header_value.trim();
    if header.len() > 7 && header[..7].eq_ignore_ascii_case("bearer ") {
        Some(header[7..].trim())
    } else {
        None
    }
}

/// Trait for claims that have a subject (user ID).
pub trait HasSubject {
    /// Get the subject (user ID) from the claims.
    fn subject(&self) -> &str;
}

/// Trait for claims that have an expiration time.
pub trait HasExpiration {
    /// Get the expiration timestamp (Unix seconds).
    fn expiration(&self) -> Option<i64>;

    /// Check if the claims have expired.
    fn is_expired(&self) -> bool {
        if let Some(exp) = self.expiration() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs() as i64;
            exp < now
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_bearer_token() {
        assert_eq!(extract_bearer_token("Bearer abc123"), Some("abc123"));
        assert_eq!(extract_bearer_token("bearer ABC"), Some("ABC"));
        assert_eq!(extract_bearer_token("BEARER token"), Some("token"));
        assert_eq!(extract_bearer_token("Bearer  spaced"), Some("spaced"));
        assert_eq!(extract_bearer_token("Basic xyz"), None);
        assert_eq!(extract_bearer_token("abc123"), None);
        assert_eq!(extract_bearer_token(""), None);
        assert_eq!(extract_bearer_token("Bearer"), None);
        // "Bearer " with no token after is treated as invalid
        assert_eq!(extract_bearer_token("Bearer "), None);
    }

    #[test]
    fn test_auth_error_display() {
        assert_eq!(
            AuthError::MissingToken.to_string(),
            "missing authentication token"
        );
        assert_eq!(AuthError::TokenExpired.to_string(), "token has expired");
        assert_eq!(
            AuthError::InvalidToken("bad".into()).to_string(),
            "invalid token: bad"
        );
    }

    #[test]
    fn test_auth_error_status_codes() {
        assert_eq!(AuthError::MissingToken.status_code(), 401);
        assert_eq!(AuthError::TokenExpired.status_code(), 401);
        assert_eq!(AuthError::ValidationFailed("".into()).status_code(), 403);
        assert_eq!(AuthError::Internal("".into()).status_code(), 500);
    }

    #[test]
    fn test_auth_context() {
        #[derive(Clone, Debug)]
        struct TestClaims {
            sub: String,
            role: String,
        }

        let ctx = AuthContext::new(
            TestClaims {
                sub: "user123".into(),
                role: "admin".into(),
            },
            "token123",
        );

        assert_eq!(ctx.claims().sub, "user123");
        assert_eq!(ctx.token(), "token123");
        assert_eq!(ctx.get(|c| c.role.clone()), "admin");
    }

    #[test]
    fn test_auth_error_predicates() {
        assert!(AuthError::MissingToken.is_missing());
        assert!(!AuthError::TokenExpired.is_missing());
        assert!(AuthError::TokenExpired.is_expired());
        assert!(!AuthError::MissingToken.is_expired());
    }

    #[derive(Clone)]
    struct MockClaims {
        exp: Option<i64>,
    }

    impl HasExpiration for MockClaims {
        fn expiration(&self) -> Option<i64> {
            self.exp
        }
    }

    #[test]
    fn test_has_expiration() {
        let past = MockClaims { exp: Some(0) };
        assert!(past.is_expired());

        let future = MockClaims {
            exp: Some(i64::MAX),
        };
        assert!(!future.is_expired());

        let no_exp = MockClaims { exp: None };
        assert!(!no_exp.is_expired());
    }
}
