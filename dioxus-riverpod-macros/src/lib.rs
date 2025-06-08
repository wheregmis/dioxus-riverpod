use proc_macro::TokenStream;
use proc_macro2::TokenStream as TokenStream2;
use quote::quote;
use syn::{FnArg, ItemFn, Pat, PatType, Result, ReturnType, Type, parse_macro_input};

/// Unified attribute macro for creating providers
/// Automatically detects provider type based on function parameters:
/// - No parameters → Future Provider  
/// - Has parameters → Family Provider
#[proc_macro_attribute]
pub fn provider(_args: TokenStream, input: TokenStream) -> TokenStream {
    let input_fn = parse_macro_input!(input as ItemFn);

    // Check if the function has parameters
    let has_params = !input_fn.sig.inputs.is_empty();

    let result = if has_params {
        // Has parameters → Family Provider
        generate_family_provider(input_fn)
    } else {
        // No parameters → Future Provider
        generate_future_provider(input_fn)
    };

    match result {
        Ok(tokens) => tokens.into(),
        Err(err) => err.to_compile_error().into(),
    }
}

fn generate_future_provider(input_fn: ItemFn) -> Result<TokenStream2> {
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
        }

        // Create a constant instance for easy usage
        // Allow snake_case for ergonomic usage while suppressing the naming warning
        #[allow(non_upper_case_globals)]
        #fn_vis const #fn_name: #struct_name = #struct_name;
    })
}

fn generate_family_provider(input_fn: ItemFn) -> Result<TokenStream2> {
    let fn_name = &input_fn.sig.ident;
    let fn_vis = &input_fn.vis;
    let fn_attrs = &input_fn.attrs;
    let fn_block = &input_fn.block;

    // Extract parameter information
    let param_info = extract_first_param(&input_fn)?;
    let param_name = &param_info.name;
    let param_type = &param_info.ty;

    // Extract return type from Result<T, E>
    let (output_type, error_type) = extract_result_types(&input_fn.sig.output)?;

    // Generate the provider struct name (convert snake_case to PascalCase + Provider)
    let struct_name = syn::Ident::new(
        &format!("{}Provider", to_pascal_case(&fn_name.to_string())),
        fn_name.span(),
    );

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
        }

        // Create a constant instance for easy usage
        // Allow snake_case for ergonomic usage while suppressing the naming warning
        #[allow(non_upper_case_globals)]
        #fn_vis const #fn_name: #struct_name = #struct_name;
    })
}

struct ParamInfo {
    name: syn::Ident,
    ty: Type,
}

fn extract_first_param(input_fn: &ItemFn) -> Result<ParamInfo> {
    let first_arg = input_fn.sig.inputs.first().ok_or_else(|| {
        syn::Error::new_spanned(
            &input_fn.sig,
            "Family provider must have at least one parameter",
        )
    })?;

    match first_arg {
        FnArg::Typed(PatType { pat, ty, .. }) => match pat.as_ref() {
            Pat::Ident(pat_ident) => Ok(ParamInfo {
                name: pat_ident.ident.clone(),
                ty: (**ty).clone(),
            }),
            _ => Err(syn::Error::new_spanned(
                pat,
                "Parameter must be a simple identifier",
            )),
        },
        FnArg::Receiver(_) => Err(syn::Error::new_spanned(
            first_arg,
            "Provider functions cannot have self parameter",
        )),
    }
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
