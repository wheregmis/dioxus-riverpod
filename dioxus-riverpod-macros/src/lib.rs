#![allow(unused_variables)] // Variables used in quote! macros aren't detected by compiler

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use std::time::Duration;
use syn::{
    FnArg, ItemFn, LitStr, Pat, PatType, Result, ReturnType, Token, Type, parse::Parse,
    parse::ParseStream, parse_macro_input,
};

/// Attribute arguments for the provider macro
#[derive(Default)]
struct ProviderArgs {
    interval: Option<Duration>,
    cache_expiration: Option<Duration>,
    stale_time: Option<Duration>,
    auto_dispose: Option<bool>,
    dispose_delay: Option<Duration>,
    inject: Vec<syn::Type>, // New: list of types to inject
}

impl Parse for ProviderArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = ProviderArgs::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "interval" => {
                    let lit: LitStr = input.parse()?;
                    let duration_str = lit.value();
                    let duration = humantime::parse_duration(&duration_str).map_err(|e| {
                        syn::Error::new_spanned(lit, format!("Invalid duration format: {}", e))
                    })?;
                    args.interval = Some(duration);
                }
                "cache_expiration" => {
                    let lit: LitStr = input.parse()?;
                    let duration_str = lit.value();
                    let duration = humantime::parse_duration(&duration_str).map_err(|e| {
                        syn::Error::new_spanned(lit, format!("Invalid duration format: {}", e))
                    })?;
                    args.cache_expiration = Some(duration);
                }
                "stale_time" => {
                    let lit: LitStr = input.parse()?;
                    let duration_str = lit.value();
                    let duration = humantime::parse_duration(&duration_str).map_err(|e| {
                        syn::Error::new_spanned(lit, format!("Invalid duration format: {}", e))
                    })?;
                    args.stale_time = Some(duration);
                }
                "auto_dispose" => {
                    let lit: syn::LitBool = input.parse()?;
                    args.auto_dispose = Some(lit.value);
                }
                "dispose_delay" => {
                    let lit: LitStr = input.parse()?;
                    let duration_str = lit.value();
                    let duration = humantime::parse_duration(&duration_str).map_err(|e| {
                        syn::Error::new_spanned(lit, format!("Invalid duration format: {}", e))
                    })?;
                    args.dispose_delay = Some(duration);
                }
                "inject" => {
                    // Parse injection types: inject = [Type1, Type2, ...]
                    let content;
                    syn::bracketed!(content in input);
                    let types = content.parse_terminated(syn::Type::parse, Token![,])?;
                    args.inject = types.into_iter().collect();
                }
                _ => return Err(syn::Error::new_spanned(ident, "Unknown argument")),
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>()?;
            }
        }

        Ok(args)
    }
}

/// Unified attribute macro for creating providers
/// Automatically detects provider type based on function parameters:
/// - No parameters → Future Provider  
/// - Has parameters → Family Provider
///
/// Supports humantime duration syntax for all timing parameters:
/// - #[provider(interval = "5s")] - refresh every 5 seconds
/// - #[provider(interval = "1min")] - refresh every minute  
/// - #[provider(interval = "30sec")] - refresh every 30 seconds
///
/// Cache expiration with humantime:
/// - #[provider(cache_expiration = "30s")] - cache expires after 30 seconds
/// - #[provider(cache_expiration = "5min")] - cache expires after 5 minutes
/// - #[provider(cache_expiration = "1h")] - cache expires after 1 hour
///
/// Stale-while-revalidate with humantime:
/// - #[provider(stale_time = "5s")] - serve stale data after 5 seconds, refresh in background
/// - #[provider(stale_time = "30sec")] - serve stale data after 30 seconds, refresh in background
/// - #[provider(stale_time = "2min")] - serve stale data after 2 minutes, refresh in background
///
/// Can combine features:
/// - #[provider(interval = "10s", cache_expiration = "1min")]
/// - #[provider(stale_time = "5s", cache_expiration = "30s")]
///
/// Supported humantime formats:
/// - "5s", "30sec", "2min", "1h", "1day"
/// - "500ms", "1.5s", "2.5min"
#[proc_macro_attribute]
pub fn provider(args: TokenStream, input: TokenStream) -> TokenStream {
    let provider_args = if args.is_empty() {
        ProviderArgs::default()
    } else {
        match syn::parse(args) {
            Ok(args) => args,
            Err(err) => return err.to_compile_error().into(),
        }
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    let result = generate_provider(input_fn, provider_args);

    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_provider(input_fn: ItemFn, provider_args: ProviderArgs) -> Result<TokenStream2> {
    let info = extract_provider_info(&input_fn)?;

    let ProviderInfo {
        fn_vis,
        fn_block,
        output_type,
        error_type,
        struct_name,
        ..
    } = &info;

    // Generate enhanced function body with dependency injection
    let enhanced_fn_block = generate_dependency_injection(&provider_args.inject, fn_block);

    // Generate interval and cache expiration implementations
    let interval_impl = generate_interval_impl(&provider_args);
    let cache_expiration_impl = generate_cache_expiration_impl(&provider_args);
    let stale_time_impl = generate_stale_time_impl(&provider_args);
    let auto_dispose_impl = generate_auto_dispose_impl(&provider_args);
    let dispose_delay_impl = generate_dispose_delay_impl(&provider_args);

    // Generate common struct and const
    let common_struct = generate_common_struct_and_const(&info);

    // Determine parameter type and implementation based on function parameters
    if input_fn.sig.inputs.is_empty() {
        // No parameters - Provider<()>
        Ok(quote! {
            #common_struct

            impl #struct_name {
                #fn_vis async fn call() -> Result<#output_type, #error_type> {
                    #enhanced_fn_block
                }
            }

            impl ::dioxus_riverpod::providers::Provider<()> for #struct_name {
                type Output = #output_type;
                type Error = #error_type;

                fn run(&self, _param: ()) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                    Self::call()
                }

                fn id(&self, _param: &()) -> String {
                    stringify!(#struct_name).to_string()
                }

                #interval_impl
                #cache_expiration_impl
                #stale_time_impl
                #auto_dispose_impl
                #dispose_delay_impl
            }
        })
    } else {
        // Has parameters - extract and handle them
        let params = extract_all_params(&input_fn)?;

        if params.len() == 1 {
            // Single parameter - Provider<ParamType>
            let param = &params[0];
            let param_name = &param.name;
            let param_type = &param.ty;

            Ok(quote! {
                #common_struct

                impl #struct_name {
                    #fn_vis async fn call(#param_name: #param_type) -> Result<#output_type, #error_type> {
                        #enhanced_fn_block
                    }
                }

                impl ::dioxus_riverpod::providers::Provider<#param_type> for #struct_name {
                    type Output = #output_type;
                    type Error = #error_type;

                    fn run(&self, #param_name: #param_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                        Self::call(#param_name)
                    }

                    fn id(&self, param: &#param_type) -> String {
                        format!("{}({:?})", stringify!(#struct_name), param)
                    }

                    #interval_impl
                    #cache_expiration_impl
                    #stale_time_impl
                    #auto_dispose_impl
                    #dispose_delay_impl
                }
            })
        } else {
            // Multiple parameters - Provider<(Type1, Type2, ...)>
            let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
            let param_types: Vec<_> = params.iter().map(|p| &p.ty).collect();

            // Generate tuple type with trailing comma for consistency
            let tuple_type = quote! { (#(#param_types,)*) };

            Ok(quote! {
                #common_struct

                impl #struct_name {
                    #fn_vis async fn call(#(#param_names: #param_types),*) -> Result<#output_type, #error_type> {
                        #enhanced_fn_block
                    }
                }

                impl ::dioxus_riverpod::providers::Provider<#tuple_type> for #struct_name {
                    type Output = #output_type;
                    type Error = #error_type;

                    fn run(&self, params: #tuple_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                        let (#(#param_names),*) = params;
                        Self::call(#(#param_names),*)
                    }

                    fn id(&self, params: &#tuple_type) -> String {
                        format!("{}({:?})", stringify!(#struct_name), params)
                    }

                    #interval_impl
                    #cache_expiration_impl
                    #stale_time_impl
                    #auto_dispose_impl
                    #dispose_delay_impl
                }
            })
        }
    }
}

fn generate_duration_impl(method_name: &str, duration: Option<Duration>) -> TokenStream2 {
    match duration {
        Some(duration) => {
            let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            let secs = duration.as_secs();
            let nanos = duration.subsec_nanos();
            quote! {
                fn #method_ident(&self) -> Option<::std::time::Duration> {
                    Some(::std::time::Duration::new(#secs, #nanos))
                }
            }
        }
        None => {
            // No duration specified, use default (None)
            quote! {}
        }
    }
}

fn generate_interval_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("interval", provider_args.interval)
}

fn generate_cache_expiration_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("cache_expiration", provider_args.cache_expiration)
}

fn generate_stale_time_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("stale_time", provider_args.stale_time)
}

fn generate_auto_dispose_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    match provider_args.auto_dispose {
        Some(true) => quote! {
            fn auto_dispose(&self) -> bool {
                true
            }
        },
        Some(false) => quote! {
            fn auto_dispose(&self) -> bool {
                false
            }
        },
        None => quote! {},
    }
}

fn generate_dispose_delay_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("dispose_delay", provider_args.dispose_delay)
}

struct ProviderInfo {
    fn_name: syn::Ident,
    fn_vis: syn::Visibility,
    fn_attrs: Vec<syn::Attribute>,
    fn_block: Box<syn::Block>,
    output_type: Type,
    error_type: Type,
    struct_name: syn::Ident,
}

struct ParamInfo {
    name: syn::Ident,
    ty: Type,
}

fn extract_provider_info(input_fn: &ItemFn) -> Result<ProviderInfo> {
    let fn_name = input_fn.sig.ident.clone();
    let fn_vis = input_fn.vis.clone();
    let fn_attrs = input_fn.attrs.clone();
    let fn_block = input_fn.block.clone();

    // Extract return type from Result<T, E>
    let (output_type, error_type) = extract_result_types(&input_fn.sig.output)?;

    // Generate the provider struct name (convert snake_case to PascalCase + Provider)
    let struct_name = syn::Ident::new(
        &format!("{}Provider", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    Ok(ProviderInfo {
        fn_name,
        fn_vis,
        fn_attrs,
        fn_block,
        output_type,
        error_type,
        struct_name,
    })
}

#[allow(unused_variables)] // Variables used in quote! macro aren't detected by compiler
#[allow(clippy::unused_unit)] // Allow clippy warnings for proc macros
fn generate_common_struct_and_const(info: &ProviderInfo) -> TokenStream2 {
    let ProviderInfo {
        fn_name,
        fn_vis,
        fn_attrs,
        struct_name,
        fn_block: _,
        output_type: _,
        error_type: _,
    } = info;

    quote! {
        #(#fn_attrs)*
        #[derive(Clone, PartialEq)]
        #fn_vis struct #struct_name;

        // Create a constant instance for easy usage
        // Allow snake_case for ergonomic usage while suppressing the naming warning
        #[allow(non_upper_case_globals)]
        #fn_vis const #fn_name: #struct_name = #struct_name;
    }
}

fn extract_all_params(input_fn: &ItemFn) -> Result<Vec<ParamInfo>> {
    if input_fn.sig.inputs.is_empty() {
        return Err(syn::Error::new_spanned(
            &input_fn.sig,
            "Family provider must have at least one parameter",
        ));
    }

    let mut params = Vec::new();

    for arg in &input_fn.sig.inputs {
        match arg {
            FnArg::Typed(PatType { pat, ty, .. }) => match pat.as_ref() {
                Pat::Ident(pat_ident) => {
                    params.push(ParamInfo {
                        name: pat_ident.ident.clone(),
                        ty: (**ty).clone(),
                    });
                }
                _ => {
                    return Err(syn::Error::new_spanned(
                        pat,
                        "Parameter must be a simple identifier",
                    ));
                }
            },
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    arg,
                    "Provider functions cannot have self parameter",
                ));
            }
        }
    }

    Ok(params)
}

fn extract_result_types(return_type: &ReturnType) -> Result<(Type, Type)> {
    match return_type {
        ReturnType::Type(_, ty) => {
            match ty.as_ref() {
                Type::Path(type_path) => {
                    let path = &type_path.path;
                    if let Some(segment) = path.segments.last() {
                        if segment.ident == "Result" {
                            match &segment.arguments {
                                syn::PathArguments::AngleBracketed(args) => {
                                    if args.args.len() == 2 {
                                        let output_type = args.args[0].clone();
                                        let error_type = args.args[1].clone();

                                        match (output_type, error_type) {
                                            (
                                                syn::GenericArgument::Type(out),
                                                syn::GenericArgument::Type(err),
                                            ) => {
                                                return Ok((out, err));
                                            }
                                            _ => {}
                                        }
                                    }
                                }
                                _ => {}
                            }
                        }
                    }
                }
                _ => {}
            }
            Err(syn::Error::new_spanned(
                ty,
                "Provider function must return Result<T, E>",
            ))
        }
        ReturnType::Default => Err(syn::Error::new_spanned(
            return_type,
            "Provider function must return Result<T, E>",
        )),
    }
}

fn to_pascal_case(s: &str) -> String {
    s.split('_')
        .map(|word| {
            let mut chars = word.chars();
            match chars.next() {
                None => String::new(),
                Some(first) => {
                    first.to_uppercase().collect::<String>() + &chars.as_str().to_lowercase()
                }
            }
        })
        .collect()
}

/// Generate dependency injection code for the function
fn generate_dependency_injection(inject_types: &[syn::Type], original_block: &syn::Block) -> syn::Block {
    if inject_types.is_empty() {
        return original_block.clone();
    }

    // Generate injection statements
    let mut injection_stmts = Vec::new();
    
    for (i, inject_type) in inject_types.iter().enumerate() {
        let var_name = syn::Ident::new(&format!("injected_{}", i), proc_macro2::Span::call_site());
        
        let injection_stmt: syn::Stmt = syn::parse_quote! {
            let #var_name = ::dioxus_riverpod::injection::inject::<#inject_type>()
                .map_err(|e| format!("Dependency injection failed for {}: {}", stringify!(#inject_type), e))?;
        };
        
        injection_stmts.push(injection_stmt);
    }

    // Create new block with injection statements + original statements
    let mut new_stmts = injection_stmts;
    new_stmts.extend(original_block.stmts.iter().cloned());

    syn::Block {
        brace_token: original_block.brace_token,
        stmts: new_stmts,
    }
}
