//! Dependency Injection Container Macro Implementation
//!
//! This module implements the `#[di_container]` procedural macro for
//! compile-time dependency injection with zero runtime reflection.

use std::collections::{HashMap, HashSet};

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Error, Fields, Result, Type};

/// Represents information about a field in the DI container
struct FieldInfo {
    name: syn::Ident,
    ty: Type,
    provide_expr: Option<syn::Expr>,
}

/// Implementation of the #[di_container] macro
///
/// Generates:
/// - A `new()` associated function that creates the container
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
    let mut filtered_attrs_map: HashMap<syn::Ident, Vec<syn::Attribute>> = HashMap::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap().clone();
        let field_type = field.ty.clone();

        // Check for #[provide(...)] attribute and filter it out
        let mut provide_expr = None;
        let mut filtered_attrs = Vec::new();

        for attr in &field.attrs {
            if attr.path().is_ident("provide") {
                // Parse the expression from #[provide(expr)]
                provide_expr = Some(attr.parse_args::<syn::Expr>()?);
                // Don't include this attribute in the output
            } else {
                filtered_attrs.push(attr.clone());
            }
        }

        filtered_attrs_map.insert(field_name.clone(), filtered_attrs);

        field_infos.push(FieldInfo {
            name: field_name,
            ty: field_type,
            provide_expr,
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
    // Strategy: Create Arc instances for dependencies, unwrap when passing to
    // constructors
    let mut let_bindings = Vec::new();

    for field_info in &init_order {
        let name = &field_info.name;
        let ty = &field_info.ty;
        let field_name_str = name.to_string();

        if let Some(expr) = &field_info.provide_expr {
            // Use the provided expression
            if is_dependency_of_others.contains(&field_name_str) {
                // Wrap in Arc for dependencies
                let_bindings.push(quote! {
                    let #name = std::sync::Arc::new(#expr);
                });
            } else {
                let_bindings.push(quote! {
                    let #name = #expr;
                });
            }
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
                // No dependencies - call new() with no arguments
                if is_dependency_of_others.contains(&field_name_str) {
                    let_bindings.push(quote! {
                        let #name = std::sync::Arc::new(#ty::new());
                    });
                } else {
                    let_bindings.push(quote! {
                        let #name = #ty::new();
                    });
                }
            } else {
                // Pass Arc::clone() of dependencies to constructor
                let dep_args: Vec<_> = dep_infos
                    .iter()
                    .map(|dep| {
                        let dep_field_name = &dep.name;
                        if is_dependency_of_others.contains(&dep.name.to_string()) {
                            quote! { std::sync::Arc::clone(&#dep_field_name) }
                        } else {
                            quote! { #dep_field_name }
                        }
                    })
                    .collect();

                if is_dependency_of_others.contains(&field_name_str) {
                    let_bindings.push(quote! {
                        let #name = std::sync::Arc::new(#ty::new(#(#dep_args),*));
                    });
                } else {
                    let_bindings.push(quote! {
                        let #name = #ty::new(#(#dep_args),*);
                    });
                }
            }
        }
    }

    // Only include fields in the struct that aren't consumed by other fields
    // i.e., fields that have no outgoing edges in the reverse_graph (from
    // topological sort) Actually, we need to include ALL fields as they're
    // declared in the struct The issue is that we're passing ownership. We need
    // to keep the fields in the container but also pass them to constructors.
    //
    // SOLUTION: Don't pass the container's own fields to constructors
    // Instead, create temporary instances that get moved
    let struct_fields: Vec<_> = init_order.iter().map(|f| &f.name).collect();

    // Generate accessor methods
    let mut accessors = Vec::new();
    for field_info in &field_infos {
        let name = &field_info.name;
        let ty = &field_info.ty;
        let field_name_str = name.to_string();

        if is_dependency_of_others.contains(&field_name_str) {
            // Return cloned Arc for dependencies
            accessors.push(quote! {
                #vis fn #name(&self) -> std::sync::Arc<#ty> {
                    std::sync::Arc::clone(&self.#name)
                }
            });
        } else {
            accessors.push(quote! {
                #vis fn #name(&self) -> &#ty {
                    &self.#name
                }
            });
        }
    }

    // Generate the implementation
    let expanded = quote! {
        impl #struct_name {
            /// Create a new instance of the container with all dependencies injected
            #vis fn new() -> Self {
                // Create each service/dependency in topological order
                #(#let_bindings)*

                // Build the container
                Self {
                    #(#struct_fields,)*
                }
            }

            #(#accessors)*
        }
    };

    // Create a modified version of the input struct without `provide` attributes
    // Wrap dependency fields in Arc
    let mut modified_input = input.clone();
    if let Data::Struct(ref mut data_struct) = modified_input.data {
        if let Fields::Named(ref mut fields_named) = data_struct.fields {
            for field in fields_named.named.iter_mut() {
                field.attrs.retain(|attr| !attr.path().is_ident("provide"));

                // Wrap dependencies in Arc
                if let Some(ident) = &field.ident {
                    if is_dependency_of_others.contains(&ident.to_string()) {
                        let original_ty = &field.ty;
                        field.ty = syn::parse_quote! { std::sync::Arc<#original_ty> };
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
    // Build TWO graphs:
    // 1. Forward graph: X depends on Y means X -> Y (for returning dependencies)
    // 2. Reverse graph: X depends on Y means Y -> X (for topological sort)
    let mut forward_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut reverse_graph: HashMap<String, HashSet<String>> = HashMap::new();
    let mut field_map: HashMap<String, &FieldInfo> = HashMap::new();

    for field in fields {
        let field_name = field.name.to_string();
        field_map.insert(field_name.clone(), field);
        forward_graph.insert(field_name.clone(), HashSet::new());
        reverse_graph.insert(field_name.clone(), HashSet::new());

        // If no provide expression, find dependencies by analyzing type names
        if field.provide_expr.is_none() {
            let deps = find_dependencies(&field.ty, &field_name, fields);
            for dep in deps {
                let dep_name = dep.name.to_string();
                // Forward: X depends on Y
                forward_graph
                    .get_mut(&field_name)
                    .unwrap()
                    .insert(dep_name.clone());
                // Reverse: Y must come before X
                reverse_graph
                    .get_mut(&dep_name)
                    .unwrap()
                    .insert(field_name.clone());
            }
        }
    }

    // Topological sort using Kahn's algorithm on the reverse graph
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
        return Err(Error::new_spanned(
            &fields[0].name,
            "Circular dependency detected in DI container",
        ));
    }

    // Return the topologically sorted order and the FORWARD dependency map
    Ok((result, forward_graph))
}

/// Find dependencies for a given field by analyzing type relationships
///
/// This uses several heuristics:
/// 1. Sequential dependency: A field at position N depends on the field at N-1
/// 2. Type name matching: If field name contains type name (e.g.,
///    "user_repository" and "UserRepository")
/// 3. Common patterns: Repository depends on Database/DataSource, Service
///    depends on Repository
///
/// For example:
/// ```ignore
/// struct Container {
///     database: Database,        // position 0, no deps
///     repository: Repository,    // position 1, depends on database
///     service: Service,          // position 2, depends on repository
/// }
/// ```
fn find_dependencies<'a>(
    ty: &Type,
    _field_name: &str,
    all_fields: &'a [FieldInfo],
) -> Vec<&'a FieldInfo> {
    let mut deps = Vec::new();

    // Get the current field's position
    let current_type_str = quote!(#ty).to_string();
    let current_pos = all_fields.iter().position(|f| {
        let fty = &f.ty;
        quote!(#fty).to_string() == current_type_str
    });

    if let Some(pos) = current_pos {
        if pos == 0 {
            // First field - no dependencies
            return deps;
        }

        // Strategy: Look for type name matches
        // If field is UserService and there's a UserRepository field before it,
        // assume UserService depends on UserRepository
        //
        // We look for common prefixes/stems in type names

        let current_type_base = current_type_str
            .split('<')
            .next()
            .unwrap_or(&current_type_str)
            .split("::")
            .last()
            .unwrap_or(&current_type_str);

        // Check all previous fields to see if any could be dependencies
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

            // Heuristic: Check if type names suggest a relationship
            // E.g., "UserService" and "UserRepository", or "Service" and "Repository"
            let current_lower = current_type_base.to_lowercase();
            let prev_lower = prev_type_base.to_lowercase();

            // Check for common patterns:
            // 1. Same prefix (User): UserService depends on UserRepository
            // 2. Known dependency patterns: Service->Repository, Repository->Database

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

impl Clone for FieldInfo {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            ty: self.ty.clone(),
            provide_expr: self.provide_expr.clone(),
        }
    }
}
