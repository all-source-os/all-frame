//! HealthCheck derive macro for automatic health check implementation
//!
//! Generates implementations of the HealthCheck trait by collecting
//! all fields that implement the Dependency trait.

use proc_macro2::TokenStream;
use quote::quote;
use syn::{parse2, Data, DeriveInput, Error, Fields, Result};

/// Configuration for health check fields
#[derive(Default)]
struct HealthFieldConfig {
    /// Whether this dependency is critical
    critical: Option<bool>,
    /// Timeout in seconds
    timeout_secs: Option<u64>,
    /// Skip this field from health checks
    skip: bool,
}

/// Parse health field attributes
fn parse_health_attrs(attrs: &[syn::Attribute]) -> Result<HealthFieldConfig> {
    let mut config = HealthFieldConfig::default();

    for attr in attrs {
        if attr.path().is_ident("health") {
            // Parse #[health(...)] attributes
            attr.parse_nested_meta(|meta| {
                if meta.path.is_ident("skip") {
                    config.skip = true;
                    Ok(())
                } else if meta.path.is_ident("critical") {
                    if meta.input.peek(syn::Token![=]) {
                        let _: syn::Token![=] = meta.input.parse()?;
                        let lit: syn::LitBool = meta.input.parse()?;
                        config.critical = Some(lit.value());
                    } else {
                        config.critical = Some(true);
                    }
                    Ok(())
                } else if meta.path.is_ident("timeout") {
                    let _: syn::Token![=] = meta.input.parse()?;
                    let lit: syn::LitStr = meta.input.parse()?;
                    let timeout_str = lit.value();
                    // Parse timeout like "5s", "10s"
                    let secs = parse_timeout(&timeout_str)
                        .map_err(|e| syn::Error::new_spanned(&lit, e))?;
                    config.timeout_secs = Some(secs);
                    Ok(())
                } else {
                    Err(meta.error("unknown health attribute"))
                }
            })?;
        }
    }

    Ok(config)
}

/// Parse a timeout string like "5s" or "10s" into seconds
fn parse_timeout(s: &str) -> std::result::Result<u64, String> {
    let s = s.trim();
    if let Some(secs) = s.strip_suffix('s') {
        secs.parse::<u64>()
            .map_err(|_| format!("invalid timeout: {}", s))
    } else {
        s.parse::<u64>()
            .map_err(|_| format!("invalid timeout: {} (use format like \"5s\")", s))
    }
}

/// Implement the HealthCheck derive macro
pub fn health_check_impl(input: TokenStream) -> Result<TokenStream> {
    let input = parse2::<DeriveInput>(input)?;
    let name = &input.ident;
    let generics = &input.generics;
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();

    let fields = match &input.data {
        Data::Struct(data) => match &data.fields {
            Fields::Named(fields) => &fields.named,
            Fields::Unnamed(_) => {
                return Err(Error::new_spanned(
                    &input,
                    "HealthCheck can only be derived for structs with named fields",
                ))
            }
            Fields::Unit => {
                return Err(Error::new_spanned(
                    &input,
                    "HealthCheck cannot be derived for unit structs",
                ))
            }
        },
        _ => {
            return Err(Error::new_spanned(
                &input,
                "HealthCheck can only be derived for structs",
            ))
        }
    };

    // Collect fields that should be included in health checks
    let mut dependency_refs = Vec::new();
    let mut wrapper_structs = Vec::new();
    let mut wrapper_field_inits = Vec::new();

    for field in fields {
        let field_name = field.ident.as_ref().unwrap();
        let config = parse_health_attrs(&field.attrs)?;

        if config.skip {
            continue;
        }

        // Generate a wrapper if timeout or critical is customized
        if config.timeout_secs.is_some() || config.critical.is_some() {
            let wrapper_name =
                syn::Ident::new(&format!("__{}HealthWrapper", field_name), field_name.span());

            let timeout_impl = if let Some(secs) = config.timeout_secs {
                quote! {
                    fn timeout(&self) -> ::std::time::Duration {
                        ::std::time::Duration::from_secs(#secs)
                    }
                }
            } else {
                quote! {}
            };

            let critical_impl = if let Some(critical) = config.critical {
                quote! {
                    fn is_critical(&self) -> bool {
                        #critical
                    }
                }
            } else {
                quote! {}
            };

            wrapper_structs.push(quote! {
                struct #wrapper_name<'a, T: ::allframe_core::health::Dependency>(&'a T);

                impl<'a, T: ::allframe_core::health::Dependency> ::allframe_core::health::Dependency for #wrapper_name<'a, T> {
                    fn name(&self) -> &str {
                        self.0.name()
                    }

                    fn check(&self) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::allframe_core::health::DependencyStatus> + Send + '_>> {
                        self.0.check()
                    }

                    #timeout_impl
                    #critical_impl
                }
            });

            wrapper_field_inits.push(quote! {
                ::std::sync::Arc::new(#wrapper_name(&self.#field_name))
            });
        } else {
            // No wrapper needed, use field directly
            dependency_refs.push(quote! {
                ::std::sync::Arc::new(FieldRef(&self.#field_name))
            });
        }
    }

    // Generate the implementation
    let all_deps: Vec<_> = dependency_refs
        .into_iter()
        .chain(wrapper_field_inits)
        .collect();

    let expanded = quote! {
        // Helper struct to wrap field references
        struct FieldRef<'a, T: ::allframe_core::health::Dependency>(&'a T);

        impl<'a, T: ::allframe_core::health::Dependency> ::allframe_core::health::Dependency for FieldRef<'a, T> {
            fn name(&self) -> &str {
                self.0.name()
            }

            fn check(&self) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::allframe_core::health::DependencyStatus> + Send + '_>> {
                self.0.check()
            }

            fn is_critical(&self) -> bool {
                self.0.is_critical()
            }

            fn timeout(&self) -> ::std::time::Duration {
                self.0.timeout()
            }
        }

        #(#wrapper_structs)*

        impl #impl_generics ::allframe_core::health::HealthCheck for #name #ty_generics #where_clause {
            fn dependencies(&self) -> ::std::vec::Vec<::std::sync::Arc<dyn ::allframe_core::health::Dependency>> {
                vec![#(#all_deps),*]
            }

            fn check_all(&self) -> ::std::pin::Pin<::std::boxed::Box<dyn ::std::future::Future<Output = ::allframe_core::health::HealthReport> + Send + '_>> {
                ::std::boxed::Box::pin(async move {
                    let start = ::std::time::Instant::now();
                    let deps = <Self as ::allframe_core::health::HealthCheck>::dependencies(self);
                    let mut results = ::std::vec::Vec::new();
                    let mut overall = ::allframe_core::health::OverallStatus::Healthy;

                    for dep in &deps {
                        let dep_start = ::std::time::Instant::now();
                        let name = dep.name().to_string();
                        let is_critical = dep.is_critical();
                        let timeout = dep.timeout();

                        let status = match ::tokio::time::timeout(timeout, dep.check()).await {
                            Ok(status) => status,
                            Err(_) => ::allframe_core::health::DependencyStatus::Unhealthy(
                                "timeout".to_string()
                            ),
                        };

                        let duration = dep_start.elapsed();

                        // Update overall status
                        match &status {
                            ::allframe_core::health::DependencyStatus::Unhealthy(_) if is_critical => {
                                overall = ::allframe_core::health::OverallStatus::Unhealthy;
                            }
                            ::allframe_core::health::DependencyStatus::Degraded(_) if overall != ::allframe_core::health::OverallStatus::Unhealthy => {
                                overall = ::allframe_core::health::OverallStatus::Degraded;
                            }
                            ::allframe_core::health::DependencyStatus::Unhealthy(_) if overall == ::allframe_core::health::OverallStatus::Healthy => {
                                overall = ::allframe_core::health::OverallStatus::Degraded;
                            }
                            _ => {}
                        }

                        results.push(::allframe_core::health::DependencyReport {
                            name,
                            status,
                            duration,
                            critical: is_critical,
                        });
                    }

                    ::allframe_core::health::HealthReport {
                        status: overall,
                        dependencies: results,
                        total_duration: start.elapsed(),
                        timestamp: ::std::time::SystemTime::now(),
                    }
                })
            }
        }
    };

    Ok(expanded)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_timeout() {
        assert_eq!(parse_timeout("5s").unwrap(), 5);
        assert_eq!(parse_timeout("10s").unwrap(), 10);
        assert_eq!(parse_timeout(" 5s ").unwrap(), 5);
        assert!(parse_timeout("invalid").is_err());
    }
}
