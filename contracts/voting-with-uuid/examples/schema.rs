use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use std::env::current_dir;
use std::fs::create_dir_all;

use cw_voting_with_uuid::msg::{
    ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, TokenStakeResponse,
};
use cw_voting_with_uuid::state::State;

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(InstantiateMsg), &out_dir);
    export_schema(&schema_for!(ExecuteMsg), &out_dir);
    export_schema(&schema_for!(PollResponse), &out_dir);
    export_schema(&schema_for!(QueryMsg), &out_dir);
    export_schema(&schema_for!(State), &out_dir);
    export_schema(&schema_for!(TokenStakeResponse), &out_dir);
}
