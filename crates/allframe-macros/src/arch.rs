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
//!
//! Enforcement uses type-name heuristics on struct fields: types whose names
//! contain "Repository", "UseCase", or "Handler" are assumed to belong to the
//! corresponding layer. This catches the vast majority of real-world violations
//! while remaining zero-cost at runtime.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Error, Fields, Item, ItemStruct, Result, Type};

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
    #[cfg(test)]
    pub fn can_depend_on(&self, other: &Layer) -> bool {
        // A layer can only depend on layers with lower numbers (inner layers)
        (*self as u8) > (*other as u8)
    }
}

/// Forbidden layer names for each layer (types at these layers cannot appear as fields)
fn forbidden_type_patterns(layer: Layer) -> &'static [(&'static str, Layer)] {
    match layer {
        Layer::Domain => &[
            ("Repository", Layer::Repository),
            ("UseCase", Layer::UseCase),
            ("Handler", Layer::Handler),
        ],
        Layer::Repository => &[
            ("UseCase", Layer::UseCase),
            ("Handler", Layer::Handler),
        ],
        Layer::UseCase => &[("Handler", Layer::Handler)],
        Layer::Handler => &[], // Handlers can depend on anything
    }
}

/// Extract the innermost type name from a type, stripping wrappers like Arc<T>,
/// Box<T>, Option<T>, Vec<T>, dyn Trait, etc.
fn extract_inner_type_name(ty: &Type) -> Vec<String> {
    let mut names = Vec::new();

    match ty {
        Type::Path(type_path) => {
            // Get the last segment (e.g., "Arc" from "std::sync::Arc<T>")
            if let Some(segment) = type_path.path.segments.last() {
                let ident = segment.ident.to_string();

                // Known wrapper types - recurse into their generic arguments
                let is_wrapper = matches!(
                    ident.as_str(),
                    "Arc" | "Box" | "Rc" | "Option" | "Vec" | "Mutex" | "RwLock"
                );

                if is_wrapper {
                    if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                        for arg in &args.args {
                            if let syn::GenericArgument::Type(inner) = arg {
                                names.extend(extract_inner_type_name(inner));
                            }
                        }
                    }
                } else {
                    names.push(ident);
                }
            }
        }
        Type::TraitObject(trait_obj) => {
            // Handle `dyn SomeTrait`
            for bound in &trait_obj.bounds {
                if let syn::TypeParamBound::Trait(trait_bound) = bound {
                    if let Some(segment) = trait_bound.path.segments.last() {
                        names.push(segment.ident.to_string());
                    }
                }
            }
        }
        Type::Reference(reference) => {
            names.extend(extract_inner_type_name(&reference.elem));
        }
        _ => {}
    }

    names
}

/// Check struct fields for layer violations
fn validate_struct_fields(s: &ItemStruct, current_layer: Layer) -> Result<()> {
    let forbidden = forbidden_type_patterns(current_layer);
    if forbidden.is_empty() {
        return Ok(());
    }

    let fields = match &s.fields {
        Fields::Named(named) => &named.named,
        _ => return Ok(()),
    };

    for field in fields {
        let type_names = extract_inner_type_name(&field.ty);
        for type_name in &type_names {
            for (pattern, violated_layer) in forbidden {
                if type_name.contains(pattern) {
                    let field_name = field
                        .ident
                        .as_ref()
                        .map(|i| i.to_string())
                        .unwrap_or_else(|| "unnamed".to_string());
                    return Err(Error::new_spanned(
                        &field.ty,
                        format!(
                            "Clean Architecture violation: {} layer type `{}` has field `{}` \
                             with type `{}` which appears to be a {} layer type. \
                             The {} layer cannot depend on the {} layer.",
                            current_layer.name(),
                            s.ident,
                            field_name,
                            type_name,
                            violated_layer.name(),
                            current_layer.name(),
                            violated_layer.name(),
                        ),
                    ));
                }
            }
        }
    }

    Ok(())
}

/// Marks a type as part of the Domain layer (Layer 1)
///
/// Domain entities have no dependencies and contain pure business logic.
/// Compile-time validation ensures domain types do not reference repository,
/// use case, or handler types.
pub fn domain_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(s) => validate_struct_fields(s, Layer::Domain)?,
        Item::Enum(_) => {}
        _ => {
            return Err(Error::new_spanned(
                &item,
                "#[domain] can only be applied to structs or enums",
            ));
        }
    }

    let layer_metadata = generate_layer_metadata(&item, Layer::Domain);

    Ok(quote! {
        #item

        #layer_metadata
    })
}

/// Marks a type as part of the Repository layer (Layer 2)
///
/// Repositories can depend on Domain entities but not on use cases or handlers.
pub fn repository_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(s) => validate_struct_fields(s, Layer::Repository)?,
        Item::Trait(_) => {}
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
/// Use cases can depend on Repositories and Domain entities, but not handlers.
pub fn use_case_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(s) => validate_struct_fields(s, Layer::UseCase)?,
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
/// Handlers can depend on Use Cases, Repositories, and Domain entities.
pub fn handler_impl(_attr: TokenStream, item: TokenStream) -> Result<TokenStream> {
    let item = parse2::<Item>(item)?;

    match &item {
        Item::Struct(_) | Item::Fn(_) => {}
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

/// Generate layer metadata as a const item
fn generate_layer_metadata(item: &Item, layer: Layer) -> TokenStream {
    let type_name = match item {
        Item::Struct(s) => &s.ident,
        Item::Trait(t) => &t.ident,
        Item::Enum(e) => &e.ident,
        Item::Fn(f) => &f.sig.ident,
        _ => return quote! {},
    };

    let layer_name = layer.name();
    let layer_number = layer as u8;

    quote! {
        #[allow(non_upper_case_globals)]
        const _: () = {
            const __ALLFRAME_LAYER: &str = #layer_name;
            const __ALLFRAME_LAYER_NUMBER: u8 = #layer_number;
            const __ALLFRAME_TYPE_NAME: &str = stringify!(#type_name);
        };
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use quote::quote;

    #[test]
    fn test_layer_hierarchy() {
        assert!(Layer::Handler > Layer::UseCase);
        assert!(Layer::UseCase > Layer::Repository);
        assert!(Layer::Repository > Layer::Domain);
    }

    #[test]
    fn test_can_depend_on() {
        assert!(Layer::Handler.can_depend_on(&Layer::UseCase));
        assert!(Layer::Handler.can_depend_on(&Layer::Repository));
        assert!(Layer::Handler.can_depend_on(&Layer::Domain));

        assert!(Layer::UseCase.can_depend_on(&Layer::Repository));
        assert!(Layer::UseCase.can_depend_on(&Layer::Domain));
        assert!(!Layer::UseCase.can_depend_on(&Layer::Handler));

        assert!(Layer::Repository.can_depend_on(&Layer::Domain));
        assert!(!Layer::Repository.can_depend_on(&Layer::UseCase));
        assert!(!Layer::Repository.can_depend_on(&Layer::Handler));

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

    #[test]
    fn test_domain_rejects_repository_field() {
        let input = quote! {
            struct BadDomain {
                repo: Arc<dyn UserRepository>,
            }
        };
        let result = domain_impl(TokenStream::new(), input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("Clean Architecture violation"), "Error: {}", err);
        assert!(err.contains("repository"), "Error should mention repository layer: {}", err);
    }

    #[test]
    fn test_domain_rejects_use_case_field() {
        let input = quote! {
            struct BadDomain {
                uc: GetUserUseCase,
            }
        };
        let result = domain_impl(TokenStream::new(), input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("use_case"), "Error: {}", err);
    }

    #[test]
    fn test_domain_rejects_handler_field() {
        let input = quote! {
            struct BadDomain {
                handler: UserHandler,
            }
        };
        let result = domain_impl(TokenStream::new(), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_domain_allows_plain_fields() {
        let input = quote! {
            struct GoodDomain {
                id: String,
                name: String,
                count: u32,
            }
        };
        let result = domain_impl(TokenStream::new(), input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_repository_rejects_use_case_field() {
        let input = quote! {
            struct BadRepo {
                uc: Arc<GetUserUseCase>,
            }
        };
        let result = repository_impl(TokenStream::new(), input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("use_case"), "Error: {}", err);
    }

    #[test]
    fn test_repository_rejects_handler_field() {
        let input = quote! {
            struct BadRepo {
                h: Box<dyn UserHandler>,
            }
        };
        let result = repository_impl(TokenStream::new(), input);
        assert!(result.is_err());
    }

    #[test]
    fn test_use_case_rejects_handler_field() {
        let input = quote! {
            struct BadUseCase {
                handler: Arc<UserHandler>,
            }
        };
        let result = use_case_impl(TokenStream::new(), input);
        assert!(result.is_err());
        let err = result.unwrap_err().to_string();
        assert!(err.contains("handler"), "Error: {}", err);
    }

    #[test]
    fn test_use_case_allows_repository_field() {
        let input = quote! {
            struct GoodUseCase {
                repo: Arc<dyn UserRepository>,
            }
        };
        let result = use_case_impl(TokenStream::new(), input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_handler_allows_any_field() {
        let input = quote! {
            struct GoodHandler {
                use_case: Arc<GetUserUseCase>,
                repo: Arc<dyn UserRepository>,
            }
        };
        let result = handler_impl(TokenStream::new(), input);
        assert!(result.is_ok());
    }

    #[test]
    fn test_extract_inner_type_through_arc() {
        let ty: Type = syn::parse_quote!(Arc<dyn UserRepository>);
        let names = extract_inner_type_name(&ty);
        assert!(names.iter().any(|n| n.contains("Repository")), "Names: {:?}", names);
    }

    #[test]
    fn test_extract_inner_type_through_box() {
        let ty: Type = syn::parse_quote!(Box<UserHandler>);
        let names = extract_inner_type_name(&ty);
        assert!(names.iter().any(|n| n.contains("Handler")), "Names: {:?}", names);
    }

    #[test]
    fn test_extract_inner_type_plain() {
        let ty: Type = syn::parse_quote!(String);
        let names = extract_inner_type_name(&ty);
        assert_eq!(names, vec!["String"]);
    }
}
