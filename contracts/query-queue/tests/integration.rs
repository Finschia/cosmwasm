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

use cosmwasm_std::{from_binary, from_slice, MessageInfo, Response, SystemError, SystemResult, ContractResult, WasmQuery, to_binary};
use cosmwasm_vm::{
    testing::{
        execute, instantiate, mock_env, mock_info, query,
        MockApi, MockQuerier, MockStorage,
    },
    Backend, Instance, InstanceOptions
};

use query_queue::contract::{
    SumResponse,
};
use query_queue::msg::{InstantiateMsg, QueryMsg, ExecuteMsg};

static WASM: &[u8] = include_bytes!("../target/wasm32-unknown-unknown/release/query_queue.wasm");

fn create_contract() -> (Instance<MockApi, MockStorage, MockQuerier>, MessageInfo) {
    let gas_limit = 500_000_000; // enough for many executions within one instance
    let instance_options = InstanceOptions {
        gas_limit,
        print_debug: false,
    };
    let mut deps = Backend{
        api: MockApi::default(),
        storage: MockStorage::new(),
        querier: MockQuerier::new(&[]),
    };
    let info = mock_info("creator", &[]);
    deps.querier.update_wasm(|query| match query {
        WasmQuery::Smart {
            contract_addr,
            msg,
        } => {
            if contract_addr != "queue_address" {
                return SystemResult::Err(SystemError::NoSuchContract {addr: contract_addr.to_string() })
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
    let (mut deps, _) = create_contract();
    let data = query(&mut deps, mock_env(), QueryMsg::Sum {}).unwrap();
    let res: SumResponse = from_binary(&data).unwrap();
    assert_eq!(res.sum, 42);
}

#[test]
fn instantiate_and_change_queue_address() {
    let (mut deps, info) = create_contract();
    let _: Response = execute(
        &mut deps,
        mock_env(),
        info,
        ExecuteMsg::ChangeAddress{ queue_address: "non_existing_address".to_string() }
    )
    .unwrap();
    let res = query(&mut deps, mock_env(), QueryMsg::Sum {});
    let expected = ContractResult::Err("Generic error: Querier system error: No such contract: non_existing_address".to_string());
    assert_eq!(res, expected);
}
