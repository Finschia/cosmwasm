use cosmwasm_schema::write_api;

use events::msg::{ExecuteMsg, InstantiateMsg};

fn main() {
    write_api! {
        execute: ExecuteMsg,
        instantiate: InstantiateMsg,
    }
}
