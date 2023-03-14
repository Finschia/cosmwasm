#[macro_use]
extern crate syn;

mod into_event;

use proc_macro::TokenStream;
use proc_macro_error::proc_macro_error;
use quote::quote;
use std::str::FromStr;

mod callable_point;
mod callable_points;
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

/// This macro generates callable point for functions marked with `#[callable_point]`
/// which can be called with dynamic link.
///
/// To use this macro, the contract must declare the import
/// `serde_json = "1.0"
/// in Cargo.toml
///
/// Functions with `#[callable_point]` are exposed to the outside world,
/// those without `#[callable_point]` are not.
///
/// For externally exposed functions, `_list_callaple_points()` is created
/// to summarize the read/write permissions of externally exposed functions
/// based on the respective function arguments `Deps` and `DepsMut`.
/// It is used to check read/write permissions.
///
/// example usage:
/// ```
/// # use cosmwasm_std::{Addr, Env, Deps, callable_points};
///
/// #[callable_points]
/// mod callable_points {
///     use cosmwasm_std::{Addr, Deps, Env};
///
///     #[callable_point] // exposed to WASM
///     fn validate_address_callable_from_other_contracts(deps: Deps, _env: Env) -> Addr {
///         // do something with deps, for example, using api.
///         deps.api.addr_validate("dummy_human_address").unwrap()
///     }
///
///     // NOT exposed to WASM
///     fn foo() -> u32 {
///         42
///     }
/// }
/// ```
#[proc_macro_error]
#[proc_macro_attribute]
pub fn callable_points(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let module = parse_macro_input!(item as syn::ItemMod);
    let module_name = module.ident;
    let body = match module.content {
        None => vec![],
        Some((_, items)) => items,
    };

    let (made, list_callable_points) = callable_points::make_callable_points(body);
    let callee_map_lit = callable_points::make_callee_map_lit(list_callable_points);

    let list_callable_points_ts = quote! {
        mod #module_name {
            #[no_mangle]
            extern "C" fn _list_callable_points() -> u32 {
                cosmwasm_std::memory::release_buffer((#callee_map_lit).to_vec()) as u32
            }

            #(#made)*

        }
    };

    TokenStream::from(list_callable_points_ts)
}

/// This macro implements functions to call dynamic linked function for attributed trait.
///
/// To use this macro, the contract must declare the import
/// `wasmer-types = { version = "2.2.1", features = ["enable-serde"] }`
/// in Cargo.toml
///
/// This macro must take an attribute specifying a struct to implement the traits for.
/// The trait must have `cosmwasm_std::Contract` as a supertrait and each
/// methods of the trait must have `&self` receiver as its first argument.
///
/// This macro can take a bool value as a named attribute `user_defined_mock`
/// When this value is true, this macro generates implement of the trait for
/// specified struct for only `target_arch = "wasm32"`.
/// So, with `user_defined_mock = true`, user can and must write mock implement of
/// the trait for specified struct with `#[cfg(not(target_arch = "wasm32"))]`.
///
/// example usage:
///
/// ```
/// use cosmwasm_std::{Addr, Contract, Deps, StdResult, dynamic_link};
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
/// // When `user_defined_mock = true` is specified, implement is generated
/// // only for "wasm32"
/// #[cfg(not(target_arch = "wasm32"))]
/// impl TraitName for ContractStruct {
///   fn callable_point_on_another_contract(&self, x: i32) -> i32 {
///     42
///   }
///
///   // validate_interface is auto generated function from `dynamic_link` macro.
///   // this function must be defined in the mock.
///   fn validate_interface(&self, dep: Deps) -> StdResult<()> {
///     Ok(())
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

/// generate an ast for `impl Into<cosmwasm::Event>` from a struct
///
/// Structure:
///
/// ```no_test
/// #[derive(IntoEvent)]
/// struct StructName {
///     field_name_1: field_type_1,
///     // if the value's type does not implement `Into<String>` trait
///     // and it implements `ToString` trait, programmers can specify
///     // to use `field_name_1.to_string()` to get string
///     // by applying `use_to_string`.
///     #[use_to_string]
///     field_name_2: field_type_2,
///     // if the value's type does not implement both `Into<String>` and
///     // `ToString` traits, programmers need specify a function
///     // to get string with `casting_fn(field_name_2)` by applying
///     // `to_string_fn(casting_fn)` attribute.
///     // this `casting_fn` needs to have the type `field_type -> String`.
///     #[to_string_fn(cast_fn_3)]
///     field_name_3: field_type_3,
/// }
/// ```
///
/// Output AST:
///
/// ```no_test
/// impl Into<cosmwasm::Event> for `StructName` {
///     fn into(self) -> Event {
///         Event::new("struct_name")
///             .add_attribute("field_name_1", self.field_value_1)
///             .add_attribute("field_name_2", self.field_value_2.to_string())
///             .add_attribute("field_name_3", casting_fn(self.field_value_3))
///     }
/// }
/// ```
#[proc_macro_derive(IntoEvent, attributes(to_string_fn, use_to_string))]
pub fn derive_into_event(input: TokenStream) -> TokenStream {
    let derive_input = parse_macro_input!(input as syn::DeriveInput);
    into_event::derive_into_event(derive_input)
}
