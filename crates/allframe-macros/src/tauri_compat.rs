//! `#[tauri_compat]` proc macro implementation
//!
//! Transforms a function with individual Tauri-style parameters into
//! an allframe handler that accepts a generated args struct.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{parse2, FnArg, Ident, ItemFn, Pat, ReturnType, Type};

/// Heuristic: checks if the last path segment is `StreamSender`.
///
/// Matches `StreamSender`, `StreamSender<T>`, `allframe_core::router::StreamSender`, etc.
/// Same trade-off as `is_state_type`.
fn is_stream_sender_type(ty: &Type) -> bool {
    if let Type::Path(type_path) = ty {
        if let Some(segment) = type_path.path.segments.last() {
            return segment.ident == "StreamSender";
        }
    }
    false
}

/// Check if the attribute tokens contain the `streaming` keyword.
fn has_streaming_attr(attr: &TokenStream) -> bool {
    attr.clone()
        .into_iter()
        .any(|tt| matches!(&tt, proc_macro2::TokenTree::Ident(ident) if ident == "streaming"))
}

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

pub fn tauri_compat_impl(attr: TokenStream, item: TokenStream) -> syn::Result<TokenStream> {
    let streaming = has_streaming_attr(&attr);
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

    // Separate params into State extractors, StreamSender, and regular args
    let mut state_params: Vec<(Ident, Type)> = Vec::new();
    let mut stream_param: Option<(Ident, Type)> = None;
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
                } else if streaming && is_stream_sender_type(&ty) {
                    if stream_param.is_some() {
                        return Err(syn::Error::new_spanned(
                            pat_type,
                            "#[tauri_compat(streaming)] supports at most one StreamSender parameter",
                        ));
                    }
                    stream_param = Some((ident, ty));
                } else {
                    arg_params.push((ident, ty));
                }
            }
        }
    }

    // If streaming is set but no StreamSender parameter found, emit a compile error
    if streaming && stream_param.is_none() {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "#[tauri_compat(streaming)] requires a StreamSender parameter",
        ));
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
    let generated_fn = if let Some((stream_ident, stream_ty)) = &stream_param {
        // Streaming mode: StreamSender goes after args struct in signature
        if state_params.is_empty() && arg_params.is_empty() {
            // Only StreamSender, no state, no args
            quote! {
                #(#fn_attrs)*
                #fn_vis #fn_asyncness fn #fn_name(#stream_ident: #stream_ty) -> #return_type
                #fn_body
            }
        } else if state_params.is_empty() {
            // Args + StreamSender, no state
            quote! {
                #(#fn_attrs)*
                #fn_vis #fn_asyncness fn #fn_name(args: #struct_name, #stream_ident: #stream_ty) -> #return_type {
                    let #struct_name { #(#field_names),* } = args;
                    #fn_body
                }
            }
        } else if arg_params.is_empty() {
            // State + StreamSender, no args
            quote! {
                #(#fn_attrs)*
                #fn_vis #fn_asyncness fn #fn_name(#(#state_names: #state_types),*, #stream_ident: #stream_ty) -> #return_type
                #fn_body
            }
        } else {
            // State + Args + StreamSender
            quote! {
                #(#fn_attrs)*
                #fn_vis #fn_asyncness fn #fn_name(#(#state_names: #state_types),*, args: #struct_name, #stream_ident: #stream_ty) -> #return_type {
                    let #struct_name { #(#field_names),* } = args;
                    #fn_body
                }
            }
        }
    } else if state_params.is_empty() && arg_params.is_empty() {
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

    #[test]
    fn test_is_stream_sender_type() {
        let ty: Type = syn::parse_str("StreamSender").unwrap();
        assert!(is_stream_sender_type(&ty));

        let ty: Type = syn::parse_str("String").unwrap();
        assert!(!is_stream_sender_type(&ty));
    }

    #[test]
    fn test_has_streaming_attr() {
        let attr: TokenStream = syn::parse_str("streaming").unwrap();
        assert!(has_streaming_attr(&attr));

        let attr: TokenStream = syn::parse_str("").unwrap();
        assert!(!has_streaming_attr(&attr));
    }

    #[test]
    fn test_streaming_basic() {
        let attr: TokenStream = syn::parse_str("streaming").unwrap();
        let item: TokenStream = syn::parse_str(
            r#"
            async fn stream_chat(prompt: String, model: String, tx: StreamSender) -> String {
                format!("{} {}", prompt, model)
            }
            "#,
        )
        .unwrap();

        let result = tauri_compat_impl(attr, item).unwrap();
        let output = result.to_string();

        // Should generate StreamChatArgs with prompt and model (not tx)
        assert!(output.contains("StreamChatArgs"));
        assert!(output.contains("prompt"));
        assert!(output.contains("model"));
        // StreamSender should appear in the function signature, not the struct
        assert!(output.contains("tx : StreamSender"));
        // The args struct should NOT contain StreamSender
        // Count occurrences: StreamSender should only appear in fn sig
        let struct_end = output.find("stream_chat").unwrap();
        let struct_part = &output[..struct_end];
        assert!(!struct_part.contains("StreamSender"));
    }

    #[test]
    fn test_streaming_with_state() {
        let attr: TokenStream = syn::parse_str("streaming").unwrap();
        let item: TokenStream = syn::parse_str(
            r#"
            async fn stream_updates(state: State<AppState>, count: u32, tx: StreamSender) -> String {
                format!("{}", count)
            }
            "#,
        )
        .unwrap();

        let result = tauri_compat_impl(attr, item).unwrap();
        let output = result.to_string();

        assert!(output.contains("StreamUpdatesArgs"));
        assert!(output.contains("count"));
        // State should be first, then args, then StreamSender
        let fn_part = &output[output.find("stream_updates").unwrap()..];
        let state_pos = fn_part.find("state").unwrap();
        let args_pos = fn_part.find("args").unwrap();
        let tx_pos = fn_part.find("tx").unwrap();
        assert!(state_pos < args_pos);
        assert!(args_pos < tx_pos);
    }

    #[test]
    fn test_streaming_no_stream_sender_error() {
        let attr: TokenStream = syn::parse_str("streaming").unwrap();
        let item: TokenStream = syn::parse_str(
            r#"
            async fn bad_stream(prompt: String) -> String {
                prompt
            }
            "#,
        )
        .unwrap();

        let result = tauri_compat_impl(attr, item);
        assert!(result.is_err());
        let err_msg = result.unwrap_err().to_string();
        assert!(err_msg.contains("StreamSender"));
    }

    #[test]
    fn test_streaming_only_stream_sender() {
        let attr: TokenStream = syn::parse_str("streaming").unwrap();
        let item: TokenStream = syn::parse_str(
            r#"
            async fn stream_all(tx: StreamSender) -> String {
                "done".to_string()
            }
            "#,
        )
        .unwrap();

        let result = tauri_compat_impl(attr, item).unwrap();
        let output = result.to_string();

        // Should have an empty args struct and fn taking only StreamSender
        assert!(output.contains("StreamAllArgs"));
        assert!(output.contains("tx : StreamSender"));
    }

    #[test]
    fn test_streaming_state_and_stream_sender_no_args() {
        let attr: TokenStream = syn::parse_str("streaming").unwrap();
        let item: TokenStream = syn::parse_str(
            r#"
            async fn stream_state(state: State<AppState>, tx: StreamSender) -> String {
                "done".to_string()
            }
            "#,
        )
        .unwrap();

        let result = tauri_compat_impl(attr, item).unwrap();
        let output = result.to_string();

        assert!(output.contains("StreamStateArgs"));
        // Function should have state and tx but no args param
        let fn_part = &output[output.find("stream_state").unwrap()..];
        assert!(fn_part.contains("state : State"));
        assert!(fn_part.contains("tx : StreamSender"));
    }

    #[test]
    fn test_non_streaming_unchanged() {
        // Ensure that a non-streaming call still works the same way
        let attr: TokenStream = syn::parse_str("").unwrap();
        let item: TokenStream = syn::parse_str(
            r#"
            async fn greet(name: String) -> String {
                name
            }
            "#,
        )
        .unwrap();

        let result = tauri_compat_impl(attr, item).unwrap();
        let output = result.to_string();

        assert!(output.contains("GreetArgs"));
        assert!(output.contains("name"));
        assert!(!output.contains("StreamSender"));
    }
}
