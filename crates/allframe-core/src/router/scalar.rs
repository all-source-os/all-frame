//! Scalar UI integration for interactive OpenAPI documentation.
//!
//! This module provides integration with Scalar (https://scalar.com), a modern
//! OpenAPI documentation UI that offers:
//! - Beautiful dark mode by default
//! - Interactive "Try It" functionality
//! - Mobile-friendly responsive design
//! - <50KB JavaScript bundle
//!
//! # Example
//!
//! ```rust
//! use allframe_core::router::{Router, ScalarConfig, ScalarTheme};
//!
//! let mut router = Router::new();
//! router.get("/users", || async { "Users".to_string() });
//!
//! // Generate Scalar HTML with default config
//! let html = router.scalar("My API", "1.0.0");
//!
//! // Or with custom configuration
//! let config = ScalarConfig::new()
//!     .theme(ScalarTheme::Auto)
//!     .show_sidebar(true)
//!     .custom_css("body { font-family: 'Inter'; }");
//! let html = router.scalar_docs(config, "My API", "1.0.0");
//! ```

use serde::{Deserialize, Serialize};

/// Scalar UI theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScalarTheme {
    /// Dark mode (default)
    #[default]
    Dark,
    /// Light mode
    Light,
    /// Auto-detect from system preferences
    Auto,
}

/// Scalar UI layout options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum ScalarLayout {
    /// Classic three-column layout
    Classic,
    /// Modern two-column layout (default)
    #[default]
    Modern,
}

/// Configuration for Scalar UI
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ScalarConfig {
    /// URL to OpenAPI spec (default: "/docs/openapi.json")
    pub spec_url: String,
    /// UI theme
    pub theme: ScalarTheme,
    /// Show sidebar navigation
    pub show_sidebar: bool,
    /// Layout style
    pub layout: ScalarLayout,
    /// Custom CSS to inject
    pub custom_css: Option<String>,
    /// Hide download button
    pub hide_download_button: bool,
    /// Hide models section
    pub hide_models: bool,
}

impl Default for ScalarConfig {
    fn default() -> Self {
        Self {
            spec_url: "/docs/openapi.json".to_string(),
            theme: ScalarTheme::Dark,
            show_sidebar: true,
            layout: ScalarLayout::Modern,
            custom_css: None,
            hide_download_button: false,
            hide_models: false,
        }
    }
}

impl ScalarConfig {
    /// Create a new ScalarConfig with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the OpenAPI spec URL
    pub fn spec_url(mut self, url: impl Into<String>) -> Self {
        self.spec_url = url.into();
        self
    }

    /// Set the UI theme
    pub fn theme(mut self, theme: ScalarTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Set whether to show sidebar
    pub fn show_sidebar(mut self, show: bool) -> Self {
        self.show_sidebar = show;
        self
    }

    /// Set the layout style
    pub fn layout(mut self, layout: ScalarLayout) -> Self {
        self.layout = layout;
        self
    }

    /// Set custom CSS
    pub fn custom_css(mut self, css: impl Into<String>) -> Self {
        self.custom_css = Some(css.into());
        self
    }

    /// Set whether to hide download button
    pub fn hide_download_button(mut self, hide: bool) -> Self {
        self.hide_download_button = hide;
        self
    }

    /// Set whether to hide models section
    pub fn hide_models(mut self, hide: bool) -> Self {
        self.hide_models = hide;
        self
    }

    /// Generate the configuration JSON for Scalar
    pub fn to_json(&self) -> serde_json::Value {
        serde_json::json!({
            "theme": self.theme,
            "layout": self.layout,
            "showSidebar": self.show_sidebar,
            "hideDownloadButton": self.hide_download_button,
            "hideModels": self.hide_models,
        })
    }
}

/// Generate Scalar HTML page
///
/// # Arguments
///
/// * `config` - Scalar configuration
/// * `title` - Page title
/// * `openapi_spec_json` - OpenAPI specification as JSON string
///
/// # Returns
///
/// Complete HTML page ready to serve
pub fn scalar_html(config: &ScalarConfig, title: &str, openapi_spec_json: &str) -> String {
    let configuration = config.to_json();

    let custom_style = if let Some(css) = &config.custom_css {
        format!("<style>{}</style>", css)
    } else {
        String::new()
    };

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <title>{title} - API Documentation</title>
    <meta charset="utf-8" />
    <meta name="viewport" content="width=device-width, initial-scale=1" />
    <style>
        body {{ margin: 0; padding: 0; }}
    </style>
    {custom_style}
</head>
<body>
    <script
        id="api-reference"
        data-configuration='{configuration}'
    >{openapi_spec}</script>
    <script src="https://cdn.jsdelivr.net/npm/@scalar/api-reference"></script>
</body>
</html>"#,
        title = title,
        custom_style = custom_style,
        configuration = configuration,
        openapi_spec = openapi_spec_json,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_scalar_theme_default() {
        assert_eq!(ScalarTheme::default(), ScalarTheme::Dark);
    }

    #[test]
    fn test_scalar_layout_default() {
        assert_eq!(ScalarLayout::default(), ScalarLayout::Modern);
    }

    #[test]
    fn test_scalar_config_default() {
        let config = ScalarConfig::default();
        assert_eq!(config.spec_url, "/docs/openapi.json");
        assert_eq!(config.theme, ScalarTheme::Dark);
        assert_eq!(config.show_sidebar, true);
        assert_eq!(config.layout, ScalarLayout::Modern);
        assert_eq!(config.custom_css, None);
        assert_eq!(config.hide_download_button, false);
        assert_eq!(config.hide_models, false);
    }

    #[test]
    fn test_scalar_config_builder() {
        let config = ScalarConfig::new()
            .spec_url("/api/openapi.json")
            .theme(ScalarTheme::Light)
            .show_sidebar(false)
            .layout(ScalarLayout::Classic)
            .custom_css("body { color: red; }")
            .hide_download_button(true)
            .hide_models(true);

        assert_eq!(config.spec_url, "/api/openapi.json");
        assert_eq!(config.theme, ScalarTheme::Light);
        assert_eq!(config.show_sidebar, false);
        assert_eq!(config.layout, ScalarLayout::Classic);
        assert_eq!(config.custom_css, Some("body { color: red; }".to_string()));
        assert_eq!(config.hide_download_button, true);
        assert_eq!(config.hide_models, true);
    }

    #[test]
    fn test_scalar_config_to_json() {
        let config = ScalarConfig::new()
            .theme(ScalarTheme::Auto)
            .layout(ScalarLayout::Classic)
            .show_sidebar(false);

        let json = config.to_json();
        assert_eq!(json["theme"], "auto");
        assert_eq!(json["layout"], "classic");
        assert_eq!(json["showSidebar"], false);
    }

    #[test]
    fn test_scalar_html_contains_title() {
        let config = ScalarConfig::default();
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains("<title>Test API - API Documentation</title>"));
    }

    #[test]
    fn test_scalar_html_contains_script_tag() {
        let config = ScalarConfig::default();
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains(r#"id="api-reference""#));
        assert!(html.contains(r#"https://cdn.jsdelivr.net/npm/@scalar/api-reference"#));
    }

    #[test]
    fn test_scalar_html_contains_configuration() {
        let config = ScalarConfig::new()
            .theme(ScalarTheme::Light)
            .show_sidebar(false);
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains(r#"data-configuration='"#));
        assert!(html.contains(r#""theme":"light""#));
        assert!(html.contains(r#""showSidebar":false"#));
    }

    #[test]
    fn test_scalar_html_contains_openapi_spec() {
        let config = ScalarConfig::default();
        let spec = r#"{"openapi":"3.1.0","info":{"title":"Test"}}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains(spec));
    }

    #[test]
    fn test_scalar_html_with_custom_css() {
        let config = ScalarConfig::new().custom_css("body { font-family: 'Inter'; }");
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains("<style>body { font-family: 'Inter'; }</style>"));
    }

    #[test]
    fn test_scalar_html_without_custom_css() {
        let config = ScalarConfig::default();
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        // Should not contain empty style tag
        assert!(!html.contains("<style></style>"));
    }
}
