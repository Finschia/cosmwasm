use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::CosmosMsg;
use crate::msg_token::{TokenRoute, TokenMsg};
use crate::msg_collection::{CollectionRoute, CollectionMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Tokenencode,
    Collectionencode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct Change {
    pub field: String,
    pub value: String,
}

impl Change {
    pub fn new(field: String, value: String) -> Self {
        return Change {field, value}
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LinkMsgWrapper<R, D> {
    pub module: Module,
    pub msg_data: MsgData<R, D>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct MsgData<R, D> {
    pub route: R,
    pub data: D,
}

impl Into<CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>>> for LinkMsgWrapper<TokenRoute, TokenMsg> {
    fn into(self) -> CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> {
        CosmosMsg::Custom(self)
    }
}

impl Into<CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>>> for LinkMsgWrapper<CollectionRoute, CollectionMsg> {
    fn into(self) -> CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
        CosmosMsg::Custom(self)
    }
}
