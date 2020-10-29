use std::env::current_dir;
use std::fs::create_dir_all;

use cosmwasm_schema::{export_schema, remove_schemas, schema_for};
use cosmwasm_ext::{LinkMsgWrapper, TokenRoute, TokenMsg, CollectionRoute, CollectionMsg};

fn main() {
    let mut out_dir = current_dir().unwrap();
    out_dir.push("schema");
    create_dir_all(&out_dir).unwrap();
    remove_schemas(&out_dir).unwrap();

    export_schema(&schema_for!(LinkMsgWrapper<TokenRoute, TokenMsg>), &out_dir);
    export_schema(&schema_for!(LinkMsgWrapper<CollectionRoute, CollectionMsg>), &out_dir);
}
