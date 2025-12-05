//! GraphiQL Playground Integration
//!
//! This module provides comprehensive GraphiQL playground integration for
//! GraphQL API documentation, similar to our Scalar integration for REST APIs.
//!
//! # Features
//!
//! - **GraphiQL Configuration**: Customizable playground settings
//! - **Theme Support**: Light and dark themes
//! - **Query History**: Persistent query history
//! - **Variables Editor**: JSON variable editing with validation
//! - **Headers Configuration**: Custom HTTP headers
//! - **Subscription Support**: WebSocket subscriptions
//! - **Schema Explorer**: Interactive schema documentation
//!
//! # Example
//!
//! ```rust
//! use allframe_core::router::{GraphiQLConfig, GraphiQLTheme, graphiql_html};
//!
//! let config = GraphiQLConfig::new()
//!     .endpoint_url("/graphql")
//!     .subscription_url("ws://localhost:3000/graphql")
//!     .theme(GraphiQLTheme::Dark)
//!     .enable_explorer(true)
//!     .enable_history(true);
//!
//! let html = graphiql_html(&config, "My GraphQL API");
//! // Serve this HTML at /graphql/playground
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// GraphiQL theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum GraphiQLTheme {
    /// Light theme
    Light,
    /// Dark theme (default)
    Dark,
}

impl Default for GraphiQLTheme {
    fn default() -> Self {
        Self::Dark
    }
}

/// GraphiQL playground configuration
///
/// Provides comprehensive configuration for the GraphiQL playground including
/// themes, subscriptions, headers, and interactive features.
#[derive(Debug, Clone)]
pub struct GraphiQLConfig {
    /// GraphQL endpoint URL (required)
    pub endpoint_url: String,

    /// WebSocket URL for subscriptions (optional)
    pub subscription_url: Option<String>,

    /// UI theme (Light or Dark)
    pub theme: GraphiQLTheme,

    /// Enable schema explorer sidebar
    pub enable_explorer: bool,

    /// Enable query history persistence
    pub enable_history: bool,

    /// Custom HTTP headers
    pub headers: HashMap<String, String>,

    /// CDN URL for GraphiQL (for version pinning)
    pub cdn_url: String,

    /// Custom CSS styling
    pub custom_css: Option<String>,
}

impl Default for GraphiQLConfig {
    fn default() -> Self {
        Self {
            endpoint_url: "/graphql".to_string(),
            subscription_url: None,
            theme: GraphiQLTheme::Dark,
            enable_explorer: true,
            enable_history: true,
            headers: HashMap::new(),
            cdn_url: "https://unpkg.com/graphiql@3.0.0/graphiql.min.css".to_string(),
            custom_css: None,
        }
    }
}

impl GraphiQLConfig {
    /// Create a new GraphiQL configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the GraphQL endpoint URL
    pub fn endpoint_url(mut self, url: impl Into<String>) -> Self {
        self.endpoint_url = url.into();
        self
    }

    /// Set the WebSocket URL for subscriptions
    pub fn subscription_url(mut self, url: impl Into<String>) -> Self {
        self.subscription_url = Some(url.into());
        self
    }

    /// Set the UI theme
    pub fn theme(mut self, theme: GraphiQLTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Enable or disable the schema explorer
    pub fn enable_explorer(mut self, enable: bool) -> Self {
        self.enable_explorer = enable;
        self
    }

    /// Enable or disable query history
    pub fn enable_history(mut self, enable: bool) -> Self {
        self.enable_history = enable;
        self
    }

    /// Add a custom HTTP header
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set the CDN URL for version pinning
    pub fn cdn_url(mut self, url: impl Into<String>) -> Self {
        self.cdn_url = url.into();
        self
    }

    /// Set custom CSS styling
    pub fn custom_css(mut self, css: impl Into<String>) -> Self {
        self.custom_css = Some(css.into());
        self
    }

    /// Convert configuration to JSON for embedding in HTML
    pub fn to_json(&self) -> serde_json::Value {
        let mut config = serde_json::json!({
            "endpoint": self.endpoint_url,
            "theme": self.theme,
            "explorer": self.enable_explorer,
            "history": self.enable_history,
        });

        if let Some(ref sub_url) = self.subscription_url {
            config["subscriptionUrl"] = serde_json::Value::String(sub_url.clone());
        }

        if !self.headers.is_empty() {
            config["headers"] = serde_json::to_value(&self.headers).unwrap();
        }

        config
    }
}

/// Generate GraphiQL playground HTML
///
/// Creates a complete HTML page with the GraphiQL playground embedded,
/// configured according to the provided settings.
///
/// # Arguments
///
/// * `config` - GraphiQL configuration
/// * `title` - Page title
///
/// # Returns
///
/// Complete HTML string ready to serve
///
/// # Example
///
/// ```rust
/// use allframe_core::router::{GraphiQLConfig, graphiql_html};
///
/// let config = GraphiQLConfig::new()
///     .endpoint_url("/graphql")
///     .theme(allframe_core::router::GraphiQLTheme::Dark);
///
/// let html = graphiql_html(&config, "My API");
/// // Serve at /graphql/playground
/// ```
pub fn graphiql_html(config: &GraphiQLConfig, title: &str) -> String {
    let config_json = serde_json::to_string(&config.to_json()).unwrap();
    let theme_class = match config.theme {
        GraphiQLTheme::Light => "graphiql-light",
        GraphiQLTheme::Dark => "graphiql-dark",
    };

    let custom_css = config.custom_css.as_deref().unwrap_or("");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <link rel="stylesheet" href="{cdn_url}" />
    <style>
        body {{
            height: 100vh;
            margin: 0;
            overflow: hidden;
        }}
        #graphiql {{
            height: 100vh;
        }}
        {custom_css}
    </style>
</head>
<body class="{theme_class}">
    <div id="graphiql">Loading GraphiQL...</div>
    <script
        crossorigin
        src="https://unpkg.com/react@18/umd/react.production.min.js"
    ></script>
    <script
        crossorigin
        src="https://unpkg.com/react-dom@18/umd/react-dom.production.min.js"
    ></script>
    <script
        crossorigin
        src="https://unpkg.com/graphiql@3.0.0/graphiql.min.js"
    ></script>
    <script>
        const config = {config_json};
        const root = ReactDOM.createRoot(document.getElementById('graphiql'));
        const fetcher = GraphiQL.createFetcher({{
            url: config.endpoint,
            subscriptionUrl: config.subscriptionUrl,
            headers: config.headers || {{}}
        }});
        root.render(
            React.createElement(GraphiQL, {{
                fetcher: fetcher,
                defaultEditorToolsVisibility: config.explorer,
                storage: config.history ? window.localStorage : null
            }})
        );
    </script>
</body>
</html>"#,
        title = title,
        cdn_url = config.cdn_url,
        theme_class = theme_class,
        custom_css = custom_css,
        config_json = config_json,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_graphiql_config_defaults() {
        let config = GraphiQLConfig::new();
        assert_eq!(config.endpoint_url, "/graphql");
        assert_eq!(config.theme, GraphiQLTheme::Dark);
        assert!(config.enable_explorer);
        assert!(config.enable_history);
        assert!(config.subscription_url.is_none());
    }

    #[test]
    fn test_graphiql_config_builder() {
        let config = GraphiQLConfig::new()
            .endpoint_url("/api/graphql")
            .subscription_url("ws://localhost:4000/graphql")
            .theme(GraphiQLTheme::Light)
            .enable_explorer(false)
            .enable_history(false)
            .add_header("Authorization", "Bearer token123");

        assert_eq!(config.endpoint_url, "/api/graphql");
        assert_eq!(
            config.subscription_url,
            Some("ws://localhost:4000/graphql".to_string())
        );
        assert_eq!(config.theme, GraphiQLTheme::Light);
        assert!(!config.enable_explorer);
        assert!(!config.enable_history);
        assert_eq!(
            config.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_graphiql_html_generation() {
        let config = GraphiQLConfig::new()
            .endpoint_url("/graphql")
            .theme(GraphiQLTheme::Dark);

        let html = graphiql_html(&config, "Test API");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test API"));
        assert!(html.contains("/graphql"));
        assert!(html.contains("graphiql-dark"));
        assert!(html.contains("GraphiQL"));
    }

    #[test]
    fn test_graphiql_html_with_subscription() {
        let config = GraphiQLConfig::new()
            .endpoint_url("/graphql")
            .subscription_url("ws://localhost:3000/graphql");

        let html = graphiql_html(&config, "Test API");

        assert!(html.contains("ws://localhost:3000/graphql"));
        assert!(html.contains("subscriptionUrl"));
    }

    #[test]
    fn test_graphiql_theme_serialization() {
        let light = GraphiQLTheme::Light;
        let dark = GraphiQLTheme::Dark;

        assert_eq!(serde_json::to_string(&light).unwrap(), "\"light\"");
        assert_eq!(serde_json::to_string(&dark).unwrap(), "\"dark\"");
    }

    #[test]
    fn test_graphiql_config_json_generation() {
        let config = GraphiQLConfig::new()
            .endpoint_url("/api/graphql")
            .theme(GraphiQLTheme::Light)
            .enable_explorer(true)
            .add_header("X-API-Key", "secret");

        let json = config.to_json();

        assert_eq!(json["endpoint"], "/api/graphql");
        assert_eq!(json["theme"], "light");
        assert_eq!(json["explorer"], true);
        assert_eq!(json["headers"]["X-API-Key"], "secret");
    }

    #[test]
    fn test_graphiql_custom_css() {
        let config = GraphiQLConfig::new().custom_css("body { background: #1a1a1a; }");

        let html = graphiql_html(&config, "Test API");

        assert!(html.contains("body { background: #1a1a1a; }"));
    }
}
