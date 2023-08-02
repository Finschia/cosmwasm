use cosmwasm_schema::write_api;

use intermediate_number::msg::{ExecuteMsg, InstantiateMsg, QueryMsg};

fn main() {
    write_api! {
        execute: ExecuteMsg,
        instantiate: InstantiateMsg,
        query: QueryMsg,
    }
}
