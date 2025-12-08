//! Dependency Injection Container Macro Implementation
//!
//! This module implements the `#[di_container]` procedural macro for
//! compile-time dependency injection with zero runtime reflection.
//!
//! # Supported Attributes
//!
//! - `#[provide(expr)]` - Use custom expression for initialization
//! - `#[provide(from_env)]` - Load from environment using FromEnv trait
//! - `#[provide(singleton)]` - Shared instance (default)
//! - `#[provide(transient)]` - New instance on each access
//! - `#[provide(async)]` - Async initialization
//! - `#[depends(field1, field2)]` - Explicit dependencies
//!
//! Multiple options can be combined: `#[provide(singleton, async)]`

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Error, Fields, Result, Type};

/// Configuration for a field's dependency injection behavior
#[derive(Default, Clone)]
struct ProvideConfig {
    /// Custom expression for initialization
    custom_expr: Option<syn::Expr>,
    /// Load from environment using FromEnv trait
    from_env: bool,
    /// Scope: singleton (true) or transient (false)
    singleton: bool,
    /// Whether initialization is async
    is_async: bool,
}

/// Represents information about a field in the DI container
#[derive(Clone)]
struct FieldInfo {
    name: syn::Ident,
    ty: Type,
    config: ProvideConfig,
    /// Explicit dependencies from #[depends(...)]
    explicit_deps: Vec<syn::Ident>,
}

/// Parse #[provide(...)] attribute
fn parse_provide_attr(attr: &syn::Attribute) -> Result<ProvideConfig> {
    let mut config = ProvideConfig {
        singleton: true, // Default to singleton
        ..Default::default()
    };

    let result = attr.parse_nested_meta(|meta| {
        if meta.path.is_ident("from_env") {
            config.from_env = true;
            Ok(())
        } else if meta.path.is_ident("singleton") {
            config.singleton = true;
            Ok(())
        } else if meta.path.is_ident("transient") {
            config.singleton = false;
            Ok(())
        } else if meta.path.is_ident("async") {
            config.is_async = true;
            Ok(())
        } else {
            // Unknown option - will try to parse as expression below
            Err(meta.error("unknown provide option"))
        }
    });

    match result {
        Ok(()) => Ok(config),
        Err(_) => {
            // If nested meta parsing fails, try to parse as expression
            // This handles #[provide(MyType::new())]
            config.custom_expr = Some(attr.parse_args::<syn::Expr>()?);
            Ok(config)
        }
    }
}

/// Parse #[depends(...)] attribute
fn parse_depends_attr(attr: &syn::Attribute) -> Result<Vec<syn::Ident>> {
    let mut deps = Vec::new();

    attr.parse_nested_meta(|meta| {
        if let Some(ident) = meta.path.get_ident() {
            deps.push(ident.clone());
            Ok(())
        } else {
            Err(meta.error("expected identifier"))
        }
    })?;

    Ok(deps)
}

/// Implementation of the #[di_container] macro
///
/// Generates:
/// - A `new()` associated function for sync containers
/// - A `build()` async associated function for async containers
/// - Accessor methods for each service
/// - Automatic dependency resolution at compile time with topological sorting
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
    let mut field_infos = Vec::new();
    let mut has_async = false;

    for field in fields {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_type = field.ty.clone();

        let mut config = ProvideConfig {
            singleton: true,
            ..Default::default()
        };
        let mut explicit_deps = Vec::new();

        for attr in &field.attrs {
            if attr.path().is_ident("provide") {
                config = parse_provide_attr(attr)?;
            } else if attr.path().is_ident("depends") {
                explicit_deps = parse_depends_attr(attr)?;
            }
        }

        if config.is_async {
            has_async = true;
        }

        field_infos.push(FieldInfo {
            name: field_name,
            ty: field_type,
            config,
            explicit_deps,
        });
    }

    // Build dependency graph and determine initialization order
    let (init_order, dependency_map) = compute_initialization_order(&field_infos)?;

    // Determine which fields are dependencies of other fields
    let mut is_dependency_of_others: HashSet<String> = HashSet::new();
    for deps in dependency_map.values() {
        for dep_name in deps {
            is_dependency_of_others.insert(dep_name.clone());
        }
    }

    // Generate field initializations in dependency order
    let mut let_bindings = Vec::new();

    for field_info in &init_order {
        let name = &field_info.name;
        let ty = &field_info.ty;
        let field_name_str = name.to_string();
        let config = &field_info.config;

        let init_expr = if let Some(expr) = &config.custom_expr {
            // Use the provided expression
            quote! { #expr }
        } else if config.from_env {
            // Use FromEnv trait (sync - FromEnv::from_env is not async)
            quote! { <#ty as ::allframe_core::di::FromEnv>::from_env()? }
        } else {
            // Get the dependencies for this field
            let dep_infos: Vec<&FieldInfo> =
                if let Some(dep_set) = dependency_map.get(&field_name_str) {
                    field_infos
                        .iter()
                        .filter(|f| dep_set.contains(&f.name.to_string()))
                        .collect()
                } else {
                    Vec::new()
                };

            if dep_infos.is_empty() {
                // No dependencies
                if config.is_async {
                    quote! { <#ty as ::allframe_core::di::AsyncInit>::init().await? }
                } else {
                    quote! { #ty::new() }
                }
            } else {
                // Pass dependencies to constructor
                let dep_args: Vec<_> = dep_infos
                    .iter()
                    .map(|dep| {
                        let dep_field_name = &dep.name;
                        if is_dependency_of_others.contains(&dep.name.to_string()) {
                            quote! { ::std::sync::Arc::clone(&#dep_field_name) }
                        } else {
                            quote! { #dep_field_name.clone() }
                        }
                    })
                    .collect();

                if config.is_async {
                    quote! { #ty::new(#(#dep_args),*).await? }
                } else {
                    quote! { #ty::new(#(#dep_args),*) }
                }
            }
        };

        // Wrap in Arc if this is a dependency of others
        if is_dependency_of_others.contains(&field_name_str) && config.singleton {
            let_bindings.push(quote! {
                let #name = ::std::sync::Arc::new(#init_expr);
            });
        } else {
            let_bindings.push(quote! {
                let #name = #init_expr;
            });
        }
    }

    let struct_fields: Vec<_> = init_order.iter().map(|f| &f.name).collect();

    // Generate accessor methods
    let mut accessors = Vec::new();
    for field_info in &field_infos {
        let name = &field_info.name;
        let ty = &field_info.ty;
        let field_name_str = name.to_string();

        if is_dependency_of_others.contains(&field_name_str) && field_info.config.singleton {
            // Return cloned Arc for singleton dependencies
            accessors.push(quote! {
                #vis fn #name(&self) -> ::std::sync::Arc<#ty> {
                    ::std::sync::Arc::clone(&self.#name)
                }
            });
        } else if !field_info.config.singleton {
            // Transient: create new instance each time
            let ty_inner = ty;
            if field_info.config.is_async {
                accessors.push(quote! {
                    #vis async fn #name(&self) -> Result<#ty_inner, ::allframe_core::di::DependencyError> {
                        <#ty_inner as ::allframe_core::di::AsyncInit>::init().await
                    }
                });
            } else {
                accessors.push(quote! {
                    #vis fn #name(&self) -> #ty_inner {
                        #ty_inner::new()
                    }
                });
            }
        } else {
            accessors.push(quote! {
                #vis fn #name(&self) -> &#ty {
                    &self.#name
                }
            });
        }
    }

    // Generate constructor method
    let constructor = if has_async {
        quote! {
            /// Build a new instance of the container with all dependencies injected
            #vis async fn build() -> Result<Self, ::allframe_core::di::DependencyError> {
                #(#let_bindings)*

                Ok(Self {
                    #(#struct_fields,)*
                })
            }
        }
    } else {
        quote! {
            /// Create a new instance of the container with all dependencies injected
            #vis fn new() -> Self {
                #(#let_bindings)*

                Self {
                    #(#struct_fields,)*
                }
            }
        }
    };

    // Generate the implementation
    let expanded = quote! {
        impl #struct_name {
            #constructor

            #(#accessors)*
        }
    };

    // Create a modified version of the input struct without DI attributes
    let mut modified_input = input.clone();
    if let Data::Struct(ref mut data_struct) = modified_input.data {
        if let Fields::Named(ref mut fields_named) = data_struct.fields {
            for field in fields_named.named.iter_mut() {
                // Remove DI-specific attributes
                field.attrs.retain(|attr| {
                    !attr.path().is_ident("provide") && !attr.path().is_ident("depends")
                });

                // Wrap singleton dependencies in Arc
                if let Some(ident) = &field.ident {
                    let field_info = field_infos.iter().find(|f| &f.name == ident);
                    if let Some(info) = field_info {
                        if is_dependency_of_others.contains(&ident.to_string())
                            && info.config.singleton
                        {
                            let original_ty = &field.ty;
                            field.ty = syn::parse_quote! { ::std::sync::Arc<#original_ty> };
                        }
                    }
                }
            }
        }
    }

    // Combine the cleaned struct definition with our implementation
    Ok(quote! {
        #modified_input
        #expanded
    })
}

/// Type alias for dependency map (field name -> set of dependency field names)
type DependencyMap = HashMap<String, HashSet<String>>;

/// Compute the initialization order for fields using topological sort
///
/// Returns a tuple of:
/// 1. The initialization order (topologically sorted fields)
/// 2. The dependency map (field name -> set of dependency field names)
fn compute_initialization_order(fields: &[FieldInfo]) -> Result<(Vec<FieldInfo>, DependencyMap)> {
    let mut forward_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut reverse_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut field_map: HashMap<String, &FieldInfo> = HashMap::new();

    for field in fields {
        let field_name = field.name.to_string();
        field_map.insert(field_name.clone(), field);
        forward_graph.insert(field_name.clone(), HashSet::new());
        reverse_graph.insert(field_name.clone(), HashSet::new());

        // Use explicit dependencies if provided, otherwise use heuristics
        if !field.explicit_deps.is_empty() {
            for dep in &field.explicit_deps {
                let dep_name = dep.to_string();
                forward_graph
                    .get_mut(&field_name)
                    .unwrap()
                    .insert(dep_name.clone());
                if let Some(rev) = reverse_graph.get_mut(&dep_name) {
                    rev.insert(field_name.clone());
                }
            }
        } else if field.config.custom_expr.is_none() {
            // Only use heuristics if no explicit deps and no custom expression
            let deps = find_dependencies(&field.ty, &field_name, fields);
            for dep in deps {
                let dep_name = dep.name.to_string();
                forward_graph
                    .get_mut(&field_name)
                    .unwrap()
                    .insert(dep_name.clone());
                reverse_graph
                    .get_mut(&dep_name)
                    .unwrap()
                    .insert(field_name.clone());
            }
        }
    }

    // Topological sort using Kahn's algorithm
    let mut in_degree: HashMap<String, usize> = HashMap::new();
    for name in reverse_graph.keys() {
        in_degree.insert(name.clone(), 0);
    }

    for deps in reverse_graph.values() {
        for dep in deps {
            *in_degree.get_mut(dep).unwrap() += 1;
        }
    }

    let mut queue: Vec<String> = in_degree
        .iter()
        .filter(|(_, &deg)| deg == 0)
        .map(|(name, _)| name.clone())
        .collect();

    let mut result = Vec::new();

    while let Some(node) = queue.pop() {
        result.push(field_map[&node].clone());

        if let Some(dependencies) = reverse_graph.get(&node) {
            for dep in dependencies {
                let deg = in_degree.get_mut(dep).unwrap();
                *deg -= 1;
                if *deg == 0 {
                    queue.push(dep.clone());
                }
            }
        }
    }

    if result.len() != fields.len() {
        // Find the cycle
        let remaining: Vec<_> = in_degree
            .iter()
            .filter(|(_, &deg)| deg > 0)
            .map(|(name, _)| name.clone())
            .collect();

        return Err(Error::new_spanned(
            &fields[0].name,
            format!(
                "Circular dependency detected in DI container involving: {:?}",
                remaining
            ),
        ));
    }

    Ok((result, forward_graph))
}

/// Find dependencies for a given field by analyzing type relationships
fn find_dependencies<'a>(
    ty: &Type,
    _field_name: &str,
    all_fields: &'a [FieldInfo],
) -> Vec<&'a FieldInfo> {
    let mut deps = Vec::new();

    let current_type_str = quote!(#ty).to_string();
    let current_pos = all_fields.iter().position(|f| {
        let fty = &f.ty;
        quote!(#fty).to_string() == current_type_str
    });

    if let Some(pos) = current_pos {
        if pos == 0 {
            return deps;
        }

        let current_type_base = current_type_str
            .split('<')
            .next()
            .unwrap_or(&current_type_str)
            .split("::")
            .last()
            .unwrap_or(&current_type_str);

        for prev_field in &all_fields[..pos] {
            let prev_type_str = {
                let t = &prev_field.ty;
                quote!(#t).to_string()
            };

            let prev_type_base = prev_type_str
                .split('<')
                .next()
                .unwrap_or(&prev_type_str)
                .split("::")
                .last()
                .unwrap_or(&prev_type_str);

            let current_lower = current_type_base.to_lowercase();
            let prev_lower = prev_type_base.to_lowercase();

            let has_common_prefix = current_lower
                .starts_with(&prev_lower[..prev_lower.len().min(4)])
                || prev_lower.starts_with(&current_lower[..current_lower.len().min(4)]);

            let is_known_pattern = (current_lower.contains("service")
                && prev_lower.contains("repository"))
                || (current_lower.contains("repository") && prev_lower.contains("database"))
                || (current_lower.contains("repository")
                    && (prev_lower.contains("database") || prev_lower.contains("box")))
                || (current_lower.contains("controller") && prev_lower.contains("service"));

            if has_common_prefix || is_known_pattern {
                deps.push(prev_field);
            }
        }
    }

    deps
}
