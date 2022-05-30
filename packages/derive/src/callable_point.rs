use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::utils::{collect_available_arg_types, has_return_value, make_typed_return};

pub fn make_callable_point(function: syn::ItemFn) -> TokenStream {
    let stub_func_name_ident = format_ident!("stub_{}", function.sig.ident);
    let args_len = function.sig.inputs.len();

    let arg_idents: Vec<_> = (0..args_len).map(|i| format_ident!("arg{}", i)).collect();
    let vec_arg_idents: Vec<_> = (0..args_len)
        .map(|i| format_ident!("vec_arg{}", i))
        .collect();
    let ptr_idents: Vec<_> = (0..args_len).map(|i| format_ident!("ptr{}", i)).collect();

    let arg_types = collect_available_arg_types(&function.sig, "callable_point".to_string());
    let renamed_param_defs: Vec<_> = ptr_idents
        .iter()
        .map(|id| {
            quote! { #id: u32 }
        })
        .collect();
    let typed_return = make_typed_return(&function.sig.output);

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

    if has_return_value(return_type) {
        quote! {
            let result = #func_name_ident(#(#arguments),*);
            let vec_result = cosmwasm_std::to_vec(&result).unwrap();
            cosmwasm_std::memory::release_buffer(vec_result) as u32
        }
    } else {
        quote! {#func_name_ident(#(#arguments),*);}
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
                fn foo() -> u64 {
                    1
                }
            };
            let result_code = make_call_origin_and_return(
                &function_foo_ret1.sig.ident,
                0,
                &function_foo_ret1.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = foo();
                let vec_result = cosmwasm_std::to_vec(&result).unwrap();
                cosmwasm_std::memory::release_buffer(vec_result) as u32
            };
            assert_eq!(expected.to_string(), result_code);
        }

        {
            let function_foo_ret2: ItemFn = parse_quote! {
                fn foo() -> (u64, u64) {
                    (1, 2)
                }
            };
            let result_code = make_call_origin_and_return(
                &function_foo_ret2.sig.ident,
                0,
                &function_foo_ret2.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = foo();
                let vec_result = cosmwasm_std::to_vec(&result).unwrap();
                cosmwasm_std::memory::release_buffer(vec_result) as u32
            };
            assert_eq!(expected.to_string(), result_code);
        }
    }
}
