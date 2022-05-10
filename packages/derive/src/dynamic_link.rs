use proc_macro2::TokenStream;
use quote::{format_ident, quote};

use crate::utils::{abort_by, collect_available_arg_types, get_return_len, make_typed_return};

macro_rules! abort_by_dynamic_link {
    ($span:expr, $($tts:tt)*) => {
        abort_by!($span,"dynamic_link", $($tts)*)
    };
}

pub fn parse_contract_name(nested_meta: &syn::NestedMeta) -> String {
    let name_value = match nested_meta {
        syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => Some(name_value),
        _ => abort_by_dynamic_link!(nested_meta, "contract_name must be a NameValue"),
    };

    match name_value {
        Some(name_value) => {
            if name_value.path.is_ident("contract_name") {
                match &name_value.lit {
                    syn::Lit::Str(literal) => literal.value(),
                    _ => abort_by_dynamic_link!(
                        name_value.lit,
                        "contract_name value is not string literal"
                    ),
                }
            } else {
                abort_by_dynamic_link!(name_value.path, "only allowed the \"contract_name\"")
            }
        }
        None => abort_by_dynamic_link!(nested_meta, "invalid attribute type"),
    }
}

pub fn generate_import_contract_declaration(
    contract_name: String,
    exist_extern_block: syn::ItemForeignMod,
) -> TokenStream {
    //if not specified the ABI(None), the default value of extern ABI is C.
    if let Some(ref extern_abi) = exist_extern_block.abi.name {
        if extern_abi.value() != "C" {
            abort_by_dynamic_link!(
                extern_abi,
                "ABI only supports the C. not recommended to specify the ABI yourself."
            );
        }
    }

    let foreign_function_decls: Vec<&syn::ForeignItemFn> = exist_extern_block
        .items
        .iter()
        .map(|foregin_item| match foregin_item {
            syn::ForeignItem::Fn(item_fn) => item_fn,
            _ => abort_by_dynamic_link!(foregin_item, "only function type is allowed."),
        })
        .collect();

    let mut new_item = TokenStream::new();
    new_item.extend(generate_extern_block(
        contract_name,
        &foreign_function_decls,
    ));
    for func_decl in foreign_function_decls {
        new_item.extend(generate_serialization_func(func_decl));
    }

    new_item
}

fn generate_extern_block(
    module_name: String,
    origin_foreign_func_decls: &[&syn::ForeignItemFn],
) -> TokenStream {
    let redeclared_funcs = origin_foreign_func_decls.iter().map(|func_decl| {
        let args_len = func_decl.sig.inputs.len();
        let stub_func_name_ident = format_ident!("stub_{}", func_decl.sig.ident);
        let renamed_param_defs: Vec<_> = (0..args_len)
            .map(|i| {
                let renamed_param_ident = format_ident!("ptr{}", i);
                quote! { #renamed_param_ident: u32 }
            })
            .collect();
        let typed_return = make_typed_return(&func_decl.sig.output, "dynamic_link".to_string());
        quote! {
            fn #stub_func_name_ident(#(#renamed_param_defs),*) #typed_return;
        }
    });

    quote! {
        #[link(wasm_import_module = #module_name)]
        extern "C" {
            #(#redeclared_funcs)*
        }
    }
}

//Defines a function that was originally imported to execute serialization and call to imported stub_xxx.
fn generate_serialization_func(origin_func_decl: &syn::ForeignItemFn) -> TokenStream {
    let func_name = &origin_func_decl.sig.ident;

    let args_len = origin_func_decl.sig.inputs.len();
    let arg_types = collect_available_arg_types(&origin_func_decl.sig, "dynamic_link".to_string());

    let renamed_param_defs: Vec<_> = (0..args_len)
        .map(|i| {
            let renamed_arg_ident = format_ident!("arg{}", i);
            let arg_type = arg_types[i];
            quote! { #renamed_arg_ident: #arg_type }
        })
        .collect();
    let arg_idents: Vec<_> = (0..args_len).map(|i| format_ident!("arg{}", i)).collect();
    let vec_arg_idents: Vec<_> = (0..args_len)
        .map(|i| format_ident!("vec_arg{}", i))
        .collect();
    let region_arg_idents: Vec<_> = (0..args_len)
        .map(|i| format_ident!("region_arg{}", i))
        .collect();

    let return_types = &origin_func_decl.sig.output;
    let call_stub_and_return =
        make_call_stub_and_return(&func_name.to_string(), args_len, return_types);
    quote! {
        fn #func_name(#(#renamed_param_defs),*) #return_types {
            #(let #vec_arg_idents = cosmwasm_std::to_vec(&#arg_idents).unwrap();)*
            #(let #region_arg_idents = cosmwasm_std::memory::release_buffer(#vec_arg_idents) as u32;)*
            unsafe {
                #call_stub_and_return
            }
        }
    }
}

fn make_call_stub_and_return(
    func_name: &str,
    args_len: usize,
    return_type: &syn::ReturnType,
) -> TokenStream {
    let ident_func_name = format_ident!("stub_{}", func_name);
    let arguments: Vec<_> = (0..args_len)
        .map(|n| format_ident!("region_arg{}", n))
        .collect();

    let return_len = get_return_len(return_type);
    match return_len {
        0 => {
            quote! {
                #ident_func_name(#(#arguments),*);
            }
        }
        1 => {
            quote! {
                let result = #ident_func_name(#(#arguments),*);
                let vec_result = cosmwasm_std::memory::consume_region(result as *mut cosmwasm_std::memory::Region);
                cosmwasm_std::from_slice(&vec_result).unwrap()
            }
        }
        _ => {
            let vec_results: Vec<_> = (0..return_len)
                .map(|n| format_ident!("vec_result{}", n))
                .collect();
            let results: Vec<_> = (0..return_len)
                .map(|n| format_ident!("result{}", n))
                .collect();
            quote! {
                let (#(#results),*) = #ident_func_name(#(#arguments),*);
                #(let #vec_results = cosmwasm_std::memory::consume_region(#results as *mut cosmwasm_std::memory::Region);)*
                (#(cosmwasm_std::from_slice(&#vec_results).unwrap()),*)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, ItemFn};

    #[test]
    fn make_call_stub_and_return_works() {
        {
            let function_foo_ret0: ItemFn = parse_quote! {
                fn foo() {
                }
            };

            let result_code = make_call_stub_and_return(
                &function_foo_ret0.sig.ident.to_string(),
                0,
                &function_foo_ret0.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                stub_foo();
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let function_foo_ret1: ItemFn = parse_quote! {
                fn foo() -> u64 {
                    1
                }
            };

            let result_code = make_call_stub_and_return(
                &function_foo_ret1.sig.ident.to_string(),
                0,
                &function_foo_ret1.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = stub_foo();
                let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                cosmwasm_std::from_slice(&vec_result).unwrap()
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let function_foo_ret2: ItemFn = parse_quote! {
                fn foo() -> (u64, u64) {
                    (1, 2)
                }
            };

            let result_code = make_call_stub_and_return(
                &function_foo_ret2.sig.ident.to_string(),
                0,
                &function_foo_ret2.sig.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let (result0, result1) = stub_foo();
                let vec_result0 = cosmwasm_std::memory::consume_region(result0 as * mut cosmwasm_std::memory::Region);
                let vec_result1 = cosmwasm_std::memory::consume_region(result1 as * mut cosmwasm_std::memory::Region);
                (cosmwasm_std::from_slice(&vec_result0).unwrap(), cosmwasm_std::from_slice(&vec_result1).unwrap())
            };
            assert_eq!(expected.to_string(), result_code);
        }
    }

    #[test]
    fn generate_serialization_func_works() {
        let test_extern: syn::ItemForeignMod = parse_quote! {
            extern {
                fn foo() -> u64;
                fn foo(a: u64, b: String) -> u64;
            }
        };

        let foreign_function_decls: Vec<&syn::ForeignItemFn> = test_extern
            .items
            .iter()
            .map(|foregin_item| match foregin_item {
                syn::ForeignItem::Fn(item_fn) => item_fn,
                _ => {
                    panic!()
                }
            })
            .collect();

        {
            let result_code = generate_serialization_func(foreign_function_decls[0]).to_string();
            let expected: TokenStream = parse_quote! {
                fn foo () -> u64 {
                    unsafe {
                        let result = stub_foo();
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
               }
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let result_code = generate_serialization_func(foreign_function_decls[1]).to_string();
            let expected: TokenStream = parse_quote! {
                fn foo (arg0 : u64 , arg1 : String) -> u64 {
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = stub_foo(region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
               }
            };
            assert_eq!(expected.to_string(), result_code);
        }
    }

    #[test]
    fn generate_extern_block_works() {
        let test_extern: syn::ItemForeignMod = parse_quote! {
            extern {
                fn foo(a: u64, b: String) -> u64;
                fn bar();
            }
        };

        let foreign_function_decls: Vec<&syn::ForeignItemFn> = test_extern
            .items
            .iter()
            .map(|foregin_item| match foregin_item {
                syn::ForeignItem::Fn(item_fn) => item_fn,
                _ => {
                    panic!()
                }
            })
            .collect();

        let result_code =
            generate_extern_block("test_contract".to_string(), &foreign_function_decls).to_string();
        let expected: TokenStream = parse_quote! {
            #[link(wasm_import_module = "test_contract")]
            extern "C" {
                fn stub_foo(ptr0: u32, ptr1: u32) -> u32;
                fn stub_bar();
            }
        };
        assert_eq!(expected.to_string(), result_code);
    }
}
