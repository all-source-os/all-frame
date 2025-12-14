//! JWT (JSON Web Token) validation.
//!
//! Provides JWT validation using the `jsonwebtoken` crate with support for
//! HS256 (HMAC) and RS256 (RSA) algorithms.
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::auth::{JwtValidator, JwtConfig, Authenticator};
//! use serde::Deserialize;
//!
//! #[derive(Debug, Clone, Deserialize)]
//! struct MyClaims {
//!     sub: String,
//!     email: Option<String>,
//!     role: Option<String>,
//! }
//!
//! // Create validator with HS256
//! let config = JwtConfig::hs256("my-secret-key")
//!     .with_issuer("my-app")
//!     .with_leeway(60);
//!
//! let validator = JwtValidator::<MyClaims>::new(config);
//!
//! // Validate a token
//! let claims = validator.authenticate("eyJ...").await?;
//! println!("User: {}", claims.sub);
//! ```

use std::marker::PhantomData;

use jsonwebtoken::{decode, DecodingKey, TokenData, Validation};
use serde::de::DeserializeOwned;

use super::{AuthError, Authenticator};

/// JWT algorithm to use for validation.
#[derive(Debug, Clone)]
pub enum JwtAlgorithm {
    /// HMAC with SHA-256 (symmetric key).
    HS256(String),
    /// HMAC with SHA-384 (symmetric key).
    HS384(String),
    /// HMAC with SHA-512 (symmetric key).
    HS512(String),
    /// RSA with SHA-256 (public key in PEM format).
    RS256(String),
    /// RSA with SHA-384 (public key in PEM format).
    RS384(String),
    /// RSA with SHA-512 (public key in PEM format).
    RS512(String),
    /// EdDSA (public key in PEM format).
    EdDSA(String),
}

impl JwtAlgorithm {
    fn jsonwebtoken_algorithm(&self) -> jsonwebtoken::Algorithm {
        match self {
            JwtAlgorithm::HS256(_) => jsonwebtoken::Algorithm::HS256,
            JwtAlgorithm::HS384(_) => jsonwebtoken::Algorithm::HS384,
            JwtAlgorithm::HS512(_) => jsonwebtoken::Algorithm::HS512,
            JwtAlgorithm::RS256(_) => jsonwebtoken::Algorithm::RS256,
            JwtAlgorithm::RS384(_) => jsonwebtoken::Algorithm::RS384,
            JwtAlgorithm::RS512(_) => jsonwebtoken::Algorithm::RS512,
            JwtAlgorithm::EdDSA(_) => jsonwebtoken::Algorithm::EdDSA,
        }
    }

    fn decoding_key(&self) -> Result<DecodingKey, AuthError> {
        match self {
            JwtAlgorithm::HS256(secret)
            | JwtAlgorithm::HS384(secret)
            | JwtAlgorithm::HS512(secret) => Ok(DecodingKey::from_secret(secret.as_bytes())),
            JwtAlgorithm::RS256(pem)
            | JwtAlgorithm::RS384(pem)
            | JwtAlgorithm::RS512(pem) => DecodingKey::from_rsa_pem(pem.as_bytes())
                .map_err(|e| AuthError::Internal(format!("Invalid RSA key: {}", e))),
            JwtAlgorithm::EdDSA(pem) => DecodingKey::from_ed_pem(pem.as_bytes())
                .map_err(|e| AuthError::Internal(format!("Invalid EdDSA key: {}", e))),
        }
    }
}

/// Configuration for JWT validation.
#[derive(Debug, Clone)]
pub struct JwtConfig {
    /// Algorithm and key for validation.
    pub algorithm: JwtAlgorithm,
    /// Expected issuer (iss claim).
    pub issuer: Option<String>,
    /// Expected audience (aud claim).
    pub audience: Option<String>,
    /// Leeway in seconds for expiration checks.
    pub leeway_seconds: u64,
    /// Whether to validate expiration.
    pub validate_exp: bool,
    /// Whether to validate not-before.
    pub validate_nbf: bool,
}

impl JwtConfig {
    /// Create a new config with HS256 algorithm.
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::auth::JwtConfig;
    ///
    /// let config = JwtConfig::hs256("my-secret-key");
    /// ```
    pub fn hs256(secret: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::HS256(secret.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create a new config with HS384 algorithm.
    pub fn hs384(secret: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::HS384(secret.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create a new config with HS512 algorithm.
    pub fn hs512(secret: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::HS512(secret.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create a new config with RS256 algorithm.
    ///
    /// # Arguments
    /// * `public_key_pem` - RSA public key in PEM format
    pub fn rs256(public_key_pem: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::RS256(public_key_pem.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create a new config with RS384 algorithm.
    pub fn rs384(public_key_pem: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::RS384(public_key_pem.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create a new config with RS512 algorithm.
    pub fn rs512(public_key_pem: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::RS512(public_key_pem.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create a new config with EdDSA algorithm.
    pub fn eddsa(public_key_pem: impl Into<String>) -> Self {
        Self {
            algorithm: JwtAlgorithm::EdDSA(public_key_pem.into()),
            issuer: None,
            audience: None,
            leeway_seconds: 0,
            validate_exp: true,
            validate_nbf: true,
        }
    }

    /// Create config from environment variables.
    ///
    /// Reads:
    /// - `JWT_SECRET` - HMAC secret (for HS256)
    /// - `JWT_PUBLIC_KEY` - RSA public key (for RS256, overrides JWT_SECRET)
    /// - `JWT_ISSUER` - Expected issuer
    /// - `JWT_AUDIENCE` - Expected audience
    /// - `JWT_LEEWAY` - Leeway in seconds (default: 60)
    pub fn from_env() -> Option<Self> {
        let config = if let Ok(public_key) = std::env::var("JWT_PUBLIC_KEY") {
            Self::rs256(public_key)
        } else if let Ok(secret) = std::env::var("JWT_SECRET") {
            Self::hs256(secret)
        } else {
            return None;
        };

        let config = if let Ok(issuer) = std::env::var("JWT_ISSUER") {
            config.with_issuer(issuer)
        } else {
            config
        };

        let config = if let Ok(audience) = std::env::var("JWT_AUDIENCE") {
            config.with_audience(audience)
        } else {
            config
        };

        let config = if let Ok(leeway) = std::env::var("JWT_LEEWAY") {
            if let Ok(seconds) = leeway.parse() {
                config.with_leeway(seconds)
            } else {
                config
            }
        } else {
            config.with_leeway(60) // Default 60 second leeway
        };

        Some(config)
    }

    /// Set the expected issuer.
    pub fn with_issuer(mut self, issuer: impl Into<String>) -> Self {
        self.issuer = Some(issuer.into());
        self
    }

    /// Set the expected audience.
    pub fn with_audience(mut self, audience: impl Into<String>) -> Self {
        self.audience = Some(audience.into());
        self
    }

    /// Set the leeway for expiration checks (in seconds).
    pub fn with_leeway(mut self, seconds: u64) -> Self {
        self.leeway_seconds = seconds;
        self
    }

    /// Disable expiration validation.
    pub fn without_exp_validation(mut self) -> Self {
        self.validate_exp = false;
        self
    }

    /// Disable not-before validation.
    pub fn without_nbf_validation(mut self) -> Self {
        self.validate_nbf = false;
        self
    }
}

/// JWT token validator.
///
/// Validates JWT tokens and extracts claims into a typed struct.
///
/// # Type Parameters
///
/// * `C` - The claims type. Must implement `DeserializeOwned`.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::auth::{JwtValidator, JwtConfig, Authenticator};
/// use serde::Deserialize;
///
/// #[derive(Debug, Clone, Deserialize)]
/// struct Claims {
///     sub: String,
///     exp: i64,
/// }
///
/// let validator = JwtValidator::<Claims>::new(JwtConfig::hs256("secret"));
/// let claims = validator.authenticate("eyJ...").await?;
/// ```
pub struct JwtValidator<C> {
    config: JwtConfig,
    decoding_key: DecodingKey,
    validation: Validation,
    _phantom: PhantomData<C>,
}

impl<C> JwtValidator<C> {
    /// Create a new JWT validator.
    ///
    /// # Panics
    ///
    /// Panics if the key in the config is invalid (e.g., malformed PEM).
    /// Use `try_new` for fallible construction.
    pub fn new(config: JwtConfig) -> Self {
        Self::try_new(config).expect("Invalid JWT configuration")
    }

    /// Try to create a new JWT validator.
    ///
    /// Returns an error if the key is invalid.
    pub fn try_new(config: JwtConfig) -> Result<Self, AuthError> {
        let decoding_key = config.algorithm.decoding_key()?;

        let mut validation = Validation::new(config.algorithm.jsonwebtoken_algorithm());
        validation.leeway = config.leeway_seconds;
        validation.validate_exp = config.validate_exp;
        validation.validate_nbf = config.validate_nbf;

        if let Some(ref iss) = config.issuer {
            validation.set_issuer(&[iss]);
        }

        if let Some(ref aud) = config.audience {
            validation.set_audience(&[aud]);
        } else {
            // By default jsonwebtoken requires audience validation
            // Disable if not configured
            validation.validate_aud = false;
        }

        Ok(Self {
            config,
            decoding_key,
            validation,
            _phantom: PhantomData,
        })
    }

    /// Get the configuration.
    pub fn config(&self) -> &JwtConfig {
        &self.config
    }
}

impl<C: Clone + Send + Sync + DeserializeOwned + 'static> JwtValidator<C> {
    /// Validate a token and return the claims.
    pub fn validate(&self, token: &str) -> Result<C, AuthError> {
        let token_data: TokenData<C> =
            decode(token, &self.decoding_key, &self.validation).map_err(|e| {
                use jsonwebtoken::errors::ErrorKind;
                match e.kind() {
                    ErrorKind::ExpiredSignature => AuthError::TokenExpired,
                    ErrorKind::InvalidSignature => AuthError::InvalidSignature,
                    ErrorKind::InvalidIssuer => AuthError::InvalidIssuer,
                    ErrorKind::InvalidAudience => AuthError::InvalidAudience,
                    ErrorKind::InvalidToken => AuthError::InvalidToken("malformed token".into()),
                    ErrorKind::InvalidAlgorithm => {
                        AuthError::InvalidToken("wrong algorithm".into())
                    }
                    _ => AuthError::InvalidToken(e.to_string()),
                }
            })?;

        Ok(token_data.claims)
    }
}

#[async_trait::async_trait]
impl<C: Clone + Send + Sync + DeserializeOwned + 'static> Authenticator for JwtValidator<C> {
    type Claims = C;

    async fn authenticate(&self, token: &str) -> Result<Self::Claims, AuthError> {
        // JWT validation is synchronous, but we implement async trait for consistency
        self.validate(token)
    }
}

/// Standard JWT claims structure.
///
/// Use this if you don't need custom claims, or as a base for your own type.
#[derive(Debug, Clone, serde::Deserialize, serde::Serialize)]
pub struct StandardClaims {
    /// Subject (usually user ID).
    pub sub: String,
    /// Expiration time (Unix timestamp).
    #[serde(default)]
    pub exp: Option<i64>,
    /// Issued at (Unix timestamp).
    #[serde(default)]
    pub iat: Option<i64>,
    /// Not before (Unix timestamp).
    #[serde(default)]
    pub nbf: Option<i64>,
    /// Issuer.
    #[serde(default)]
    pub iss: Option<String>,
    /// Audience.
    #[serde(default)]
    pub aud: Option<String>,
}

impl super::HasSubject for StandardClaims {
    fn subject(&self) -> &str {
        &self.sub
    }
}

impl super::HasExpiration for StandardClaims {
    fn expiration(&self) -> Option<i64> {
        self.exp
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // Test token created with secret "test-secret", sub "user123", exp far in future
    const TEST_SECRET: &str = "test-secret-that-is-long-enough-for-hs256";

    fn create_test_token(claims: &impl serde::Serialize) -> String {
        use jsonwebtoken::{encode, EncodingKey, Header};
        encode(
            &Header::default(),
            claims,
            &EncodingKey::from_secret(TEST_SECRET.as_bytes()),
        )
        .unwrap()
    }

    #[derive(Debug, Clone, serde::Serialize, serde::Deserialize, PartialEq)]
    struct TestClaims {
        sub: String,
        exp: i64,
    }

    #[test]
    fn test_jwt_config_hs256() {
        let config = JwtConfig::hs256("secret");
        assert!(matches!(config.algorithm, JwtAlgorithm::HS256(_)));
        assert!(config.issuer.is_none());
        assert!(config.audience.is_none());
    }

    #[test]
    fn test_jwt_config_builder() {
        let config = JwtConfig::hs256("secret")
            .with_issuer("my-app")
            .with_audience("my-audience")
            .with_leeway(120);

        assert_eq!(config.issuer, Some("my-app".to_string()));
        assert_eq!(config.audience, Some("my-audience".to_string()));
        assert_eq!(config.leeway_seconds, 120);
    }

    #[test]
    fn test_jwt_validator_valid_token() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        };
        let token = create_test_token(&claims);

        let validator = JwtValidator::<TestClaims>::new(JwtConfig::hs256(TEST_SECRET));

        let result = validator.validate(&token);
        assert!(result.is_ok());

        let decoded = result.unwrap();
        assert_eq!(decoded.sub, "user123");
    }

    #[test]
    fn test_jwt_validator_expired_token() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: 0, // Expired long ago
        };
        let token = create_test_token(&claims);

        let validator = JwtValidator::<TestClaims>::new(JwtConfig::hs256(TEST_SECRET));

        let result = validator.validate(&token);
        assert!(matches!(result, Err(AuthError::TokenExpired)));
    }

    #[test]
    fn test_jwt_validator_wrong_secret() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        };
        let token = create_test_token(&claims);

        let validator = JwtValidator::<TestClaims>::new(JwtConfig::hs256("wrong-secret"));

        let result = validator.validate(&token);
        assert!(matches!(result, Err(AuthError::InvalidSignature)));
    }

    #[test]
    fn test_jwt_validator_invalid_token() {
        let validator = JwtValidator::<TestClaims>::new(JwtConfig::hs256(TEST_SECRET));

        let result = validator.validate("not-a-jwt");
        assert!(matches!(result, Err(AuthError::InvalidToken(_))));
    }

    #[tokio::test]
    async fn test_jwt_validator_authenticator_trait() {
        let claims = TestClaims {
            sub: "user123".to_string(),
            exp: (chrono::Utc::now() + chrono::Duration::hours(1)).timestamp(),
        };
        let token = create_test_token(&claims);

        let validator = JwtValidator::<TestClaims>::new(JwtConfig::hs256(TEST_SECRET));

        // Use via Authenticator trait
        let result = validator.authenticate(&token).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().sub, "user123");
    }

    #[test]
    fn test_standard_claims() {
        let claims = StandardClaims {
            sub: "user123".to_string(),
            exp: Some(1234567890),
            iat: Some(1234567800),
            nbf: None,
            iss: Some("my-app".to_string()),
            aud: None,
        };

        assert_eq!(claims.sub, "user123");
        assert_eq!(claims.exp, Some(1234567890));

        // Test HasSubject trait
        use super::super::HasSubject;
        assert_eq!(claims.subject(), "user123");

        // Test HasExpiration trait
        use super::super::HasExpiration;
        assert_eq!(claims.expiration(), Some(1234567890));
    }
}
