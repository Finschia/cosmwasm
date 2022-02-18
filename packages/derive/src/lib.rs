#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
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


#[proc_macro_attribute]
pub fn callable_point(_attr: TokenStream, mut item: TokenStream) -> TokenStream {
    let cloned = item.clone();
    let function = parse_macro_input!(cloned as syn::ItemFn);
    let name = function.sig.ident.to_string();


    let args_len = function.sig.inputs.len();
    let arg_types: Vec<&syn::Type> = function.sig.inputs.iter().map(|arg| {
        match arg {
            syn::FnArg::Receiver(_) => panic!("callable_point Method type are not allowed."),
            syn::FnArg::Typed(arg_info) => {
                match arg_info.ty.as_ref() {
                    syn::Type::BareFn(_) => panic!("callable_point function type by parameter are not allowed."),
                    _ => arg_info.ty.as_ref(),
                }
            }
        }
    }).collect();


    // E.g. "ptr0: u32, ptr1: u32, ptr2: u32, "
    let typed_ptrs = (0..args_len).fold(String::new(), |acc, i| format!("{}ptr{}: u32, ", acc, i));
  
    // E.g. "let vec_arg0: Vec<u8> = unsafe { consume_region(ptr0 as *mut Region) };\n let vec_arg1 ..."
    let vec_args = (0..args_len).fold(String::new(), |acc, i| {
        format!("{}let vec_arg{}: Vec<u8> = unsafe {{ cosmwasm_std::memory::consume_region(ptr{} as *mut cosmwasm_std::memory::Region) }};", acc, i, i)
    });

    // E.g. "let arg0: $ArgType = from_slice(vec_arg0));\n let arg1: ..."
    let converted_args = (0..args_len).fold(String::new(), |acc, i| {
        let arg_type = arg_types[i];
        format!("{}let arg{}: {} = cosmwasm_std::from_slice(&vec_arg{}).unwrap();",
        acc,
        i,
        quote!{#arg_type}.to_string(),
        i
        )
    });

    // E.g. "arg0, arg1, arg2, "
    let pass_args = (0..args_len).fold(String::new(), |acc, i| format!("{}arg{}, ", acc, i));

    let new_code = format!(
        r##"
        // The stub function does not create a new module unlike entry_point macro.
        // This is because the argument type cannot be found if you define it as a new module.
        #[cfg(target_arch = "wasm32")]
            #[no_mangle]
            extern "C" fn stub_{name}({typed_ptrs}) -> u32 {{
                {vec_args}
                {converted_args}

                // call the original function
                let result = {name}({pass_args});
                let vec_result = cosmwasm_std::to_vec(&result).unwrap();
                cosmwasm_std::memory::release_buffer(vec_result) as u32
            }}
    "##,
        name = name,
        typed_ptrs = typed_ptrs,
        vec_args = vec_args,
        converted_args = converted_args,
        pass_args = pass_args,

    );
    let entry = TokenStream::from_str(&new_code).unwrap();
    item.extend(entry);
    item
}
