use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Ident, ItemTrait, Meta, NestedMeta, Signature, TraitItem, TypeParamBound};

use crate::utils::{abort_by, collect_available_arg_types, has_return_value, make_typed_return};

macro_rules! abort_by_dynamic_link {
    ($span:expr, $($tts:tt)*) => {
        abort_by!($span,"dynamic_link", $($tts)*)
    };
}

pub fn parse_contract_struct_id(nested_meta: &syn::NestedMeta) -> Ident {
    match nested_meta {
        NestedMeta::Meta(Meta::Path(path)) => match path.get_ident() {
            Some(id) => id.clone(),
            None => abort_by_dynamic_link!(
                nested_meta,
                "contract struct id must be specified like `#[dynamic_link(CalleeContract)]`"
            ),
        },
        _ => abort_by_dynamic_link!(
            nested_meta,
            "contract struct id must be specified like `#[dynamic_link(CalleeContract)]`"
        ),
    }
}

pub fn generate_import_contract_declaration(
    contract_struct_id: &Ident,
    trait_def: &ItemTrait,
) -> TokenStream {
    if !has_supertrait_contract(trait_def) {
        abort_by_dynamic_link!(
            trait_def,
            "dynamic link trait must has `cosmwasm_std::Contract` as one of its supertraits."
        )
    }

    let mut signatures: Vec<&Signature> = vec![];
    for item in &trait_def.items {
        match item {
            TraitItem::Method(method) => signatures.push(&method.sig),
            _ => abort_by_dynamic_link!(
                item,
                "other than method cannot be defined in dynamic link traits."
            ),
        }
    }

    let extern_block = generate_extern_block(contract_struct_id.to_string(), &signatures);
    let implement_block = generate_implements(&trait_def.ident, contract_struct_id, &signatures);

    quote! {
        #extern_block

        #trait_def

        #implement_block
    }
}

fn has_supertrait_contract(trait_def: &ItemTrait) -> bool {
    trait_def.supertraits.iter().any(|sb| match sb {
        TypeParamBound::Trait(tb) => tb.path.segments.last().unwrap().ident == "Contract",
        _ => false,
    })
}

fn generate_extern_block(module_name: String, methods: &[&Signature]) -> TokenStream {
    let stub_funcs = methods.iter().map(|signature| {
        let args_len = signature.inputs.len() - 1;
        let stub_func_name_ident = format_ident!("stub_{}", signature.ident);
        let renamed_param_defs: Vec<_> = (0..args_len)
            .map(|i| {
                let renamed_param_ident = format_ident!("ptr{}", i);
                quote! { #renamed_param_ident: u32 }
            })
            .collect();
        let typed_return = make_typed_return(&signature.output);
        quote! {
            fn #stub_func_name_ident(addr: u32 #(, #renamed_param_defs)*) #typed_return;
        }
    });

    quote! {
        #[link(wasm_import_module = #module_name)]
        extern "C" {
            #(#stub_funcs)*
        }
    }
}

fn generate_implements(trait_id: &Ident, struct_id: &Ident, methods: &[&Signature]) -> TokenStream {
    let impl_funcs = methods.iter().map(|sig| generate_serialization_func(sig));
    quote! {
        impl #trait_id for #struct_id {
            #(#impl_funcs)*
        }
    }
}

//Defines a function that was originally imported to execute serialization and call to imported stub_xxx.
fn generate_serialization_func(signature: &Signature) -> TokenStream {
    let func_name = &signature.ident;

    let args_len = signature.inputs.len() - 1;
    let arg_types = collect_available_arg_types(signature, "dynamic_link".to_string());

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

    let return_types = &signature.output;
    let call_stub_and_return =
        make_call_stub_and_return(func_name, &region_arg_idents, return_types);
    quote! {
        fn #func_name(&self #(, #renamed_param_defs)*) #return_types {
            let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
            #(let #vec_arg_idents = cosmwasm_std::to_vec(&#arg_idents).unwrap();)*
            let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
            #(let #region_arg_idents = cosmwasm_std::memory::release_buffer(#vec_arg_idents) as u32;)*
            unsafe {
                #call_stub_and_return
            }
        }
    }
}

fn make_call_stub_and_return(
    func_id: &Ident,
    arg_idents: &[Ident],
    return_type: &syn::ReturnType,
) -> TokenStream {
    let stub_func_id = format_ident!("stub_{}", func_id);
    if has_return_value(return_type) {
        quote! {
            let result = #stub_func_id(region_addr #(, #arg_idents)*);
            let vec_result = cosmwasm_std::memory::consume_region(result as *mut cosmwasm_std::memory::Region);
            cosmwasm_std::from_slice(&vec_result).unwrap()
        }
    } else {
        quote! {
            #stub_func_id(region_addr #(, #arg_idents)*);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{parse_quote, ItemTrait, Signature};

    #[test]
    fn make_call_stub_and_return_works() {
        {
            let sig_foo_ret0: Signature = parse_quote! {
                fn foo()
            };

            let result_code =
                make_call_stub_and_return(&sig_foo_ret0.ident, &[], &sig_foo_ret0.output)
                    .to_string();

            let expected: TokenStream = parse_quote! {
                stub_foo(region_addr);
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let sig_foo_ret1: Signature = parse_quote! {
                fn foo() -> u64
            };

            let result_code =
                make_call_stub_and_return(&sig_foo_ret1.ident, &[], &sig_foo_ret1.output)
                    .to_string();

            let expected: TokenStream = parse_quote! {
                let result = stub_foo(region_addr);
                let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                cosmwasm_std::from_slice(&vec_result).unwrap()
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let sig_foo_ret2: Signature = parse_quote! {
                fn foo() -> (u64, u64)
            };

            let result_code =
                make_call_stub_and_return(&sig_foo_ret2.ident, &[], &sig_foo_ret2.output)
                    .to_string();

            let expected: TokenStream = parse_quote! {
                let result = stub_foo(region_addr);
                let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                cosmwasm_std::from_slice(&vec_result).unwrap()
            };
            assert_eq!(expected.to_string(), result_code);
        }
    }

    #[test]
    fn generate_serialization_func_works() {
        let test_trait: ItemTrait = parse_quote! {
            trait Callee: Contract {
                fn foo(&self) -> u64;
                fn bar(&self, a: u64, b: String) -> u64;
                fn foobar(&self, a: u64, b: String) -> (u64, String);
            }
        };

        let method_sigs: Vec<&Signature> = test_trait
            .items
            .iter()
            .map(|item| match item {
                syn::TraitItem::Method(method) => &method.sig,
                _ => {
                    panic!()
                }
            })
            .collect();

        {
            let result_code = generate_serialization_func(method_sigs[0]).to_string();
            let expected: TokenStream = parse_quote! {
                fn foo (&self) -> u64 {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    unsafe {
                        let result = stub_foo(region_addr);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
               }
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let result_code = generate_serialization_func(method_sigs[1]).to_string();
            let expected: TokenStream = parse_quote! {
                fn bar (&self, arg0: u64 , arg1: String) -> u64 {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = stub_bar(region_addr, region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
               }
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let result_code = generate_serialization_func(method_sigs[2]).to_string();
            let expected: TokenStream = parse_quote! {
                fn foobar(&self, arg0: u64, arg1: String) -> (u64, String) {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = stub_foobar(region_addr, region_arg0, region_arg1);
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
        let test_trait: ItemTrait = parse_quote! {
            trait Callee: Contract {
                fn foo(&self, a: u64, b: String) -> u64;
                fn bar(&self);
                fn foobar(&self, a: u64, b: String) -> (u64, String);
            }
        };

        let method_sigs: Vec<&Signature> = test_trait
            .items
            .iter()
            .map(|item| match item {
                syn::TraitItem::Method(method) => &method.sig,
                _ => {
                    panic!()
                }
            })
            .collect();

        let result_code =
            generate_extern_block("test_contract".to_string(), &method_sigs).to_string();
        let expected: TokenStream = parse_quote! {
            #[link(wasm_import_module = "test_contract")]
            extern "C" {
                fn stub_foo(addr: u32, ptr0: u32, ptr1: u32) -> u32;
                fn stub_bar(addr: u32);
                fn stub_foobar(addr: u32, ptr0: u32, ptr1: u32) -> u32;
            }
        };
        assert_eq!(expected.to_string(), result_code);
    }

    #[test]
    fn generate_implements_works() {
        let test_trait: ItemTrait = parse_quote! {
            trait Callee: Contract {
                fn foo(&self, a: u64, b: String) -> u64;
                fn bar(&self);
                fn foobar(&self, a: u64, b: String) -> (u64, String);
            }
        };

        let method_sigs: Vec<&Signature> = test_trait
            .items
            .iter()
            .map(|item| match item {
                syn::TraitItem::Method(method) => &method.sig,
                _ => {
                    panic!()
                }
            })
            .collect();

        let result_code = generate_implements(
            &test_trait.ident,
            &format_ident!("CalleeContract"),
            &method_sigs,
        )
        .to_string();
        let expected: TokenStream = parse_quote! {
            impl Callee for CalleeContract {
                fn foo(&self, arg0: u64, arg1: String) -> u64 {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = stub_foo(region_addr, region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
                }

                fn bar(&self) {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    unsafe {
                        stub_bar(region_addr);
                    }
                }

                fn foobar(&self, arg0: u64, arg1: String) -> (u64, String) {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = stub_foobar(region_addr, region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
                }
            }
        };
        assert_eq!(expected.to_string(), result_code);
    }
}
