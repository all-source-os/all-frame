//! OpenTelemetry macros for automatic tracing instrumentation
//!
//! Provides the #[traced] macro that automatically instruments functions
//! with OpenTelemetry spans for distributed tracing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, Item, ItemFn, Result};

/// Configuration for the traced macro
struct TracedConfig {
    /// Custom span name (defaults to function name)
    name: Option<String>,
    /// Skip certain parameters from being recorded
    skip: Vec<String>,
    /// Record the return value
    ret: bool,
    /// Record errors
    err: bool,
    /// Span level (trace, debug, info, warn, error)
    level: String,
}

impl Default for TracedConfig {
    fn default() -> Self {
        Self {
            name: None,
            skip: Vec::new(),
            ret: false,
            err: true,
            level: "info".to_string(),
        }
    }
}

/// Parse traced macro attributes
fn parse_traced_attrs(attr: TokenStream) -> Result<TracedConfig> {
    let mut config = TracedConfig::default();

    if attr.is_empty() {
        return Ok(config);
    }

    // Parse key-value pairs: #[traced(name = "custom", skip(arg1), ret, err, level = "debug")]
    let attr_str = attr.to_string();
    for part in attr_str.split(',') {
        let part = part.trim();
        if part.is_empty() {
            continue;
        }

        if let Some((key, value)) = part.split_once('=') {
            let key = key.trim();
            let value = value.trim().trim_matches('"');
            match key {
                "name" => config.name = Some(value.to_string()),
                "level" => config.level = value.to_string(),
                _ => {}
            }
        } else if part == "ret" {
            config.ret = true;
        } else if part == "err" {
            config.err = true;
        } else if part.starts_with("skip(") && part.ends_with(')') {
            let skip_args = &part[5..part.len() - 1];
            config.skip = skip_args
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }
    }

    Ok(config)
}

/// Mark a function to be automatically traced
pub fn traced_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;
    let config = parse_traced_attrs(attr)?;

    match &item {
        Item::Fn(func) => {
            let traced_fn = instrument_function(func, &config)?;
            Ok(traced_fn)
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[traced] can only be applied to functions",
        )),
    }
}

fn instrument_function(func: &ItemFn, config: &TracedConfig) -> Result<TokenStream> {
    let func_name = &func.sig.ident;
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    // Use function name as span name if not specified
    let span_name = config
        .name
        .clone()
        .unwrap_or_else(|| func_name.to_string());

    // Build skip list
    let skip_args = if config.skip.is_empty() {
        quote! {}
    } else {
        let skips: Vec<_> = config.skip.iter().map(|s| {
            let ident = syn::Ident::new(s, proc_macro2::Span::call_site());
            quote! { #ident }
        }).collect();
        quote! { skip(#(#skips),*), }
    };

    // Build level
    let level = match config.level.as_str() {
        "trace" => quote! { tracing::Level::TRACE },
        "debug" => quote! { tracing::Level::DEBUG },
        "warn" => quote! { tracing::Level::WARN },
        "error" => quote! { tracing::Level::ERROR },
        _ => quote! { tracing::Level::INFO },
    };

    // Build ret/err options
    let ret_opt = if config.ret {
        quote! { ret, }
    } else {
        quote! {}
    };

    let err_opt = if config.err {
        quote! { err, }
    } else {
        quote! {}
    };

    // Generate the instrumented function
    Ok(quote! {
        #(#attrs)*
        #[tracing::instrument(
            name = #span_name,
            level = #level,
            #skip_args
            #ret_opt
            #err_opt
        )]
        #vis #sig #block
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = TracedConfig::default();
        assert!(config.name.is_none());
        assert!(config.skip.is_empty());
        assert!(!config.ret);
        assert!(config.err);
        assert_eq!(config.level, "info");
    }

    #[test]
    fn test_parse_empty_attrs() {
        let attr = TokenStream::new();
        let config = parse_traced_attrs(attr).unwrap();
        assert!(config.name.is_none());
    }
}
