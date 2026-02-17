//! Saga procedural macros for AllFrame
//!
//! This module provides macros for reducing boilerplate in saga implementations,
//! including step definitions, output extraction, and saga containers.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{
    parse::{Parse, ParseStream},
    punctuated::Punctuated,
    token::Comma,
    Data, DataStruct, DeriveInput, Expr, ExprLit, Field, Fields, FieldsNamed,
    Ident, Lit, Meta, MetaNameValue, Result as SynResult,
};

/// Arguments for the `#[saga_step]` attribute
#[derive(Debug)]
struct SagaStepArgs {
    name: String,
    timeout_seconds: Option<u64>,
    requires_compensation: bool,
}

impl Parse for SagaStepArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut name = None;
        let mut timeout_seconds = None;
        let mut requires_compensation = true; // Default to true

        let parsed = Punctuated::<Meta, Comma>::parse_terminated(input)?;
        for meta in parsed {
            match meta {
                Meta::NameValue(MetaNameValue { path, eq_token: _, value }) => {
                    let ident = path.get_ident().ok_or_else(|| {
                        syn::Error::new_spanned(&path, "Expected identifier")
                    })?;

                    match ident.to_string().as_str() {
                        "name" => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value {
                                name = Some(lit_str.value());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "Expected string literal for name"
                                ));
                            }
                        }
                        "timeout_seconds" => {
                            if let Expr::Lit(ExprLit { lit: Lit::Int(lit_int), .. }) = value {
                                timeout_seconds = Some(lit_int.base10_parse()?);
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "Expected integer literal for timeout_seconds"
                                ));
                            }
                        }
                        "requires_compensation" => {
                            if let Expr::Lit(ExprLit { lit: Lit::Bool(lit_bool), .. }) = value {
                                requires_compensation = lit_bool.value();
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "Expected boolean literal for requires_compensation"
                                ));
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "Unknown attribute key"
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &meta,
                        "Expected name=value pairs"
                    ));
                }
            }
        }

        let name = name.ok_or_else(|| {
            syn::Error::new(input.span(), "Missing required 'name' attribute")
        })?;

        Ok(SagaStepArgs {
            name,
            timeout_seconds,
            requires_compensation,
        })
    }
}

/// Arguments for the `#[saga]` attribute
#[derive(Debug)]
struct SagaArgs {
    name: Option<String>,
    data_field: Option<String>,
}

/// Arguments for the `#[saga_workflow]` attribute
#[derive(Debug)]
struct SagaWorkflowArgs {
    saga_type: String,
}

impl Parse for SagaArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let mut name = None;
        let mut data_field = None;

        let parsed = Punctuated::<Meta, Comma>::parse_terminated(input)?;
        for meta in parsed {
            match meta {
                Meta::NameValue(MetaNameValue { path, eq_token: _, value }) => {
                    let ident = path.get_ident().ok_or_else(|| {
                        syn::Error::new_spanned(&path, "Expected identifier")
                    })?;

                    match ident.to_string().as_str() {
                        "name" => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value {
                                name = Some(lit_str.value());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "Expected string literal for name"
                                ));
                            }
                        }
                        "data_field" => {
                            if let Expr::Lit(ExprLit { lit: Lit::Str(lit_str), .. }) = value {
                                data_field = Some(lit_str.value());
                            } else {
                                return Err(syn::Error::new_spanned(
                                    &value,
                                    "Expected string literal for data_field"
                                ));
                            }
                        }
                        _ => {
                            return Err(syn::Error::new_spanned(
                                ident,
                                "Unknown attribute key"
                            ));
                        }
                    }
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        &meta,
                        "Expected name=value pairs"
                    ));
                }
            }
        }

        Ok(SagaArgs { name, data_field })
    }
}

impl Parse for SagaWorkflowArgs {
    fn parse(input: ParseStream) -> SynResult<Self> {
        let saga_type: Ident = input.parse()?;
        Ok(SagaWorkflowArgs {
            saga_type: saga_type.to_string(),
        })
    }
}

/// Generate Debug impl that skips #[inject] fields
fn generate_debug_impl(struct_name: &Ident, fields: &FieldsNamed) -> TokenStream {
    let debug_fields = fields.named.iter().filter_map(|field| {
        // Check if field has #[inject] attribute
        let has_inject = field.attrs.iter().any(|attr| {
            attr.path().is_ident("inject")
        });

        if has_inject {
            // Skip inject fields in debug output
            None
        } else {
            let field_name = field.ident.as_ref()?;
            Some(quote! {
                .field(stringify!(#field_name), &self.#field_name)
            })
        }
    });

    quote! {
        impl std::fmt::Debug for #struct_name {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(stringify!(#struct_name))
                    #(#debug_fields)*
                    .finish()
            }
        }
    }
}

/// Generate SagaStep trait implementation
fn generate_saga_step_impl(
    struct_name: &Ident,
    args: &SagaStepArgs,
) -> TokenStream {
    let step_name = &args.name;
    let timeout_seconds = args.timeout_seconds.unwrap_or(30);
    let requires_compensation = args.requires_compensation;

    quote! {
        #[async_trait::async_trait]
        impl allframe_core::cqrs::MacroSagaStep for #struct_name {
            fn name(&self) -> &str {
                #step_name
            }

            fn timeout_seconds(&self) -> u64 {
                #timeout_seconds
            }

            fn requires_compensation(&self) -> bool {
                #requires_compensation
            }

            async fn execute(&self, ctx: &allframe_core::cqrs::SagaContext) -> allframe_core::cqrs::StepExecutionResult {
                // This will be implemented by the user in a separate impl block
                todo!("Implement execute method in separate impl block")
            }

            async fn compensate(&self, ctx: &allframe_core::cqrs::SagaContext) -> allframe_core::cqrs::CompensationResult {
                // This will be implemented by the user in a separate impl block
                if #requires_compensation {
                    todo!("Implement compensate method in separate impl block")
                } else {
                    allframe_core::cqrs::CompensationResult::not_needed()
                }
            }
        }
    }
}

/// Extract #[inject] fields and saga_data field from struct
fn extract_special_fields<'a>(args: &SagaArgs, fields: &'a FieldsNamed) -> (Vec<&'a Field>, Option<&'a Field>) {
    let mut inject_fields = Vec::new();
    let mut saga_data_field = None;

    for field in &fields.named {
        let field_name = field.ident.as_ref().unwrap().to_string();
        let has_inject = field.attrs.iter().any(|attr| {
            attr.path().segments.len() == 1 && attr.path().segments[0].ident == "inject"
        });

        // Check if this field is the saga data field (either by attribute or by parameter)
        let is_saga_data = field.attrs.iter().any(|attr| {
            attr.path().segments.len() == 1 && attr.path().segments[0].ident == "saga_data"
        }) || args.data_field.as_ref().map(|df| df == &field_name).unwrap_or(false);

        if has_inject {
            inject_fields.push(field);
        }
        if is_saga_data {
            if saga_data_field.is_some() {
                panic!("Only one field can be marked as saga data");
            }
            saga_data_field = Some(field);
        }
    }

    // If no saga data field specified, assume the first non-inject field is saga data
    if saga_data_field.is_none() && !fields.named.is_empty() {
        for field in &fields.named {
            let has_inject = field.attrs.iter().any(|attr| {
                attr.path().segments.len() == 1 && attr.path().segments[0].ident == "inject"
            });
            if !has_inject {
                saga_data_field = Some(field);
                break;
            }
        }
    }

    (inject_fields, saga_data_field)
}

/// Generate Saga trait implementation for saga containers
fn generate_saga_impl(
    struct_name: &Ident,
    args: &SagaArgs,
    _inject_fields: &[&Field],
    saga_data_field: Option<&Field>,
) -> TokenStream {
    let saga_name = args.name.as_ref()
        .cloned()
        .unwrap_or_else(|| struct_name.to_string());

    let saga_data_access = if let Some(field) = saga_data_field {
        let field_name = field.ident.as_ref().unwrap();
        quote! { &self.#field_name }
    } else {
        quote! { panic!("No #[saga_data] field found") }
    };

    let user_id_access = if let Some(field) = saga_data_field {
        let field_name = field.ident.as_ref().unwrap();
        quote! { &self.#field_name.user_id }
    } else {
        quote! { panic!("No #[saga_data] field found") }
    };

    // Generate steps implementation
    // Use workflow_steps method if available (from #[saga_workflow])
    let steps_impl = quote! {
        self.workflow_steps().unwrap_or_else(|| {
            // Fallback: create steps from inject fields
            vec![
                // TODO: Auto-create steps from inject fields
                // For now, manual implementation required
            ]
        })
    };

    // Generate Saga trait implementation
    // Assumes Saga trait is available at runtime (from allframe-core)
    quote! {
        impl #struct_name {
            // Default implementation - returns None if no workflow defined
            // Can be overridden by #[saga_workflow] macro
            fn workflow_steps(&self) -> Option<Vec<std::sync::Arc<dyn allframe_core::cqrs::MacroSagaStep>>> {
                None
            }
        }

        #[async_trait::async_trait]
        impl allframe_core::cqrs::Saga for #struct_name {
            fn saga_type(&self) -> &'static str {
                #saga_name
            }

            fn steps(&self) -> Vec<std::sync::Arc<dyn allframe_core::cqrs::MacroSagaStep>> {
                #steps_impl
            }

            fn initial_data(&self) -> serde_json::Value {
                serde_json::to_value(#saga_data_access).unwrap_or_default()
            }

            fn user_id(&self) -> &str {
                #user_id_access
            }
        }
    }
}

/// Generate constructor for saga container
fn generate_saga_constructor(
    struct_name: &Ident,
    inject_fields: &[&Field],
    saga_data_field: Option<&Field>,
) -> TokenStream {
    let inject_params = inject_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        quote! { #field_name: #field_type }
    });

    let inject_assignments = inject_fields.iter().map(|field| {
        let field_name = field.ident.as_ref().unwrap();
        quote! { #field_name }
    });

    let saga_data_param = if let Some(field) = saga_data_field {
        let field_name = field.ident.as_ref().unwrap();
        let field_type = &field.ty;
        quote! { #field_name: #field_type }
    } else {
        quote! { data: serde_json::Value }
    };

    let saga_data_assignment = if let Some(field) = saga_data_field {
        let field_name = field.ident.as_ref().unwrap();
        quote! { #field_name }
    } else {
        quote! { data }
    };

    quote! {
        impl #struct_name {
            pub fn new(#saga_data_param, #(#inject_params),*) -> Self {
                Self {
                    #saga_data_assignment,
                    #(#inject_assignments),*
                }
            }
        }
    }
}

/// Implementation of `#[saga_step]` attribute macro
pub fn saga_step_impl(attr: TokenStream, item: TokenStream) -> SynResult<TokenStream> {
    let args = syn::parse2::<SagaStepArgs>(attr)?;
    let input = syn::parse2::<syn::ItemStruct>(item)?;

    let struct_name = &input.ident;
    let fields = match &input.fields {
        Fields::Named(fields) => fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input.fields,
                "SagaStep structs must have named fields"
            ));
        }
    };

    let debug_impl = generate_debug_impl(struct_name, fields);
    let saga_step_impl = generate_saga_step_impl(struct_name, &args);

    Ok(quote! {
        #input
        #debug_impl
        #saga_step_impl
    })
}

/// Implementation of `#[saga]` attribute macro
pub fn saga_impl(attr: TokenStream, item: TokenStream) -> SynResult<TokenStream> {
    let args = syn::parse2::<SagaArgs>(attr)?;
    let input: syn::ItemStruct = syn::parse2(item)?;

    let struct_name = &input.ident;

    // Extract field information
    let (inject_fields, saga_data_field) = match &input.fields {
        Fields::Named(fields) => extract_special_fields(&args, fields),
        _ => {
            return Err(syn::Error::new_spanned(
                &input.fields,
                "Saga structs must have named fields"
            ));
        }
    };

    // For now, just use the input as-is (TODO: clean attributes)

    let constructor = generate_saga_constructor(struct_name, &inject_fields, saga_data_field);
    let saga_impl = generate_saga_impl(struct_name, &args, &inject_fields, saga_data_field);

    Ok(quote! {
        #input
        #constructor
        #saga_impl
    })
}

/// Implementation of `#[derive(StepOutput)]` derive macro
pub fn derive_step_output(input: TokenStream) -> SynResult<TokenStream> {
    let input = syn::parse2::<DeriveInput>(input)?;
    let struct_name = &input.ident;

    let fields = match &input.data {
        Data::Struct(DataStruct { fields: Fields::Named(fields), .. }) => fields,
        _ => {
            return Err(syn::Error::new_spanned(
                &input,
                "StepOutput can only be derived for structs with named fields"
            ));
        }
    };

    let field_names: Vec<_> = fields.named.iter().map(|field| {
        field.ident.as_ref().unwrap()
    }).collect();

    Ok(quote! {
        impl allframe_core::cqrs::StepOutput for #struct_name {
            fn from_context(ctx: &allframe_core::cqrs::SagaContext, step_name: &str) -> Result<Self, allframe_core::cqrs::SagaError> {
                let value = ctx.step_outputs.get(step_name)
                    .ok_or_else(|| allframe_core::cqrs::SagaError::StepOutputNotFound {
                        step_name: step_name.to_string()
                    })?;

                let #struct_name { #(#field_names),* } = serde_json::from_value(value.clone())
                    .map_err(|e| allframe_core::cqrs::SagaError::StepOutputParse {
                        step_name: step_name.to_string(),
                        error: e.to_string()
                    })?;

                Ok(#struct_name { #(#field_names),* })
            }
        }

        impl From<#struct_name> for allframe_core::cqrs::StepExecutionResult {
            fn from(output: #struct_name) -> allframe_core::cqrs::StepExecutionResult {
                allframe_core::cqrs::StepExecutionResult::Success {
                    output: Some(serde_json::to_value(output).unwrap_or_default())
                }
            }
        }
    })
}

/// Implementation of `#[saga_workflow]` attribute macro
pub fn saga_workflow_impl(attr: TokenStream, item: TokenStream) -> SynResult<TokenStream> {
    let args = syn::parse2::<SagaWorkflowArgs>(attr)?;
    let input = syn::parse2::<syn::ItemEnum>(item)?;

    let saga_type = &args.saga_type;
    let _enum_name = &input.ident;

    // Create step constructor calls that take the saga as parameter
    // This allows steps to extract dependencies from the saga
    let step_constructors = input.variants.iter().map(|variant| {
        let step_name = &variant.ident;
        // Convert enum variant to step constructor function name
        let constructor_name = format!("create_{}_step", to_snake_case(&step_name.to_string()));
        let constructor_ident = syn::Ident::new(&constructor_name, variant.ident.span());

        quote! {
            self.#constructor_ident()
        }
    });

    let steps_impl = quote! {
        vec![
            #(#step_constructors,)*
        ]
    };

    // Generate an impl block for the saga type that provides the steps
    let saga_ident = syn::Ident::new(saga_type, input.ident.span());

    // Also generate the step constructor methods
    let step_constructor_methods = input.variants.iter().map(|variant| {
        let step_name = &variant.ident;
        let constructor_name = format!("create_{}_step", to_snake_case(&step_name.to_string()));
        let constructor_ident = syn::Ident::new(&constructor_name, variant.ident.span());

        let step_struct_name = format!("{}Step", step_name);
        let step_struct_ident = syn::Ident::new(&step_struct_name, variant.ident.span());

        quote! {
            fn #constructor_ident(&self) -> std::sync::Arc<dyn allframe_core::cqrs::MacroSagaStep> {
                // User must implement this method to create the step with proper dependencies
                todo!("Implement {} to create {} with dependencies from saga", stringify!(#constructor_ident), stringify!(#step_struct_ident))
            }
        }
    });

    Ok(quote! {
        #input

        impl #saga_ident {
            #(#step_constructor_methods)*

            fn workflow_steps(&self) -> Option<Vec<std::sync::Arc<dyn allframe_core::cqrs::MacroSagaStep>>> {
                Some(#steps_impl)
            }
        }
    })
}

/// Convert PascalCase to snake_case
fn to_snake_case(s: &str) -> String {
    let mut result = String::new();
    for (i, ch) in s.char_indices() {
        if ch.is_uppercase() && i > 0 {
            result.push('_');
        }
        result.push(ch.to_lowercase().next().unwrap());
    }
    result
}

/// Implementation of `extract_output!` macro
///
/// NOTE: This is a proc-macro crate, so we can't define declarative macros.
/// Instead, this provides a compile error directing users to use the StepOutput trait.
///
/// For ergonomic output extraction, use:
/// - `MyOutputType::from_context(ctx, "StepName")?)` for typed extraction
/// - `ctx.get_step_output("StepName")` for raw JSON access
pub fn extract_output_impl(_input: TokenStream) -> SynResult<TokenStream> {
    Ok(quote! {
        compile_error!("extract_output! is not available in proc-macro context. Use StepOutput::from_context() for type-safe extraction or ctx.get_step_output() for raw access");
    })
}

