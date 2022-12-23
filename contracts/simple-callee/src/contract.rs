use cosmwasm_std::{callable_point, entry_point, DepsMut, Env, MessageInfo, Response};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[callable_point]
fn succeed(_deps: DepsMut, _env: Env) {
    ()
}

#[callable_point]
fn fail(_deps: DepsMut, _env: Env) {
    panic!()
}
