//! Scalar UI integration for interactive OpenAPI documentation.
//!
//! This module provides integration with Scalar (<https://scalar.com>), a modern
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
    /// CDN URL for Scalar JS (default: jsdelivr with latest version)
    pub cdn_url: String,
    /// SRI hash for CDN integrity verification (optional but recommended)
    pub sri_hash: Option<String>,
    /// Fallback CDN URL if primary fails (optional)
    pub fallback_cdn_url: Option<String>,
    /// Proxy URL for "Try It" requests to avoid CORS issues (optional)
    pub proxy_url: Option<String>,
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
            cdn_url: "https://cdn.jsdelivr.net/npm/@scalar/api-reference".to_string(),
            sri_hash: None,
            fallback_cdn_url: None,
            proxy_url: None,
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

    /// Set custom CDN URL for Scalar JS
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::ScalarConfig;
    ///
    /// let config = ScalarConfig::new()
    ///     .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0");
    /// ```
    pub fn cdn_url(mut self, url: impl Into<String>) -> Self {
        self.cdn_url = url.into();
        self
    }

    /// Set SRI hash for CDN integrity verification
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::ScalarConfig;
    ///
    /// let config = ScalarConfig::new()
    ///     .sri_hash("sha384-abc123...");
    /// ```
    pub fn sri_hash(mut self, hash: impl Into<String>) -> Self {
        self.sri_hash = Some(hash.into());
        self
    }

    /// Set fallback CDN URL if primary fails
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::ScalarConfig;
    ///
    /// let config = ScalarConfig::new()
    ///     .fallback_cdn_url("https://unpkg.com/@scalar/api-reference");
    /// ```
    pub fn fallback_cdn_url(mut self, url: impl Into<String>) -> Self {
        self.fallback_cdn_url = Some(url.into());
        self
    }

    /// Set proxy URL for "Try It" requests
    ///
    /// A proxy is recommended to avoid CORS issues when making requests
    /// to your API from the documentation interface.
    ///
    /// # Example
    ///
    /// ```rust
    /// use allframe_core::router::ScalarConfig;
    ///
    /// let config = ScalarConfig::new()
    ///     .proxy_url("https://proxy.scalar.com");
    /// ```
    pub fn proxy_url(mut self, url: impl Into<String>) -> Self {
        self.proxy_url = Some(url.into());
        self
    }

    /// Generate the configuration JSON for Scalar
    pub fn to_json(&self) -> serde_json::Value {
        let mut config = serde_json::json!({
            "theme": self.theme,
            "layout": self.layout,
            "showSidebar": self.show_sidebar,
            "hideDownloadButton": self.hide_download_button,
            "hideModels": self.hide_models,
        });

        // Add proxy URL if provided (for "Try It" functionality)
        if let Some(ref proxy) = self.proxy_url {
            config["proxy"] = serde_json::Value::String(proxy.clone());
        }

        config
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

    // Build script tag with SRI if provided
    let script_attrs = if let Some(sri) = &config.sri_hash {
        format!(
            r#"src="{}" integrity="{}" crossorigin="anonymous""#,
            config.cdn_url, sri
        )
    } else {
        format!(r#"src="{}""#, config.cdn_url)
    };

    // Build fallback script if provided
    let fallback_script = if let Some(fallback_url) = &config.fallback_cdn_url {
        format!(
            r#"
    <script>
        // Fallback CDN loader
        window.addEventListener('error', function(e) {{
            if (e.target.tagName === 'SCRIPT' && e.target.src.includes('scalar')) {{
                console.warn('Primary CDN failed, loading from fallback...');
                var fallback = document.createElement('script');
                fallback.src = '{}';
                document.body.appendChild(fallback);
            }}
        }}, true);
    </script>"#,
            fallback_url
        )
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
    {custom_style}{fallback_script}
</head>
<body>
    <script
        id="api-reference"
        data-configuration='{configuration}'
    >{openapi_spec}</script>
    <script {script_attrs}></script>
</body>
</html>"#,
        title = title,
        custom_style = custom_style,
        fallback_script = fallback_script,
        configuration = configuration,
        openapi_spec = openapi_spec_json,
        script_attrs = script_attrs,
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
        assert_eq!(
            config.cdn_url,
            "https://cdn.jsdelivr.net/npm/@scalar/api-reference"
        );
        assert_eq!(config.sri_hash, None);
        assert_eq!(config.fallback_cdn_url, None);
        assert_eq!(config.proxy_url, None);
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

    #[test]
    fn test_scalar_config_with_cdn_url() {
        let config = ScalarConfig::new()
            .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0");

        assert_eq!(
            config.cdn_url,
            "https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0"
        );
    }

    #[test]
    fn test_scalar_config_with_sri_hash() {
        let config = ScalarConfig::new().sri_hash("sha384-abc123def456");

        assert_eq!(config.sri_hash, Some("sha384-abc123def456".to_string()));
    }

    #[test]
    fn test_scalar_config_with_fallback_cdn() {
        let config =
            ScalarConfig::new().fallback_cdn_url("https://unpkg.com/@scalar/api-reference");

        assert_eq!(
            config.fallback_cdn_url,
            Some("https://unpkg.com/@scalar/api-reference".to_string())
        );
    }

    #[test]
    fn test_scalar_html_with_sri() {
        let config = ScalarConfig::new()
            .cdn_url("https://cdn.jsdelivr.net/npm/@scalar/api-reference@1.25.0")
            .sri_hash("sha384-abc123");
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains("integrity=\"sha384-abc123\""));
        assert!(html.contains("crossorigin=\"anonymous\""));
    }

    #[test]
    fn test_scalar_html_with_fallback() {
        let config =
            ScalarConfig::new().fallback_cdn_url("https://unpkg.com/@scalar/api-reference");
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(html.contains("Fallback CDN loader"));
        assert!(html.contains("https://unpkg.com/@scalar/api-reference"));
        assert!(html.contains("window.addEventListener('error'"));
    }

    #[test]
    fn test_scalar_html_without_fallback() {
        let config = ScalarConfig::default();
        let spec = r#"{"openapi":"3.1.0"}"#;
        let html = scalar_html(&config, "Test API", spec);

        assert!(!html.contains("Fallback CDN loader"));
        assert!(!html.contains("window.addEventListener('error'"));
    }

    #[test]
    fn test_scalar_config_with_proxy() {
        let config = ScalarConfig::new().proxy_url("https://proxy.scalar.com");

        assert_eq!(
            config.proxy_url,
            Some("https://proxy.scalar.com".to_string())
        );
    }

    #[test]
    fn test_scalar_config_to_json_with_proxy() {
        let config = ScalarConfig::new()
            .proxy_url("https://proxy.scalar.com")
            .show_sidebar(false);

        let json = config.to_json();
        assert_eq!(json["proxy"], "https://proxy.scalar.com");
        assert_eq!(json["showSidebar"], false);
    }

    #[test]
    fn test_scalar_config_to_json_without_proxy() {
        let config = ScalarConfig::default();
        let json = config.to_json();

        assert!(json.get("proxy").is_none());
    }
}
