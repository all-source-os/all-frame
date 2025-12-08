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
        Data::Enum(_) => {
            // For enums, just use Debug formatting with *** for sensitive fields
            Err(syn::Error::new_spanned(
                input,
                "Obfuscate derive is not yet supported for enums. Please implement Obfuscate \
                 manually.",
            ))
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
    fn test_obfuscate_enum_not_supported() {
        let input: TokenStream = quote! {
            enum MyEnum {
                A,
                B,
            }
        };

        let result = obfuscate_impl(input);
        assert!(result.is_err());
    }
}
