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
/// Wraps an async function with retry logic using `RetryExecutor`.
pub fn retry_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let config = parse_retry_attr(attr)?;
    let func: ItemFn = parse2(item)?;

    let func_name = &func.sig.ident;
    let func_name_str = func_name.to_string();
    let visibility = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    // Build RetryConfig
    let max_retries = config.max_retries.unwrap_or(3);
    let initial_interval_ms = config.initial_interval_ms.unwrap_or(500);
    let max_interval_ms = config.max_interval_ms.unwrap_or(30000);
    let multiplier = config.multiplier.unwrap_or(2.0);

    Ok(quote! {
        #(#attrs)*
        #visibility #sig {
            use allframe_core::resilience::{RetryExecutor, RetryConfig, RetryError};
            use std::time::Duration;

            let __retry_config = RetryConfig::new(#max_retries)
                .with_initial_interval(Duration::from_millis(#initial_interval_ms))
                .with_max_interval(Duration::from_millis(#max_interval_ms))
                .with_multiplier(#multiplier);

            let __executor = RetryExecutor::new(__retry_config);

            __executor.execute(#func_name_str, || async {
                #block
            }).await.map_err(|e| e.last_error)
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

    let func_name = &func.sig.ident;
    let func_name_str = config.name.unwrap_or_else(|| func_name.to_string());
    let visibility = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    let failure_threshold = config.failure_threshold.unwrap_or(5);
    let success_threshold = config.success_threshold.unwrap_or(3);
    let timeout_ms = config.timeout_ms.unwrap_or(30000);

    Ok(quote! {
        #(#attrs)*
        #visibility #sig {
            use allframe_core::resilience::{
                CircuitBreaker, CircuitBreakerConfig, CircuitBreakerError
            };
            use std::time::Duration;
            use std::sync::OnceLock;

            static __CIRCUIT_BREAKER: OnceLock<CircuitBreaker> = OnceLock::new();

            let __cb = __CIRCUIT_BREAKER.get_or_init(|| {
                let config = CircuitBreakerConfig::new(#failure_threshold)
                    .with_success_threshold(#success_threshold)
                    .with_timeout(Duration::from_millis(#timeout_ms));
                CircuitBreaker::new(#func_name_str, config)
            });

            __cb.call(|| async {
                #block
            }).await.map_err(|e| match e {
                CircuitBreakerError::CircuitOpen(err) => {
                    std::io::Error::new(std::io::ErrorKind::Other, err.to_string())
                }
                CircuitBreakerError::Inner(err) => err,
            })
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

    Ok(quote! {
        #(#attrs)*
        #visibility #sig {
            use allframe_core::resilience::{RateLimiter, RateLimitError};
            use std::sync::OnceLock;

            static __RATE_LIMITER: OnceLock<RateLimiter> = OnceLock::new();

            let __limiter = __RATE_LIMITER.get_or_init(|| {
                RateLimiter::new(#rps, #burst)
            });

            __limiter.check().map_err(|e| {
                std::io::Error::new(std::io::ErrorKind::Other, e.to_string())
            })?;

            #block
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
