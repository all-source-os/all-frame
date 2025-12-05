//! Error derive macros for automatic gRPC status conversion
//!
//! Provides the #[derive(GrpcError)] macro that automatically generates
//! `From<Error> for tonic::Status` implementations.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Attribute, Data, DeriveInput, Error, Fields, Ident, Result};

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
        let grpc_code = extract_grpc_code(&variant.attrs)?
            .unwrap_or_else(|| "INTERNAL".to_string());

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
                ::allframe_core::tonic::Status::#code_ident(err.to_string())
            }
        });
    }

    let expanded = quote! {
        impl ::core::convert::From<#name> for ::allframe_core::tonic::Status {
            fn from(err: #name) -> Self {
                match &err {
                    #(#match_arms),*
                }
            }
        }
    };

    Ok(expanded)
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
