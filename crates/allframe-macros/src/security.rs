//! Security-related macros for AllFrame.
//!
//! Provides derive macros for safe logging of sensitive data.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Fields, Ident, LitStr};

/// Implementation of the `#[derive(Obfuscate)]` macro.
///
/// Generates an `Obfuscate` trait implementation that obfuscates fields
/// marked with `#[sensitive]`.
pub fn obfuscate_impl(input: TokenStream) -> syn::Result<TokenStream> {
    let input: DeriveInput = parse2(input)?;
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let format_fields = generate_format_fields(&input)?;

    Ok(quote! {
        impl #impl_generics allframe_core::security::Obfuscate for #name #ty_generics #where_clause {
            fn obfuscate(&self) -> String {
                format!(#format_fields)
            }
        }
    })
}

fn generate_format_fields(input: &DeriveInput) -> syn::Result<TokenStream> {
    let name = &input.ident;
    let name_str = name.to_string();

    match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => {
                // Note: We need double-escaped braces because:
                // 1. format!() here unescapes {{ to {
                // 2. The resulting string is used as a format literal in generated code
                // So we need {{ in the final string, which requires {{{{ here
                let mut format_parts = vec![format!("{} {{{{ ", name_str)];
                let mut format_args = Vec::new();
                let mut first = true;

                for field in &fields.named {
                    let field_name = field.ident.as_ref().unwrap();
                    let field_name_str = field_name.to_string();

                    // Check for #[sensitive] attribute
                    let is_sensitive = field
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("sensitive"));

                    // Check for #[obfuscate(with = "...")] attribute
                    let custom_obfuscator = get_obfuscate_with(&field.attrs)?;

                    if !first {
                        format_parts.push(", ".to_string());
                    }
                    first = false;

                    if is_sensitive {
                        format_parts.push(format!("{}: ***", field_name_str));
                    } else if let Some(obfuscator) = custom_obfuscator {
                        format_parts.push(format!("{}: {{}}", field_name_str));
                        let obf_ident: Ident = syn::parse_str(&obfuscator)?;
                        format_args.push(quote! { #obf_ident(&self.#field_name) });
                    } else {
                        format_parts.push(format!("{}: {{:?}}", field_name_str));
                        format_args.push(quote! { self.#field_name });
                    }
                }

                format_parts.push(" }}".to_string());
                let format_str = format_parts.join("");
                let format_lit = LitStr::new(&format_str, proc_macro2::Span::call_site());

                Ok(quote! {
                    #format_lit, #(#format_args),*
                })
            }
            Fields::Unnamed(fields) => {
                let mut format_parts = vec![format!("{}(", name_str)];
                let mut format_args = Vec::new();

                for (i, field) in fields.unnamed.iter().enumerate() {
                    let is_sensitive = field
                        .attrs
                        .iter()
                        .any(|attr| attr.path().is_ident("sensitive"));

                    if i > 0 {
                        format_parts.push(", ".to_string());
                    }

                    let index = syn::Index::from(i);
                    if is_sensitive {
                        format_parts.push("***".to_string());
                    } else {
                        format_parts.push("{}".to_string());
                        format_args.push(quote! { self.#index });
                    }
                }

                format_parts.push(")".to_string());
                let format_str = format_parts.join("");
                let format_lit = LitStr::new(&format_str, proc_macro2::Span::call_site());

                Ok(quote! {
                    #format_lit, #(#format_args),*
                })
            }
            Fields::Unit => {
                let format_lit = LitStr::new(&name_str, proc_macro2::Span::call_site());
                Ok(quote! { #format_lit })
            }
        },
        Data::Enum(data) => {
            // For enums, generate match arms that obfuscate #[sensitive] fields per variant
            let match_arms = data.variants.iter().map(|variant| {
                let variant_name = &variant.ident;
                let variant_str = variant_name.to_string();

                match &variant.fields {
                    Fields::Named(fields) => {
                        let field_names: Vec<_> = fields
                            .named
                            .iter()
                            .map(|f| f.ident.as_ref().unwrap())
                            .collect();

                        let mut format_parts = vec![format!("{}::{} {{{{ ", name_str, variant_str)];
                        let mut format_args = Vec::<proc_macro2::TokenStream>::new();
                        let mut first = true;

                        for field in &fields.named {
                            let field_name = field.ident.as_ref().unwrap();
                            let field_name_str = field_name.to_string();
                            let is_sensitive = field
                                .attrs
                                .iter()
                                .any(|attr| attr.path().is_ident("sensitive"));

                            if !first {
                                format_parts.push(", ".to_string());
                            }
                            first = false;

                            if is_sensitive {
                                format_parts.push(format!("{}: ***", field_name_str));
                            } else {
                                format_parts.push(format!("{}: {{:?}}", field_name_str));
                                format_args.push(quote! { #field_name });
                            }
                        }

                        format_parts.push(" }}".to_string());
                        let format_str = format_parts.join("");
                        let format_lit =
                            LitStr::new(&format_str, proc_macro2::Span::call_site());

                        quote! {
                            #name::#variant_name { #(#field_names),* } => {
                                format!(#format_lit, #(#format_args),*)
                            }
                        }
                    }
                    Fields::Unnamed(fields) => {
                        let bindings: Vec<_> = (0..fields.unnamed.len())
                            .map(|i| {
                                Ident::new(&format!("_f{}", i), proc_macro2::Span::call_site())
                            })
                            .collect();

                        let mut format_parts =
                            vec![format!("{}::{}(", name_str, variant_str)];
                        let mut format_args = Vec::<proc_macro2::TokenStream>::new();

                        for (i, field) in fields.unnamed.iter().enumerate() {
                            let is_sensitive = field
                                .attrs
                                .iter()
                                .any(|attr| attr.path().is_ident("sensitive"));
                            let binding = &bindings[i];

                            if i > 0 {
                                format_parts.push(", ".to_string());
                            }

                            if is_sensitive {
                                format_parts.push("***".to_string());
                            } else {
                                format_parts.push("{:?}".to_string());
                                format_args.push(quote! { #binding });
                            }
                        }

                        format_parts.push(")".to_string());
                        let format_str = format_parts.join("");
                        let format_lit =
                            LitStr::new(&format_str, proc_macro2::Span::call_site());

                        quote! {
                            #name::#variant_name(#(#bindings),*) => {
                                format!(#format_lit, #(#format_args),*)
                            }
                        }
                    }
                    Fields::Unit => {
                        let variant_full = format!("{}::{}", name_str, variant_str);
                        let lit = LitStr::new(&variant_full, proc_macro2::Span::call_site());
                        quote! {
                            #name::#variant_name => #lit.to_string()
                        }
                    }
                }
            });

            let format_lit = LitStr::new(&name_str, proc_macro2::Span::call_site());
            let _ = format_lit; // suppress unused warning
            Ok(quote! {
                match self {
                    #(#match_arms,)*
                }
            })
        }
        Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "Obfuscate cannot be derived for unions",
        )),
    }
}

fn get_obfuscate_with(attrs: &[syn::Attribute]) -> syn::Result<Option<String>> {
    for attr in attrs {
        if attr.path().is_ident("obfuscate") {
            let mut result = None;
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("with") {
                    let value: LitStr = meta.value()?.parse()?;
                    result = Some(value.value());
                }
                Ok(())
            })?;
            return Ok(result);
        }
    }
    Ok(None)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_obfuscate_struct_with_sensitive() {
        let input: TokenStream = quote! {
            struct Config {
                host: String,
                #[sensitive]
                password: String,
                port: u16,
            }
        };

        let result = obfuscate_impl(input);
        assert!(result.is_ok());
        let output = result.unwrap().to_string();
        assert!(output.contains("Obfuscate"));
        assert!(output.contains("obfuscate"));
    }

    #[test]
    fn test_obfuscate_struct_unit() {
        let input: TokenStream = quote! {
            struct Empty;
        };

        let result = obfuscate_impl(input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_obfuscate_enum_unit_variants() {
        let input: TokenStream = quote! {
            enum Status {
                Active,
                Inactive,
            }
        };

        let result = obfuscate_impl(input);
        assert!(result.is_ok(), "Enum Obfuscate should work: {:?}", result.err());
        let output = result.unwrap().to_string();
        assert!(output.contains("Obfuscate"));
        assert!(output.contains("Active"));
        assert!(output.contains("Inactive"));
    }

    #[test]
    fn test_obfuscate_enum_with_sensitive_fields() {
        let input: TokenStream = quote! {
            enum Credential {
                Password {
                    username: String,
                    #[sensitive]
                    password: String,
                },
                Token {
                    #[sensitive]
                    value: String,
                },
                None,
            }
        };

        let result = obfuscate_impl(input);
        assert!(result.is_ok(), "Enum with sensitive fields should work: {:?}", result.err());
        let output = result.unwrap().to_string();
        assert!(output.contains("***"), "Should obfuscate sensitive fields");
        assert!(output.contains("username"), "Should show non-sensitive fields");
    }
}
