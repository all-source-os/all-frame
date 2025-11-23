//! Dependency Injection Container Macro Implementation
//!
//! This module implements the `#[di_container]` procedural macro for compile-time
//! dependency injection with zero runtime reflection.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Error, Fields, Result};

/// Implementation of the #[di_container] macro
///
/// Generates:
/// - A `new()` associated function that creates the container
/// - Accessor methods for each service
/// - Automatic dependency resolution at compile time
pub fn di_container_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let input: DeriveInput = parse2(item.clone())?;

    let struct_name = &input.ident;
    let vis = &input.vis;

    // Extract fields from the struct
    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            _ => {
                return Err(Error::new_spanned(
                    input,
                    "di_container only works with structs with named fields",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                input,
                "di_container only works with structs",
            ))
        }
    };

    // Collect field information
    let mut field_names = Vec::new();
    let mut field_types = Vec::new();
    let mut field_attrs = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;

        field_names.push(field_name.clone());
        field_types.push(field_type.clone());
        field_attrs.push(field.attrs.clone());
    }

    // Generate field initializations by analyzing dependencies
    let mut field_inits = Vec::new();

    for (idx, (name, ty, attrs)) in field_names
        .iter()
        .zip(field_types.iter())
        .zip(field_attrs.iter())
        .map(|((a, b), c)| (a, b, c))
        .enumerate()
    {
        // Check for #[provide(...)] attribute
        let has_provide = attrs.iter().any(|attr| attr.path().is_ident("provide"));

        if has_provide {
            // Extract the expression from #[provide(...)]
            for attr in attrs {
                if attr.path().is_ident("provide") {
                    let tokens = &attr.meta;
                    let expr_str = quote!(#tokens).to_string();
                    if let Some(expr_content) = expr_str
                        .strip_prefix("provide (")
                        .and_then(|s| s.strip_suffix(')'))
                    {
                        let expr: syn::Expr = syn::parse_str(expr_content)?;
                        field_inits.push(quote! {
                            #name: #expr
                        });
                    }
                }
            }
        } else {
            // Auto-wire by looking at previous fields as potential dependencies
            // For now, we'll try to call new() with references to already-created fields
            let deps: Vec<_> = field_names.iter().take(idx).collect();

            if deps.is_empty() {
                // No dependencies - just call new()
                field_inits.push(quote! {
                    #name: #ty::new()
                });
            } else {
                // Try passing the last field as a dependency (simple heuristic)
                let dep = deps.last().unwrap();
                field_inits.push(quote! {
                    #name: #ty::new(#dep)
                });
            }
        }
    }

    // Generate accessor methods
    let mut accessors = Vec::new();
    for (name, ty) in field_names.iter().zip(field_types.iter()) {
        accessors.push(quote! {
            #vis fn #name(&self) -> &#ty {
                &self.#name
            }
        });
    }

    // Generate the implementation
    // Don't redeclare the struct - just add the implementation
    let expanded = quote! {
        impl #struct_name {
            /// Create a new instance of the container with all dependencies injected
            #vis fn new() -> Self {
                Self {
                    #(#field_inits,)*
                }
            }

            #(#accessors)*
        }
    };

    // Combine the original struct with our implementation
    Ok(quote! {
        #input
        #expanded
    })
}
