//! `#[tauri_compat]` proc macro implementation
//!
//! Transforms a function with individual Tauri-style parameters into
//! an allframe handler that accepts a generated args struct.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, FnArg, Ident, ItemFn, Pat, ReturnType, Type};

/// Heuristic: checks if the last path segment is `Option`.
///
/// This matches `Option<T>`, `std::option::Option<T>`, etc. but could
/// false-positive on a custom type named `Option`. This is the same
/// trade-off Tauri's own macros make and is acceptable for the common case.
fn is_option_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "Option";
        }
    }
    false
}

/// Heuristic: checks if the last path segment is `State`.
///
/// Matches `State<Arc<S>>`, `allframe_core::router::State<...>`, etc.
/// Same trade-off as `is_option_type` — a custom type named `State` would
/// be misclassified. In practice this is fine because `State` in handler
/// signatures almost always refers to the framework extractor.
fn is_state_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "State";
        }
    }
    false
}

/// Convert snake_case to PascalCase
fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(c) => c.to_uppercase().chain(chars).collect(),
            }
        })
        .collect()
}

pub fn tauri_compat_impl(_attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let input_fn: ItemFn = parse2(item)?;

    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_body = &input_fn.block;
    let fn_asyncness = &input_fn.sig.asyncness;

    // Determine return type
    let return_type = match &input_fn.sig.output {
        ReturnType::Default => quote! { () },
        ReturnType::Type(_, ty) => quote! { #ty },
    };

    // Separate params into State extractors vs regular args
    let mut state_params: Vec<(Ident, Type)> = Vec::new();
    let mut arg_params: Vec<(Ident, Type)> = Vec::new();

    for arg in &input_fn.sig.inputs {
        match arg {
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    arg,
                    "#[tauri_compat] cannot be applied to methods with self",
                ));
            }
            FnArg::Typed(pat_type) => {
                let ident = match pat_type.pat.as_ref() {
                    Pat::Ident(pat_ident) => pat_ident.ident.clone(),
                    _ => {
                        return Err(syn::Error::new_spanned(
                            &pat_type.pat,
                            "#[tauri_compat] requires simple parameter names",
                        ));
                    }
                };
                let ty = pat_type.ty.as_ref().clone();

                if is_state_type(&ty) {
                    state_params.push((ident, ty));
                } else {
                    arg_params.push((ident, ty));
                }
            }
        }
    }

    // Generate the args struct name: fn_name -> FnNameArgs
    let struct_name = format_ident!("{}Args", to_pascal_case(&fn_name.to_string()));

    // Build struct fields with serde attributes for Option types.
    // Field visibility matches the function's visibility.
    let struct_fields: Vec<TokenStream> = arg_params
        .iter()
        .map(|(name, ty)| {
            if is_option_type(ty) {
                quote! {
                    #[serde(default)]
                    #fn_vis #name: #ty
                }
            } else {
                quote! { #fn_vis #name: #ty }
            }
        })
        .collect();

    // Generate the args struct (empty struct if no args)
    let args_struct = if arg_params.is_empty() {
        quote! {
            #[derive(serde::Deserialize)]
            #[allow(dead_code)]
            #fn_vis struct #struct_name;
        }
    } else {
        quote! {
            #[derive(serde::Deserialize)]
            #[allow(dead_code)]
            #fn_vis struct #struct_name {
                #(#struct_fields),*
            }
        }
    };

    // Build destructure pattern
    let field_names: Vec<&Ident> = arg_params.iter().map(|(name, _)| name).collect();
    let state_names: Vec<&Ident> = state_params.iter().map(|(name, _)| name).collect();
    let state_types: Vec<&Type> = state_params.iter().map(|(_, ty)| ty).collect();

    // Generate the rewritten function
    let generated_fn = if state_params.is_empty() && arg_params.is_empty() {
        // No args, no state
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_asyncness fn #fn_name() -> #return_type
            #fn_body
        }
    } else if state_params.is_empty() {
        // Args only, no state
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_asyncness fn #fn_name(args: #struct_name) -> #return_type {
                let #struct_name { #(#field_names),* } = args;
                #fn_body
            }
        }
    } else if arg_params.is_empty() {
        // State only, no args
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_asyncness fn #fn_name(#(#state_names: #state_types),*) -> #return_type
            #fn_body
        }
    } else {
        // Both state and args
        quote! {
            #(#fn_attrs)*
            #fn_vis #fn_asyncness fn #fn_name(#(#state_names: #state_types),*, args: #struct_name) -> #return_type {
                let #struct_name { #(#field_names),* } = args;
                #fn_body
            }
        }
    };

    Ok(quote! {
        #args_struct
        #generated_fn
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_to_pascal_case() {
        assert_eq!(to_pascal_case("greet"), "Greet");
        assert_eq!(to_pascal_case("get_user"), "GetUser");
        assert_eq!(to_pascal_case("create_new_item"), "CreateNewItem");
    }
}
