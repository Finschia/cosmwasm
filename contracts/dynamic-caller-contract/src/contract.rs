use cosmwasm_std::{
    entry_point, to_vec, Binary, Deps, DepsMut, Env, MessageInfo, Response, StdResult,
    Uint128,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

#[link(wasm_import_module = "dynamic_callee_contract")]
extern "C" {
    fn pong(int: u64) -> u64;
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
    let pong_ret: u64;
    unsafe {
        pong_ret = pong(ping_num.u128() as u64);
    }

    let mut res = Response::default();
    res.add_attribute("returned_pong", pong_ret.to_string());
    Ok(res)
}

#[entry_point]
pub fn query(_deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {}
}
