//! `#[allframe_handler]` attribute macro
//!
//! Marks a function as an AllFrame router handler. This serves two purposes:
//!
//! 1. **Suppresses dead_code warnings** — The Rust compiler cannot trace usage
//!    through the `router.register(name, handler_fn)` closure chain, so handler
//!    functions appear unused. This macro adds `#[allow(dead_code)]` to silence
//!    that false positive without hiding real dead code elsewhere.
//!
//! 2. **Validates the handler signature** — Both sync and async handlers are
//!    supported. In streaming mode, a `StreamSender` parameter must be present.
//!
//! # Usage
//!
//! ```rust,ignore
//! use allframe_macros::allframe_handler;
//!
//! #[allframe_handler]
//! async fn get_user(args: GetUserArgs) -> String {
//!     format!(r#"{{"name":"Alice"}}"#)
//! }
//!
//! #[allframe_handler]
//! fn compute_hash(args: HashArgs) -> String {
//!     format!(r#"{{"hash":"{}"}}"#, hash(&args.data))
//! }
//!
//! #[allframe_handler(streaming)]
//! async fn stream_data(tx: StreamSender) -> String {
//!     tx.send("chunk".to_string()).await.ok();
//!     "done".to_string()
//! }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, ItemFn, Meta, Result};

/// Returns `true` if the type path ends with `StreamSender`.
fn has_stream_sender_param(sig: &syn::Signature) -> bool {
    sig.inputs.iter().any(|arg| {
        if let syn::FnArg::Typed(pat_type) = arg {
            if let syn::Type::Path(type_path) = pat_type.ty.as_ref() {
                return type_path
                    .path
                    .segments
                    .last()
                    .is_some_and(|seg| seg.ident == "StreamSender");
            }
        }
        false
    })
}

/// Returns `true` if the attribute contains the `streaming` keyword.
fn is_streaming(attr: &TokenStream) -> bool {
    if attr.is_empty() {
        return false;
    }
    match parse2::<Meta>(attr.clone()) {
        Ok(Meta::Path(path)) => path
            .segments
            .last()
            .is_some_and(|seg| seg.ident == "streaming"),
        _ => false,
    }
}

pub fn allframe_handler_impl(attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let func: ItemFn = parse2(item)?;
    let streaming = is_streaming(&attr);

    // Validate: streaming handlers must be async (StreamSender requires .await)
    if streaming && func.sig.asyncness.is_none() {
        return Err(Error::new_spanned(
            func.sig.fn_token,
            "#[allframe_handler(streaming)] functions must be async",
        ));
    }

    // Validate: streaming handlers must have a StreamSender parameter
    if streaming && !has_stream_sender_param(&func.sig) {
        return Err(Error::new_spanned(
            &func.sig,
            "#[allframe_handler(streaming)] requires a `StreamSender` parameter",
        ));
    }

    // Validate: non-streaming handlers should not have StreamSender
    if !streaming && has_stream_sender_param(&func.sig) {
        return Err(Error::new_spanned(
            &func.sig,
            "handler has a `StreamSender` parameter but is not marked as streaming; \
             use #[allframe_handler(streaming)]",
        ));
    }

    Ok(quote! {
        #[allow(dead_code)]
        #func
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_basic_handler_adds_allow_dead_code() {
        let input = quote! {
            async fn get_user() -> String {
                "alice".to_string()
            }
        };
        let result = allframe_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(
            output.contains("allow (dead_code)") || output.contains("allow(dead_code)"),
            "should add #[allow(dead_code)], got: {output}"
        );
        assert!(output.contains("async fn get_user"));
    }

    #[test]
    fn test_handler_preserves_function_body() {
        let input = quote! {
            async fn echo() -> String {
                "hello".to_string()
            }
        };
        let result = allframe_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("hello"));
    }

    #[test]
    fn test_handler_preserves_visibility() {
        let input = quote! {
            pub async fn public_handler() -> String {
                "ok".to_string()
            }
        };
        let result = allframe_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("pub async fn public_handler"));
    }

    #[test]
    fn test_handler_preserves_args() {
        let input = quote! {
            async fn greet(args: GreetArgs) -> String {
                format!("hi {}", args.name)
            }
        };
        let result = allframe_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("args : GreetArgs"));
    }

    #[test]
    fn test_allows_sync_function() {
        let input = quote! {
            fn sync_handler() -> String {
                "ok".to_string()
            }
        };
        let result = allframe_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(
            output.contains("allow (dead_code)") || output.contains("allow(dead_code)"),
            "should add #[allow(dead_code)], got: {output}"
        );
        assert!(output.contains("fn sync_handler"));
    }

    #[test]
    fn test_streaming_rejects_sync_function() {
        let attr = quote! { streaming };
        let input = quote! {
            fn sync_stream(tx: StreamSender) -> String {
                "no".to_string()
            }
        };
        let result = allframe_handler_impl(attr, input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("must be async"),
            "expected 'must be async' error, got: {err}"
        );
    }

    #[test]
    fn test_streaming_handler_with_stream_sender() {
        let attr = quote! { streaming };
        let input = quote! {
            async fn stream_data(tx: StreamSender) -> String {
                "done".to_string()
            }
        };
        let result = allframe_handler_impl(attr, input).unwrap();
        let output = result.to_string();
        assert!(output.contains("allow (dead_code)") || output.contains("allow(dead_code)"));
        assert!(output.contains("async fn stream_data"));
    }

    #[test]
    fn test_streaming_without_stream_sender_fails() {
        let attr = quote! { streaming };
        let input = quote! {
            async fn bad_stream() -> String {
                "no sender".to_string()
            }
        };
        let result = allframe_handler_impl(attr, input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("StreamSender"),
            "expected StreamSender error, got: {err}"
        );
    }

    #[test]
    fn test_non_streaming_with_stream_sender_fails() {
        let input = quote! {
            async fn accidental_stream(tx: StreamSender) -> String {
                "oops".to_string()
            }
        };
        let result = allframe_handler_impl(TokenStream::new(), input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(
            err.contains("not marked as streaming"),
            "expected hint to use streaming attr, got: {err}"
        );
    }

    #[test]
    fn test_streaming_with_args_and_sender() {
        let attr = quote! { streaming };
        let input = quote! {
            async fn stream_greet(args: GreetArgs, tx: StreamSender) -> String {
                "done".to_string()
            }
        };
        let result = allframe_handler_impl(attr, input).unwrap();
        let output = result.to_string();
        assert!(output.contains("allow (dead_code)") || output.contains("allow(dead_code)"));
        assert!(output.contains("args : GreetArgs"));
        assert!(output.contains("tx : StreamSender"));
    }

    #[test]
    fn test_is_streaming_empty_attr() {
        assert!(!is_streaming(&TokenStream::new()));
    }

    #[test]
    fn test_is_streaming_with_keyword() {
        let attr = quote! { streaming };
        assert!(is_streaming(&attr));
    }

    #[test]
    fn test_is_streaming_with_other_keyword() {
        let attr = quote! { something_else };
        assert!(!is_streaming(&attr));
    }
}
