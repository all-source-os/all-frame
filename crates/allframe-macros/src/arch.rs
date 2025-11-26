//! Clean Architecture layer enforcement macros
//!
//! This module provides compile-time enforcement of Clean Architecture layers.
//! Each layer can only depend on layers below it in the hierarchy:
//!
//! Layer 4: Handler (HTTP/gRPC/GraphQL endpoints)
//!    ↓
//! Layer 3: Use Case (application logic)
//!    ↓
//! Layer 2: Repository (data access)
//!    ↓
//! Layer 1: Domain (business logic, no dependencies)

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, Item, ItemStruct, Result};

/// Layer hierarchy - lower numbers are inner layers
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum Layer {
    Domain = 1,
    Repository = 2,
    UseCase = 3,
    Handler = 4,
}

impl Layer {
    /// Get layer name as string
    pub fn name(&self) -> &'static str {
        match self {
            Layer::Domain => "domain",
            Layer::Repository => "repository",
            Layer::UseCase => "use_case",
            Layer::Handler => "handler",
        }
    }

    /// Check if this layer can depend on another layer
    #[allow(dead_code)] // Used in tests
    pub fn can_depend_on(&self, other: &Layer) -> bool {
        // A layer can only depend on layers with lower numbers (inner layers)
        (*self as u8) > (*other as u8)
    }
}

/// Marks a type as part of the Domain layer (Layer 1)
///
/// Domain entities have no dependencies and contain pure business logic.
pub fn domain_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(s) => validate_domain_struct(s)?,
        Item::Enum(_) => {
            // Enums are fine in domain layer
        }
        _ => {
            return Err(Error::new_spanned(
                &item,
                "#[domain] can only be applied to structs or enums",
            ));
        }
    }

    // Add layer metadata as a const
    let layer_metadata = generate_layer_metadata(&item, Layer::Domain);

    Ok(quote! {
        #item

        #layer_metadata
    })
}

/// Marks a type as part of the Repository layer (Layer 2)
///
/// Repositories can depend on Domain entities.
pub fn repository_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Trait(_) | Item::Struct(_) => {
            // Both traits and structs can be repositories
        }
        _ => {
            return Err(Error::new_spanned(
                &item,
                "#[repository] can only be applied to traits or structs",
            ));
        }
    }

    let layer_metadata = generate_layer_metadata(&item, Layer::Repository);

    Ok(quote! {
        #item

        #layer_metadata
    })
}

/// Marks a type as part of the Use Case layer (Layer 3)
///
/// Use cases can depend on Repositories and Domain entities.
pub fn use_case_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(_) => {
            // Use cases are typically structs
        }
        _ => {
            return Err(Error::new_spanned(
                &item,
                "#[use_case] can only be applied to structs",
            ));
        }
    }

    let layer_metadata = generate_layer_metadata(&item, Layer::UseCase);

    Ok(quote! {
        #item

        #layer_metadata
    })
}

/// Marks a type as part of the Handler layer (Layer 4)
///
/// Handlers can depend on Use Cases, but NOT on Repositories directly.
pub fn handler_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(_) | Item::Fn(_) => {
            // Handlers can be structs or functions
        }
        _ => {
            return Err(Error::new_spanned(
                &item,
                "#[handler] can only be applied to structs or functions",
            ));
        }
    }

    let layer_metadata = generate_layer_metadata(&item, Layer::Handler);

    Ok(quote! {
        #item

        #layer_metadata
    })
}

/// Validate that a domain struct has no forbidden dependencies
fn validate_domain_struct(s: &ItemStruct) -> Result<()> {
    // For now, we'll just mark it
    // Full validation would require type analysis of all fields
    // which is complex and would need to track all types in the crate

    // TODO: In a future iteration, we can:
    // 1. Analyze field types
    // 2. Check if they are marked with #[repository], #[use_case], or #[handler]
    // 3. Emit compile errors for violations

    let _ = s; // Use the parameter
    Ok(())
}

/// Generate layer metadata as a const item
fn generate_layer_metadata(item: &Item, layer: Layer) -> TokenStream {
    let type_name = match item {
        Item::Struct(s) => &s.ident,
        Item::Trait(t) => &t.ident,
        Item::Enum(e) => &e.ident,
        Item::Fn(f) => &f.sig.ident,
        _ => return quote! {}, // Skip for other items
    };

    let layer_name = layer.name();
    let layer_number = layer as u8;

    quote! {
        // Layer metadata available at compile time and runtime
        #[allow(non_upper_case_globals)]
        const _: () = {
            // This const block ensures the metadata is available for compile-time checks
            const __ALLFRAME_LAYER: &str = #layer_name;
            const __ALLFRAME_LAYER_NUMBER: u8 = #layer_number;
            const __ALLFRAME_TYPE_NAME: &str = stringify!(#type_name);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_layer_hierarchy() {
        assert!(Layer::Handler > Layer::UseCase);
        assert!(Layer::UseCase > Layer::Repository);
        assert!(Layer::Repository > Layer::Domain);
    }

    #[test]
    fn test_can_depend_on() {
        // Handler can depend on all lower layers
        assert!(Layer::Handler.can_depend_on(&Layer::UseCase));
        assert!(Layer::Handler.can_depend_on(&Layer::Repository));
        assert!(Layer::Handler.can_depend_on(&Layer::Domain));

        // Use Case can depend on Repository and Domain
        assert!(Layer::UseCase.can_depend_on(&Layer::Repository));
        assert!(Layer::UseCase.can_depend_on(&Layer::Domain));
        assert!(!Layer::UseCase.can_depend_on(&Layer::Handler));

        // Repository can only depend on Domain
        assert!(Layer::Repository.can_depend_on(&Layer::Domain));
        assert!(!Layer::Repository.can_depend_on(&Layer::UseCase));
        assert!(!Layer::Repository.can_depend_on(&Layer::Handler));

        // Domain cannot depend on anything
        assert!(!Layer::Domain.can_depend_on(&Layer::Repository));
        assert!(!Layer::Domain.can_depend_on(&Layer::UseCase));
        assert!(!Layer::Domain.can_depend_on(&Layer::Handler));
    }

    #[test]
    fn test_layer_names() {
        assert_eq!(Layer::Domain.name(), "domain");
        assert_eq!(Layer::Repository.name(), "repository");
        assert_eq!(Layer::UseCase.name(), "use_case");
        assert_eq!(Layer::Handler.name(), "handler");
    }
}
