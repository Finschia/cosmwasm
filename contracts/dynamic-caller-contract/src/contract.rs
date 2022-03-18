use cosmwasm_std::{
    dynamic_link, entry_point, to_vec, Binary, Deps, DepsMut, Env, MessageInfo,
    Response, StdResult, Uint128,
};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[derive(Serialize, Deserialize)]
pub struct ExampleStruct {
    pub str_field: String,
    pub u64_field: u64,
}
impl fmt::Display for ExampleStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.str_field, self.u64_field)
    }
}

#[dynamic_link(contract_name = "dynamic_callee_contract")]
extern "C" {
    fn pong(ping_num: u64) -> u64;
    fn pong_with_struct(example: ExampleStruct) -> ExampleStruct;
}

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.storage
        .set(b"dynamic_callee_contract", &to_vec(&msg.callee_addr)?);

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Ping { ping_num } => try_ping(deps, ping_num),
    }
}

pub fn try_ping(_deps: DepsMut, ping_num: Uint128) -> Result<Response, ContractError> {
    let pong_ret = pong(ping_num.u128() as u64);
    let struct_ret = pong_with_struct(ExampleStruct {
        str_field: String::from("hello"),
        u64_field: 100u64,
    });

    let mut res = Response::default();
    res.add_attribute("returned_pong", pong_ret.to_string());
    res.add_attribute("returned_pong_with_struct", struct_ret.to_string());
    Ok(res)
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
