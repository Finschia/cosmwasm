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

use cosmwasm_std::{coins, Response};
use cosmwasm_vm::testing::{execute, instantiate, mock_env, mock_info, mock_instance};

use cw_voting_with_uuid::msg::{ExecuteMsg, InstantiateMsg};

static WASM: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/cw_voting_with_uuid.wasm");
const DENOM: &str = "voting_token";

#[test]
fn compare_gas_spent() {
    let mut deps = mock_instance(WASM, &[]);
    let env = mock_env();
    let creator = String::from("creator");
    let info = mock_info(&creator, &coins(1000, "earth"));

    let msg = InstantiateMsg {
        denom: String::from(DENOM),
    };
    let res: Response = instantiate(&mut deps, env.clone(), info.clone(), msg).unwrap();
    assert_eq!(res.messages.len(), 0);

    let uuid_msg = ExecuteMsg::MakeUuid {};
    let before_gas1 = deps.get_gas_left();
    let _execute_res: Response = execute(&mut deps, env.clone(), info.clone(), uuid_msg).unwrap();
    let gas_used_uuid = before_gas1 - deps.get_gas_left();

    let seq_msg = ExecuteMsg::MakeSequenceId {};
    let before_gas2 = deps.get_gas_left();
    let _execute_res: Response = execute(&mut deps, env, info, seq_msg).unwrap();
    let gas_used_seq_id = before_gas2 - deps.get_gas_left();

    assert!(gas_used_seq_id < gas_used_uuid);
    println!("gas_seq_id {} gas_uuid {}", gas_used_seq_id, gas_used_uuid);
}
