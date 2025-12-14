//! Tonic (gRPC) integration for authentication.
//!
//! Provides interceptors for using AllFrame auth with gRPC services.
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::auth::{AuthInterceptor, JwtValidator, JwtConfig};
//! use tonic::transport::Server;
//!
//! #[derive(Clone, serde::Deserialize)]
//! struct Claims {
//!     sub: String,
//! }
//!
//! let validator = JwtValidator::<Claims>::new(JwtConfig::hs256("secret"));
//! let interceptor = AuthInterceptor::new(validator);
//!
//! // Use with a service
//! let service = MyServiceServer::with_interceptor(impl, interceptor);
//! ```

use std::{marker::PhantomData, sync::Arc};

use tonic::{metadata::MetadataMap, Request, Status};

use super::{extract_bearer_token, AuthContext, AuthError, Authenticator};

/// gRPC interceptor for authentication.
///
/// Validates the bearer token from the `authorization` metadata and
/// adds the auth context to request extensions.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::auth::{AuthInterceptor, JwtValidator, JwtConfig};
///
/// let validator = JwtValidator::<Claims>::new(JwtConfig::hs256("secret"));
/// let interceptor = AuthInterceptor::new(validator);
///
/// // Required auth - rejects unauthenticated requests
/// let service = MyServiceServer::with_interceptor(impl, interceptor.required());
///
/// // Optional auth - allows unauthenticated requests
/// let service = MyServiceServer::with_interceptor(impl, interceptor.optional());
/// ```
pub struct AuthInterceptor<A, C> {
    authenticator: Arc<A>,
    required: bool,
    _phantom: PhantomData<C>,
}

impl<A, C> Clone for AuthInterceptor<A, C> {
    fn clone(&self) -> Self {
        Self {
            authenticator: self.authenticator.clone(),
            required: self.required,
            _phantom: PhantomData,
        }
    }
}

impl<A, C> AuthInterceptor<A, C>
where
    A: Authenticator<Claims = C>,
    C: Clone + Send + Sync + 'static,
{
    /// Create a new auth interceptor (required by default).
    pub fn new(authenticator: A) -> Self {
        Self {
            authenticator: Arc::new(authenticator),
            required: true,
            _phantom: PhantomData,
        }
    }

    /// Make authentication required (rejects unauthenticated requests).
    pub fn required(mut self) -> Self {
        self.required = true;
        self
    }

    /// Make authentication optional (allows unauthenticated requests).
    pub fn optional(mut self) -> Self {
        self.required = false;
        self
    }

    /// Extract and validate the token from request metadata.
    fn authenticate_request<T>(&self, request: &Request<T>) -> Result<Option<C>, Status> {
        let metadata = request.metadata();

        // Try to get the authorization header
        let token = match extract_token_from_metadata(metadata) {
            Some(token) => token,
            None => {
                if self.required {
                    return Err(auth_error_to_status(AuthError::MissingToken));
                }
                return Ok(None);
            }
        };

        // Validate the token synchronously
        // Note: We use block_in_place since tonic interceptors are sync
        let authenticator = self.authenticator.clone();
        let token_owned = token.to_string();

        // For sync validation (like JWT), we can use blocking
        let result = tokio::task::block_in_place(|| {
            tokio::runtime::Handle::current()
                .block_on(async { authenticator.authenticate(&token_owned).await })
        });

        match result {
            Ok(claims) => Ok(Some(claims)),
            Err(e) => {
                if self.required {
                    Err(auth_error_to_status(e))
                } else {
                    Ok(None)
                }
            }
        }
    }
}

impl<A, C> tonic::service::Interceptor for AuthInterceptor<A, C>
where
    A: Authenticator<Claims = C> + 'static,
    C: Clone + Send + Sync + 'static,
{
    fn call(&mut self, mut request: Request<()>) -> Result<Request<()>, Status> {
        match self.authenticate_request(&request)? {
            Some(claims) => {
                // Extract token for the context
                let token = extract_token_from_metadata(request.metadata())
                    .unwrap_or("")
                    .to_string();

                let ctx = AuthContext::new(claims, token);
                request.extensions_mut().insert(ctx);
            }
            None => {
                // Optional auth, no token present
            }
        }

        Ok(request)
    }
}

/// Extract bearer token from gRPC metadata.
fn extract_token_from_metadata(metadata: &MetadataMap) -> Option<&str> {
    metadata
        .get("authorization")
        .and_then(|v| v.to_str().ok())
        .and_then(extract_bearer_token)
}

/// Convert an auth error to a tonic Status.
fn auth_error_to_status(error: AuthError) -> Status {
    match error {
        AuthError::MissingToken => Status::unauthenticated("missing authentication token"),
        AuthError::InvalidToken(msg) => Status::unauthenticated(format!("invalid token: {}", msg)),
        AuthError::TokenExpired => Status::unauthenticated("token has expired"),
        AuthError::InvalidSignature => Status::unauthenticated("invalid token signature"),
        AuthError::InvalidIssuer => Status::unauthenticated("invalid token issuer"),
        AuthError::InvalidAudience => Status::unauthenticated("invalid token audience"),
        AuthError::ValidationFailed(msg) => {
            Status::permission_denied(format!("validation failed: {}", msg))
        }
        AuthError::Internal(msg) => Status::internal(format!("auth error: {}", msg)),
    }
}

/// Extension trait for extracting auth context from gRPC requests.
pub trait GrpcAuthExt<T> {
    /// Get the auth context if present.
    fn auth_context<C: Clone + Send + Sync + 'static>(&self) -> Option<&AuthContext<C>>;

    /// Get the claims if authenticated.
    fn claims<C: Clone + Send + Sync + 'static>(&self) -> Option<&C> {
        self.auth_context::<C>().map(|ctx| &ctx.claims)
    }

    /// Get the claims, returning an error if not authenticated.
    fn require_auth<C: Clone + Send + Sync + 'static>(&self) -> Result<&C, Status> {
        self.claims::<C>()
            .ok_or_else(|| Status::unauthenticated("authentication required"))
    }
}

impl<T> GrpcAuthExt<T> for Request<T> {
    fn auth_context<C: Clone + Send + Sync + 'static>(&self) -> Option<&AuthContext<C>> {
        self.extensions().get::<AuthContext<C>>()
    }
}

/// Simple function-based interceptor for quick auth setup.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::auth::tonic::auth_interceptor;
///
/// let validator = JwtValidator::<Claims>::new(config);
///
/// // Create a simple interceptor function
/// let intercept = auth_interceptor(validator, true);
///
/// let service = MyServiceServer::with_interceptor(impl, intercept);
/// ```
pub fn auth_interceptor<A, C>(
    authenticator: A,
    required: bool,
) -> impl FnMut(Request<()>) -> Result<Request<()>, Status> + Clone
where
    A: Authenticator<Claims = C> + 'static,
    C: Clone + Send + Sync + 'static,
{
    let mut interceptor = AuthInterceptor::new(authenticator);
    if !required {
        interceptor = interceptor.optional();
    }

    move |req| {
        let mut i = interceptor.clone();
        tonic::service::Interceptor::call(&mut i, req)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_token_from_metadata() {
        let mut metadata = MetadataMap::new();

        // No token
        assert!(extract_token_from_metadata(&metadata).is_none());

        // Add bearer token
        metadata.insert("authorization", "Bearer test123".parse().unwrap());
        assert_eq!(extract_token_from_metadata(&metadata), Some("test123"));
    }

    #[test]
    fn test_auth_error_to_status() {
        let status = auth_error_to_status(AuthError::MissingToken);
        assert_eq!(status.code(), tonic::Code::Unauthenticated);

        let status = auth_error_to_status(AuthError::TokenExpired);
        assert_eq!(status.code(), tonic::Code::Unauthenticated);

        let status = auth_error_to_status(AuthError::ValidationFailed("test".into()));
        assert_eq!(status.code(), tonic::Code::PermissionDenied);

        let status = auth_error_to_status(AuthError::Internal("error".into()));
        assert_eq!(status.code(), tonic::Code::Internal);
    }

    #[test]
    fn test_grpc_auth_ext() {
        #[derive(Clone)]
        struct Claims {
            sub: String,
        }

        let mut request = Request::new(());

        // No auth initially
        assert!(request.auth_context::<Claims>().is_none());
        assert!(request.claims::<Claims>().is_none());
        assert!(request.require_auth::<Claims>().is_err());

        // Add auth context
        request.extensions_mut().insert(AuthContext::new(
            Claims {
                sub: "user123".to_string(),
            },
            "token",
        ));

        // Now available
        assert!(request.auth_context::<Claims>().is_some());
        assert_eq!(request.claims::<Claims>().unwrap().sub, "user123");
        assert!(request.require_auth::<Claims>().is_ok());
    }
}
