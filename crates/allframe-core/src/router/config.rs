//! Configuration-driven protocol selection
//!
//! This module enables AllFrame's key differentiator: write handlers once,
//! expose them via multiple protocols through configuration alone.

use serde::{Deserialize, Serialize};

/// Router configuration with protocol selection
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RouterConfig {
    /// Server configuration
    pub server: ServerConfig,
}

/// Server configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ServerConfig {
    /// Enabled protocols
    pub protocols: Vec<String>,

    /// REST-specific configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub rest: Option<RestConfig>,

    /// GraphQL-specific configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub graphql: Option<GraphQLConfig>,

    /// gRPC-specific configuration
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub grpc: Option<GrpcConfig>,
}

/// REST protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RestConfig {
    /// Port to listen on
    #[serde(default = "default_rest_port")]
    pub port: u16,

    /// Path prefix for all REST endpoints
    #[serde(default = "default_rest_prefix")]
    pub path_prefix: String,
}

fn default_rest_port() -> u16 {
    8080
}

fn default_rest_prefix() -> String {
    "/api/v1".to_string()
}

/// GraphQL protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphQLConfig {
    /// Port to listen on
    #[serde(default = "default_graphql_port")]
    pub port: u16,

    /// GraphQL endpoint path
    #[serde(default = "default_graphql_path")]
    pub path: String,

    /// Enable GraphiQL playground
    #[serde(default)]
    pub playground: bool,
}

fn default_graphql_port() -> u16 {
    8081
}

fn default_graphql_path() -> String {
    "/graphql".to_string()
}

/// gRPC protocol configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GrpcConfig {
    /// Port to listen on
    #[serde(default = "default_grpc_port")]
    pub port: u16,

    /// Enable reflection API
    #[serde(default)]
    pub reflection: bool,
}

fn default_grpc_port() -> u16 {
    9090
}

impl RouterConfig {
    /// Parse configuration from TOML string
    pub fn from_toml(toml: &str) -> Result<Self, String> {
        toml::from_str(toml).map_err(|e| format!("Failed to parse config: {}", e))
    }

    /// Parse configuration from TOML file
    pub fn from_file(path: &str) -> Result<Self, String> {
        let contents =
            std::fs::read_to_string(path).map_err(|e| format!("Failed to read file: {}", e))?;
        Self::from_toml(&contents)
    }

    /// Get enabled protocols
    pub fn protocols(&self) -> &[String] {
        &self.server.protocols
    }

    /// Check if a protocol is enabled
    pub fn has_protocol(&self, protocol: &str) -> bool {
        self.server.protocols.contains(&protocol.to_string())
    }

    /// Get REST configuration
    pub fn rest(&self) -> Option<&RestConfig> {
        self.server.rest.as_ref()
    }

    /// Get GraphQL configuration
    pub fn graphql(&self) -> Option<&GraphQLConfig> {
        self.server.graphql.as_ref()
    }

    /// Get gRPC configuration
    pub fn grpc(&self) -> Option<&GrpcConfig> {
        self.server.grpc.as_ref()
    }
}

impl RestConfig {
    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the path prefix
    pub fn path_prefix(&self) -> &str {
        &self.path_prefix
    }
}

impl GraphQLConfig {
    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Get the path
    pub fn path(&self) -> &str {
        &self.path
    }

    /// Check if playground is enabled
    pub fn playground(&self) -> bool {
        self.playground
    }
}

impl GrpcConfig {
    /// Get the port
    pub fn port(&self) -> u16 {
        self.port
    }

    /// Check if reflection is enabled
    pub fn reflection(&self) -> bool {
        self.reflection
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_basic_config() {
        let toml = r#"
            [server]
            protocols = ["rest", "graphql"]
        "#;

        let config = RouterConfig::from_toml(toml).unwrap();
        assert_eq!(config.protocols().len(), 2);
        assert!(config.has_protocol("rest"));
        assert!(config.has_protocol("graphql"));
        assert!(!config.has_protocol("grpc"));
    }

    #[test]
    fn test_parse_full_config() {
        let toml = r#"
            [server]
            protocols = ["rest", "graphql", "grpc"]

            [server.rest]
            port = 8080
            path_prefix = "/api/v1"

            [server.graphql]
            port = 8081
            path = "/graphql"
            playground = true

            [server.grpc]
            port = 9090
            reflection = true
        "#;

        let config = RouterConfig::from_toml(toml).unwrap();

        // Check protocols
        assert_eq!(config.protocols().len(), 3);
        assert!(config.has_protocol("rest"));
        assert!(config.has_protocol("graphql"));
        assert!(config.has_protocol("grpc"));

        // Check REST config
        let rest = config.rest().unwrap();
        assert_eq!(rest.port(), 8080);
        assert_eq!(rest.path_prefix(), "/api/v1");

        // Check GraphQL config
        let graphql = config.graphql().unwrap();
        assert_eq!(graphql.port(), 8081);
        assert_eq!(graphql.path(), "/graphql");
        assert!(graphql.playground());

        // Check gRPC config
        let grpc = config.grpc().unwrap();
        assert_eq!(grpc.port(), 9090);
        assert!(grpc.reflection());
    }

    #[test]
    fn test_parse_minimal_config() {
        let toml = r#"
            [server]
            protocols = ["rest"]
        "#;

        let config = RouterConfig::from_toml(toml).unwrap();
        assert_eq!(config.protocols().len(), 1);
        assert!(config.has_protocol("rest"));
        assert!(config.rest().is_none()); // No explicit REST config
    }

    #[test]
    fn test_default_values() {
        let toml = r#"
            [server]
            protocols = ["rest", "graphql", "grpc"]

            [server.rest]
            [server.graphql]
            [server.grpc]
        "#;

        let config = RouterConfig::from_toml(toml).unwrap();

        // Check default REST values
        let rest = config.rest().unwrap();
        assert_eq!(rest.port(), 8080);
        assert_eq!(rest.path_prefix(), "/api/v1");

        // Check default GraphQL values
        let graphql = config.graphql().unwrap();
        assert_eq!(graphql.port(), 8081);
        assert_eq!(graphql.path(), "/graphql");
        assert!(!graphql.playground()); // Default is false

        // Check default gRPC values
        let grpc = config.grpc().unwrap();
        assert_eq!(grpc.port(), 9090);
        assert!(!grpc.reflection()); // Default is false
    }
}
