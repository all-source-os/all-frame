//! Resilience-related macros for AllFrame.
//!
//! Provides attribute macros for retry, circuit breaker, and rate limiting.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, ItemFn, LitInt, LitStr};

/// Configuration parsed from `#[retry(...)]` attributes.
#[derive(Default)]
struct RetryConfig {
    max_retries: Option<u32>,
    initial_interval_ms: Option<u64>,
    max_interval_ms: Option<u64>,
    multiplier: Option<f64>,
}

/// Implementation of the `#[retry]` attribute macro.
///
/// **⚠️ DEPRECATED**: This macro uses the old architecture that violates Clean
/// Architecture principles. Consider migrating to the new resilience system for
/// better testability and maintainability.
///
/// For migration guidance, see: https://docs.allframe.rs/guides/MIGRATION_GUIDE.html
///
/// Wraps an async function with retry logic using the new Clean Architecture
/// approach. The macro now uses ResilienceOrchestrator internally while
/// maintaining backward compatibility.
pub fn retry_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let config = parse_retry_attr(attr)?;
    let func: ItemFn = parse2(item)?;

    let visibility = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    // Build retry policy configuration
    let max_retries = config.max_retries.unwrap_or(3);
    let initial_interval_ms = config.initial_interval_ms.unwrap_or(500);
    let max_interval_ms = config.max_interval_ms.unwrap_or(30000);
    let multiplier = config.multiplier.unwrap_or(2.0);

    // Generate deprecation warning
    let deprecation_warning = quote! {
        #[deprecated(
            since = "0.1.13",
            note = "The #[retry] macro uses legacy architecture. Consider migrating to the new Clean Architecture resilience system. See: https://docs.allframe.rs/guides/MIGRATION_GUIDE.html"
        )]
    };

    Ok(quote! {
        #deprecation_warning
        #(#attrs)*
        #visibility #sig {
            // Import the new architectural components
            #[cfg(feature = "resilience")]
            {
                use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
                use allframe_core::domain::resilience::{ResiliencePolicy, BackoffStrategy};

                // Create orchestrator (lazy initialization for performance)
                static ORCHESTRATOR: std::sync::OnceLock<std::sync::Arc<dyn ResilienceOrchestrator + Send + Sync>> = std::sync::OnceLock::new();
                let orchestrator = ORCHESTRATOR.get_or_init(|| {
                    std::sync::Arc::new(DefaultResilienceOrchestrator::new())
                });

                // Build the retry policy using new architecture
                let policy = ResiliencePolicy::Retry {
                    max_attempts: #max_retries,
                    backoff: BackoffStrategy::Exponential {
                        initial_delay: std::time::Duration::from_millis(#initial_interval_ms),
                        multiplier: #multiplier,
                        max_delay: Some(std::time::Duration::from_millis(#max_interval_ms)),
                        jitter: true,
                    },
                };

                // Execute with new architecture
                // The orchestration returns the same Result type as the original function
                // This maintains backward compatibility
                match orchestrator.execute_with_policy(policy, || async {
                    #block
                }).await {
                    Ok(result) => result,
                    Err(_) => {
                        // For backward compatibility, panic on orchestration errors
                        // The old macro would typically let the operation's own errors bubble up
                        panic!("Resilience orchestration failed");
                    }
                }
            }

            // Fallback for when resilience features are not enabled
            #[cfg(not(feature = "resilience"))]
            {
                compile_error!("The #[retry] macro requires the 'resilience' feature. Enable it in Cargo.toml or migrate to the new Clean Architecture approach.");
            }
        }
    })
}

fn parse_retry_attr(attr: TokenStream) -> syn::Result<RetryConfig> {
    let mut config = RetryConfig::default();

    if attr.is_empty() {
        return Ok(config);
    }

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("max_retries") {
            let value: LitInt = meta.value()?.parse()?;
            config.max_retries = Some(value.base10_parse()?);
        } else if meta.path.is_ident("initial_interval_ms") {
            let value: LitInt = meta.value()?.parse()?;
            config.initial_interval_ms = Some(value.base10_parse()?);
        } else if meta.path.is_ident("max_interval_ms") {
            let value: LitInt = meta.value()?.parse()?;
            config.max_interval_ms = Some(value.base10_parse()?);
        } else if meta.path.is_ident("multiplier") {
            let value: syn::LitFloat = meta.value()?.parse()?;
            config.multiplier = Some(value.base10_parse()?);
        }
        Ok(())
    });

    parse2::<syn::parse::Nothing>(attr.clone())
        .ok()
        .map(|_| ())
        .or_else(|| syn::parse::Parser::parse2(parser, attr).ok());

    Ok(config)
}

/// Configuration parsed from `#[circuit_breaker(...)]` attributes.
#[derive(Default)]
struct CircuitBreakerConfig {
    name: Option<String>,
    failure_threshold: Option<u32>,
    success_threshold: Option<u32>,
    timeout_ms: Option<u64>,
}

/// Implementation of the `#[circuit_breaker]` attribute macro.
///
/// Wraps a function with circuit breaker logic.
pub fn circuit_breaker_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let config = parse_circuit_breaker_attr(attr)?;
    let func: ItemFn = parse2(item)?;

    let visibility = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    let failure_threshold = config.failure_threshold.unwrap_or(5);
    let success_threshold = config.success_threshold.unwrap_or(3);
    let timeout_ms = config.timeout_ms.unwrap_or(30000);

    // Generate deprecation warning for circuit breaker macro
    let deprecation_warning = quote! {
        #[deprecated(
            since = "0.1.13",
            note = "The #[circuit_breaker] macro uses legacy architecture. Consider migrating to the new Clean Architecture resilience system. See: https://docs.allframe.rs/guides/MIGRATION_GUIDE.html"
        )]
    };

    Ok(quote! {
        #deprecation_warning
        #(#attrs)*
        #visibility #sig {
            // Import the new architectural components
            #[cfg(feature = "resilience")]
            {
                use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
                use allframe_core::domain::resilience::ResiliencePolicy;
                use std::time::Duration;

                // Create orchestrator (lazy initialization for performance)
                static ORCHESTRATOR: std::sync::OnceLock<std::sync::Arc<dyn ResilienceOrchestrator + Send + Sync>> = std::sync::OnceLock::new();
                let orchestrator = ORCHESTRATOR.get_or_init(|| {
                    std::sync::Arc::new(DefaultResilienceOrchestrator::new())
                });

                // Build the circuit breaker policy using new architecture
                let policy = ResiliencePolicy::CircuitBreaker {
                    failure_threshold: #failure_threshold,
                    recovery_timeout: Duration::from_millis(#timeout_ms),
                    success_threshold: #success_threshold,
                };

                // Execute with new architecture
                match orchestrator.execute_with_policy(policy, || async {
                    #block
                }).await {
                    Ok(result) => result,
                    Err(_) => {
                        // For backward compatibility, panic on circuit breaker errors
                        panic!("Circuit breaker error in legacy macro");
                    }
                }
            }

            // Fallback for when resilience features are not enabled
            #[cfg(not(feature = "resilience"))]
            {
                compile_error!("The #[circuit_breaker] macro requires the 'resilience' feature. Enable it in Cargo.toml or migrate to the new Clean Architecture approach.");
            }
        }
    })
}

fn parse_circuit_breaker_attr(attr: TokenStream) -> syn::Result<CircuitBreakerConfig> {
    let mut config = CircuitBreakerConfig::default();

    if attr.is_empty() {
        return Ok(config);
    }

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("name") {
            let value: LitStr = meta.value()?.parse()?;
            config.name = Some(value.value());
        } else if meta.path.is_ident("failure_threshold") {
            let value: LitInt = meta.value()?.parse()?;
            config.failure_threshold = Some(value.base10_parse()?);
        } else if meta.path.is_ident("success_threshold") {
            let value: LitInt = meta.value()?.parse()?;
            config.success_threshold = Some(value.base10_parse()?);
        } else if meta.path.is_ident("timeout_ms") {
            let value: LitInt = meta.value()?.parse()?;
            config.timeout_ms = Some(value.base10_parse()?);
        }
        Ok(())
    });

    parse2::<syn::parse::Nothing>(attr.clone())
        .ok()
        .map(|_| ())
        .or_else(|| syn::parse::Parser::parse2(parser, attr).ok());

    Ok(config)
}

/// Configuration parsed from `#[rate_limited(...)]` attributes.
#[derive(Default)]
struct RateLimitConfig {
    rps: Option<u32>,
    burst: Option<u32>,
}

/// Implementation of the `#[rate_limited]` attribute macro.
///
/// Wraps a function with rate limiting logic.
pub fn rate_limited_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let config = parse_rate_limit_attr(attr)?;
    let func: ItemFn = parse2(item)?;

    let visibility = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    let rps = config.rps.unwrap_or(100);
    let burst = config.burst.unwrap_or(10);

    // Generate deprecation warning for rate limiting macro
    let deprecation_warning = quote! {
        #[deprecated(
            since = "0.1.13",
            note = "The #[rate_limited] macro uses legacy architecture. Consider migrating to the new Clean Architecture resilience system. See: https://docs.allframe.rs/guides/MIGRATION_GUIDE.html"
        )]
    };

    Ok(quote! {
        #deprecation_warning
        #(#attrs)*
        #visibility #sig {
            // Import the new architectural components
            #[cfg(feature = "resilience")]
            {
                use allframe_core::application::resilience::{ResilienceOrchestrator, DefaultResilienceOrchestrator};
                use allframe_core::domain::resilience::ResiliencePolicy;

                // Create orchestrator (lazy initialization for performance)
                static ORCHESTRATOR: std::sync::OnceLock<std::sync::Arc<dyn ResilienceOrchestrator + Send + Sync>> = std::sync::OnceLock::new();
                let orchestrator = ORCHESTRATOR.get_or_init(|| {
                    std::sync::Arc::new(DefaultResilienceOrchestrator::new())
                });

                // Build the rate limiting policy using new architecture
                let policy = ResiliencePolicy::RateLimit {
                    requests_per_second: #rps,
                    burst_capacity: #burst,
                };

                // Execute with new architecture
                match orchestrator.execute_with_policy(policy, || async {
                    #block
                }).await {
                    Ok(result) => result,
                    Err(_) => {
                        // For backward compatibility, panic on rate limiting errors
                        panic!("Rate limiting error in legacy macro");
                    }
                }
            }

            // Fallback for when resilience features are not enabled
            #[cfg(not(feature = "resilience"))]
            {
                compile_error!("The #[rate_limited] macro requires the 'resilience' feature. Enable it in Cargo.toml or migrate to the new Clean Architecture approach.");
            }
        }
    })
}

fn parse_rate_limit_attr(attr: TokenStream) -> syn::Result<RateLimitConfig> {
    let mut config = RateLimitConfig::default();

    if attr.is_empty() {
        return Ok(config);
    }

    let parser = syn::meta::parser(|meta| {
        if meta.path.is_ident("rps") {
            let value: LitInt = meta.value()?.parse()?;
            config.rps = Some(value.base10_parse()?);
        } else if meta.path.is_ident("burst") {
            let value: LitInt = meta.value()?.parse()?;
            config.burst = Some(value.base10_parse()?);
        }
        Ok(())
    });

    parse2::<syn::parse::Nothing>(attr.clone())
        .ok()
        .map(|_| ())
        .or_else(|| syn::parse::Parser::parse2(parser, attr).ok());

    Ok(config)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_retry_impl_basic() {
        let attr = TokenStream::new();
        let item = quote! {
            async fn fetch_data() -> Result<String, std::io::Error> {
                Ok("data".to_string())
            }
        };

        let result = retry_impl(attr, item);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("RetryExecutor"));
        assert!(output.contains("RetryConfig"));
    }

    #[test]
    fn test_circuit_breaker_impl_basic() {
        let attr = TokenStream::new();
        let item = quote! {
            async fn call_api() -> Result<String, std::io::Error> {
                Ok("response".to_string())
            }
        };

        let result = circuit_breaker_impl(attr, item);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("CircuitBreaker"));
    }

    #[test]
    fn test_rate_limited_impl_basic() {
        let attr = TokenStream::new();
        let item = quote! {
            fn handle_request() -> Result<(), std::io::Error> {
                Ok(())
            }
        };

        let result = rate_limited_impl(attr, item);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("RateLimiter"));
    }
}
