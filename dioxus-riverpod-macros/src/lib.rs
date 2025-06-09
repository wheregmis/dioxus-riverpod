#![allow(unused_variables)] // Variables used in quote! macros aren't detected by compiler

use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{
    FnArg, ItemFn, LitInt, Pat, PatType, Result, ReturnType, Token, Type, parse::Parse,
    parse::ParseStream, parse_macro_input,
};

/// Attribute arguments for the provider macro
#[derive(Default)]
struct ProviderArgs {
    interval_secs: Option<u64>,
    interval_millis: Option<u64>,
    cache_expiration_secs: Option<u64>,
    cache_expiration_millis: Option<u64>,
    stale_time_secs: Option<u64>,
    stale_time_millis: Option<u64>,
}

impl Parse for ProviderArgs {
    fn parse(input: ParseStream) -> Result<Self> {
        let mut args = ProviderArgs::default();

        while !input.is_empty() {
            let ident: syn::Ident = input.parse()?;
            input.parse::<Token![=]>()?;

            match ident.to_string().as_str() {
                "interval_secs" => {
                    let lit: LitInt = input.parse()?;
                    args.interval_secs = Some(lit.base10_parse()?);
                }
                "interval_millis" => {
                    let lit: LitInt = input.parse()?;
                    args.interval_millis = Some(lit.base10_parse()?);
                }
                "cache_expiration_secs" => {
                    let lit: LitInt = input.parse()?;
                    args.cache_expiration_secs = Some(lit.base10_parse()?);
                }
                "cache_expiration_millis" => {
                    let lit: LitInt = input.parse()?;
                    args.cache_expiration_millis = Some(lit.base10_parse()?);
                }
                "stale_time_secs" => {
                    let lit: LitInt = input.parse()?;
                    args.stale_time_secs = Some(lit.base10_parse()?);
                }
                "stale_time_millis" => {
                    let lit: LitInt = input.parse()?;
                    args.stale_time_millis = Some(lit.base10_parse()?);
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
/// Supports interval configuration:
/// - #[provider(interval_secs = 5)] - refresh every 5 seconds
/// - #[provider(interval_millis = 1000)] - refresh every 1000 milliseconds
///
/// Supports cache expiration:
/// - #[provider(cache_expiration_secs = 30)] - cache expires after 30 seconds
/// - #[provider(cache_expiration_millis = 5000)] - cache expires after 5000 milliseconds
///
/// Supports stale-while-revalidate:
/// - #[provider(stale_time_secs = 5)] - serve stale data after 5 seconds, refresh in background
/// - #[provider(stale_time_millis = 3000)] - serve stale data after 3000 milliseconds, refresh in background
///
/// Can combine features:
/// - #[provider(interval_secs = 10, cache_expiration_secs = 60)]
/// - #[provider(stale_time_secs = 5, cache_expiration_secs = 30)]
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

    // Generate interval and cache expiration implementations
    let interval_impl = generate_interval_impl(&provider_args);
    let cache_expiration_impl = generate_cache_expiration_impl(&provider_args);
    let stale_time_impl = generate_stale_time_impl(&provider_args);

    // Generate common struct and const
    let common_struct = generate_common_struct_and_const(&info);

    // Determine parameter type and implementation based on function parameters
    if input_fn.sig.inputs.is_empty() {
        // No parameters - Provider<()>
        Ok(quote! {
            #common_struct

            impl #struct_name {
                #fn_vis async fn call() -> Result<#output_type, #error_type> {
                    #fn_block
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
                        #fn_block
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
                        #fn_block
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
                }
            })
        }
    }
}

fn generate_duration_impl(
    method_name: &str,
    secs: Option<u64>,
    millis: Option<u64>,
) -> TokenStream2 {
    match (secs, millis) {
        (Some(secs), _) => {
            // Prefer seconds over millis if both specified
            let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            quote! {
                fn #method_ident(&self) -> Option<::std::time::Duration> {
                    Some(::std::time::Duration::from_secs(#secs))
                }
            }
        }
        (None, Some(millis)) => {
            let method_ident = syn::Ident::new(method_name, proc_macro2::Span::call_site());
            quote! {
                fn #method_ident(&self) -> Option<::std::time::Duration> {
                    Some(::std::time::Duration::from_millis(#millis))
                }
            }
        }
        (None, None) => {
            // No duration specified, use default (None)
            quote! {}
        }
    }
}

fn generate_interval_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl(
        "interval",
        provider_args.interval_secs,
        provider_args.interval_millis,
    )
}

fn generate_cache_expiration_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl(
        "cache_expiration",
        provider_args.cache_expiration_secs,
        provider_args.cache_expiration_millis,
    )
}

fn generate_stale_time_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    generate_duration_impl(
        "stale_time",
        provider_args.stale_time_secs,
        provider_args.stale_time_millis,
    )
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
