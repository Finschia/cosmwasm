use cosmwasm_schema::write_api;

use dynamic_callee_contract::msg::{ExecuteMsg, InstantiateMsg};

fn main() {
    write_api! {
        execute: ExecuteMsg,
        instantiate: InstantiateMsg,
    }
}
