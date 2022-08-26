use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::utils::{abort_by, collect_available_arg_types, has_return_value, make_typed_return};

pub fn make_callable_point(function: syn::ItemFn) -> TokenStream {
    let function_name_ident = &function.sig.ident;
    let mod_name_ident = format_ident!("__wasm_export_{}", function_name_ident);
    // The first argument is `deps`, the rest is region pointers
    if function.sig.inputs.len() < 2 {
        abort_by!(
            function,
            "callable_point",
            "callable_point function needs `deps` typed `Deps` or `DepsMut` as the first argument and `env` typed `Env` as the second argument."
        )
    }
    let args_len = function.sig.inputs.len() - 1;

    let arg_idents: Vec<_> = (0..args_len).map(|i| format_ident!("arg{}", i)).collect();
    let vec_arg_idents: Vec<_> = (0..args_len)
        .map(|i| format_ident!("vec_arg{}", i))
        .collect();
    let ptr_idents: Vec<_> = (0..args_len).map(|i| format_ident!("ptr{}", i)).collect();

    let orig_arg_types = collect_available_arg_types(&function.sig, "callable_point".to_string());
    let arg_types = &orig_arg_types[1..];

    let is_dep_mutable = match &orig_arg_types[0] {
        syn::Type::Path(p) => {
            if p.path.is_ident("Deps") {
                false
            } else if p.path.is_ident("DepsMut") {
                true
            } else {
                abort_by!(
                    function, "callable_point",
                    "the first argument of callable_point function needs `deps` typed `Deps` or `DepsMut`"
                )
            }
        }
        _ => {
            abort_by!(
                function, "callable_point",
                "the first argument of callable_point function needs `deps` typed `Deps` or `DepsMut`"
            )
        }
    };

    match &orig_arg_types[1] {
        syn::Type::Path(p) => {
            if !p.path.is_ident("Env") {
                abort_by!(
                    function,
                    "callable_point",
                    "the second argument of callable_point function needs `env` typed `Env`"
                )
            }
        }
        _ => {
            abort_by!(
                function,
                "callable_point",
                "the second argument of callable_point function needs `env` typed `Env`"
            )
        }
    };

    let renamed_param_defs: Vec<_> = ptr_idents
        .iter()
        .map(|id| {
            quote! { #id: u32 }
        })
        .collect();
    let typed_return = make_typed_return(&function.sig.output);

    let call_origin_return = make_call_origin_and_return(
        is_dep_mutable,
        function_name_ident,
        args_len,
        &function.sig.output,
    );

    let deps_def = if is_dep_mutable {
        quote! { let mut deps = cosmwasm_std::make_dependencies() }
    } else {
        quote! { let deps = cosmwasm_std::make_dependencies() }
    };

    quote! {
        #[cfg(target_arch = "wasm32")]
        mod #mod_name_ident {
            use super::*;

            #[no_mangle]
            extern "C" fn #function_name_ident(#(#renamed_param_defs),*) #typed_return {
                #(let #vec_arg_idents: Vec<u8> = unsafe { cosmwasm_std::memory::consume_region(#ptr_idents as *mut cosmwasm_std::memory::Region)};)*
                #(let #arg_idents: #arg_types = cosmwasm_std::from_slice(&#vec_arg_idents).unwrap();)*

                #deps_def;

                #call_origin_return
            }
        }
    }
}

fn make_call_origin_and_return(
    is_dep_mutable: bool,
    func_name_ident: &syn::Ident,
    args_len: usize,
    return_type: &syn::ReturnType,
) -> TokenStream {
    let arguments: Vec<_> = (0..args_len).map(|n| format_ident!("arg{}", n)).collect();

    let call_func = if is_dep_mutable {
        quote! { super::#func_name_ident(deps.as_mut() #(, #arguments)*) }
    } else {
        quote! { super::#func_name_ident(deps.as_ref() #(, #arguments)*) }
    };

    if has_return_value(return_type) {
        quote! {
            let result = #call_func;
            let vec_result = cosmwasm_std::to_vec(&result).unwrap();
            cosmwasm_std::memory::release_buffer(vec_result) as u32
        }
    } else {
        call_func
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, ItemFn};

    #[test]
    fn make_call_origin_and_return_works() {
        {
            let function_foo_ret1: ItemFn = parse_quote! {
                fn foo(deps: DepsMut) -> u64 {
                    1
                }
            };
            let result_code = make_call_origin_and_return(
                true,
                &function_foo_ret1.sig.ident,
                0,
                &function_foo_ret1.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = super::foo(deps.as_mut());
                let vec_result = cosmwasm_std::to_vec(&result).unwrap();
                cosmwasm_std::memory::release_buffer(vec_result) as u32
            };
            assert_eq!(expected.to_string(), result_code);
        }

        {
            let function_foo_ret2: ItemFn = parse_quote! {
                fn foo(deps: Deps) -> (u64, u64) {
                    (1, 2)
                }
            };
            let result_code = make_call_origin_and_return(
                false,
                &function_foo_ret2.sig.ident,
                0,
                &function_foo_ret2.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = super::foo(deps.as_ref());
                let vec_result = cosmwasm_std::to_vec(&result).unwrap();
                cosmwasm_std::memory::release_buffer(vec_result) as u32
            };
            assert_eq!(expected.to_string(), result_code);
        }
    }
}
