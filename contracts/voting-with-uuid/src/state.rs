use cosmwasm_std::{
    from_json, storage_keys::namespace_with_key, to_json_vec, Addr, StdError, StdResult, Storage,
    Uint128, Uuid,
};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

static CONFIG_KEY: &[u8] = b"config";
static POLL_KEY: &[u8] = b"polls";
static BANK_KEY: &[u8] = b"bank";

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct State {
    pub denom: String,
    pub owner: Addr,
    pub staked_tokens: Uint128,
}

#[derive(Default, Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct TokenManager {
    pub token_balance: Uint128,              // total staked balance
    pub locked_tokens: Vec<(Uuid, Uint128)>, //maps poll_id to weight voted
    pub participated_polls: Vec<Uuid>,       // poll_id
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Voter {
    pub vote: String,
    pub weight: Uint128,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub enum PollStatus {
    InProgress,
    Tally,
    Passed,
    Rejected,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, Eq, JsonSchema)]
pub struct Poll {
    pub creator: Addr,
    pub status: PollStatus,
    pub quorum_percentage: Option<u8>,
    pub yes_votes: Uint128,
    pub no_votes: Uint128,
    pub voters: Vec<Addr>,
    pub voter_info: Vec<Voter>,
    pub end_height: u64,
    pub start_height: Option<u64>,
    pub description: String,
}

pub fn save_config(storage: &mut dyn Storage, item: &State) -> StdResult<()> {
    storage.set(CONFIG_KEY, &to_json_vec(item)?);
    Ok(())
}

pub fn load_config(storage: &dyn Storage) -> StdResult<State> {
    storage
        .get(CONFIG_KEY)
        .ok_or_else(|| StdError::not_found("config"))
        .and_then(from_json)
}

pub fn save_poll(storage: &mut dyn Storage, key: &Uuid, poll: &Poll) -> StdResult<()> {
    storage.set(
        &namespace_with_key(&[POLL_KEY], key.as_bytes()),
        &to_json_vec(poll)?,
    );
    Ok(())
}

pub fn may_load_poll(storage: &dyn Storage, key: &Uuid) -> StdResult<Option<Poll>> {
    storage
        .get(&namespace_with_key(&[POLL_KEY], key.as_bytes()))
        .map(from_json)
        .transpose()
}

pub fn load_poll(storage: &dyn Storage, key: &Uuid) -> StdResult<Poll> {
    may_load_poll(storage, key)?.ok_or_else(|| StdError::not_found(format!("poll {key:?}")))
}

pub fn save_bank(
    storage: &mut dyn Storage,
    key: &Addr,
    token_manager: &TokenManager,
) -> StdResult<()> {
    storage.set(
        &namespace_with_key(&[BANK_KEY], key.as_bytes()),
        &to_json_vec(token_manager)?,
    );
    Ok(())
}

pub fn may_load_bank(storage: &dyn Storage, key: &Addr) -> StdResult<Option<TokenManager>> {
    storage
        .get(&namespace_with_key(&[BANK_KEY], key.as_bytes()))
        .map(from_json)
        .transpose()
}

pub fn load_bank(storage: &dyn Storage, key: &Addr) -> StdResult<TokenManager> {
    may_load_bank(storage, key)?.ok_or_else(|| StdError::not_found(format!("bank {key:?}")))
}
