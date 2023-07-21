use cosmwasm_std::{
    dynamic_link, entry_point, from_slice, to_binary, to_vec, wasm_execute, Addr, Binary, Contract,
    Deps, DepsMut, Env, MessageInfo, Reply, Response, SubMsg,
};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg, NumberExecuteMsg, NumberResponse, QueryMsg};

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
        ExecuteMsg::SubmsgReplyAdd { value } => handle_submsg_reply_add(deps.as_ref(), value),
        ExecuteMsg::SubmsgReplySub { value } => handle_submsg_reply_sub(deps.as_ref(), value),
        ExecuteMsg::SubmsgReplyMul { value } => handle_submsg_reply_mul(deps.as_ref(), value),
        ExecuteMsg::LogQuery {} => handle_log_query(deps.as_ref()),
        ExecuteMsg::LogQueryDyn {} => handle_log_query_dyn(deps.as_ref()),
    }
}

fn handle_add(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
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
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
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
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
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

fn handle_submsg_reply_add(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let execute_msg = SubMsg::reply_on_success(
        wasm_execute(address, &NumberExecuteMsg::Add { value: by }, vec![])?,
        0,
    );
    let response = Response::default().add_submessage(execute_msg);
    Ok(response)
}

fn handle_submsg_reply_sub(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let execute_msg = SubMsg::reply_on_success(
        wasm_execute(address, &NumberExecuteMsg::Sub { value: by }, vec![])?,
        0,
    );
    let response = Response::default().add_submessage(execute_msg);
    Ok(response)
}

fn handle_submsg_reply_mul(deps: Deps, by: i32) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let execute_msg = SubMsg::reply_on_success(
        wasm_execute(address, &NumberExecuteMsg::Mul { value: by }, vec![])?,
        0,
    );
    let response = Response::default().add_submessage(execute_msg);
    Ok(response)
}

fn handle_log_query(deps: Deps) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let res: NumberResponse = deps
        .querier
        .query_wasm_smart(address, &QueryMsg::Number {})?;

    let response = Response::default().add_attribute("value_by_query", res.value.to_string());

    Ok(response)
}

fn handle_log_query_dyn(deps: Deps) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let contract = NumberContract { address };
    let value_dyn = contract.number();

    let response = Response::default().add_attribute("value_by_query_dyn", value_dyn.to_string());

    Ok(response)
}

#[entry_point]
pub fn reply(deps: DepsMut, _env: Env, _msg: Reply) -> Result<Response, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let contract = NumberContract {
        address: address.clone(),
    };
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
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let contract = NumberContract { address };
    let value = contract.number();
    Ok(NumberResponse { value })
}

// This function is used to check if an appropriate error has been occurred in the VM,
// when a caller with read-only permissions attempts to call a callable point with read/write permissions.
// https://github.com/Finschia/cosmwasm/blob/03abb0871ca5cfe8b874561795bc59d12562002f/packages/vm/src/dynamic_link.rs#L333-L336
fn query_add(deps: Deps, by: i32) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let contract = NumberContract { address };
    contract.add(by);
    let value = contract.number();
    Ok(NumberResponse { value })
}

// This function is used to check if an appropriate error has been occurred in the VM,
// when a caller with read-only permissions attempts to call a callable point with read/write permissions.
// https://github.com/Finschia/cosmwasm/blob/03abb0871ca5cfe8b874561795bc59d12562002f/packages/vm/src/dynamic_link.rs#L333-L336
fn query_sub(deps: Deps, by: i32) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
    let contract = NumberContract { address };
    contract.sub(by);
    let value = contract.number();
    Ok(NumberResponse { value })
}

// This function is used to check if an appropriate error has been occurred in the VM,
// when a caller with read-only permissions attempts to call a callable point with read/write permissions.
// https://github.com/Finschia/cosmwasm/blob/03abb0871ca5cfe8b874561795bc59d12562002f/packages/vm/src/dynamic_link.rs#L333-L336
fn query_mul(deps: Deps, by: i32) -> Result<NumberResponse, ContractError> {
    let address: Addr = from_slice(
        &deps
            .storage
            .get(ADDRESS_KEY)
            .ok_or(ContractError::StorageError)?,
    )?;
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
    )?;
    let response: NumberResponse = deps
        .querier
        .query_wasm_smart(address, &QueryMsg::Number {})?;
    Ok(response)
}
