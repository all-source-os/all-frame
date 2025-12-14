//! Axum integration for authentication.
//!
//! Provides extractors and middleware for using AllFrame auth with Axum.
//!
//! # Example
//!
//! ```rust,ignore
//! use allframe_core::auth::{AuthenticatedUser, AuthLayer, JwtValidator, JwtConfig};
//! use axum::{Router, routing::get, Extension};
//!
//! #[derive(Clone, serde::Deserialize)]
//! struct Claims {
//!     sub: String,
//!     role: String,
//! }
//!
//! async fn protected_handler(
//!     AuthenticatedUser(claims): AuthenticatedUser<Claims>,
//! ) -> String {
//!     format!("Hello, {}!", claims.sub)
//! }
//!
//! // Setup
//! let validator = JwtValidator::<Claims>::new(JwtConfig::hs256("secret"));
//!
//! let app = Router::new()
//!     .route("/protected", get(protected_handler))
//!     .layer(AuthLayer::new(validator));
//! ```

use std::{
    future::Future,
    marker::PhantomData,
    pin::Pin,
    sync::Arc,
    task::{Context, Poll},
};

use super::{extract_bearer_token, AuthContext, AuthError, Authenticator};

/// Extractor for authenticated requests.
///
/// Extracts and validates the bearer token from the Authorization header,
/// returning the claims on success.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::auth::AuthenticatedUser;
///
/// async fn handler(AuthenticatedUser(claims): AuthenticatedUser<MyClaims>) -> String {
///     format!("User ID: {}", claims.sub)
/// }
/// ```
///
/// # Extracting Optional Auth
///
/// Wrap in `Option` to make auth optional:
///
/// ```rust,ignore
/// async fn handler(auth: Option<AuthenticatedUser<MyClaims>>) -> String {
///     match auth {
///         Some(AuthenticatedUser(claims)) => format!("Hello, {}", claims.sub),
///         None => "Hello, anonymous!".to_string(),
///     }
/// }
/// ```
#[derive(Debug, Clone)]
pub struct AuthenticatedUser<C>(pub C);

impl<C> AuthenticatedUser<C> {
    /// Get the claims.
    pub fn claims(&self) -> &C {
        &self.0
    }

    /// Unwrap into the inner claims.
    pub fn into_inner(self) -> C {
        self.0
    }
}

impl<C> std::ops::Deref for AuthenticatedUser<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Layer for adding authentication to a router.
///
/// This layer validates the Authorization header on each request and
/// stores the auth context in request extensions.
///
/// # Example
///
/// ```rust,ignore
/// use allframe_core::auth::{AuthLayer, JwtValidator, JwtConfig};
///
/// let validator = JwtValidator::<MyClaims>::new(JwtConfig::hs256("secret"));
///
/// let app = Router::new()
///     .route("/protected", get(handler))
///     .layer(AuthLayer::new(validator));
/// ```
#[derive(Clone)]
pub struct AuthLayer<A> {
    authenticator: Arc<A>,
}

impl<A> AuthLayer<A> {
    /// Create a new auth layer with the given authenticator.
    pub fn new(authenticator: A) -> Self {
        Self {
            authenticator: Arc::new(authenticator),
        }
    }
}

impl<S, A> tower::Layer<S> for AuthLayer<A>
where
    A: Clone,
{
    type Service = AuthService<S, A>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            authenticator: self.authenticator.clone(),
        }
    }
}

/// Service that performs authentication.
#[derive(Clone)]
pub struct AuthService<S, A> {
    inner: S,
    authenticator: Arc<A>,
}

impl<S, A, ReqBody> tower::Service<hyper::Request<ReqBody>> for AuthService<S, A>
where
    S: tower::Service<hyper::Request<ReqBody>> + Clone + Send + 'static,
    S::Future: Send,
    A: Authenticator + 'static,
    ReqBody: Send + 'static,
{
    type Response = S::Response;
    type Error = S::Error;
    type Future = Pin<Box<dyn Future<Output = Result<Self::Response, Self::Error>> + Send>>;

    fn poll_ready(&mut self, cx: &mut Context<'_>) -> Poll<Result<(), Self::Error>> {
        self.inner.poll_ready(cx)
    }

    fn call(&mut self, mut req: hyper::Request<ReqBody>) -> Self::Future {
        let authenticator = self.authenticator.clone();
        let mut inner = self.inner.clone();

        Box::pin(async move {
            // Extract token from Authorization header
            if let Some(auth_header) = req.headers().get(hyper::header::AUTHORIZATION) {
                if let Ok(header_str) = auth_header.to_str() {
                    if let Some(token) = extract_bearer_token(header_str) {
                        // Validate token
                        if let Ok(claims) = authenticator.authenticate(token).await {
                            let ctx = AuthContext::new(claims, token);
                            req.extensions_mut().insert(ctx);
                        }
                    }
                }
            }

            inner.call(req).await
        })
    }
}

/// Optional auth layer that doesn't reject unauthenticated requests.
///
/// Use this when you want to allow both authenticated and unauthenticated
/// access, but still make auth info available when present.
#[derive(Clone)]
pub struct OptionalAuthLayer<A> {
    authenticator: Arc<A>,
}

impl<A> OptionalAuthLayer<A> {
    /// Create a new optional auth layer.
    pub fn new(authenticator: A) -> Self {
        Self {
            authenticator: Arc::new(authenticator),
        }
    }
}

impl<S, A> tower::Layer<S> for OptionalAuthLayer<A>
where
    A: Clone,
{
    type Service = AuthService<S, A>;

    fn layer(&self, inner: S) -> Self::Service {
        AuthService {
            inner,
            authenticator: self.authenticator.clone(),
        }
    }
}

/// Extension trait for extracting auth context from request extensions.
pub trait AuthExt {
    /// Get the auth context if present.
    fn auth_context<C: Clone + Send + Sync + 'static>(&self) -> Option<&AuthContext<C>>;

    /// Get the claims if authenticated.
    fn claims<C: Clone + Send + Sync + 'static>(&self) -> Option<&C> {
        self.auth_context::<C>().map(|ctx| &ctx.claims)
    }
}

impl<B> AuthExt for hyper::Request<B> {
    fn auth_context<C: Clone + Send + Sync + 'static>(&self) -> Option<&AuthContext<C>> {
        self.extensions().get::<AuthContext<C>>()
    }
}

/// Rejection type for authentication failures.
#[derive(Debug)]
pub struct AuthRejection {
    /// The authentication error.
    pub error: AuthError,
}

impl AuthRejection {
    /// Create a new rejection from an auth error.
    pub fn new(error: AuthError) -> Self {
        Self { error }
    }
}

impl std::fmt::Display for AuthRejection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.error)
    }
}

impl std::error::Error for AuthRejection {}

// Note: Full axum::response::IntoResponse implementation would require
// axum as a dependency. Users can implement this in their own code:
//
// impl axum::response::IntoResponse for AuthRejection {
//     fn into_response(self) -> axum::response::Response {
//         let status = match self.error.status_code() {
//             401 => axum::http::StatusCode::UNAUTHORIZED,
//             403 => axum::http::StatusCode::FORBIDDEN,
//             _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
//         };
//         (status, self.error.to_string()).into_response()
//     }
// }

/// Marker type for required authentication.
///
/// Used to create extractors that reject unauthenticated requests.
#[derive(Debug, Clone, Copy)]
pub struct Required;

/// Marker type for optional authentication.
///
/// Used to create extractors that allow unauthenticated requests.
#[derive(Debug, Clone, Copy)]
pub struct Optional;

/// Generic auth extractor with configurable requirement.
#[derive(Debug, Clone)]
pub struct Auth<C, R = Required> {
    /// The auth context (None if optional and unauthenticated).
    pub context: Option<AuthContext<C>>,
    _requirement: PhantomData<R>,
}

impl<C: Clone> Auth<C, Required> {
    /// Get the claims (always present for Required auth).
    pub fn claims(&self) -> &C {
        &self.context.as_ref().unwrap().claims
    }

    /// Get the original token.
    pub fn token(&self) -> &str {
        self.context.as_ref().unwrap().token()
    }
}

impl<C> Auth<C, Optional> {
    /// Get the claims if authenticated.
    pub fn claims(&self) -> Option<&C> {
        self.context.as_ref().map(|ctx| &ctx.claims)
    }

    /// Check if authenticated.
    pub fn is_authenticated(&self) -> bool {
        self.context.is_some()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_authenticated_user() {
        #[derive(Clone, Debug, PartialEq)]
        struct Claims {
            sub: String,
        }

        let user = AuthenticatedUser(Claims {
            sub: "user123".to_string(),
        });

        assert_eq!(user.claims().sub, "user123");
        assert_eq!(user.sub, "user123"); // via Deref

        let claims = user.into_inner();
        assert_eq!(claims.sub, "user123");
    }

    #[test]
    fn test_auth_rejection() {
        let rejection = AuthRejection::new(AuthError::MissingToken);
        assert!(rejection.to_string().contains("missing"));
    }

    #[test]
    fn test_auth_ext_trait() {
        #[derive(Clone)]
        struct Claims {
            sub: String,
        }

        let mut req = hyper::Request::builder()
            .body(())
            .unwrap();

        // No auth context initially
        assert!(req.auth_context::<Claims>().is_none());
        assert!(req.claims::<Claims>().is_none());

        // Add auth context
        req.extensions_mut().insert(AuthContext::new(
            Claims {
                sub: "user123".to_string(),
            },
            "token",
        ));

        // Now available
        assert!(req.auth_context::<Claims>().is_some());
        assert_eq!(req.claims::<Claims>().unwrap().sub, "user123");
    }
}
