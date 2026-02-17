//! Error derive macros for automatic gRPC status conversion
//!
//! Provides the #[derive(GrpcError)] macro that automatically generates
//! `From<Error> for tonic::Status` implementations.
//!
//! ## Usage
//!
//! ```rust,ignore
//! use allframe::GrpcError;
//!
//! #[derive(Debug, thiserror::Error, GrpcError)]
//! pub enum MyError {
//!     #[error("Not found: {0}")]
//!     #[grpc(NOT_FOUND)]
//!     NotFound(String),
//!
//!     #[error("Internal error")]
//!     #[grpc(INTERNAL)]
//!     Internal,
//! }
//! ```
//!
//! ## Custom Crate Path
//!
//! If you're using `allframe-core` directly instead of `allframe`, specify the
//! crate path:
//!
//! ```rust,ignore
//! #[derive(GrpcError)]
//! #[grpc_error(crate = "allframe_core")]
//! pub enum MyError { ... }
//! ```

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Attribute, Data, DeriveInput, Error, Fields, Ident, Lit, Meta, Result};

/// Supported gRPC status codes
const VALID_CODES: &[&str] = &[
    "OK",
    "CANCELLED",
    "UNKNOWN",
    "INVALID_ARGUMENT",
    "DEADLINE_EXCEEDED",
    "NOT_FOUND",
    "ALREADY_EXISTS",
    "PERMISSION_DENIED",
    "RESOURCE_EXHAUSTED",
    "FAILED_PRECONDITION",
    "ABORTED",
    "OUT_OF_RANGE",
    "UNIMPLEMENTED",
    "INTERNAL",
    "UNAVAILABLE",
    "DATA_LOSS",
    "UNAUTHENTICATED",
];

/// Implement the GrpcError derive macro
pub fn grpc_error_impl(input: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    let name = &input.ident;

    // Extract custom crate path from #[grpc_error(crate = "...")] attribute
    // Default to "allframe" for users of the main crate
    let crate_path = extract_crate_path(&input.attrs)?.unwrap_or_else(|| "allframe".to_string());
    let crate_ident = Ident::new(&crate_path, proc_macro2::Span::call_site());

    let data = match &input.data {
        Data::Enum(data) => data,
        _ => {
            return Err(Error::new_spanned(
                &input,
                "GrpcError can only be derived for enums",
            ))
        }
    };

    let mut match_arms = Vec::new();

    for variant in &data.variants {
        let variant_name = &variant.ident;
        let grpc_code =
            extract_grpc_code(&variant.attrs)?.unwrap_or_else(|| "INTERNAL".to_string());

        // Validate the code
        if !VALID_CODES.contains(&grpc_code.as_str()) {
            return Err(Error::new_spanned(
                variant,
                format!(
                    "Invalid gRPC code '{}'. Valid codes are: {}",
                    grpc_code,
                    VALID_CODES.join(", ")
                ),
            ));
        }

        let code_ident = Ident::new(&grpc_code.to_lowercase(), variant_name.span());

        // Generate pattern based on variant fields
        let pattern = match &variant.fields {
            Fields::Unit => quote! { #name::#variant_name },
            Fields::Unnamed(_) => quote! { #name::#variant_name(..) },
            Fields::Named(_) => quote! { #name::#variant_name { .. } },
        };

        match_arms.push(quote! {
            #pattern => {
                ::#crate_ident::tonic::Status::#code_ident(err.to_string())
            }
        });
    }

    let expanded = quote! {
        impl ::core::convert::From<#name> for ::#crate_ident::tonic::Status {
            fn from(err: #name) -> Self {
                match &err {
                    #(#match_arms),*
                }
            }
        }
    };

    Ok(expanded)
}

/// Extract the crate path from #[grpc_error(crate = "...")] attribute
fn extract_crate_path(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        if attr.path().is_ident("grpc_error") {
            // Parse #[grpc_error(crate = "path")]
            let meta = attr.parse_args::<Meta>()?;
            if let Meta::NameValue(nv) = meta {
                if nv.path.is_ident("crate") {
                    if let syn::Expr::Lit(syn::ExprLit {
                        lit: Lit::Str(lit_str),
                        ..
                    }) = &nv.value
                    {
                        return Ok(Some(lit_str.value()));
                    }
                }
            }
        }
    }
    Ok(None)
}

/// Extract the gRPC code from #[grpc(CODE)] attribute
fn extract_grpc_code(attrs: &[Attribute]) -> Result<Option<String>> {
    for attr in attrs {
        if attr.path().is_ident("grpc") {
            // Parse the attribute content
            // Expected format: #[grpc(CODE)] or #[grpc(CODE_NAME)]
            let code: syn::Ident = attr.parse_args()?;
            return Ok(Some(code.to_string().to_uppercase()));
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_valid_codes() {
        assert!(VALID_CODES.contains(&"INTERNAL"));
        assert!(VALID_CODES.contains(&"UNAUTHENTICATED"));
        assert!(VALID_CODES.contains(&"NOT_FOUND"));
    }
}
