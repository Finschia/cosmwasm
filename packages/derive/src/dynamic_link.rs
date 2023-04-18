use proc_macro2::{Span, TokenStream};
use quote::{format_ident, quote};
use syn::{
    parse_quote, AttributeArgs, Ident, ItemTrait, Lit, Meta, NestedMeta, ReturnType, Signature,
    TraitItem, TypeParamBound,
};
use wasmer_types::{ExportType, FunctionType, Type};

use crate::utils::{abort_by, collect_available_arg_types, has_return_value, make_typed_return};

macro_rules! abort_by_dynamic_link {
    ($span:expr, $($tts:tt)*) => {
        abort_by!($span,"dynamic_link", $($tts)*)
    };
}

pub fn parse_attributes(attr_args: AttributeArgs) -> (Ident, bool) {
    let mut struct_id: Option<Ident> = None;
    let mut does_use_mock: Option<bool> = None;
    for nested_meta in attr_args {
        match &nested_meta {
            NestedMeta::Meta(Meta::Path(path)) => match path.get_ident() {
                Some(id) => struct_id = Some(id.clone()),
                None => abort_by_dynamic_link!(
                    nested_meta,
                    "`dynamic_link` macro cannot take unnamed attributes other than contract struct id"
                ),
            },
            NestedMeta::Meta(Meta::NameValue(mnv)) => {
                if !mnv.path.is_ident("user_defined_mock") {
                    abort_by_dynamic_link!(
                        nested_meta,
                        "other named attribute than `user_defined_mock` cannot be used in `dynamic_link` macro."
                    )
                }
                match &mnv.lit {
                    Lit::Bool(b) => does_use_mock = Some(b.value),
                    _ => abort_by_dynamic_link!(
                        nested_meta,
                        "`user_defined_mock` attribute can take only bool value"
                    )
                }
            },
            _ => abort_by_dynamic_link!(
                nested_meta,
                "`dynamic_link` macro cannot take attributes other than contract struct id or `user_defined_mock` attribute."
            ),
        }
    }
    let res_id = match struct_id {
        Some(id) => id,
        None => panic!(
            "`dynamic_link` macro needs contract struct id as an unnamed attribute like `#[dynamic_link(CalleeContract)]`"
        )
    };
    let res_mock = does_use_mock.unwrap_or(false);
    (res_id, res_mock)
}

pub fn generate_import_contract_declaration(
    contract_struct_id: &Ident,
    trait_def: &ItemTrait,
    does_use_user_defined_mock: bool,
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

    let module_name = &format!("dynamiclinked_{}", &contract_struct_id.to_string());
    let extern_block = generate_extern_block(module_name, &signatures);
    let implement_block = generate_implements(
        module_name,
        &trait_def.ident,
        contract_struct_id,
        &signatures,
    );

    let mut new_trait_def = trait_def.clone();
    let method_validate_interface: TraitItem = parse_quote! {
        fn validate_interface(&self, deps: cosmwasm_std::Deps) -> cosmwasm_std::StdResult<()>;
    };
    new_trait_def.items.push(method_validate_interface);

    if does_use_user_defined_mock {
        quote! {
            #new_trait_def

            #[cfg(target_arch = "wasm32")]
            #extern_block

            #[cfg(target_arch = "wasm32")]
            #implement_block
        }
    } else {
        quote! {
            #new_trait_def

            #extern_block

            #implement_block
        }
    }
}

fn generate_imported_module_id(module_name: &str) -> Ident {
    format_ident!("__wasm_imported_{}", module_name.to_ascii_lowercase())
}

fn has_supertrait_contract(trait_def: &ItemTrait) -> bool {
    trait_def.supertraits.iter().any(|sb| match sb {
        TypeParamBound::Trait(tb) => tb.path.segments.last().unwrap().ident == "Contract",
        _ => false,
    })
}

fn generate_extern_block(module_name: &str, methods: &[&Signature]) -> TokenStream {
    let module_name_ident = generate_imported_module_id(module_name);
    let funcs = methods.iter().map(|signature| {
        let func_ident = &signature.ident;
        let args_len = signature.inputs.len() - 1;
        let renamed_param_defs: Vec<_> = (0..args_len)
            .map(|i| {
                let renamed_param_ident = format_ident!("ptr{}", i);
                quote! { #renamed_param_ident: u32 }
            })
            .collect();
        let typed_return = make_typed_return(&signature.output);
        quote! {
            pub(crate) fn #func_ident(addr: u32 #(, #renamed_param_defs)*) #typed_return;
        }
    });

    quote! {
        mod #module_name_ident {
            #[link(wasm_import_module = #module_name)]
            extern "C" {
                #(#funcs)*
            }
        }
    }
}

fn generate_validate_interface_method(methods: &[&Signature]) -> TokenStream {
    let interface: Vec<ExportType<FunctionType>> = methods
        .iter()
        .map(|sig| {
            let name = sig.ident.to_string();
            // -1 for &self and +1 for arg `env`, so equals to len()
            let input_len = sig.inputs.len();
            let result_len = match sig.output {
                ReturnType::Default => 0_usize,
                ReturnType::Type(..) => 1_usize,
            };
            ExportType::new(
                &name,
                FunctionType::new(vec![Type::I32; input_len], vec![Type::I32; result_len]),
            )
        })
        .collect();
    let serialized_interface = serde_json::to_vec(&interface).unwrap();
    let interface_lit = syn::LitByteStr::new(&serialized_interface, Span::call_site());
    quote! {
        fn validate_interface(&self, deps: cosmwasm_std::Deps) -> cosmwasm_std::StdResult<()> {
            let address = self.get_address();
            deps.api.validate_dynamic_link_interface(&address, #interface_lit)
        }
    }
}

fn generate_implements(
    module_name: &str,
    trait_id: &Ident,
    struct_id: &Ident,
    methods: &[&Signature],
) -> TokenStream {
    let impl_funcs = methods
        .iter()
        .map(|sig| generate_serialization_func(module_name, sig));
    let impl_validate_interface = generate_validate_interface_method(methods);
    quote! {
        impl #trait_id for #struct_id {
            #(#impl_funcs)*
            #impl_validate_interface
        }
    }
}

//Defines a function that was originally imported to execute serialization and call to imported functions.
fn generate_serialization_func(module_name: &str, signature: &Signature) -> TokenStream {
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
    let call_function_and_return =
        make_call_function_and_return(module_name, func_name, &region_arg_idents, return_types);
    quote! {
        fn #func_name(&self #(, #renamed_param_defs)*) #return_types {
            let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
            #(let #vec_arg_idents = cosmwasm_std::to_vec(&#arg_idents).unwrap();)*
            let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
            #(let #region_arg_idents = cosmwasm_std::memory::release_buffer(#vec_arg_idents) as u32;)*
            unsafe {
                #call_function_and_return
            }
        }
    }
}

fn make_call_function_and_return(
    module_name: &str,
    func_id: &Ident,
    arg_idents: &[Ident],
    return_type: &syn::ReturnType,
) -> TokenStream {
    let imported_module_name_ident = generate_imported_module_id(module_name);

    if has_return_value(return_type) {
        quote! {
            let result = #imported_module_name_ident::#func_id(region_addr #(, #arg_idents)*);
            let vec_result = cosmwasm_std::memory::consume_region(result as *mut cosmwasm_std::memory::Region);
            cosmwasm_std::from_slice(&vec_result).unwrap()
        }
    } else {
        quote! {
            #imported_module_name_ident::#func_id(region_addr #(, #arg_idents)*);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::{ItemTrait, Signature};

    #[test]
    fn make_call_function_and_return_works() {
        let module_name = "callee_contract";
        let module_id = generate_imported_module_id(module_name);

        {
            let sig_foo_ret0: Signature = parse_quote! {
                fn foo()
            };

            let result_code = make_call_function_and_return(
                module_name,
                &sig_foo_ret0.ident,
                &[],
                &sig_foo_ret0.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                #module_id::foo(region_addr);
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let sig_foo_ret1: Signature = parse_quote! {
                fn foo() -> u64
            };

            let result_code = make_call_function_and_return(
                module_name,
                &sig_foo_ret1.ident,
                &[],
                &sig_foo_ret1.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = #module_id::foo(region_addr);
                let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                cosmwasm_std::from_slice(&vec_result).unwrap()
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let sig_foo_ret2: Signature = parse_quote! {
                fn foo() -> (u64, u64)
            };

            let result_code = make_call_function_and_return(
                module_name,
                &sig_foo_ret2.ident,
                &[],
                &sig_foo_ret2.output,
            )
            .to_string();

            let expected: TokenStream = parse_quote! {
                let result = #module_id::foo(region_addr);
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

        let module_name = "callee_contract";
        let module_id = generate_imported_module_id(module_name);

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
            let result_code = generate_serialization_func(module_name, method_sigs[0]).to_string();
            let expected: TokenStream = parse_quote! {
                fn foo (&self) -> u64 {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    unsafe {
                        let result = #module_id::foo(region_addr);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
               }
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let result_code = generate_serialization_func(module_name, method_sigs[1]).to_string();
            let expected: TokenStream = parse_quote! {
                fn bar (&self, arg0: u64 , arg1: String) -> u64 {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = #module_id::bar(region_addr, region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
               }
            };
            assert_eq!(expected.to_string(), result_code);
        }
        {
            let result_code = generate_serialization_func(module_name, method_sigs[2]).to_string();
            let expected: TokenStream = parse_quote! {
                fn foobar(&self, arg0: u64, arg1: String) -> (u64, String) {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let vec_arg0 = cosmwasm_std::to_vec(&arg0).unwrap();
                    let vec_arg1 = cosmwasm_std::to_vec(&arg1).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    let region_arg0 = cosmwasm_std::memory::release_buffer(vec_arg0) as u32;
                    let region_arg1 = cosmwasm_std::memory::release_buffer(vec_arg1) as u32;
                    unsafe {
                        let result = #module_id::foobar(region_addr, region_arg0, region_arg1);
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

        let module_name = "callee_contract";
        let module_id = generate_imported_module_id(module_name);
        let result_code = generate_extern_block(module_name, &method_sigs).to_string();
        let expected: TokenStream = parse_quote! {
            mod #module_id {
                #[link(wasm_import_module = #module_name)]
                extern "C" {
                    pub(crate) fn foo(addr: u32, ptr0: u32, ptr1: u32) -> u32;
                    pub(crate) fn bar(addr: u32);
                    pub(crate) fn foobar(addr: u32, ptr0: u32, ptr1: u32) -> u32;
                }
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

        let module_name = "callee_contract";
        let module_id = generate_imported_module_id(module_name);

        let result_code = generate_implements(
            module_name,
            &test_trait.ident,
            &format_ident!("CalleeContract"),
            &method_sigs,
        )
        .to_string();
        let expected_interface: Vec<wasmer_types::ExportType<wasmer_types::FunctionType>> = vec![
            ExportType::new("foo", ([Type::I32; 3], [Type::I32; 1]).into()),
            ExportType::new("bar", ([Type::I32; 1], [Type::I32; 0]).into()),
            ExportType::new("foobar", ([Type::I32; 3], [Type::I32; 1]).into()),
        ];
        let serialized_interface = serde_json::to_vec(&expected_interface).unwrap();
        let interface_lit = syn::LitByteStr::new(&serialized_interface, Span::call_site());

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
                        let result = #module_id::foo(region_addr, region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
                }

                fn bar(&self) {
                    let vec_addr = cosmwasm_std::to_vec(&self.get_address()).unwrap();
                    let region_addr = cosmwasm_std::memory::release_buffer(vec_addr) as u32;
                    unsafe {
                        #module_id::bar(region_addr);
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
                        let result = #module_id::foobar(region_addr, region_arg0, region_arg1);
                        let vec_result = cosmwasm_std::memory::consume_region(result as * mut cosmwasm_std::memory::Region);
                        cosmwasm_std::from_slice(&vec_result).unwrap()
                    }
                }

                fn validate_interface(&self, deps: cosmwasm_std::Deps) -> cosmwasm_std::StdResult<()> {
                    let address = self.get_address();
                    deps.api.validate_dynamic_link_interface(&address, #interface_lit)
                }
            }
        };
        assert_eq!(expected.to_string(), result_code);
    }
}
