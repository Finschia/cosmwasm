//! This integration test tries to run and call the generated wasm.
//! It depends on a Wasm build being available, which you can create with `cargo wasm`.
//! Then running `cargo integration-test` will validate we can properly call into that generated Wasm.
//!
//! You can easily convert unit tests to integration tests as follows:
//! 1. Copy them over verbatim
//! 2. Then change
//!      let mut deps = mock_dependencies(20, &[]);
//!    to
//!      let mut deps = mock_instance(WASM, &[]);
//! 3. If you access raw storage, where ever you see something like:
//!      deps.storage.get(CONFIG_KEY).expect("no data stored");
//!    replace it with:
//!      deps.with_storage(|store| {
//!          let data = store.get(CONFIG_KEY).expect("no data stored");
//!          //...
//!      });
//! 4. Anywhere you see query(&deps, ...) you must replace it with query(&mut deps, ...)

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{
    from_binary, from_slice, to_binary, Binary, ContractResult, MessageInfo, Response, SystemError,
    SystemResult, WasmQuery,
};
use cosmwasm_vm::{
    testing::{
        execute, instantiate, mock_env, mock_info, mock_instance_with_gas_limit, query, MockApi,
        MockQuerier, MockStorage,
    },
    Backend, Instance, InstanceOptions, Storage, VmResult,
};

use query_queue::msg::{ExecuteMsg, InstantiateMsg, QueryMsg, RawResponse, SumResponse};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/query_queue.wasm");
static QUEUE_WASM: &[u8] =
    include_bytes!("../../queue/target/wasm32-unknown-unknown/release/queue.wasm");

fn create_contract() -> (Instance<MockApi, MockStorage, MockQuerier>, MessageInfo) {
    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let instance_options = InstanceOptions {
        gas_limit,
        print_debug: false,
    };
    let mut deps = Backend {
        api: MockApi::default(),
        storage: MockStorage::new(),
        querier: MockQuerier::new(&[]),
    };
    let info = mock_info("creator", &[]);
    deps.querier.update_wasm(|query_msg| match query_msg {
        WasmQuery::Smart { contract_addr, msg } => {
            if contract_addr != "queue_address" {
                return SystemResult::Err(SystemError::NoSuchContract {
                    addr: contract_addr.to_string(),
                });
            };
            let q_msg: QueryMsg = from_slice(msg).unwrap();
            match q_msg {
                QueryMsg::Sum {} => SystemResult::Ok(ContractResult::Ok(
                    to_binary(&SumResponse { sum: 42 }).unwrap(),
                )),
                _ => SystemResult::Err(SystemError::Unknown {}),
            }
        }
        _ => SystemResult::Err(SystemError::Unknown {}),
    });
    let mut instance = Instance::from_code(WASM, deps, instance_options, None).unwrap();
    let res: Response = instantiate(
        &mut instance,
        mock_env(),
        info.clone(),
        InstantiateMsg {
            queue_address: "queue_address".to_string(),
        },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());
    (instance, info)
}

#[test]
fn instantiate_and_query() {
    let (mut instance, _) = create_contract();
    let data = query(&mut instance, mock_env(), QueryMsg::Sum {}).unwrap();
    let res: SumResponse = from_binary(&data).unwrap();
    assert_eq!(res.sum, 42);
}

#[test]
fn instantiate_and_change_queue_address() {
    let (mut instance, info) = create_contract();
    let _: Response = execute(
        &mut instance,
        mock_env(),
        info,
        ExecuteMsg::ChangeAddress {
            queue_address: "non_existing_address".to_string(),
        },
    )
    .unwrap();
    let res = query(&mut instance, mock_env(), QueryMsg::Sum {});
    let expected = ContractResult::Err(
        "Generic error: Querier system error: No such contract: non_existing_address".to_string(),
    );
    assert_eq!(res, expected);
}

fn create_queue_contract_and_push_42() -> (Instance<MockApi, MockStorage, MockQuerier>, MessageInfo)
{
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    struct InstantiateMsg {}
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    enum ExecuteMsg {
        Enqueue { value: i32 },
    }

    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let mut deps = mock_instance_with_gas_limit(QUEUE_WASM, gas_limit);
    let creator = String::from("creator");
    let info = mock_info(&creator, &[]);
    let res: Response =
        instantiate(&mut deps, mock_env(), info.clone(), InstantiateMsg {}).unwrap();
    assert_eq!(0, res.messages.len());
    let res: Response = execute(
        &mut deps,
        mock_env(),
        info.clone(),
        ExecuteMsg::Enqueue { value: 42 },
    )
    .unwrap();
    assert_eq!(0, res.messages.len());
    (deps, info)
}

fn create_integrated_query_contract() -> (Instance<MockApi, MockStorage, MockQuerier>, MessageInfo)
{
    #[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
    #[serde(rename_all = "snake_case")]
    pub enum QueryMsg {
        Count {},
        Sum {},
        Reducer {},
        List {},
    }

    let gas_limit = 1_000_000_000_000; // ~1ms, enough for many executions within one instance
    let instance_options = InstanceOptions {
        gas_limit,
        print_debug: false,
    };
    let mut deps = Backend {
        api: MockApi::default(),
        storage: MockStorage::new(),
        querier: MockQuerier::new(&[]),
    };
    let info = mock_info("creator", &[]);
    deps.querier.update_wasm(|query_msg| {
        let (mut queue_instance, _) = create_queue_contract_and_push_42();
        match query_msg {
            WasmQuery::Smart { contract_addr, msg } => {
                if contract_addr != "queue_address" {
                    return SystemResult::Err(SystemError::NoSuchContract {
                        addr: contract_addr.to_string(),
                    });
                };
                let q_msg: QueryMsg = from_slice(msg).unwrap();
                let res = query(&mut queue_instance, mock_env(), q_msg);
                SystemResult::Ok(res)
            }
            WasmQuery::Raw { contract_addr, key } => {
                if contract_addr != "queue_address" {
                    return SystemResult::Err(SystemError::NoSuchContract {
                        addr: contract_addr.to_string(),
                    });
                };
                let data = queue_instance
                    .with_storage(|storage| VmResult::Ok(storage.get(key).0.unwrap()))
                    .unwrap();
                if data.is_none() {
                    return SystemResult::Ok(ContractResult::Ok(Binary::from(b"null")));
                };
                SystemResult::Ok(ContractResult::Ok(Binary::from(data.unwrap())))
            }
            _ => SystemResult::Err(SystemError::Unknown {}),
        }
    });
    let mut instance = Instance::from_code(WASM, deps, instance_options, None).unwrap();
    let res: Response = instantiate(
        &mut instance,
        mock_env(),
        info.clone(),
        InstantiateMsg {
            queue_address: "queue_address".to_string(),
        },
    )
    .unwrap();

    assert_eq!(0, res.messages.len());
    (instance, info)
}

#[test]
fn integration_query_contract_queue() {
    let (mut query_instance, _) = create_integrated_query_contract();
    let data = query(&mut query_instance, mock_env(), QueryMsg::Sum {}).unwrap();
    let res: SumResponse = from_binary(&data).unwrap();
    assert_eq!(res.sum, 42);
    let data = query(&mut query_instance, mock_env(), QueryMsg::Raw { key: 0 }).unwrap();
    let res: RawResponse = from_binary(&data).unwrap();
    assert_eq!(res.item, Some(42));
}
