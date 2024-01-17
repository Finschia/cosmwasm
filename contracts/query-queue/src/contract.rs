use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    entry_point, from_json, to_json_binary, to_json_vec, Deps, DepsMut, Env, MessageInfo,
    QueryResponse, Response, StdResult, Storage,
};

use crate::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

static CONFIG_KEY: &[u8] = b"config";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
struct Config {
    pub queue_address: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct RawResponse {
    pub item: Option<i32>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct CountResponse {
    pub count: u32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct SumResponse {
    pub sum: i32,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
// the Vec contains pairs for every element in the queue
// (value of item i, sum of all elements where value > value[i])
pub struct ReducerResponse {
    pub counters: Vec<(i32, i32)>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct ListResponse {
    /// List an empty range, both bounded
    pub empty: Vec<u32>,
    /// List all IDs lower than 0x20
    pub early: Vec<u32>,
    /// List all IDs starting from 0x20
    pub late: Vec<u32>,
}

// Item of queue
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Item {
    pub value: i32,
}

fn write_queue_address(storage: &mut dyn Storage, addr: String) {
    let config = Config {
        queue_address: addr,
    };
    storage.set(CONFIG_KEY, &to_json_vec(&config).unwrap());
}

fn read_queue_address(storage: &dyn Storage) -> String {
    let config: Config = from_json(storage.get(CONFIG_KEY).unwrap()).unwrap();
    config.queue_address
}

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> StdResult<Response> {
    write_queue_address(deps.storage, msg.queue_address);
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> StdResult<Response> {
    match msg {
        ExecuteMsg::ChangeAddress { queue_address } => handle_change_address(deps, queue_address),
    }
}

fn handle_change_address(deps: DepsMut, address: String) -> StdResult<Response> {
    write_queue_address(deps.storage, address);
    Ok(Response::default())
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<QueryResponse> {
    match msg {
        QueryMsg::Raw { key } => to_json_binary(&query_raw(deps, key)?),
        QueryMsg::Count {} => to_json_binary(&query_count(deps, msg)?),
        QueryMsg::Sum {} => to_json_binary(&query_sum(deps, msg)?),
        QueryMsg::Reducer {} => to_json_binary(&query_reducer(deps, msg)?),
        QueryMsg::List {} => to_json_binary(&query_list(deps, msg)?),
    }
}

fn query_raw(deps: Deps, key: u32) -> StdResult<RawResponse> {
    let address = read_queue_address(deps.storage);
    let response = deps.querier.query_wasm_raw(address, key.to_be_bytes())?;
    match response {
        None => Ok(RawResponse { item: None }),
        Some(v) => {
            let i: Item = from_json(v.as_slice())?;
            Ok(RawResponse {
                item: Some(i.value),
            })
        }
    }
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

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{ContractResult, OwnedDeps, SystemError, SystemResult, WasmQuery};

    static QUEUE_ADDRESS: &str = "queue";

    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    struct RawQueryResponse {
        value: u32,
    }

    fn create_contract() -> OwnedDeps<MockStorage, MockApi, MockQuerier> {
        let mut deps = mock_dependencies();
        let info = mock_info("creator", &[]);
        let res = instantiate(
            deps.as_mut(),
            mock_env(),
            info,
            InstantiateMsg {
                queue_address: QUEUE_ADDRESS.to_string(),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());
        deps.querier.update_wasm(|query| match query {
            WasmQuery::Smart {
                contract_addr: _,
                msg,
            } => {
                let q_msg: QueryMsg = from_json(msg).unwrap();
                match q_msg {
                    QueryMsg::Count {} => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&CountResponse { count: 1 }).unwrap(),
                    )),
                    QueryMsg::Sum {} => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&SumResponse { sum: 42 }).unwrap(),
                    )),
                    QueryMsg::Reducer {} => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&ReducerResponse {
                            counters: vec![(42, 0)],
                        })
                        .unwrap(),
                    )),
                    QueryMsg::List {} => SystemResult::Ok(ContractResult::Ok(
                        to_json_binary(&ListResponse {
                            empty: vec![],
                            early: vec![0],
                            late: vec![],
                        })
                        .unwrap(),
                    )),
                    _ => SystemResult::Err(SystemError::Unknown {}),
                }
            }
            WasmQuery::Raw {
                contract_addr: _,
                key: _,
            } => SystemResult::Ok(ContractResult::Ok(
                to_json_binary(&RawQueryResponse { value: 42 }).unwrap(),
            )),
            _ => SystemResult::Err(SystemError::Unknown {}),
        });
        deps
    }

    #[test]
    fn test_instantiate() {
        let deps = create_contract();
        let config: Config = from_json(deps.storage.get(CONFIG_KEY).unwrap()).unwrap();
        assert_eq!(config.queue_address, QUEUE_ADDRESS);
    }

    #[test]
    fn test_queue_raw() {
        let deps = create_contract();
        assert_eq!(
            query_raw(deps.as_ref(), 42).unwrap(),
            RawResponse { item: Some(42) }
        );
    }

    #[test]
    fn test_queue_smart() {
        let deps = create_contract();
        assert_eq!(
            query_sum(deps.as_ref(), QueryMsg::Sum {}).unwrap(),
            SumResponse { sum: 42 }
        );
    }
}
