use cosmwasm_schema::write_api;

use simple_callee::msg::{ExecuteMsg, InstantiateMsg};

fn main() {
    write_api! {
        execute: ExecuteMsg,
        instantiate: InstantiateMsg,
    }
}
