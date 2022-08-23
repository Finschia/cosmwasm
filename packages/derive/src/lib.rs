#[macro_use]
extern crate syn;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use std::str::FromStr;

mod callable_point;
mod contract;
mod dynamic_link;
mod utils;
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
pub fn entry_point(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut res = item.clone();
    let function = parse_macro_input!(item as syn::ItemFn);
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
    res.extend(entry);
    res
}

#[proc_macro_error]
#[proc_macro_attribute]
pub fn callable_point(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let function = parse_macro_input!(item as syn::ItemFn);
    let mut res = TokenStream::from(quote! {
        #[allow(dead_code)]
        #function
    });

    let maked = callable_point::make_callable_point(function);
    res.extend(TokenStream::from(maked));
    res
}

/// This macro implements dynamic call functions for attributed trait.
///
/// This macro must take an attribute specifying a struct to implement the traits for.
/// The trait must have `cosmwasm_std::Contract` as a supertrait and each
/// methods of the trait must have `&self` receiver as its first argument.
///
/// This macro can take a bool value as a named attribute `user_defined_mock`
/// When this value is true, this macro generates implement of the trait for specified struct for only `target_arch = "wasm32"`.
/// So, with `user_defined_mock = true`, user can and must write mock implement of the trait for specified struct with `#[cfg(not(target_arch = "wasm32"))]`.
///
/// example usage:
///
/// ```
/// use cosmwasm_std::{Addr, Contract, dynamic_link};
///
/// #[derive(Contract)]
/// struct ContractStruct {
///   address: Addr
/// }
///
/// #[dynamic_link(ContractStruct, user_defined_mock = true)]
/// trait TraitName: Contract {
///   fn callable_point_on_another_contract(&self, x: i32) -> i32;
/// }
///
/// // When `user_defined_mock = true` is specified, implement is generated only for "wasm32"
/// #[cfg(not(target_arch = "wasm32"))]
/// impl TraitName for ContractStruct {
///   fn callable_point_on_another_contract(&self, x: i32) -> i32 {
///     do_something_as_a_mock()
///   }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn dynamic_link(attr: TokenStream, item: TokenStream) -> TokenStream {
    let attr_args = parse_macro_input!(attr as syn::AttributeArgs);

    let (contract_struct_id, does_use_user_defined_mock) =
        dynamic_link::parse_attributes(attr_args);
    let trait_def = parse_macro_input!(item as syn::ItemTrait);
    dynamic_link::generate_import_contract_declaration(
        &contract_struct_id,
        &trait_def,
        does_use_user_defined_mock,
    )
    .into()
}

/// This derive macro is for implementing `cosmwasm_std::Contract`
///
/// This implements `get_address` and `set_address` for address field.
/// Address field is selected as following
/// 1. If there is a field attributed with `#[address]`, the field will
///    be used as the address field.
/// 2. Choose a field by field name. The priority of the name is
///    "contract_address" -> "contract_addr" -> "address" -> "addr".
#[proc_macro_derive(Contract, attributes(address))]
pub fn derive_contract(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    contract::derive_contract(derive_input).into()
}
