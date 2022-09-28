use cosmwasm_std::{
    dynamic_link, entry_point, from_slice, to_binary, to_vec, Addr, Binary, Contract, Deps,
    DepsMut, Env, MessageInfo, Response,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, NumberResponse, QueryMsg};

const ADDRESS_KEY: &[u8] = b"number-address";

#[derive(Contract)]
struct NumberContract {
    address: Addr,
}

#[dynamic_link(NumberContract)]
trait Number: Contract {
    fn add(&self, by: i32);
    fn sub(&self, by: i32);
    fn mul(&self, by: i32);
    fn number(&self) -> i32;
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.storage.set(ADDRESS_KEY, &to_vec(&msg.callee_addr)?);
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Add { value } => handle_add(deps.as_ref(), value),
        ExecuteMsg::Sub { value } => handle_sub(deps.as_ref(), value),
        ExecuteMsg::Mul { value } => handle_mul(deps.as_ref(), value),
    }
}

fn handle_add(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract {
        address: address.clone(),
    };
    contract.add(by);
    let value_dyn = contract.number();
    let res: NumberResponse = deps
        .querier
        .query_wasm_smart(address, &QueryMsg::Number {})?;

    let response = Response::default()
        .add_attribute("value_by_dynamic", value_dyn.to_string())
        .add_attribute("value_by_query", res.value.to_string());

    Ok(response)
}

fn handle_sub(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract {
        address: address.clone(),
    };
    contract.sub(by);
    let value_dyn = contract.number();
    let res: NumberResponse = deps
        .querier
        .query_wasm_smart(address, &QueryMsg::Number {})?;

    let response = Response::default()
        .add_attribute("value_by_dynamic", value_dyn.to_string())
        .add_attribute("value_by_query", res.value.to_string());

    Ok(response)
}

fn handle_mul(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract {
        address: address.clone(),
    };
    contract.mul(by);
    let value_dyn = contract.number();
    let res: NumberResponse = deps
        .querier
        .query_wasm_smart(address, &QueryMsg::Number {})?;

    let response = Response::default()
        .add_attribute("value_by_dynamic", value_dyn.to_string())
        .add_attribute("value_by_query", res.value.to_string());

    Ok(response)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Add { value } => Ok(to_binary(&query_add(deps, value)?)?),
        QueryMsg::Sub { value } => Ok(to_binary(&query_sub(deps, value)?)?),
        QueryMsg::Mul { value } => Ok(to_binary(&query_mul(deps, value)?)?),
        QueryMsg::NumberDyn {} => Ok(to_binary(&query_number_dyn(deps)?)?),
        QueryMsg::Number {} => Ok(to_binary(&query_number(deps)?)?),
    }
}

fn query_number_dyn(deps: Deps) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract { address };
    let value = contract.number();
    Ok(NumberResponse { value })
}

fn query_add(deps: Deps, by: i32) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract { address };
    contract.add(by);
    let value = contract.number();
    Ok(NumberResponse { value })
}

fn query_sub(deps: Deps, by: i32) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract { address };
    contract.sub(by);
    let value = contract.number();
    Ok(NumberResponse { value })
}

fn query_mul(deps: Deps, by: i32) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(&deps.storage.get(ADDRESS_KEY).unwrap())?;
    let contract = NumberContract { address };
    contract.mul(by);
    let value = contract.number();
    Ok(NumberResponse { value })
}

fn query_number(deps: Deps) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )
    .unwrap();
    let response: NumberResponse = deps
        .querier
        .query_wasm_smart(address, &QueryMsg::Number {})?;
    Ok(response)
}
