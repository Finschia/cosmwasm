use cosmwasm_std::{Attribute, Event};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Event {
        #[serde(rename = "type")]
        ty: String,
        attributes: Vec<Attribute>,
    },
    Events {
        events: Vec<Event>,
    },
    Attribute {
        key: String,
        value: String,
    },
    Attributes {
        attributes: Vec<Attribute>,
    },
}
