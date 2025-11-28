//! HTTP method types for type-safe routing
//!
//! This module provides a type-safe representation of HTTP methods
//! to replace stringly-typed method handling.

use std::fmt;

/// HTTP method for REST routes
///
/// Represents standard HTTP methods with compile-time type safety.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Method {
    /// GET method
    GET,
    /// POST method
    POST,
    /// PUT method
    PUT,
    /// DELETE method
    DELETE,
    /// PATCH method
    PATCH,
    /// HEAD method
    HEAD,
    /// OPTIONS method
    OPTIONS,
}

impl Method {
    /// Convert Method to string representation
    pub fn as_str(&self) -> &'static str {
        match self {
            Method::GET => "GET",
            Method::POST => "POST",
            Method::PUT => "PUT",
            Method::DELETE => "DELETE",
            Method::PATCH => "PATCH",
            Method::HEAD => "HEAD",
            Method::OPTIONS => "OPTIONS",
        }
    }
}

impl fmt::Display for Method {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

impl From<Method> for String {
    fn from(method: Method) -> String {
        method.as_str().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_method_as_str() {
        assert_eq!(Method::GET.as_str(), "GET");
        assert_eq!(Method::POST.as_str(), "POST");
        assert_eq!(Method::PUT.as_str(), "PUT");
        assert_eq!(Method::DELETE.as_str(), "DELETE");
        assert_eq!(Method::PATCH.as_str(), "PATCH");
        assert_eq!(Method::HEAD.as_str(), "HEAD");
        assert_eq!(Method::OPTIONS.as_str(), "OPTIONS");
    }

    #[test]
    fn test_method_display() {
        assert_eq!(format!("{}", Method::GET), "GET");
        assert_eq!(format!("{}", Method::POST), "POST");
    }

    #[test]
    fn test_method_into_string() {
        let method: String = Method::GET.into();
        assert_eq!(method, "GET");
    }

    #[test]
    fn test_method_equality() {
        assert_eq!(Method::GET, Method::GET);
        assert_ne!(Method::GET, Method::POST);
    }

    #[test]
    fn test_method_clone() {
        let method1 = Method::POST;
        let method2 = method1;
        assert_eq!(method1, method2);
    }
}
