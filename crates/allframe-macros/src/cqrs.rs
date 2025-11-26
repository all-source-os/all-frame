//! CQRS macros for Command Query Responsibility Segregation
//!
//! Provides macros for:
//! - #[command] - Mark a struct as a command
//! - #[query] - Mark a struct as a query
//! - #[event] - Mark an enum/struct as an event
//! - #[command_handler] - Mark a function as a command handler
//! - #[query_handler] - Mark a function as a query handler

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, Item, Result};

/// Mark a struct as a Command
pub fn command_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(_) => {
            // Add Command marker trait implementation
            let output = quote! {
                #item

                // Marker trait implementation will be added here
            };
            Ok(output)
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[command] can only be applied to structs",
        )),
    }
}

/// Mark a struct as a Query
pub fn query_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(_) => {
            // Add Query marker trait implementation
            let output = quote! {
                #item

                // Marker trait implementation will be added here
            };
            Ok(output)
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[query] can only be applied to structs",
        )),
    }
}

/// Mark an enum or struct as an Event
pub fn event_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Enum(_) | Item::Struct(_) => {
            // Add Event marker trait implementation
            let output = quote! {
                #item

                // Marker trait implementation will be added here
            };
            Ok(output)
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[event] can only be applied to enums or structs",
        )),
    }
}

/// Mark a function as a Command Handler
pub fn command_handler_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Fn(_) => {
            // Just pass through for now - handlers are just regular functions
            Ok(quote! { #item })
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[command_handler] can only be applied to functions",
        )),
    }
}

/// Mark a function as a Query Handler
pub fn query_handler_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Fn(_) => {
            // Just pass through for now - handlers are just regular functions
            Ok(quote! { #item })
        }
        _ => Err(Error::new_spanned(
            &item,
            "#[query_handler] can only be applied to functions",
        )),
    }
}
