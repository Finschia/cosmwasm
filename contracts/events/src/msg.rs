use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Attribute, Event};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
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
