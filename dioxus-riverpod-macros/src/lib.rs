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

    // Check if the function has parameters
    let has_params = !input_fn.sig.inputs.is_empty();

    let result = if has_params {
        // Has parameters → Family Provider
        generate_family_provider(input_fn, provider_args)
    } else {
        // No parameters → Future Provider
        generate_future_provider(input_fn, provider_args)
    };

    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_future_provider(input_fn: ItemFn, provider_args: ProviderArgs) -> Result<TokenStream2> {
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;

    // Extract return type from Result<T, E>
    let (output_type, error_type) = extract_result_types(&input_fn.sig.output)?;

    // Generate the provider struct name (convert snake_case to PascalCase + Provider)
    let struct_name = syn::Ident::new(
        &format!("{}Provider", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    // Generate interval implementation
    let interval_impl = generate_interval_impl(&provider_args);

    Ok(quote! {
        #(#fn_attrs)*
        #[derive(Clone, PartialEq)]
        #fn_vis struct #struct_name;

        impl #struct_name {
            #fn_vis async fn call() -> Result<#output_type, #error_type> {
                #fn_block
            }
        }

        impl ::dioxus_riverpod::providers::FutureProvider for #struct_name {
            type Output = #output_type;
            type Error = #error_type;

            fn run(&self) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                Self::call()
            }

            fn id(&self) -> String {
                stringify!(#struct_name).to_string()
            }

            #interval_impl
        }

        // Create a constant instance for easy usage
        // Allow snake_case for ergonomic usage while suppressing the naming warning
        #[allow(non_upper_case_globals)]
        #fn_vis const #fn_name: #struct_name = #struct_name;
    })
}

fn generate_family_provider(input_fn: ItemFn, provider_args: ProviderArgs) -> Result<TokenStream2> {
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;

    // Extract all parameters (support multiple parameters)
    let params = extract_all_params(&input_fn)?;

    // Extract return type from Result<T, E>
    let (output_type, error_type) = extract_result_types(&input_fn.sig.output)?;

    // Generate the provider struct name (convert snake_case to PascalCase + Provider)
    let struct_name = syn::Ident::new(
        &format!("{}Provider", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

    // Generate interval implementation
    let interval_impl = generate_interval_impl(&provider_args);

    // Handle single vs multiple parameters
    if params.len() == 1 {
        // Single parameter - keep existing behavior
        let param = &params[0];
        let param_name = &param.name;
        let param_type = &param.ty;

        Ok(quote! {
            #(#fn_attrs)*
            #[derive(Clone, PartialEq)]
            #fn_vis struct #struct_name;

            impl #struct_name {
                #fn_vis async fn call(#param_name: #param_type) -> Result<#output_type, #error_type> {
                    #fn_block
                }
            }

            impl ::dioxus_riverpod::providers::FamilyProvider<#param_type> for #struct_name {
                type Output = #output_type;
                type Error = #error_type;

                fn run(&self, #param_name: #param_type) -> impl ::std::future::Future<Output = Result<Self::Output, Self::Error>> + Send {
                    Self::call(#param_name)
                }

                fn id(&self, param: &#param_type) -> String {
                    format!("{}({:?})", stringify!(#struct_name), param)
                }

                #interval_impl
            }

            // Create a constant instance for easy usage
            // Allow snake_case for ergonomic usage while suppressing the naming warning
            #[allow(non_upper_case_globals)]
            #fn_vis const #fn_name: #struct_name = #struct_name;
        })
    } else {
        // Multiple parameters - create a tuple type
        let param_names: Vec<_> = params.iter().map(|p| &p.name).collect();
        let param_types: Vec<_> = params.iter().map(|p| &p.ty).collect();

        // Create tuple type for parameters
        let tuple_type = if param_types.len() == 2 {
            quote! { (#(#param_types),*) }
        } else {
            quote! { (#(#param_types,)*) }
        };

        Ok(quote! {
            #(#fn_attrs)*
            #[derive(Clone, PartialEq)]
            #fn_vis struct #struct_name;

            impl #struct_name {
                #fn_vis async fn call(#(#param_names: #param_types),*) -> Result<#output_type, #error_type> {
                    #fn_block
                }
            }

            impl ::dioxus_riverpod::providers::FamilyProvider<#tuple_type> for #struct_name {
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
            }

            // Create a constant instance for easy usage
            // Allow snake_case for ergonomic usage while suppressing the naming warning
            #[allow(non_upper_case_globals)]
            #fn_vis const #fn_name: #struct_name = #struct_name;
        })
    }
}

fn generate_interval_impl(provider_args: &ProviderArgs) -> TokenStream2 {
    match (provider_args.interval_secs, provider_args.interval_millis) {
        (Some(secs), None) => {
            quote! {
                fn interval(&self) -> Option<::std::time::Duration> {
                    Some(::std::time::Duration::from_secs(#secs))
                }
            }
        }
        (None, Some(millis)) => {
            quote! {
                fn interval(&self) -> Option<::std::time::Duration> {
                    Some(::std::time::Duration::from_millis(#millis))
                }
            }
        }
        (Some(secs), Some(_millis)) => {
            // If both are specified, prefer seconds and ignore millis
            quote! {
                fn interval(&self) -> Option<::std::time::Duration> {
                    Some(::std::time::Duration::from_secs(#secs))
                }
            }
        }
        (None, None) => {
            // No interval specified, use default (None)
            quote! {}
        }
    }
}

struct ParamInfo {
    name: syn::Ident,
    ty: Type,
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
