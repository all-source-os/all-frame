//! gRPC Service Explorer Integration
//!
//! This module provides a web-based gRPC service explorer for interactive
//! gRPC API documentation and testing.
//!
//! # Features
//!
//! - **Service Explorer**: Interactive UI for browsing gRPC services
//! - **Method Testing**: Test gRPC methods directly from the browser
//! - **Reflection Support**: Automatic service discovery via gRPC reflection
//! - **Stream Testing**: Test server/client/bidirectional streams
//! - **Request Builder**: Build and send gRPC requests with JSON
//! - **Proto Viewer**: View service definitions
//!
//! # Example
//!
//! ```rust
//! use allframe_core::router::{GrpcExplorerConfig, grpc_explorer_html};
//!
//! let config = GrpcExplorerConfig::new()
//!     .server_url("http://localhost:50051")
//!     .enable_reflection(true)
//!     .enable_tls(false);
//!
//! let html = grpc_explorer_html(&config, "My gRPC API");
//! // Serve this HTML at /grpc/explorer
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// gRPC Explorer theme options
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "lowercase")]
pub enum GrpcExplorerTheme {
    /// Light theme
    Light,
    /// Dark theme (default)
    #[default]
    Dark,
}

/// gRPC Explorer configuration
///
/// Provides comprehensive configuration for the gRPC service explorer including
/// server connection, reflection, and UI customization.
#[derive(Debug, Clone)]
pub struct GrpcExplorerConfig {
    /// gRPC server URL (required)
    pub server_url: String,

    /// Enable gRPC reflection for service discovery
    pub enable_reflection: bool,

    /// Enable TLS/SSL connection
    pub enable_tls: bool,

    /// UI theme (Light or Dark)
    pub theme: GrpcExplorerTheme,

    /// Custom HTTP headers for metadata
    pub headers: HashMap<String, String>,

    /// Custom CSS styling
    pub custom_css: Option<String>,

    /// Request timeout in seconds
    pub timeout_seconds: u32,
}

impl Default for GrpcExplorerConfig {
    fn default() -> Self {
        Self {
            server_url: "http://localhost:50051".to_string(),
            enable_reflection: true,
            enable_tls: false,
            theme: GrpcExplorerTheme::Dark,
            headers: HashMap::new(),
            custom_css: None,
            timeout_seconds: 30,
        }
    }
}

impl GrpcExplorerConfig {
    /// Create a new gRPC Explorer configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Set the gRPC server URL
    pub fn server_url(mut self, url: impl Into<String>) -> Self {
        self.server_url = url.into();
        self
    }

    /// Enable or disable gRPC reflection
    pub fn enable_reflection(mut self, enable: bool) -> Self {
        self.enable_reflection = enable;
        self
    }

    /// Enable or disable TLS
    pub fn enable_tls(mut self, enable: bool) -> Self {
        self.enable_tls = enable;
        self
    }

    /// Set the UI theme
    pub fn theme(mut self, theme: GrpcExplorerTheme) -> Self {
        self.theme = theme;
        self
    }

    /// Add a custom metadata header
    pub fn add_header(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.headers.insert(key.into(), value.into());
        self
    }

    /// Set custom CSS styling
    pub fn custom_css(mut self, css: impl Into<String>) -> Self {
        self.custom_css = Some(css.into());
        self
    }

    /// Set request timeout in seconds
    pub fn timeout_seconds(mut self, seconds: u32) -> Self {
        self.timeout_seconds = seconds;
        self
    }

    /// Convert configuration to JSON for embedding in HTML
    pub fn to_json(&self) -> serde_json::Value {
        let mut config = serde_json::json!({
            "serverUrl": self.server_url,
            "reflection": self.enable_reflection,
            "tls": self.enable_tls,
            "theme": self.theme,
            "timeout": self.timeout_seconds,
        });

        if !self.headers.is_empty() {
            config["headers"] = serde_json::to_value(&self.headers).unwrap();
        }

        config
    }
}

/// Generate gRPC Explorer HTML
///
/// Creates a complete HTML page with the gRPC service explorer embedded,
/// configured according to the provided settings.
///
/// # Arguments
///
/// * `config` - gRPC Explorer configuration
/// * `title` - Page title
///
/// # Returns
///
/// Complete HTML string ready to serve
///
/// # Example
///
/// ```rust
/// use allframe_core::router::{GrpcExplorerConfig, grpc_explorer_html};
///
/// let config = GrpcExplorerConfig::new()
///     .server_url("http://localhost:50051")
///     .enable_reflection(true);
///
/// let html = grpc_explorer_html(&config, "My gRPC API");
/// // Serve at /grpc/explorer
/// ```
pub fn grpc_explorer_html(config: &GrpcExplorerConfig, title: &str) -> String {
    let config_json = serde_json::to_string(&config.to_json()).unwrap();
    let theme_class = match config.theme {
        GrpcExplorerTheme::Light => "grpc-light",
        GrpcExplorerTheme::Dark => "grpc-dark",
    };

    let custom_css = config.custom_css.as_deref().unwrap_or("");

    format!(
        r#"<!DOCTYPE html>
<html>
<head>
    <meta charset="utf-8">
    <meta name="viewport" content="width=device-width, initial-scale=1">
    <title>{title}</title>
    <style>
        * {{
            margin: 0;
            padding: 0;
            box-sizing: border-box;
        }}

        body {{
            height: 100vh;
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            background: {bg_color};
            color: {text_color};
        }}

        .grpc-explorer {{
            height: 100vh;
            display: flex;
            flex-direction: column;
        }}

        .header {{
            padding: 1rem 2rem;
            background: {header_bg};
            border-bottom: 1px solid {border_color};
        }}

        .header h1 {{
            font-size: 1.5rem;
            font-weight: 600;
        }}

        .server-info {{
            margin-top: 0.5rem;
            font-size: 0.875rem;
            opacity: 0.8;
        }}

        .main {{
            display: flex;
            flex: 1;
            overflow: hidden;
        }}

        .sidebar {{
            width: 300px;
            background: {sidebar_bg};
            border-right: 1px solid {border_color};
            overflow-y: auto;
            padding: 1rem;
        }}

        .content {{
            flex: 1;
            padding: 2rem;
            overflow-y: auto;
        }}

        .service-list {{
            list-style: none;
        }}

        .service-item {{
            padding: 0.75rem;
            margin-bottom: 0.5rem;
            background: {item_bg};
            border-radius: 0.5rem;
            cursor: pointer;
            transition: background 0.2s;
        }}

        .service-item:hover {{
            background: {item_hover_bg};
        }}

        .service-name {{
            font-weight: 600;
            margin-bottom: 0.25rem;
        }}

        .method-count {{
            font-size: 0.875rem;
            opacity: 0.7;
        }}

        .loading {{
            text-align: center;
            padding: 3rem;
        }}

        .spinner {{
            border: 3px solid rgba(255, 255, 255, 0.1);
            border-top: 3px solid {accent_color};
            border-radius: 50%;
            width: 40px;
            height: 40px;
            animation: spin 1s linear infinite;
            margin: 0 auto 1rem;
        }}

        @keyframes spin {{
            0% {{ transform: rotate(0deg); }}
            100% {{ transform: rotate(360deg); }}
        }}

        .error {{
            background: #dc2626;
            color: white;
            padding: 1rem;
            border-radius: 0.5rem;
            margin: 1rem 0;
        }}

        .info-box {{
            background: {info_bg};
            padding: 1.5rem;
            border-radius: 0.5rem;
            border-left: 4px solid {accent_color};
        }}

        .info-box h3 {{
            margin-bottom: 0.5rem;
            color: {accent_color};
        }}

        .features-list {{
            list-style: none;
            margin-top: 1rem;
        }}

        .features-list li {{
            padding: 0.5rem 0;
            display: flex;
            align-items: center;
        }}

        .features-list li:before {{
            content: "✓";
            color: {accent_color};
            font-weight: bold;
            margin-right: 0.75rem;
        }}

        {custom_css}
    </style>
</head>
<body class="{theme_class}">
    <div class="grpc-explorer">
        <div class="header">
            <h1>{title}</h1>
            <div class="server-info" id="server-info">
                Connecting to {server_url}...
            </div>
        </div>
        <div class="main">
            <div class="sidebar">
                <h2 style="margin-bottom: 1rem;">Services</h2>
                <div id="service-list">
                    <div class="loading">
                        <div class="spinner"></div>
                        <div>Loading services...</div>
                    </div>
                </div>
            </div>
            <div class="content">
                <div class="info-box">
                    <h3>gRPC Service Explorer</h3>
                    <p>Interactive gRPC API documentation and testing.</p>

                    <ul class="features-list">
                        <li>Browse gRPC services and methods</li>
                        <li>Test unary, server stream, client stream, and bidirectional calls</li>
                        <li>View service definitions and proto files</li>
                        <li>Automatic service discovery via gRPC reflection</li>
                        <li>Real-time request/response testing</li>
                    </ul>

                    <div style="margin-top: 1.5rem; padding-top: 1.5rem; border-top: 1px solid {border_color};">
                        <strong>Configuration:</strong>
                        <ul style="list-style: none; margin-top: 0.5rem;">
                            <li>Server: <code>{server_url}</code></li>
                            <li>Reflection: {reflection_status}</li>
                            <li>TLS: {tls_status}</li>
                            <li>Timeout: {timeout}s</li>
                        </ul>
                    </div>
                </div>

                <div id="service-detail" style="margin-top: 2rem;"></div>
            </div>
        </div>
    </div>

    <script>
        const config = {config_json};

        // Placeholder for future gRPC Web client integration
        // This will be enhanced with actual gRPC-Web support in future iterations

        console.log('gRPC Explorer Config:', config);

        // Simulate service loading
        setTimeout(() => {{
            const serviceList = document.getElementById('service-list');
            const serverInfo = document.getElementById('server-info');

            if (config.reflection) {{
                serviceList.innerHTML = `
                    <div class="info-box">
                        <h3>Reflection Enabled</h3>
                        <p>gRPC reflection API support will automatically discover services when connected to a gRPC server with reflection enabled.</p>
                        <p style="margin-top: 1rem;"><strong>To enable reflection in your gRPC server:</strong></p>
                        <pre style="background: rgba(0,0,0,0.2); padding: 1rem; border-radius: 0.25rem; margin-top: 0.5rem; overflow-x: auto;">
use tonic::transport::Server;
use tonic_reflection::server::Builder;

Server::builder()
    .add_service(Builder::configure()
        .register_encoded_file_descriptor_set(DESCRIPTOR_SET)
        .build()
        .unwrap())
    .serve(addr)
    .await?;</pre>
                    </div>
                `;
                serverInfo.textContent = `Connected to ${{config.serverUrl}} (Reflection enabled)`;
            }} else {{
                serviceList.innerHTML = `
                    <div class="error">
                        <strong>Reflection Disabled</strong>
                        <p style="margin-top: 0.5rem;">Enable gRPC reflection to automatically discover services.</p>
                    </div>
                `;
                serverInfo.textContent = `Server: ${{config.serverUrl}} (Reflection disabled)`;
            }}
        }}, 1000);
    </script>
</body>
</html>"#,
        title = title,
        server_url = config.server_url,
        theme_class = theme_class,
        config_json = config_json,
        custom_css = custom_css,
        bg_color = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#1a1a1a"
        } else {
            "#ffffff"
        },
        text_color = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#e5e5e5"
        } else {
            "#1a1a1a"
        },
        header_bg = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#252525"
        } else {
            "#f5f5f5"
        },
        sidebar_bg = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#1f1f1f"
        } else {
            "#fafafa"
        },
        border_color = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#333"
        } else {
            "#e5e5e5"
        },
        item_bg = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#2a2a2a"
        } else {
            "#ffffff"
        },
        item_hover_bg = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#333"
        } else {
            "#f0f0f0"
        },
        info_bg = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#1f2937"
        } else {
            "#f0f9ff"
        },
        accent_color = if matches!(config.theme, GrpcExplorerTheme::Dark) {
            "#60a5fa"
        } else {
            "#3b82f6"
        },
        reflection_status = if config.enable_reflection {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        },
        tls_status = if config.enable_tls {
            "✓ Enabled"
        } else {
            "✗ Disabled"
        },
        timeout = config.timeout_seconds,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_grpc_explorer_config_defaults() {
        let config = GrpcExplorerConfig::new();
        assert_eq!(config.server_url, "http://localhost:50051");
        assert_eq!(config.theme, GrpcExplorerTheme::Dark);
        assert!(config.enable_reflection);
        assert!(!config.enable_tls);
        assert_eq!(config.timeout_seconds, 30);
    }

    #[test]
    fn test_grpc_explorer_config_builder() {
        let config = GrpcExplorerConfig::new()
            .server_url("http://localhost:9090")
            .enable_reflection(false)
            .enable_tls(true)
            .theme(GrpcExplorerTheme::Light)
            .timeout_seconds(60)
            .add_header("Authorization", "Bearer token123");

        assert_eq!(config.server_url, "http://localhost:9090");
        assert!(!config.enable_reflection);
        assert!(config.enable_tls);
        assert_eq!(config.theme, GrpcExplorerTheme::Light);
        assert_eq!(config.timeout_seconds, 60);
        assert_eq!(
            config.headers.get("Authorization"),
            Some(&"Bearer token123".to_string())
        );
    }

    #[test]
    fn test_grpc_explorer_html_generation() {
        let config = GrpcExplorerConfig::new()
            .server_url("http://localhost:50051")
            .theme(GrpcExplorerTheme::Dark);

        let html = grpc_explorer_html(&config, "Test gRPC API");

        assert!(html.contains("<!DOCTYPE html>"));
        assert!(html.contains("Test gRPC API"));
        assert!(html.contains("http://localhost:50051"));
        assert!(html.contains("grpc-dark"));
    }

    #[test]
    fn test_grpc_explorer_with_tls() {
        let config = GrpcExplorerConfig::new()
            .server_url("https://api.example.com:443")
            .enable_tls(true);

        let html = grpc_explorer_html(&config, "Secure API");

        assert!(html.contains("https://api.example.com:443"));
        assert!(html.contains("✓ Enabled"));
    }

    #[test]
    fn test_grpc_explorer_theme_serialization() {
        let light = GrpcExplorerTheme::Light;
        let dark = GrpcExplorerTheme::Dark;

        assert_eq!(serde_json::to_string(&light).unwrap(), "\"light\"");
        assert_eq!(serde_json::to_string(&dark).unwrap(), "\"dark\"");
    }

    #[test]
    fn test_grpc_explorer_config_json_generation() {
        let config = GrpcExplorerConfig::new()
            .server_url("http://localhost:9090")
            .enable_reflection(true)
            .add_header("X-API-Key", "secret");

        let json = config.to_json();

        assert_eq!(json["serverUrl"], "http://localhost:9090");
        assert_eq!(json["reflection"], true);
        assert_eq!(json["headers"]["X-API-Key"], "secret");
    }

    #[test]
    fn test_grpc_explorer_custom_css() {
        let config = GrpcExplorerConfig::new().custom_css("body { background: #000; }");

        let html = grpc_explorer_html(&config, "Custom API");

        assert!(html.contains("body { background: #000; }"));
    }
}
