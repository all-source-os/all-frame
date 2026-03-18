//! CQRS macros for Command Query Responsibility Segregation
//!
//! Provides macros for:
//! - #[command] - Mark a struct as a command (generates `Command` trait impl)
//! - #[query] - Mark a struct as a query (generates `Query` trait impl)
//! - #[event] - Mark an enum/struct as an event (generates `EventTypeName` + `Event` trait impls)
//! - #[command_handler] - Mark a function as a command handler
//! - #[query_handler] - Mark a function as a query handler

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, Item, ItemEnum, ItemStruct, Result};

/// Extract the ident from a struct item
fn struct_ident(s: &ItemStruct) -> &syn::Ident {
    &s.ident
}

/// Extract the ident from an enum item
fn enum_ident(e: &ItemEnum) -> &syn::Ident {
    &e.ident
}

/// Mark a struct as a Command
///
/// Generates `impl allframe_core::cqrs::Command for StructName {}`.
/// The struct must be `Send + Sync + 'static` (enforced by the trait bounds).
pub fn command_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(s) => {
            let name = struct_ident(s);
            let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();
            let output = quote! {
                #item

                impl #impl_generics allframe_core::cqrs::Command for #name #ty_generics #where_clause {}
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
///
/// Generates `impl allframe_core::cqrs::Query for StructName {}`.
/// The struct must be `Send + Sync + 'static` (enforced by the trait bounds).
pub fn query_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(s) => {
            let name = struct_ident(s);
            let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();
            let output = quote! {
                #item

                impl #impl_generics allframe_core::cqrs::Query for #name #ty_generics #where_clause {}
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
///
/// Generates `impl EventTypeName for TypeName {}` and `impl Event for TypeName {}`.
/// The type must already derive `Clone, Serialize, Deserialize` to satisfy Event's
/// supertraits.
pub fn event_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Enum(e) => {
            let name = enum_ident(e);
            let (impl_generics, ty_generics, where_clause) = e.generics.split_for_impl();
            let output = quote! {
                #item

                impl #impl_generics allframe_core::cqrs::EventTypeName for #name #ty_generics #where_clause {}
                impl #impl_generics allframe_core::cqrs::Event for #name #ty_generics #where_clause {}
            };
            Ok(output)
        }
        Item::Struct(s) => {
            let name = struct_ident(s);
            let (impl_generics, ty_generics, where_clause) = s.generics.split_for_impl();
            let output = quote! {
                #item

                impl #impl_generics allframe_core::cqrs::EventTypeName for #name #ty_generics #where_clause {}
                impl #impl_generics allframe_core::cqrs::Event for #name #ty_generics #where_clause {}
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
///
/// Validates that the item is a function and passes it through.
/// Command handlers implement the `CommandHandler` trait on structs;
/// this macro serves as documentation and compile-time validation.
pub fn command_handler_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Fn(_) => Ok(quote! { #item }),
        _ => Err(Error::new_spanned(
            &item,
            "#[command_handler] can only be applied to functions",
        )),
    }
}

/// Mark a function as a Query Handler
///
/// Validates that the item is a function and passes it through.
/// Query handlers implement the `QueryHandler` trait on structs;
/// this macro serves as documentation and compile-time validation.
pub fn query_handler_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Fn(_) => Ok(quote! { #item }),
        _ => Err(Error::new_spanned(
            &item,
            "#[query_handler] can only be applied to functions",
        )),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_command_generates_trait_impl() {
        let input = quote! {
            struct CreateUser {
                name: String,
            }
        };
        let result = command_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("Command"), "Should generate Command impl");
        assert!(output.contains("CreateUser"), "Should reference the struct name");
    }

    #[test]
    fn test_command_rejects_enum() {
        let input = quote! {
            enum NotAStruct {
                A,
                B,
            }
        };
        let result = command_impl(TokenStream::new(), input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("structs"));
    }

    #[test]
    fn test_query_generates_trait_impl() {
        let input = quote! {
            struct GetUser {
                id: String,
            }
        };
        let result = query_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("Query"), "Should generate Query impl");
        assert!(output.contains("GetUser"), "Should reference the struct name");
    }

    #[test]
    fn test_query_rejects_enum() {
        let input = quote! {
            enum NotAStruct { A }
        };
        let result = query_impl(TokenStream::new(), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_event_on_enum_generates_trait_impls() {
        let input = quote! {
            enum UserEvent {
                Created { id: String },
                Updated { name: String },
            }
        };
        let result = event_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("EventTypeName"), "Should generate EventTypeName impl");
        assert!(output.contains("Event"), "Should generate Event impl");
        assert!(output.contains("UserEvent"), "Should reference the enum name");
    }

    #[test]
    fn test_event_on_struct_generates_trait_impls() {
        let input = quote! {
            struct UserCreated {
                id: String,
            }
        };
        let result = event_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("EventTypeName"));
        assert!(output.contains("Event"));
    }

    #[test]
    fn test_event_rejects_function() {
        let input = quote! {
            fn not_a_type() {}
        };
        let result = event_impl(TokenStream::new(), input);
        assert!(result.is_err());
        assert!(result.unwrap_err().to_string().contains("enums or structs"));
    }

    #[test]
    fn test_command_handler_passes_through_function() {
        let input = quote! {
            fn handle_create(cmd: CreateUser) -> Result<(), Error> {
                Ok(())
            }
        };
        let result = command_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("handle_create"));
    }

    #[test]
    fn test_command_handler_rejects_struct() {
        let input = quote! {
            struct NotAFunction;
        };
        let result = command_handler_impl(TokenStream::new(), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_query_handler_passes_through_function() {
        let input = quote! {
            fn handle_get(q: GetUser) -> UserResult {
                todo!()
            }
        };
        let result = query_handler_impl(TokenStream::new(), input).unwrap();
        let output = result.to_string();
        assert!(output.contains("handle_get"));
    }

    #[test]
    fn test_query_handler_rejects_struct() {
        let input = quote! {
            struct NotAFunction;
        };
        let result = query_handler_impl(TokenStream::new(), input);
        assert!(result.is_err());
    }
}
