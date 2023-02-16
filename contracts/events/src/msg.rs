use cosmwasm_std::{Addr, Attribute, Event};
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
    EventDyn {
        address: Addr,
        #[serde(rename = "type")]
        ty: String,
        attributes: Vec<Attribute>,
    },
    EventsDyn {
        address: Addr,
        events: Vec<Event>,
    },
    AttributeDyn {
        address: Addr,
        key: String,
        value: String,
    },
    AttributesDyn {
        address: Addr,
        attributes: Vec<Attribute>,
    },
}
