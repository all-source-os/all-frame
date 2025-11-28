//! Documentation serving helpers
//!
//! This module provides helpers for serving API documentation.
//! The actual HTTP serving is left to the user's choice of framework
//! (Axum, Actix, etc.), but this module provides the data and formatting.

use crate::router::Router;
use serde_json::Value;

/// Documentation configuration
///
/// Configures how API documentation should be served.
#[derive(Debug, Clone)]
pub struct DocsConfig {
    /// Path where documentation will be served (e.g., "/docs")
    pub path: String,
    /// API title
    pub title: String,
    /// API version
    pub version: String,
    /// Optional API description
    pub description: Option<String>,
}

impl DocsConfig {
    /// Create a new documentation configuration
    pub fn new(path: impl Into<String>, title: impl Into<String>, version: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            title: title.into(),
            version: version.into(),
            description: None,
        }
    }

    /// Set the API description
    pub fn with_description(mut self, description: impl Into<String>) -> Self {
        self.description = Some(description.into());
        self
    }

    /// Get the OpenAPI spec path
    pub fn openapi_path(&self) -> String {
        format!("{}/openapi.json", self.path.trim_end_matches('/'))
    }
}

impl Router {
    /// Get documentation configuration helper
    ///
    /// Returns a DocsConfig that can be used to serve documentation.
    pub fn docs_config(&self, path: &str, title: &str, version: &str) -> DocsConfig {
        DocsConfig::new(path, title, version)
    }

    /// Generate OpenAPI JSON for serving at /docs/openapi.json
    ///
    /// This is a convenience method that generates the OpenAPI spec
    /// in JSON format ready to be served via HTTP.
    pub fn openapi_json(&self, title: &str, version: &str) -> String {
        let spec = self.to_openapi(title, version);
        serde_json::to_string_pretty(&spec).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate OpenAPI JSON with description
    pub fn openapi_json_with_description(
        &self,
        title: &str,
        version: &str,
        description: &str,
    ) -> String {
        let spec = self.to_openapi_with_description(title, version, description);
        serde_json::to_string_pretty(&spec).unwrap_or_else(|_| "{}".to_string())
    }

    /// Generate a basic HTML page for documentation
    ///
    /// Returns a simple HTML page that can serve as a landing page
    /// for API documentation. In production, you'd want to use
    /// a proper documentation UI like Scalar or Swagger UI.
    pub fn docs_html(&self, config: &DocsConfig) -> String {
        let openapi_path = config.openapi_path();

        format!(
            r#"<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>{title} - API Documentation</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, "Segoe UI", Roboto, sans-serif;
            max-width: 800px;
            margin: 40px auto;
            padding: 20px;
            line-height: 1.6;
        }}
        h1 {{ color: #333; }}
        .info {{ background: #f5f5f5; padding: 20px; border-radius: 8px; }}
        .links {{ margin-top: 30px; }}
        .links a {{
            display: inline-block;
            margin: 10px 10px 10px 0;
            padding: 10px 20px;
            background: #007bff;
            color: white;
            text-decoration: none;
            border-radius: 4px;
        }}
        .links a:hover {{ background: #0056b3; }}
    </style>
</head>
<body>
    <h1>{title}</h1>
    <div class="info">
        <p><strong>Version:</strong> {version}</p>
        {description}
    </div>
    <div class="links">
        <a href="{openapi_path}">OpenAPI Specification</a>
    </div>
    <p>
        <small>Built with <a href="https://github.com/all-source-os/all-frame">AllFrame</a></small>
    </p>
</body>
</html>"#,
            title = config.title,
            version = config.version,
            description = config
                .description
                .as_ref()
                .map(|d| format!("<p>{}</p>", d))
                .unwrap_or_default(),
            openapi_path = openapi_path
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_docs_config_creation() {
        let config = DocsConfig::new("/docs", "My API", "1.0.0");

        assert_eq!(config.path, "/docs");
        assert_eq!(config.title, "My API");
        assert_eq!(config.version, "1.0.0");
        assert_eq!(config.description, None);
    }

    #[test]
    fn test_docs_config_with_description() {
        let config = DocsConfig::new("/docs", "My API", "1.0.0")
            .with_description("A great API");

        assert_eq!(config.description, Some("A great API".to_string()));
    }

    #[test]
    fn test_openapi_path() {
        let config = DocsConfig::new("/docs", "API", "1.0");
        assert_eq!(config.openapi_path(), "/docs/openapi.json");
    }

    #[test]
    fn test_openapi_path_with_trailing_slash() {
        let config = DocsConfig::new("/docs/", "API", "1.0");
        assert_eq!(config.openapi_path(), "/docs/openapi.json");
    }

    #[tokio::test]
    async fn test_router_docs_config() {
        let router = Router::new();
        let config = router.docs_config("/api-docs", "Test API", "2.0.0");

        assert_eq!(config.path, "/api-docs");
        assert_eq!(config.title, "Test API");
        assert_eq!(config.version, "2.0.0");
    }

    #[tokio::test]
    async fn test_openapi_json() {
        let mut router = Router::new();
        router.get("/users", || async { "Users".to_string() });

        let json = router.openapi_json("Test API", "1.0.0");

        assert!(json.contains("\"openapi\": \"3.1.0\""));
        assert!(json.contains("\"title\": \"Test API\""));
        assert!(json.contains("\"/users\""));
    }

    #[tokio::test]
    async fn test_openapi_json_with_description() {
        let router = Router::new();
        let json = router.openapi_json_with_description(
            "Test API",
            "1.0.0",
            "A test API",
        );

        assert!(json.contains("\"description\": \"A test API\""));
    }

    #[tokio::test]
    async fn test_docs_html() {
        let router = Router::new();
        let config = DocsConfig::new("/docs", "My API", "1.0.0");
        let html = router.docs_html(&config);

        assert!(html.contains("<title>My API - API Documentation</title>"));
        assert!(html.contains("Version:</strong> 1.0.0"));
        assert!(html.contains("href=\"/docs/openapi.json\""));
    }

    #[tokio::test]
    async fn test_docs_html_with_description() {
        let router = Router::new();
        let config = DocsConfig::new("/docs", "My API", "1.0.0")
            .with_description("A great API");
        let html = router.docs_html(&config);

        assert!(html.contains("A great API"));
    }

    #[tokio::test]
    async fn test_docs_html_contains_allframe_link() {
        let router = Router::new();
        let config = DocsConfig::new("/docs", "API", "1.0");
        let html = router.docs_html(&config);

        assert!(html.contains("AllFrame"));
        assert!(html.contains("github.com/all-source-os/all-frame"));
    }

    #[tokio::test]
    async fn test_openapi_json_is_valid_json() {
        let mut router = Router::new();
        router.get("/test", || async { "Test".to_string() });

        let json = router.openapi_json("API", "1.0");
        let parsed: Value = serde_json::from_str(&json).expect("Should be valid JSON");

        assert_eq!(parsed["openapi"], "3.1.0");
        assert_eq!(parsed["info"]["title"], "API");
    }
}
