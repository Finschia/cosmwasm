use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    to_binary, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdResult,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, QueueQueryMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RawResponse {
    pub value: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct SmartResponse {
    pub response: String,
}

// A no-op, just empty data
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Ok(Response::default())
}

pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::DoNothing {} => handle_do_nothing(),
    }
}

fn handle_do_nothing() -> StdResult<Response> {
    Ok(Response::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Raw { address, key } => to_binary(&query_raw(deps, address, key)?),
        QueryMsg::Smart { address, msg } => to_binary(&query_smart(deps, address, msg)?),
    }
}

fn query_raw(deps: Deps, address: String, key: u8) -> StdResult<RawResponse> {
    let response: Option<Vec<u8>> = deps
        .querier
        .query_wasm_raw(address, (vec![key]).as_slice())?;
    let value = std::str::from_utf8(response.unwrap_or_default().as_slice())?.to_string();
    Ok(RawResponse { value })
}
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
struct CountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
struct SumResponse {
    pub sum: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
// the Vec contains pairs for every element in the queue
// (value of item i, sum of all elements where value > value[i])
struct ReducerResponse {
    pub counters: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, JsonSchema, Debug)]
struct ListResponse {
    /// List an empty range, both bounded
    pub empty: Vec<u32>,
    /// List all IDs lower than 0x20
    pub early: Vec<u32>,
    /// List all IDs starting from 0x20
    pub late: Vec<u32>,
}

fn query_smart(deps: Deps, address: String, msg: QueueQueryMsg) -> StdResult<SmartResponse> {
    match msg {
        QueueQueryMsg::Count {} => {
            let response: CountResponse = deps.querier.query_wasm_smart(address, &msg)?;
            Ok(SmartResponse {
                response: format!("count: {}", response.count),
            })
        }
        QueueQueryMsg::Sum {} => {
            let response: SumResponse = deps.querier.query_wasm_smart(address, &msg)?;
            Ok(SmartResponse {
                response: format!("sum: {}", response.sum),
            })
        }
        QueueQueryMsg::Reducer {} => {
            let response: ReducerResponse = deps.querier.query_wasm_smart(address, &msg)?;
            Ok(SmartResponse {
                response: format!("sum: {:?}", response.counters),
            })
        }
        QueueQueryMsg::List {} => {
            let response: ListResponse = deps.querier.query_wasm_smart(address, &msg)?;
            Ok(SmartResponse {
                response: format!(
                    "empty: {:?}\nearly: {:?}\nlate: {:?}",
                    response.empty, response.early, response.late
                ),
            })
        }
    }
}
