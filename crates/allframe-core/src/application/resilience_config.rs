//! Configuration layer for resilience policies.
//!
//! This module provides TOML/YAML-based configuration for resilience policies,
//! allowing runtime configuration of resilience behavior without code changes.
//!
//! # Example TOML Configuration
//!
//! ```toml
//! [resilience]
//! enabled = true
//!
//! [resilience.policies]
//! default = { retry = { max_attempts = 3 } }
//! database = { retry = { max_attempts = 5 }, circuit_breaker = { failure_threshold = 10 } }
//! external_api = { retry = { max_attempts = 2 }, timeout = { duration_seconds = 30 } }
//!
//! [resilience.services]
//! payment_service = "database"
//! email_service = "external_api"
//! cache_service = "default"
//! ```
//!
//! # Example YAML Configuration
//!
//! ```yaml
//! resilience:
//!   enabled: true
//!   policies:
//!     default:
//!       retry:
//!         max_attempts: 3
//!     database:
//!       retry:
//!         max_attempts: 5
//!       circuit_breaker:
//!         failure_threshold: 10
//!   services:
//!     payment_service: database
//!     email_service: external_api
//! ```

use std::{collections::HashMap, time::Duration};

use serde::{Deserialize, Serialize};

use crate::domain::resilience::{BackoffStrategy, ResiliencePolicy};

/// Top-level resilience configuration
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct ResilienceConfig {
    /// Whether resilience is enabled globally
    #[serde(default = "default_enabled")]
    pub enabled: bool,

    /// Named policy configurations
    #[serde(default)]
    pub policies: HashMap<String, PolicyConfig>,

    /// Service-to-policy mappings
    #[serde(default)]
    pub services: HashMap<String, String>,
}

fn default_enabled() -> bool {
    true
}

/// Configuration for a single resilience policy
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum PolicyConfig {
    /// Simple policy (single resilience mechanism)
    Simple(SimplePolicyConfig),

    /// Combined policy (multiple resilience mechanisms)
    Combined {
        /// Retry configuration
        retry: Option<RetryConfig>,

        /// Circuit breaker configuration
        circuit_breaker: Option<CircuitBreakerConfig>,

        /// Rate limiting configuration
        rate_limit: Option<RateLimitConfig>,

        /// Timeout configuration
        timeout: Option<TimeoutConfig>,
    },
}

/// Simple policy configuration for single resilience mechanisms
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum SimplePolicyConfig {
    /// No resilience
    None,

    /// Retry configuration
    Retry(RetryConfig),

    /// Circuit breaker configuration
    CircuitBreaker(CircuitBreakerConfig),

    /// Rate limiting configuration
    RateLimit(RateLimitConfig),

    /// Timeout configuration
    Timeout(TimeoutConfig),
}

/// Retry policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RetryConfig {
    /// Maximum number of retry attempts
    pub max_attempts: u32,

    /// Backoff strategy configuration
    #[serde(flatten)]
    pub backoff: BackoffConfig,
}

/// Backoff strategy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "backoff_type", rename_all = "snake_case")]
pub enum BackoffConfig {
    /// Fixed delay between attempts
    Fixed {
        /// Delay in milliseconds
        delay_ms: u64,
    },

    /// Exponential backoff
    Exponential {
        /// Initial delay in milliseconds
        initial_delay_ms: u64,

        /// Backoff multiplier
        #[serde(default = "default_multiplier")]
        multiplier: f64,

        /// Maximum delay in milliseconds (optional)
        max_delay_ms: Option<u64>,

        /// Whether to add jitter
        #[serde(default = "default_jitter")]
        jitter: bool,
    },

    /// Linear backoff
    Linear {
        /// Initial delay in milliseconds
        initial_delay_ms: u64,

        /// Increment in milliseconds per attempt
        increment_ms: u64,

        /// Maximum delay in milliseconds (optional)
        max_delay_ms: Option<u64>,
    },
}

fn default_multiplier() -> f64 {
    2.0
}

fn default_jitter() -> bool {
    true
}

/// Circuit breaker policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct CircuitBreakerConfig {
    /// Number of failures before opening the circuit
    pub failure_threshold: u32,

    /// Recovery timeout in seconds
    pub recovery_timeout_seconds: u64,

    /// Number of successes required to close the circuit
    #[serde(default = "default_success_threshold")]
    pub success_threshold: u32,
}

fn default_success_threshold() -> u32 {
    3
}

/// Rate limiting policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// Maximum requests per second
    pub requests_per_second: u32,

    /// Burst capacity (additional requests allowed)
    #[serde(default = "default_burst_capacity")]
    pub burst_capacity: u32,
}

fn default_burst_capacity() -> u32 {
    10
}

/// Timeout policy configuration
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct TimeoutConfig {
    /// Timeout duration in seconds
    pub duration_seconds: u64,
}

impl ResilienceConfig {
    /// Load configuration from a TOML string
    pub fn from_toml(content: &str) -> Result<Self, ResilienceConfigError> {
        toml::from_str(content).map_err(|e| ResilienceConfigError::Toml(e.to_string()))
    }

    /// Load configuration from a YAML string
    pub fn from_yaml(_content: &str) -> Result<Self, ResilienceConfigError> {
        // YAML support is not currently available - serde_yaml is not a dependency
        Err(ResilienceConfigError::Yaml(
            "YAML support not available".to_string(),
        ))
    }

    /// Load configuration from a file (auto-detects TOML vs YAML based on
    /// extension)
    pub fn from_file(path: &std::path::Path) -> Result<Self, ResilienceConfigError> {
        let content =
            std::fs::read_to_string(path).map_err(|e| ResilienceConfigError::Io(e.to_string()))?;

        match path.extension().and_then(|s| s.to_str()) {
            Some("toml") => Self::from_toml(&content),
            Some("yaml") | Some("yml") => Self::from_yaml(&content),
            _ => Err(ResilienceConfigError::UnsupportedFormat(
                path.display().to_string(),
            )),
        }
    }

    /// Get the policy for a specific service
    pub fn get_policy_for_service(&self, service_name: &str) -> Option<ResiliencePolicy> {
        if !self.enabled {
            return Some(ResiliencePolicy::None);
        }

        // Check if service has a specific policy mapping
        let policy_name = self.services.get(service_name)?;

        // Get the policy configuration
        let policy_config = self.policies.get(policy_name)?;

        Some(policy_config.to_policy())
    }

    /// Get a named policy directly
    pub fn get_policy(&self, policy_name: &str) -> Option<ResiliencePolicy> {
        if !self.enabled {
            return Some(ResiliencePolicy::None);
        }

        self.policies
            .get(policy_name)
            .map(|config| config.to_policy())
    }

    /// Get the default policy (or None if no policies configured)
    pub fn get_default_policy(&self) -> ResiliencePolicy {
        if !self.enabled {
            return ResiliencePolicy::None;
        }

        self.policies
            .get("default")
            .map(|config| config.to_policy())
            .unwrap_or(ResiliencePolicy::None)
    }
}

impl PolicyConfig {
    /// Convert configuration to a resilience policy
    pub fn to_policy(&self) -> ResiliencePolicy {
        match self {
            PolicyConfig::Simple(simple) => match simple {
                SimplePolicyConfig::None => ResiliencePolicy::None,
                SimplePolicyConfig::Retry(config) => ResiliencePolicy::Retry {
                    max_attempts: config.max_attempts,
                    backoff: config.backoff.to_backoff_strategy(),
                },
                SimplePolicyConfig::CircuitBreaker(config) => ResiliencePolicy::CircuitBreaker {
                    failure_threshold: config.failure_threshold,
                    recovery_timeout: Duration::from_secs(config.recovery_timeout_seconds),
                    success_threshold: config.success_threshold,
                },
                SimplePolicyConfig::RateLimit(config) => ResiliencePolicy::RateLimit {
                    requests_per_second: config.requests_per_second,
                    burst_capacity: config.burst_capacity,
                },
                SimplePolicyConfig::Timeout(config) => ResiliencePolicy::Timeout {
                    duration: Duration::from_secs(config.duration_seconds),
                },
            },

            PolicyConfig::Combined {
                retry,
                circuit_breaker,
                rate_limit,
                timeout,
            } => {
                let mut policies = Vec::new();

                if let Some(config) = retry {
                    policies.push(ResiliencePolicy::Retry {
                        max_attempts: config.max_attempts,
                        backoff: config.backoff.to_backoff_strategy(),
                    });
                }

                if let Some(config) = circuit_breaker {
                    policies.push(ResiliencePolicy::CircuitBreaker {
                        failure_threshold: config.failure_threshold,
                        recovery_timeout: Duration::from_secs(config.recovery_timeout_seconds),
                        success_threshold: config.success_threshold,
                    });
                }

                if let Some(config) = rate_limit {
                    policies.push(ResiliencePolicy::RateLimit {
                        requests_per_second: config.requests_per_second,
                        burst_capacity: config.burst_capacity,
                    });
                }

                if let Some(config) = timeout {
                    policies.push(ResiliencePolicy::Timeout {
                        duration: Duration::from_secs(config.duration_seconds),
                    });
                }

                match policies.len() {
                    0 => ResiliencePolicy::None,
                    1 => policies.into_iter().next().unwrap(),
                    _ => ResiliencePolicy::Combined { policies },
                }
            }
        }
    }
}

impl BackoffConfig {
    /// Convert configuration to a backoff strategy
    pub fn to_backoff_strategy(&self) -> BackoffStrategy {
        match self {
            BackoffConfig::Fixed { delay_ms } => BackoffStrategy::Fixed {
                delay: Duration::from_millis(*delay_ms),
            },

            BackoffConfig::Exponential {
                initial_delay_ms,
                multiplier,
                max_delay_ms,
                jitter,
            } => BackoffStrategy::Exponential {
                initial_delay: Duration::from_millis(*initial_delay_ms),
                multiplier: *multiplier,
                max_delay: max_delay_ms.map(Duration::from_millis),
                jitter: *jitter,
            },

            BackoffConfig::Linear {
                initial_delay_ms,
                increment_ms,
                max_delay_ms,
            } => BackoffStrategy::Linear {
                initial_delay: Duration::from_millis(*initial_delay_ms),
                increment: Duration::from_millis(*increment_ms),
                max_delay: max_delay_ms.map(Duration::from_millis),
            },
        }
    }
}

/// Errors that can occur during configuration loading
#[derive(thiserror::Error, Debug)]
pub enum ResilienceConfigError {
    #[error("IO error: {0}")]
    Io(String),

    #[error("TOML parsing error: {0}")]
    Toml(String),

    #[error("YAML parsing error: {0}")]
    Yaml(String),

    #[error("Unsupported file format: {0}")]
    UnsupportedFormat(String),
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_simple_retry_policy_config() {
        let config = PolicyConfig::Simple(SimplePolicyConfig::Retry(RetryConfig {
            max_attempts: 3,
            backoff: BackoffConfig::Exponential {
                initial_delay_ms: 100,
                multiplier: 2.0,
                max_delay_ms: Some(10000),
                jitter: true,
            },
        }));

        let policy = config.to_policy();
        match policy {
            ResiliencePolicy::Retry {
                max_attempts,
                backoff: BackoffStrategy::Exponential { .. },
            } => {
                assert_eq!(max_attempts, 3);
            }
            _ => panic!("Expected retry policy"),
        }
    }

    #[test]
    fn test_combined_policy_config() {
        let config = PolicyConfig::Combined {
            retry: Some(RetryConfig {
                max_attempts: 3,
                backoff: BackoffConfig::Fixed { delay_ms: 1000 },
            }),
            circuit_breaker: Some(CircuitBreakerConfig {
                failure_threshold: 5,
                recovery_timeout_seconds: 30,
                success_threshold: 2,
            }),
            rate_limit: None,
            timeout: Some(TimeoutConfig {
                duration_seconds: 60,
            }),
        };

        let policy = config.to_policy();
        match policy {
            ResiliencePolicy::Combined { policies } => {
                assert_eq!(policies.len(), 3);
            }
            _ => panic!("Expected combined policy"),
        }
    }

    #[test]
    fn test_resilience_config_from_toml() {
        let toml_content = r#"
            enabled = true

            [policies.default]
            retry = { max_attempts = 3, backoff_type = "exponential", initial_delay_ms = 100 }

            [policies.database]
            retry = { max_attempts = 5, backoff_type = "fixed", delay_ms = 1000 }
            circuit_breaker = { failure_threshold = 10, recovery_timeout_seconds = 30 }

            [services]
            payment_service = "database"
            cache_service = "default"
        "#;

        let config = ResilienceConfig::from_toml(toml_content).unwrap();
        assert!(config.enabled);
        assert_eq!(config.policies.len(), 2);
        assert_eq!(config.services.len(), 2);

        // Test service policy resolution
        let payment_policy = config.get_policy_for_service("payment_service").unwrap();
        match payment_policy {
            ResiliencePolicy::Combined { policies } => {
                assert_eq!(policies.len(), 2); // retry + circuit breaker
            }
            _ => panic!("Expected combined policy for payment service"),
        }
    }

    #[test]
    fn test_backoff_config_conversion() {
        let fixed_config = BackoffConfig::Fixed { delay_ms: 500 };
        let fixed_strategy = fixed_config.to_backoff_strategy();
        match fixed_strategy {
            BackoffStrategy::Fixed { delay } => {
                assert_eq!(delay, Duration::from_millis(500));
            }
            _ => panic!("Expected fixed backoff"),
        }

        let exp_config = BackoffConfig::Exponential {
            initial_delay_ms: 100,
            multiplier: 2.0,
            max_delay_ms: Some(5000),
            jitter: true,
        };
        let exp_strategy = exp_config.to_backoff_strategy();
        match exp_strategy {
            BackoffStrategy::Exponential {
                initial_delay,
                multiplier,
                max_delay,
                jitter,
            } => {
                assert_eq!(initial_delay, Duration::from_millis(100));
                assert_eq!(multiplier, 2.0);
                assert_eq!(max_delay, Some(Duration::from_millis(5000)));
                assert!(jitter);
            }
            _ => panic!("Expected exponential backoff"),
        }
    }

    #[test]
    fn test_config_disabled_behavior() {
        let config = ResilienceConfig::from_toml("enabled = false").unwrap();
        assert!(!config.enabled);

        // All policies should return None when disabled
        assert_eq!(config.get_default_policy(), ResiliencePolicy::None);
        assert_eq!(config.get_policy("any"), Some(ResiliencePolicy::None));
        assert_eq!(
            config.get_policy_for_service("any"),
            Some(ResiliencePolicy::None)
        );
    }
}
