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
    inject: Vec<syn::Type>, // New: list of types to inject
    compose: Vec<syn::Ident>, // New: list of provider functions to compose
}

/// Attribute arguments for the mutation macro
#[derive(Default)]
struct MutationArgs {
    invalidates: Vec<syn::Ident>, // List of provider functions to invalidate
    inject: Vec<syn::Type>, // List of types to inject
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
                "inject" => {
                    // Parse injection types: inject = [Type1, Type2, ...]
                    let content;
                    syn::bracketed!(content in input);
                    let types = content.parse_terminated(syn::Type::parse, Token![,])?;
                    args.inject = types.into_iter().collect();
                }
                "compose" => {
                    // Parse compose list: compose = [provider1, provider2, ...]
                    let content;
                    syn::bracketed!(content in input);
                    let providers = content.parse_terminated(syn::Ident::parse, Token![,])?;
                    args.compose = providers.into_iter().collect();
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

impl Parse for MutationArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = MutationArgs::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "invalidates" => {
                    // Parse invalidation list: invalidates = [provider1, provider2, ...]
                    let content;
                    syn::bracketed!(content in input);
                    let providers = content.parse_terminated(syn::Ident::parse, Token![,])?;
                    args.invalidates = providers.into_iter().collect();
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
/// Dependency injection:
/// - #[provider(inject = [ApiClient, Database])] - automatically inject dependencies
///
/// Composable providers:
/// - #[provider(compose = [fetch_user, fetch_permissions])] - compose multiple providers
/// - Composed providers run in parallel and their results are available as variables
/// - Example: fetch_user_result and fetch_permissions_result
///
/// Can combine features:
/// - #[provider(interval = "10s", cache_expiration = "1min")]
/// - #[provider(stale_time = "5s", cache_expiration = "30s")]
/// - #[provider(compose = [fetch_user, fetch_settings], inject = [ApiClient])]
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

/// Attribute macro for creating mutations
///
/// Mutations are operations that modify data and can invalidate provider caches.
/// They support dependency injection and automatic cache invalidation.
///
/// ## Examples
///
/// Basic mutation:
/// ```rust
/// #[mutation]
/// async fn create_user(data: UserData) -> Result<User, String> {
///     api_client.create_user(data).await
/// }
/// ```
///
/// Mutation with cache invalidation:
/// ```rust
/// #[mutation(invalidates = [fetch_users, fetch_user_stats])]
/// async fn update_user(user_id: u32, data: UserData) -> Result<User, String> {
///     api_client.update_user(user_id, data).await
/// }
/// ```
///
/// Mutation with dependency injection:
/// ```rust
/// #[mutation(inject = [ApiClient, Logger], invalidates = [fetch_user])]
/// async fn delete_user(user_id: u32) -> Result<(), String> {
///     // ApiClient and Logger are automatically injected
///     api_client.delete_user(user_id).await
/// }
/// ```
#[proc_macro_attribute]
pub fn mutation(args: TokenStream, input: TokenStream) -> TokenStream {
    let mutation_args = if args.is_empty() {
        MutationArgs::default()
    } else {
        match syn::parse(args) {
            Ok(args) => args,
            Err(err) => return err.to_compile_error().into(),
        }
    };

    let input_fn = parse_macro_input!(input as ItemFn);

    let result = generate_mutation(input_fn, mutation_args);

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

    // Extract parameters once
    let params = extract_all_params(&input_fn)?;

    // Generate enhanced function body with dependency injection and composition
    let enhanced_fn_block = generate_enhanced_function_body(&provider_args.inject, &provider_args.compose, &params, fn_block);

    // Generate interval and cache expiration implementations
    let interval_impl = generate_interval_impl(&provider_args);
    let cache_expiration_impl = generate_cache_expiration_impl(&provider_args);
    let stale_time_impl = generate_stale_time_impl(&provider_args);

    // Generate common struct and const
    let common_struct = generate_common_struct_and_const(&info);

    // Determine parameter type and implementation based on function parameters
    if params.is_empty() {
        // No parameters - Provider<()>
        Ok(quote! {
            #common_struct

            impl #struct_name {
                #fn_vis async fn call() -> Result<#output_type, #error_type> {
                    #enhanced_fn_block
                }
            }

            impl ::dioxus_provider::hooks::Provider<()> for #struct_name {
                type Output = #output_type;
                type Error = #error_type;

                fn run(&self, _param: ()) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                    Self::call()
                }

                #interval_impl
                #cache_expiration_impl
                #stale_time_impl
            }
        })
    } else if params.len() == 1 {
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

            impl ::dioxus_provider::hooks::Provider<#param_type> for #struct_name {
                type Output = #output_type;
                type Error = #error_type;

                fn run(&self, #param_name: #param_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                    Self::call(#param_name)
                }

                #interval_impl
                #cache_expiration_impl
                #stale_time_impl
            }
        })
    } else {
        // Multiple parameters - Provider<(Param1, Param2, ...)>
        let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
        let param_types: Vec<_> = params.iter().map(|p| &p.ty).collect();
        let tuple_type = quote! { (#(#param_types,)*) };

        Ok(quote! {
            #common_struct

            impl #struct_name {
                #fn_vis async fn call(#(#param_names: #param_types,)*) -> Result<#output_type, #error_type> {
                    #enhanced_fn_block
                }
            }

            impl ::dioxus_provider::hooks::Provider<#tuple_type> for #struct_name {
                type Output = #output_type;
                type Error = #error_type;

                fn run(&self, params: #tuple_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                    let (#(#param_names,)*) = params;
                    Self::call(#(#param_names,)*)
                }

                #interval_impl
                #cache_expiration_impl
                #stale_time_impl
            }
        })
    }
}

fn generate_mutation(input_fn: ItemFn, mutation_args: MutationArgs) -> Result<TokenStream2> {
    let info = extract_provider_info(&input_fn)?;

    let ProviderInfo {
        fn_vis,
        fn_block,
        output_type,
        error_type,
        struct_name,
        ..
    } = &info;

    // Generate enhanced function body with dependency injection and composition
    let enhanced_fn_block = generate_enhanced_function_body(&mutation_args.inject, &[], &[], fn_block);

    // Generate invalidation implementation
    let invalidation_impl = generate_invalidation_impl(&mutation_args);

    // Generate common struct and const
    let common_struct = generate_common_struct_and_const(&info);

    // Determine parameter type and implementation based on function parameters
    if input_fn.sig.inputs.is_empty() {
        // No parameters - Mutation<()>
        Ok(quote! {
            #common_struct

            impl #struct_name {
                #fn_vis async fn call() -> Result<#output_type, #error_type> {
                    #enhanced_fn_block
                }
            }

            impl ::dioxus_provider::mutation::Mutation<()> for #struct_name {
                type Output = #output_type;
                type Error = #error_type;

                fn mutate(&self, _input: ()) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                    Self::call()
                }

                #invalidation_impl
            }
        })
    } else {
        // Has parameters - extract and handle them
        let params = extract_all_params(&input_fn)?;

        if params.len() == 1 {
            // Single parameter - Mutation<ParamType>
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

                impl ::dioxus_provider::mutation::Mutation<#param_type> for #struct_name {
                    type Output = #output_type;
                    type Error = #error_type;

                    fn mutate(&self, #param_name: #param_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                        Self::call(#param_name)
                    }

                    #invalidation_impl
                }
            })
        } else {
            // Multiple parameters - Mutation<(Param1, Param2, ...)>
            let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
            let param_types: Vec<_> = params.iter().map(|p| &p.ty).collect();
            let tuple_type = quote! { (#(#param_types,)*) };

            Ok(quote! {
                #common_struct

                impl #struct_name {
                    #fn_vis async fn call(#(#param_names: #param_types,)*) -> Result<#output_type, #error_type> {
                        #enhanced_fn_block
                    }
                }

                impl ::dioxus_provider::mutation::Mutation<#tuple_type> for #struct_name {
                    type Output = #output_type;
                    type Error = #error_type;

                    fn mutate(&self, input: #tuple_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                        let (#(#param_names,)*) = input;
                        Self::call(#(#param_names,)*)
                    }

                    #invalidation_impl
                }
            })
        }
    }
}

/// Generate duration implementation for provider methods
fn generate_duration_impl(method_name: &str, duration: Option<Duration>) -> TokenStream2 {
    if let Some(duration) = duration {
        let duration_secs = duration.as_secs();
        let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());

        quote! {
            fn #method_ident(&self) -> Option<::std::time::Duration> {
                Some(::std::time::Duration::from_secs(#duration_secs))
            }
        }
    } else {
        quote! {}
    }
}

/// Generate interval implementation
fn generate_interval_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("interval", provider_args.interval)
}

/// Generate cache expiration implementation
fn generate_cache_expiration_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("cache_expiration", provider_args.cache_expiration)
}

/// Generate stale time implementation
fn generate_stale_time_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl("stale_time", provider_args.stale_time)
}

/// Generate invalidation implementation for mutations
fn generate_invalidation_impl(mutation_args: &MutationArgs) -> TokenStream2 {
    if mutation_args.invalidates.is_empty() {
        quote! {}
    } else {
        let provider_calls: Vec<_> = mutation_args.invalidates
            .iter()
            .map(|provider_fn| {
                quote! {
                    ::dioxus_provider::mutation::provider_cache_key_simple(#provider_fn())
                }
            })
            .collect();

        quote! {
            fn invalidates(&self) -> Vec<String> {
                vec![#(#provider_calls,)*]
            }
        }
    }
}

/// Information extracted from the provider function
struct ProviderInfo {
    fn_vis: syn::Visibility,
    fn_attrs: Vec<syn::Attribute>,
    fn_block: Box<syn::Block>,
    output_type: Type,
    error_type: Type,
    struct_name: syn::Ident,
    fn_name: syn::Ident,
}

/// Information about a function parameter
struct ParamInfo {
    name: syn::Ident,
    ty: Type,
}

/// Extract provider information from the input function
fn extract_provider_info(input_fn: &ItemFn) -> Result<ProviderInfo> {
    let fn_name = input_fn.sig.ident.clone();
    let fn_vis = input_fn.vis.clone();
    let fn_attrs = input_fn.attrs.clone();
    let fn_block = input_fn.block.clone();

    let (output_type, error_type) = extract_result_types(&input_fn.sig.output)?;
    let struct_name = syn::Ident::new(
        &to_pascal_case(&fn_name.to_string()),
        proc_macro2::Span::call_site(),
    );

    Ok(ProviderInfo {
        fn_vis,
        fn_attrs,
        fn_block,
        output_type,
        error_type,
        struct_name,
        fn_name,
    })
}

/// Generate common struct and const for the provider
fn generate_common_struct_and_const(info: &ProviderInfo) -> TokenStream2 {
    let struct_name = &info.struct_name;
    let fn_attrs = &info.fn_attrs;
    let fn_name = &info.fn_name;

    quote! {
        #[derive(Clone, PartialEq)]
        #(#fn_attrs)*
        pub struct #struct_name;

        impl Default for #struct_name {
            fn default() -> Self {
                Self
            }
        }

        // Generate a function that returns an instance of the struct
        pub fn #fn_name() -> #struct_name {
            #struct_name
        }
    }
}

/// Extract all parameters from the function signature
fn extract_all_params(input_fn: &ItemFn) -> Result<Vec<ParamInfo>> {
    let mut params = Vec::new();

    for input in &input_fn.sig.inputs {
        match input {
            FnArg::Typed(PatType { pat, ty, .. }) => {
                if let Pat::Ident(pat_ident) = &**pat {
                    params.push(ParamInfo {
                        name: pat_ident.ident.clone(),
                        ty: (**ty).clone(),
                    });
                } else {
                    return Err(syn::Error::new_spanned(
                        pat,
                        "Only simple parameter names are supported",
                    ));
                }
            }
            FnArg::Receiver(_) => {
                return Err(syn::Error::new_spanned(
                    input,
                    "Methods with self parameter are not supported",
                ));
            }
        }
    }

    Ok(params)
}

/// Extract result types from the function return type
fn extract_result_types(return_type: &ReturnType) -> Result<(Type, Type)> {
    match return_type {
        ReturnType::Default => Err(syn::Error::new_spanned(
            return_type,
            "Provider functions must return Result<T, E>",
        )),
        ReturnType::Type(_, ty) => {
            if let Type::Path(type_path) = &**ty {
                if let Some(segment) = type_path.path.segments.last() {
                    if segment.ident == "Result" {
                        if let syn::PathArguments::AngleBracketed(args) = &segment.arguments {
                            if args.args.len() == 2 {
                                let mut args_iter = args.args.iter();

                                let output_type = match args_iter.next().unwrap() {
                                    syn::GenericArgument::Type(ty) => ty.clone(),
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            args,
                                            "Result must have type arguments",
                                        ));
                                    }
                                };

                                let error_type = match args_iter.next().unwrap() {
                                    syn::GenericArgument::Type(ty) => ty.clone(),
                                    _ => {
                                        return Err(syn::Error::new_spanned(
                                            args,
                                            "Result must have type arguments",
                                        ));
                                    }
                                };

                                return Ok((output_type, error_type));
                            }
                        }
                    }
                }
            }

            Err(syn::Error::new_spanned(
                return_type,
                "Provider functions must return Result<T, E>",
            ))
        }
    }
}

/// Convert a string to PascalCase
fn to_pascal_case(s: &str) -> String {
    let mut result = String::new();
    let mut capitalize_next = true;

    for c in s.chars() {
        if c == '_' {
            capitalize_next = true;
        } else if capitalize_next {
            result.push(c.to_ascii_uppercase());
            capitalize_next = false;
        } else {
            result.push(c);
        }
    }

    result
}

/// Generate enhanced function body with dependency injection and composition
fn generate_enhanced_function_body(
    inject_types: &[syn::Type], 
    compose_providers: &[syn::Ident],
    params: &[ParamInfo],
    original_block: &syn::Block
) -> syn::Block {
    let mut statements = Vec::new();
    
    // Add dependency injection statements
    if !inject_types.is_empty() {
        let injection_stmts: Vec<_> = inject_types
            .iter()
            .map(|ty| {
                let var_name = syn::Ident::new(
                    &format!("injected_{}", to_pascal_case(&quote!(#ty).to_string().to_lowercase())),
                    proc_macro2::Span::call_site(),
                );
                
                syn::parse_quote! {
                    let #var_name = ::dioxus_provider::injection::inject::<#ty>()
                        .map_err(|e| format!("Dependency injection failed for {}: {}", stringify!(#ty), e))?;
                }
            })
            .collect();
        
        statements.extend(injection_stmts);
    }
    
    // Add composition statements
    if !compose_providers.is_empty() {
        let composition_statements = generate_composition_statements(compose_providers, params);
        statements.extend(composition_statements);
    }
    
    // Add original function body statements
    statements.extend(original_block.stmts.clone());
    
    syn::Block {
        brace_token: original_block.brace_token,
        stmts: statements,
    }
}

/// Generate composition statements that can be directly added to a statement list
fn generate_composition_statements(compose_providers: &[syn::Ident], params: &[ParamInfo]) -> Vec<syn::Stmt> {
    if compose_providers.is_empty() {
        return vec![];
    }

    let mut statements = Vec::new();

    // Generate variable names for composed results
    let result_vars: Vec<_> = compose_providers
        .iter()
        .map(|provider| {
            syn::Ident::new(
                &format!("{}_result", provider.to_string()),
                proc_macro2::Span::call_site(),
            )
        })
        .collect();

    // Generate provider calls based on parameter count
    if params.is_empty() {
        // No parameters - call providers with ()
        let provider_calls: Vec<_> = compose_providers
            .iter()
            .map(|provider| {
                quote! {
                    async { #provider().run(()).await }
                }
            })
            .collect();

        let join_stmt: syn::Stmt = syn::parse_quote! {
            let (#(#result_vars,)*) = ::futures::join!(
                #(#provider_calls,)*
            );
        };
        statements.push(join_stmt);
    } else if params.len() == 1 {
        // Single parameter - clone it inside each async block
        let param_name = &params[0].name;
        let provider_calls: Vec<_> = compose_providers
            .iter()
            .map(|provider| {
                quote! {
                    async { 
                        let param = #param_name.clone(); 
                        #provider().run(param).await 
                    }
                }
            })
            .collect();

        let join_stmt: syn::Stmt = syn::parse_quote! {
            let (#(#result_vars,)*) = ::futures::join!(
                #(#provider_calls,)*
            );
        };
        statements.push(join_stmt);
    } else {
        // Multiple parameters - clone each parameter inside each async block
        let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
        let provider_calls: Vec<_> = compose_providers
            .iter()
            .map(|provider| {
                quote! {
                    async { 
                        let params = (#(#param_names.clone(),)*); 
                        #provider().run(params).await 
                    }
                }
            })
            .collect();

        let join_stmt: syn::Stmt = syn::parse_quote! {
            let (#(#result_vars,)*) = ::futures::join!(
                #(#provider_calls,)*
            );
        };
        statements.push(join_stmt);
    }

    statements
}
