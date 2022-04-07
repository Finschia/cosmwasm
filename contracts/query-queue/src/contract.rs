use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_slice, to_binary, to_vec, Deps, DepsMut, Env, MessageInfo, QueryResponse, Response, StdResult, Storage,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
struct Config {
    pub queue_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct RawResponse {
    pub value: String,
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

fn write_queue_address(storage: &mut dyn Storage, addr: String) {
    let config = Config { queue_address: addr };
    storage.set(CONFIG_KEY, &to_vec(&config).unwrap());
}

fn read_queue_address(storage: &dyn Storage) -> String {
    let config: Config = from_slice(&storage.get(CONFIG_KEY).unwrap()).unwrap();
    config.queue_address
}

// A no-op, just empty data
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    write_queue_address(deps.storage, msg.queue_address);
    Ok(Response::default())
}

pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ChangeAddress { queue_address } => handle_change_address(deps, queue_address)
    }
}

fn handle_change_address(deps: DepsMut, address: String) -> StdResult<Response> {
    write_queue_address(deps.storage, address);
    Ok(Response::default())
}

pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Raw { key } => to_binary(&query_raw(deps, key)?),
        QueryMsg::Count { } => to_binary(&query_count(deps, msg)?),
        QueryMsg::Sum { } => to_binary(&query_sum(deps, msg)?),
        QueryMsg::Reducer { } => to_binary(&query_reducer(deps, msg)?),
        QueryMsg::List { } => to_binary(&query_list(deps, msg)?),
    }
}

fn query_raw(deps: Deps, key: u8) -> StdResult<RawResponse> {
    let address = read_queue_address(deps.storage);
    let response: Option<Vec<u8>> = deps
        .querier
        .query_wasm_raw(address, (vec![key]).as_slice())?;
    let value = std::str::from_utf8(response.unwrap_or_default().as_slice())?.to_string();
    Ok(RawResponse { value })
}

fn query_count(deps: Deps, msg: QueryMsg) -> StdResult<CountResponse> {
    let address = read_queue_address(deps.storage);
    let response: CountResponse = deps.querier.query_wasm_smart(address, &msg)?;
    Ok(response)
}

fn query_sum(deps: Deps, msg: QueryMsg) -> StdResult<SumResponse> {
    let address = read_queue_address(deps.storage);
    let response: SumResponse = deps.querier.query_wasm_smart(address, &msg)?;
    Ok(response)
}

fn query_reducer(deps: Deps, msg: QueryMsg) -> StdResult<ReducerResponse> {
    let address = read_queue_address(deps.storage);
    let response: ReducerResponse = deps.querier.query_wasm_smart(address, &msg)?;
    Ok(response)
}

fn query_list(deps: Deps, msg: QueryMsg) -> StdResult<ListResponse> {
    let address = read_queue_address(deps.storage);
    let response: ListResponse = deps.querier.query_wasm_smart(address, &msg)?;
    Ok(response)
}
