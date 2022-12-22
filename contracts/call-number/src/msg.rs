use cosmwasm_std::Addr;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InstantiateMsg {
    pub callee_addr: Addr,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    Add { value: i32 },
    Sub { value: i32 },
    Mul { value: i32 },
    SubmsgReplyAdd { value: i32 },
    SubmsgReplySub { value: i32 },
    SubmsgReplyMul { value: i32 },
    LogQuery {},
    LogQueryDyn {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum NumberExecuteMsg {
    Add { value: i32 },
    Sub { value: i32 },
    Mul { value: i32 },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Add { value: i32 },
    Sub { value: i32 },
    Mul { value: i32 },
    NumberDyn {},
    Number {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NumberResponse {
    pub value: i32,
}
