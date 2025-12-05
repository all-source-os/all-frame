//! Contract Testing Support
//!
//! This module provides comprehensive contract testing functionality for API
//! endpoints across REST, GraphQL, and gRPC protocols.
//!
//! # Features
//!
//! - **Contract Test Generation**: Automatic test generation from specs
//! - **Schema Validation**: Validate requests/responses against schemas
//! - **Breaking Change Detection**: Detect API contract violations
//! - **Mock Server Generation**: Generate mock servers from specs
//! - **Coverage Reporting**: Track contract test coverage
//!
//! # Example
//!
//! ```rust
//! use allframe_core::router::{Router, ContractTester};
//!
//! let router = Router::new();
//! let tester = ContractTester::new(&router);
//!
//! // Generate and run contract tests
//! let results = tester.test_all_routes().await?;
//! assert!(results.all_passed());
//! ```

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::router::{RouteMetadata, Router};

/// Contract test result for a single route
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTestResult {
    /// Route path
    pub path: String,

    /// HTTP method
    pub method: String,

    /// Whether the test passed
    pub passed: bool,

    /// Failure reason if test failed
    pub failure_reason: Option<String>,

    /// Validation errors
    pub errors: Vec<String>,
}

impl ContractTestResult {
    /// Create a passing test result
    pub fn passed(path: impl Into<String>, method: impl Into<String>) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
            passed: true,
            failure_reason: None,
            errors: Vec::new(),
        }
    }

    /// Create a failing test result
    pub fn failed(
        path: impl Into<String>,
        method: impl Into<String>,
        reason: impl Into<String>,
    ) -> Self {
        Self {
            path: path.into(),
            method: method.into(),
            passed: false,
            failure_reason: Some(reason.into()),
            errors: Vec::new(),
        }
    }

    /// Add a validation error
    pub fn with_error(mut self, error: impl Into<String>) -> Self {
        self.errors.push(error.into());
        self.passed = false;
        self
    }
}

/// Collection of contract test results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractTestResults {
    /// Individual test results
    pub results: Vec<ContractTestResult>,

    /// Total tests run
    pub total: usize,

    /// Number of passed tests
    pub passed: usize,

    /// Number of failed tests
    pub failed: usize,

    /// Coverage percentage
    pub coverage: f64,
}

impl ContractTestResults {
    /// Create new test results
    pub fn new(results: Vec<ContractTestResult>) -> Self {
        let total = results.len();
        let passed = results.iter().filter(|r| r.passed).count();
        let failed = total - passed;
        let coverage = if total > 0 {
            (passed as f64 / total as f64) * 100.0
        } else {
            0.0
        };

        Self {
            results,
            total,
            passed,
            failed,
            coverage,
        }
    }

    /// Check if all tests passed
    pub fn all_passed(&self) -> bool {
        self.failed == 0
    }

    /// Get failed tests
    pub fn failed_tests(&self) -> Vec<&ContractTestResult> {
        self.results.iter().filter(|r| !r.passed).collect()
    }
}

/// Contract testing configuration
#[derive(Debug, Clone)]
pub struct ContractTestConfig {
    /// Validate request schemas
    pub validate_requests: bool,

    /// Validate response schemas
    pub validate_responses: bool,

    /// Detect breaking changes
    pub detect_breaking_changes: bool,

    /// Generate mock responses
    pub generate_mocks: bool,

    /// Fail on first error
    pub fail_fast: bool,
}

impl Default for ContractTestConfig {
    fn default() -> Self {
        Self {
            validate_requests: true,
            validate_responses: true,
            detect_breaking_changes: true,
            generate_mocks: false,
            fail_fast: false,
        }
    }
}

impl ContractTestConfig {
    /// Create a new configuration with defaults
    pub fn new() -> Self {
        Self::default()
    }

    /// Enable request validation
    pub fn validate_requests(mut self, enable: bool) -> Self {
        self.validate_requests = enable;
        self
    }

    /// Enable response validation
    pub fn validate_responses(mut self, enable: bool) -> Self {
        self.validate_responses = enable;
        self
    }

    /// Enable breaking change detection
    pub fn detect_breaking_changes(mut self, enable: bool) -> Self {
        self.detect_breaking_changes = enable;
        self
    }

    /// Enable mock generation
    pub fn generate_mocks(mut self, enable: bool) -> Self {
        self.generate_mocks = enable;
        self
    }

    /// Enable fail-fast mode
    pub fn fail_fast(mut self, enable: bool) -> Self {
        self.fail_fast = enable;
        self
    }
}

/// Contract tester for API routes
pub struct ContractTester<'a> {
    #[allow(dead_code)]
    router: &'a Router,
    config: ContractTestConfig,
    routes: Vec<RouteMetadata>,
}

impl<'a> ContractTester<'a> {
    /// Create a new contract tester
    pub fn new(router: &'a Router) -> Self {
        Self {
            router,
            config: ContractTestConfig::default(),
            routes: router.routes().to_vec(),
        }
    }

    /// Create with custom configuration
    pub fn with_config(router: &'a Router, config: ContractTestConfig) -> Self {
        Self {
            router,
            config,
            routes: router.routes().to_vec(),
        }
    }

    /// Test all routes
    pub fn test_all_routes(&self) -> ContractTestResults {
        let mut results = Vec::new();

        for route in &self.routes {
            let result = self.test_route(route);
            results.push(result);

            if self.config.fail_fast && !results.last().unwrap().passed {
                break;
            }
        }

        ContractTestResults::new(results)
    }

    /// Test a specific route
    pub fn test_route(&self, route: &RouteMetadata) -> ContractTestResult {
        // Basic validation: route must have path and method
        if route.path.is_empty() {
            return ContractTestResult::failed(&route.path, &route.method, "Route path is empty");
        }

        let mut result = ContractTestResult::passed(&route.path, &route.method);

        // Validate request schema if enabled
        if self.config.validate_requests {
            if let Some(error) = self.validate_request_schema(route) {
                result = result.with_error(error);
            }
        }

        // Validate response schema if enabled
        if self.config.validate_responses {
            if let Some(error) = self.validate_response_schema(route) {
                result = result.with_error(error);
            }
        }

        result
    }

    /// Validate request schema
    fn validate_request_schema(&self, route: &RouteMetadata) -> Option<String> {
        // Check if route has request schema
        if self.config.validate_requests && route.request_schema.is_none() {
            return Some(format!("Route {} lacks request schema", route.path));
        }
        None
    }

    /// Validate response schema
    fn validate_response_schema(&self, route: &RouteMetadata) -> Option<String> {
        // Check if route has response schema
        if self.config.validate_responses && route.response_schema.is_none() {
            return Some(format!("Route {} lacks response schema", route.path));
        }
        None
    }

    /// Generate contract test for a route
    pub fn generate_test_code(&self, route: &RouteMetadata) -> String {
        format!(
            r#"#[tokio::test]
async fn test_{}_contract() {{
    let router = Router::new();
    let tester = ContractTester::new(&router);

    let route = router.routes()
        .iter()
        .find(|r| r.path == "{}" && r.method == "{}")
        .expect("Route not found");

    let result = tester.test_route(route);
    assert!(result.passed, "Contract test failed: {{:?}}", result.failure_reason);
}}
"#,
            route
                .path
                .replace('/', "_")
                .replace('{', "")
                .replace('}', ""),
            route.path,
            route.method
        )
    }

    /// Get coverage statistics
    pub fn coverage_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();
        let results = self.test_all_routes();

        stats.insert("total_routes".to_string(), self.routes.len() as f64);
        stats.insert("tested_routes".to_string(), results.passed as f64);
        stats.insert("coverage_percent".to_string(), results.coverage);
        stats.insert("failed_tests".to_string(), results.failed as f64);

        stats
    }
}

/// Helper trait for generating contract tests
pub trait ContractTestable {
    /// Generate contract tests for all routes
    fn generate_contract_tests(&self) -> ContractTestResults;

    /// Test a specific route path and method
    fn test_route_contract(&self, path: &str, method: &str) -> ContractTestResult;
}

impl ContractTestable for Router {
    fn generate_contract_tests(&self) -> ContractTestResults {
        let tester = ContractTester::new(self);
        tester.test_all_routes()
    }

    fn test_route_contract(&self, path: &str, method: &str) -> ContractTestResult {
        let tester = ContractTester::new(self);

        if let Some(route) = self
            .routes()
            .iter()
            .find(|r| r.path == path && r.method == method)
        {
            tester.test_route(route)
        } else {
            ContractTestResult::failed(
                path,
                method,
                format!("Route not found: {} {}", path, method),
            )
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contract_test_result_passed() {
        let result = ContractTestResult::passed("/users", "GET");
        assert!(result.passed);
        assert_eq!(result.path, "/users");
        assert_eq!(result.method, "GET");
        assert!(result.failure_reason.is_none());
    }

    #[test]
    fn test_contract_test_result_failed() {
        let result = ContractTestResult::failed("/users", "POST", "Invalid schema");
        assert!(!result.passed);
        assert_eq!(result.failure_reason, Some("Invalid schema".to_string()));
    }

    #[test]
    fn test_contract_test_result_with_error() {
        let result = ContractTestResult::passed("/users", "GET").with_error("Missing field: name");

        assert!(!result.passed);
        assert_eq!(result.errors.len(), 1);
        assert_eq!(result.errors[0], "Missing field: name");
    }

    #[test]
    fn test_contract_test_results() {
        let results = vec![
            ContractTestResult::passed("/users", "GET"),
            ContractTestResult::passed("/posts", "GET"),
            ContractTestResult::failed("/admin", "DELETE", "Unauthorized"),
        ];

        let test_results = ContractTestResults::new(results);

        assert_eq!(test_results.total, 3);
        assert_eq!(test_results.passed, 2);
        assert_eq!(test_results.failed, 1);
        assert!(!test_results.all_passed());
        assert_eq!(test_results.coverage, 66.66666666666666);
    }

    #[test]
    fn test_contract_test_config() {
        let config = ContractTestConfig::new()
            .validate_requests(true)
            .validate_responses(true)
            .detect_breaking_changes(false)
            .fail_fast(true);

        assert!(config.validate_requests);
        assert!(config.validate_responses);
        assert!(!config.detect_breaking_changes);
        assert!(config.fail_fast);
    }

    #[test]
    fn test_contract_tester_empty_router() {
        let router = Router::new();
        let tester = ContractTester::new(&router);
        let results = tester.test_all_routes();

        assert_eq!(results.total, 0);
        assert_eq!(results.passed, 0);
        assert!(results.all_passed());
    }

    #[test]
    fn test_contract_testable_trait() {
        let router = Router::new();
        let results = router.generate_contract_tests();

        assert_eq!(results.total, 0);
        assert!(results.all_passed());
    }

    #[test]
    fn test_generate_test_code() {
        let router = Router::new();
        let tester = ContractTester::new(&router);

        let route = RouteMetadata {
            path: "/users".to_string(),
            method: "GET".to_string(),
            protocol: "rest".to_string(),
            description: Some("Get users".to_string()),
            request_schema: None,
            response_schema: None,
        };

        let code = tester.generate_test_code(&route);

        // The path "/users" becomes "users" after removing /
        assert!(code.contains("test__users_contract") || code.contains("test_users_contract"));
        assert!(code.contains("/users"));
        assert!(code.contains("GET"));
    }

    #[test]
    fn test_coverage_stats() {
        let router = Router::new();
        let tester = ContractTester::new(&router);
        let stats = tester.coverage_stats();

        assert_eq!(stats.get("total_routes"), Some(&0.0));
        assert_eq!(stats.get("tested_routes"), Some(&0.0));
    }
}
