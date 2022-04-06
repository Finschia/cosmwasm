use cosmwasm_std::{
    callable_point, entry_point, Binary, Deps, DepsMut, Env, GlobalApi, MessageInfo, Response,
    StdResult,
};
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

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
    return x + 1;
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
fn pong_env() -> Env {
    GlobalApi::env()
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {}
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
