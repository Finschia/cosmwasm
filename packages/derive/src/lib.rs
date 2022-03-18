#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use proc_macro_error::{abort, proc_macro_error};
use quote::quote;
use std::str::FromStr;

/// This attribute macro generates the boilerplate required to call into the
/// contract-specific logic from the entry-points to the Wasm module.
///
/// It should be added to the contract's init, handle, migrate and query implementations
/// like this:
/// ```
/// # use cosmwasm_std::{
/// #     Storage, Api, Querier, DepsMut, Deps, entry_point, Env, StdError, MessageInfo,
/// #     Response, QueryResponse,
/// # };
/// #
/// # type InstantiateMsg = ();
/// # type ExecuteMsg = ();
/// # type QueryMsg = ();
///
/// #[entry_point]
/// pub fn instantiate(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: InstantiateMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// #[entry_point]
/// pub fn handle(
///     deps: DepsMut,
///     env: Env,
///     info: MessageInfo,
///     msg: ExecuteMsg,
/// ) -> Result<Response, StdError> {
/// #   Ok(Default::default())
/// }
///
/// #[entry_point]
/// pub fn query(
///     deps: Deps,
///     env: Env,
///     msg: QueryMsg,
/// ) -> Result<QueryResponse, StdError> {
/// #   Ok(Default::default())
/// }
/// ```
///
/// where `InstantiateMsg`, `ExecuteMsg`, and `QueryMsg` are contract defined
/// types that implement `DeserializeOwned + JsonSchema`.
///
/// This is an alternative implementation of `cosmwasm_std::create_entry_points!(contract)`
/// and `cosmwasm_std::create_entry_points_with_migration!(contract)`
/// and should not be used together.
#[proc_macro_attribute]
pub fn entry_point(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);
    let name = function.sig.ident.to_string();
    // The first argument is `deps`, the rest is region pointers
    let args = function.sig.inputs.len() - 1;

    // E.g. "ptr0: u32, ptr1: u32, ptr2: u32, "
    let typed_ptrs = (0..args).fold(String::new(), |acc, i| format!("{}ptr{}: u32, ", acc, i));
    // E.g. "ptr0, ptr1, ptr2, "
    let ptrs = (0..args).fold(String::new(), |acc, i| format!("{}ptr{}, ", acc, i));

    let new_code = format!(
        r##"
        #[cfg(target_arch = "wasm32")]
        mod __wasm_export_{name} {{ // new module to avoid conflict of function name
            #[no_mangle]
            extern "C" fn {name}({typed_ptrs}) -> u32 {{
                cosmwasm_std::do_{name}(&super::{name}, {ptrs})
            }}
        }}
    "##,
        name = name,
        typed_ptrs = typed_ptrs,
        ptrs = ptrs
    );
    let entry = TokenStream::from_str(&new_code).unwrap();
    item.extend(entry);
    item
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn callable_point(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);
    let name = function.sig.ident.to_string();

    let args_len = function.sig.inputs.len();
    let arg_types = collect_available_arg_types(&function.sig);

    // E.g. "ptr0: u32, ptr1: u32, ptr2: u32, "
    let typed_ptrs = (0..args_len).fold(String::new(), |acc, i| format!("{}ptr{}: u32, ", acc, i));

    // E.g. "let vec_arg0: Vec<u8> = unsafe { consume_region(ptr0 as *mut Region) };\n let vec_arg1 ..."
    let vec_args = (0..args_len).fold(String::new(), |acc, i| {
        format!("{}let vec_arg{}: Vec<u8> = unsafe {{ cosmwasm_std::memory::consume_region(ptr{} as *mut cosmwasm_std::memory::Region) }};", acc, i, i)
    });

    // E.g. "let arg0: $ArgType = from_slice(vec_arg0));\n let arg1: ..."
    let converted_args = (0..args_len).fold(String::new(), |acc, i| {
        let arg_type = arg_types[i];
        format!(
            "{}let arg{}: {} = cosmwasm_std::from_slice(&vec_arg{}).unwrap();",
            acc,
            i,
            quote! {#arg_type}.to_string(),
            i
        )
    });

    let typed_return = make_typed_return(&function.sig.output);
    let call_origin_return = make_call_origin_and_return(&name, args_len, &function.sig.output);

    let new_code = format!(
        r##"
        // The stub function does not create a new module unlike entry_point macro.
        // This is because the argument type cannot be found if you define it as a new module.
        #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            extern "C" fn stub_{name}({typed_ptrs}){typed_return} {{
                {vec_args}
                {converted_args}
                {call_origin_return}
            }}
    "##,
        name = name,
        typed_ptrs = typed_ptrs,
        typed_return = typed_return,
        vec_args = vec_args,
        converted_args = converted_args,
        call_origin_return = call_origin_return,
    );
    let entry = TokenStream::from_str(&new_code).unwrap();
    item.extend(entry);
    item
}

fn make_call_origin_and_return(
    func_name: &str,
    args_len: usize,
    return_type: &syn::ReturnType,
) -> String {
    let pass_args = (0..args_len).fold(String::new(), |acc, i| format!("{}arg{}, ", acc, i));
    let return_len = get_return_len(return_type);
    match return_len {
        0 => format!(
            "{func_name}({pass_args});",
            func_name = func_name,
            pass_args = pass_args
        ),
        1 => {
            format!(
                "let result = {func_name}({pass_args});
            let vec_result = cosmwasm_std::to_vec(&result).unwrap();
            cosmwasm_std::memory::release_buffer(vec_result) as u32",
                func_name = func_name,
                pass_args = pass_args,
            )
        }
        _ => {
            let tuple_returns =
                (0..return_len).fold(String::new(), |acc, i| format!("{}result{}, ", acc, i));
            let vec_returns = (0..return_len).fold(String::new(), |acc, i| {
                format!(
                    "{}let vec_result{i} = cosmwasm_std::to_vec(&result{i}).unwrap();\n",
                    acc,
                    i = i
                )
            });
            let tuple_from_slice_returns = (0..return_len).fold(String::new(), |acc, i| {
                format!(
                    "{}cosmwasm_std::memory::release_buffer(vec_result{}) as u32, ",
                    acc, i
                )
            });
            format!(
                "let ({tuple_returns}) = {func_name}({pass_args});
            {vec_returns}
            ({tuple_from_slice_returns})
            ",
                func_name = func_name,
                pass_args = pass_args,
                tuple_returns = tuple_returns,
                vec_returns = vec_returns,
                tuple_from_slice_returns = tuple_from_slice_returns,
            )
        }
    }
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn dynamic_link(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);
    if attr_args.len() != 1 {
        panic!("too many attributes");
    }

    let contract_name = parse_contract_name(&attr_args[0]);

    generate_import_contract_declaration(contract_name, item)
}

fn parse_contract_name(nested_meta: &syn::NestedMeta) -> String {
    let name_value = match nested_meta {
        syn::NestedMeta::Meta(syn::Meta::NameValue(name_value)) => Some(name_value),
        _ => abort!(nested_meta, "contract_name must be a NameValue"),
    };

    match name_value {
        Some(name_value) => {
            if name_value.path.is_ident("contract_name") {
                match &name_value.lit {
                    syn::Lit::Str(literal) => literal.value(),
                    _ => abort!(name_value.lit, "contract_name value is not string literal"),
                }
            } else {
                abort!(name_value.path, "only allowed the \"contract_name\"")
            }
        }
        None => abort!(nested_meta, "invliad attribute type"),
    }
}

fn generate_import_contract_declaration(contract_name: String, item: TokenStream) -> TokenStream {
    let extern_block = parse_macro_input!(item as syn::ItemForeignMod);
    let foreign_function_decls: Vec<&syn::ForeignItemFn> = extern_block
        .items
        .iter()
        .map(|foregin_item| match foregin_item {
            syn::ForeignItem::Fn(item_fn) => item_fn,
            _ => abort!(foregin_item, "only function type is allowed."),
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
    let redeclared_funcs =
        origin_foreign_func_decls
            .iter()
            .fold(String::new(), |acc, func_decl| {
                let args_len = func_decl.sig.inputs.len();
                let typed_ptrs =
                    (0..args_len).fold(String::new(), |acc, i| format!("{}ptr{}: u32, ", acc, i));
                let typed_return = make_typed_return(&func_decl.sig.output);

                format!(
                    "{}fn stub_{}({}){};\n",
                    acc, func_decl.sig.ident, typed_ptrs, typed_return
                )
            });

    let new_extern_block = format!(
        r#"
            #[link(wasm_import_module = "{module_name}")]
            extern "C" {{
            {redeclared_funcs}
            }}
        "#,
        module_name = module_name,
        redeclared_funcs = redeclared_funcs,
    );

    TokenStream::from_str(&new_extern_block).unwrap()
}

//Defines a function that was originally imported to execute serialization and call to imported stub_xxx.
fn generate_serialization_func(origin_func_decl: &syn::ForeignItemFn) -> TokenStream {
    let func_name = origin_func_decl.sig.ident.to_string();

    let args_len = origin_func_decl.sig.inputs.len();
    let arg_types = collect_available_arg_types(&origin_func_decl.sig);

    let renamed_args = (0..args_len).fold(String::new(), |acc, i| {
        let arg_type = arg_types[i];
        format!("{}arg{}: {}, ", acc, i, quote!(#arg_type).to_string())
    });

    let vec_args = (0..args_len).fold(String::new(), |acc, i| {
        format!(
            "{}let vec_arg{} = cosmwasm_std::to_vec(&arg{}).unwrap();",
            acc, i, i
        )
    });
    let region_args = (0..args_len).fold(String::new(), |acc, i| {
        format!(
            "{}let region_arg{} = cosmwasm_std::memory::release_buffer(vec_arg{}) as u32;",
            acc, i, i
        )
    });

    let return_types = &origin_func_decl.sig.output;
    let call_stub_and_return = make_call_stub_and_return(&func_name, args_len, return_types);
    let replaced_new_code = format!(
        r##"
            //replace function for serialization
            fn {func_name}({renamed_args}) {origin_return} {{
                {vec_args}
                {region_args}
                unsafe {{
                    {call_stub_and_return}
                }}
            }}
        "##,
        func_name = func_name,
        renamed_args = renamed_args,
        origin_return = quote!(#return_types).to_string(),
        vec_args = vec_args,
        region_args = region_args,
        call_stub_and_return = call_stub_and_return,
    );
    TokenStream::from_str(&replaced_new_code).unwrap()
}

fn make_call_stub_and_return(
    func_name: &str,
    args_len: usize,
    return_type: &syn::ReturnType,
) -> String {
    let pass_args = (0..args_len).fold(String::new(), |acc, i| format!("{}region_arg{}, ", acc, i));

    let return_len = get_return_len(return_type);
    match return_len {
        0 => format!(
            "stub_{func_name}({pass_args});",
            func_name = func_name,
            pass_args = pass_args
        ),
        1 => {
            format!("let result = stub_{func_name}({pass_args});
            let vec_result = cosmwasm_std::memory::consume_region(result as *mut cosmwasm_std::memory::Region);    
            cosmwasm_std::from_slice(&vec_result).unwrap()",
            func_name = func_name,
            pass_args = pass_args,
            )
        }
        _ => {
            let tuple_returns =
                (0..return_len).fold(String::new(), |acc, i| format!("{}result{}, ", acc, i));
            let vec_returns = (0..return_len).fold(String::new(), |acc, i| {
                format!("{}let vec_result{i} = cosmwasm_std::memory::consume_region(result{i} as *mut cosmwasm_std::memory::Region);\n", acc, i=i)
            });
            let tuple_from_slice_returns = (0..return_len).fold(String::new(), |acc, i| {
                format!(
                    "{}cosmwasm_std::from_slice(&vec_result{}).unwrap(), ",
                    acc, i
                )
            });
            format!(
                "let ({tuple_returns}) = stub_{func_name}({pass_args});
            {vec_returns}
            ({tuple_from_slice_returns})
            ",
                func_name = func_name,
                pass_args = pass_args,
                tuple_returns = tuple_returns,
                vec_returns = vec_returns,
                tuple_from_slice_returns = tuple_from_slice_returns,
            )
        }
    }
}

fn collect_available_arg_types(func_sig: &syn::Signature) -> Vec<&syn::Type> {
    func_sig
        .inputs
        .iter()
        .map(|arg| match arg {
            syn::FnArg::Receiver(_) => abort!(arg, "method type is not allowed."),
            syn::FnArg::Typed(arg_info) => match arg_info.ty.as_ref() {
                syn::Type::BareFn(_) => abort!(arg, "function type by parameter is not allowed."),
                _ => arg_info.ty.as_ref(),
            },
        })
        .collect()
}

fn make_typed_return(return_type: &syn::ReturnType) -> String {
    let return_types_len = get_return_len(return_type);
    match return_types_len {
        0 => String::from(""),
        1 => String::from(" -> u32"),
        //TODO: see (https://github.com/line/cosmwasm/issues/156)
        _ => abort!(return_type, "Cannot support returning tuple type yet"), //_ => format!(" -> ({})", (0..return_types_len).fold(String::new(), |acc, _| format!("{}u32, ", acc))),
    }
}
fn get_return_len(return_type: &syn::ReturnType) -> usize {
    match return_type {
        syn::ReturnType::Default => 0,
        syn::ReturnType::Type(_, return_type) => match return_type.as_ref() {
            syn::Type::Tuple(tuple) => tuple.elems.len(),
            _ => 1,
        },
    }
}
