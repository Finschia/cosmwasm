#![allow(clippy::field_reassign_with_default)] // see https://github.com/CosmWasm/cosmwasm/issues/685

use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{Binary, CosmosMsg, CustomQuery, HumanAddr, QueryRequest, SubMsg};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    /// if set, returns CallbackMsg::InstantiateCallback{} to the caller with this contract's address
    /// and this id
    pub callback_id: Option<String>,
}

/// This is what we return upon init if callback is set
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CallbackMsg {
    /// This type must match ExecuteMsg::InitCallback from [ibc-reflect](https://github.com/line/cosmwasm/blob/develop/contracts/ibc-reflect/src/msg.rs)
    InitCallback {
        /// Callback ID provided in the InstantiateMsg
        id: String,
        /// contract_addr is the address of this contract
        contract_addr: HumanAddr,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    ReflectMsg { msgs: Vec<CosmosMsg<CustomMsg>> },
    ReflectSubCall { msgs: Vec<SubMsg<CustomMsg>> },
    ChangeOwner { owner: HumanAddr },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Owner {},
    /// This will call out to SpecialQuery::Capitalized
    Capitalized {
        text: String,
    },
    /// Queries the blockchain and returns the result untouched
    Chain {
        request: QueryRequest<SpecialQuery>,
    },
    /// Queries another contract and returns the data
    Raw {
        contract: HumanAddr,
        key: Binary,
    },
    /// If there was a previous ReflectSubCall with this ID, returns cosmwasm_std::Reply
    SubCallResult {
        id: u64,
    },
}

// We define a custom struct for each query response

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct OwnerResponse {
    pub owner: HumanAddr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct CapitalizedResponse {
    pub text: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct ChainResponse {
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct RawResponse {
    /// The returned value of the raw query. Empty data can be the
    /// result of a non-existent key or an empty value. We cannot
    /// differentiate those two cases in cross contract queries.
    pub data: Binary,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// CustomMsg is an override of CosmosMsg::Custom to show this works and can be extended in the contract
pub enum CustomMsg {
    Debug(String),
    Raw(Binary),
}

impl Into<CosmosMsg<CustomMsg>> for CustomMsg {
    fn into(self) -> CosmosMsg<CustomMsg> {
        CosmosMsg::Custom(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// An implementation of QueryRequest::Custom to show this works and can be extended in the contract
pub enum SpecialQuery {
    Ping {},
    Capitalized { text: String },
}

impl CustomQuery for SpecialQuery {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
/// The response data for all `SpecialQuery`s
pub struct SpecialResponse {
    pub msg: String,
}
