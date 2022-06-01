use cosmwasm_std::{
    callable_point, dynamic_link, entry_point, DepsMut, Env, GlobalApi, MessageInfo, Response,
    Addr, to_vec,
};
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[callable_point]
fn pong(x: u64) -> u64 {
    x + 1
}

#[derive(Serialize, Deserialize)]
pub struct ExampleStruct {
    pub str_field: String,
    pub u64_field: u64,
}

#[callable_point]
fn pong_with_struct(example: ExampleStruct) -> ExampleStruct {
    ExampleStruct {
        str_field: example.str_field + " world",
        u64_field: example.u64_field + 1,
    }
}

#[callable_point]
fn pong_with_tuple(input: (String, i32)) -> (String, i32) {
    (input.0 + " world", input.1 + 1)
}

#[callable_point]
fn pong_with_tuple_takes_2_args(input1: String, input2: i32) -> (String, i32) {
    (input1 + " world", input2 + 1)
}

#[callable_point]
fn pong_env() -> Env {
    GlobalApi::env()
}

#[dynamic_link(contract_name = "dynamic_caller_contract")]
extern "C" {
    fn should_never_be_called();
}

#[callable_point]
fn reentrancy(addr: Addr) {
    GlobalApi::with_deps_mut(|deps|{
        deps.storage
        .set(b"dynamic_caller_contract", &to_vec(&addr).unwrap());
    });
    should_never_be_called()
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}
