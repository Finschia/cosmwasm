use crate::state::PollStatus;
use cosmwasm_std::{Uint128, Uuid};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct InstantiateMsg {
    pub denom: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ExecuteMsg {
    CastVote {
        poll_id: Uuid,
        vote: String,
        weight: Uint128,
    },
    StakeVotingTokens {},
    WithdrawVotingTokens {
        amount: Option<Uint128>,
    },
    CreatePoll {
        quorum_percentage: Option<u8>,
        description: String,
        start_height: Option<u64>,
        end_height: Option<u64>,
    },
    EndPoll {
        poll_id: Uuid,
    },
    MakeUuid {},
    MakeSequenceId {},
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    Config {},
    TokenStake { address: String },
    Poll { poll_id: Uuid },
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct PollResponse {
    pub creator: String,
    pub status: PollStatus,
    pub quorum_percentage: Option<u8>,
    pub end_height: Option<u64>,
    pub start_height: Option<u64>,
    pub description: String,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct CreatePollResponse {
    pub poll_id: Uuid,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct PollCountResponse {
    pub poll_count: u64,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct TokenStakeResponse {
    pub token_balance: Uint128,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct MakeUuidResponse {
    pub uuid: Uuid,
}

#[derive(Serialize, Deserialize, Clone, PartialEq, Eq, JsonSchema)]
pub struct MakeSequenceIdResponse {
    pub seq_id: u64,
}
