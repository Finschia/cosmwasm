use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::utils::{collect_available_arg_types, get_return_len, make_typed_return};

pub fn make_callable_point(function: syn::ItemFn) -> TokenStream {
    let stub_func_name_ident = format_ident!("stub_{}", function.sig.ident);
    let args_len = function.sig.inputs.len();

    let arg_idents: Vec<_> = (0..args_len).map(|i| format_ident!("arg{}", i)).collect();
    let vec_arg_idents: Vec<_> = (0..args_len)
        .map(|i| format_ident!("vec_arg{}", i))
        .collect();
    let ptr_idents: Vec<_> = (0..args_len).map(|i| format_ident!("ptr{}", i)).collect();

    let arg_types = collect_available_arg_types(&function.sig, "callable_point".to_string());
    let renamed_param_defs: Vec<_> = (0..args_len)
        .map(|i| {
            let renamed_param_ident = format_ident!("ptr{}", i);
            quote! { #renamed_param_ident: u32 }
        })
        .collect();
    let typed_return = make_typed_return(&function.sig.output, "callable_point".to_string());

    let call_origin_return =
        make_call_origin_and_return(&function.sig.ident, args_len, &function.sig.output);

    quote! {
        #[cfg(target_arch = "wasm32")]
        #[no_mangle]
        extern "C" fn #stub_func_name_ident(#(#renamed_param_defs),*) #typed_return {
            #(let #vec_arg_idents: Vec<u8> = unsafe { cosmwasm_std::memory::consume_region(#ptr_idents as *mut cosmwasm_std::memory::Region)};)*
            #(let #arg_idents: #arg_types = cosmwasm_std::from_slice(&#vec_arg_idents).unwrap();)*
            #call_origin_return
        }
    }
}

fn make_call_origin_and_return(
    func_name_ident: &syn::Ident,
    args_len: usize,
    return_type: &syn::ReturnType,
) -> TokenStream {
    let arguments: Vec<_> = (0..args_len).map(|n| format_ident!("arg{}", n)).collect();
    let return_len = get_return_len(return_type);

    match return_len {
        0 => quote! {#func_name_ident(#(#arguments),*);},
        1 => {
            quote! {
                let result = #func_name_ident(#(#arguments),*);
                let vec_result = cosmwasm_std::to_vec(&result).unwrap();
                cosmwasm_std::memory::release_buffer(vec_result) as u32
            }
        }
        _ => {
            let results: Vec<_> = (0..return_len)
                .map(|n| format_ident!("result{}", n))
                .collect();
            let vec_results: Vec<_> = (0..return_len)
                .map(|n| format_ident!("vec_result{}", n))
                .collect();

            quote! {
                let (#(#results),*) = #func_name_ident(#(#arguments),*);
                #(let #vec_results = cosmwasm_std::to_vec(&#results).unwrap();)*
                (#(cosmwasm_std::memory::release_buffer(#vec_results) as u32),*)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, ItemFn};

    const PART_CALL_TO_ORIGIN_ARG0: &str = "foo () ;";
    const PART_CALL_TO_VEC: &str = "cosmwasm_std :: to_vec (";
    const PART_CALL_TO_RELEASE_BUFFER: &str = "cosmwasm_std :: memory :: release_buffer (";

    #[test]
    fn make_call_origin_and_return_works() {
        {
            let function_foo_ret1: ItemFn = parse_quote! {
                fn foo() -> u64 {
                    1
                }
            };
            /* generated:
            let result = foo () ;
            let vec_result = cosmwasm_std :: to_vec (& result) . unwrap () ;
            cosmwasm_std :: memory :: release_buffer (vec_result) as u32
            */
            let result_code = make_call_origin_and_return(
                &function_foo_ret1.sig.ident,
                0,
                &function_foo_ret1.sig.output,
            )
            .to_string();
            assert_eq!(result_code.matches(PART_CALL_TO_ORIGIN_ARG0).count(), 1);
            assert_eq!(result_code.matches(PART_CALL_TO_VEC).count(), 1);
            assert_eq!(result_code.matches(PART_CALL_TO_RELEASE_BUFFER).count(), 1);
        }

        {
            let function_foo_ret2: ItemFn = parse_quote! {
                fn foo() -> (u64, u64) {
                    (1, 2)
                }
            };
            /*
            let (result0 , result1) = foo () ;
            let vec_result0 = cosmwasm_std :: to_vec (& result0) . unwrap () ;
            let vec_result1 = cosmwasm_std :: to_vec (& result1) . unwrap () ;
            (cosmwasm_std :: memory :: release_buffer (vec_result0) as u32 , cosmwasm_std :: memory :: release_buffer (vec_result1) as u32)
            */
            let result_code = make_call_origin_and_return(
                &function_foo_ret2.sig.ident,
                0,
                &function_foo_ret2.sig.output,
            )
            .to_string();
            assert_eq!(result_code.matches(PART_CALL_TO_ORIGIN_ARG0).count(), 1);
            assert_eq!(result_code.matches(PART_CALL_TO_VEC).count(), 2);
            assert_eq!(result_code.matches(PART_CALL_TO_RELEASE_BUFFER).count(), 2);
        }
    }
}
