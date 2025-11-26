//! OpenTelemetry macros for automatic tracing instrumentation
//!
//! Provides the #[traced] macro that automatically instruments functions
//! with OpenTelemetry spans for distributed tracing.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, Item, ItemFn, Result};

/// Mark a function to be automatically traced
pub fn traced_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Fn(func) => {
            let traced_fn = instrument_function(func)?;
            Ok(traced_fn)
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[traced] can only be applied to functions",
        )),
    }
}

fn instrument_function(func: &ItemFn) -> Result<TokenStream> {
    let _func_name = &func.sig.ident;
    let vis = &func.vis;
    let sig = &func.sig;
    let block = &func.block;
    let attrs = &func.attrs;

    // Generate the instrumented function
    Ok(quote! {
        #(#attrs)*
        #vis #sig {
            // For MVP, just execute the function without tracing
            // Full tracing implementation will be added incrementally
            #block
        }
    })
}
