use crate::coin_helpers::validate_sent_sufficient_coin;
use crate::error::ContractError;
use crate::msg::{
    CreatePollResponse, ExecuteMsg, InstantiateMsg, PollResponse, QueryMsg, TokenStakeResponse,
};
use crate::state::{
    load_bank, load_config, load_poll, may_load_bank, may_load_poll, save_bank, save_config,
    save_poll, Poll, PollStatus, State, TokenManager, Voter,
};
use cosmwasm_std::{
    attr, coin, entry_point, new_uuid, to_json_binary, Addr, BankMsg, Binary, Coin, CosmosMsg,
    Deps, DepsMut, Env, MessageInfo, Response, StdError, StdResult, Storage, Uint128, Uuid,
};

pub const VOTING_TOKEN: &str = "voting_token";
pub const DEFAULT_END_HEIGHT_BLOCKS: &u64 = &100_800_u64;
const MIN_STAKE_AMOUNT: u128 = 1;
const MIN_DESC_LENGTH: u64 = 3;
const MAX_DESC_LENGTH: u64 = 64;

#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    let state = State {
        denom: msg.denom,
        owner: info.sender,
        staked_tokens: Uint128::zero(),
    };

    save_config(deps.storage, &state)?;
    Ok(Response::default())
}

#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::StakeVotingTokens {} => stake_voting_tokens(deps, env, info),
        ExecuteMsg::WithdrawVotingTokens { amount } => {
            withdraw_voting_tokens(deps, env, info, amount)
        }
        ExecuteMsg::CastVote {
            poll_id,
            vote,
            weight,
        } => cast_vote(deps, env, info, poll_id, vote, weight),
        ExecuteMsg::EndPoll { poll_id } => end_poll(deps, env, info, poll_id),
        ExecuteMsg::CreatePoll {
            quorum_percentage,
            description,
            start_height,
            end_height,
        } => create_poll(
            deps,
            env,
            info,
            quorum_percentage,
            description,
            start_height,
            end_height,
        ),
        ExecuteMsg::MakeUuid {} => make_uuid(deps, env, info),
        ExecuteMsg::MakeSequenceId {} => make_seq_id(deps, env, info),
    }
}

pub fn stake_voting_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
) -> Result<Response, ContractError> {
    let sender_key = info.sender;

    let mut token_manager: TokenManager =
        may_load_bank(deps.storage, &sender_key)?.unwrap_or_default();

    let mut state = load_config(deps.storage)?;

    validate_sent_sufficient_coin(&info.funds, Some(coin(MIN_STAKE_AMOUNT, &state.denom)))?;
    let funds = info
        .funds
        .iter()
        .find(|coin| coin.denom.eq(&state.denom))
        .unwrap();

    token_manager.token_balance += funds.amount;

    let staked_tokens = state.staked_tokens.u128() + funds.amount.u128();
    state.staked_tokens = Uint128::from(staked_tokens);
    save_config(deps.storage, &state)?;

    save_bank(deps.storage, &sender_key, &token_manager)?;

    Ok(Response::default())
}

// Withdraw amount if not staked. By default all funds will be withdrawn.
pub fn withdraw_voting_tokens(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    amount: Option<Uint128>,
) -> Result<Response, ContractError> {
    let sender_key = info.sender;

    if let Some(mut token_manager) = may_load_bank(deps.storage, &sender_key)? {
        let largest_staked = locked_amount(&sender_key, deps.storage);
        let withdraw_amount = amount.unwrap_or(token_manager.token_balance);
        if largest_staked + withdraw_amount > token_manager.token_balance {
            let max_amount = token_manager.token_balance.checked_sub(largest_staked)?;
            Err(ContractError::ExcessiveWithdraw { max_amount })
        } else {
            let balance = token_manager.token_balance.checked_sub(withdraw_amount)?;
            token_manager.token_balance = balance;

            save_bank(deps.storage, &sender_key, &token_manager)?;

            let mut state = load_config(deps.storage)?;
            let staked_tokens = state.staked_tokens.checked_sub(withdraw_amount)?;
            state.staked_tokens = staked_tokens;
            save_config(deps.storage, &state)?;

            Ok(send_tokens(
                &sender_key,
                vec![coin(withdraw_amount.u128(), &state.denom)],
                "approve",
            ))
        }
    } else {
        Err(ContractError::PollNoStake {})
    }
}

/// validate_description returns an error if the description is invalid
fn validate_description(description: &str) -> Result<(), ContractError> {
    if (description.len() as u64) < MIN_DESC_LENGTH {
        Err(ContractError::DescriptionTooShort {
            min_desc_length: MIN_DESC_LENGTH,
        })
    } else if (description.len() as u64) > MAX_DESC_LENGTH {
        Err(ContractError::DescriptionTooLong {
            max_desc_length: MAX_DESC_LENGTH,
        })
    } else {
        Ok(())
    }
}

/// validate_quorum_percentage returns an error if the quorum_percentage is invalid
/// (we require 0-100)
fn validate_quorum_percentage(quorum_percentage: Option<u8>) -> Result<(), ContractError> {
    match quorum_percentage {
        Some(qp) => {
            if qp > 100 {
                return Err(ContractError::PollQuorumPercentageMismatch {
                    quorum_percentage: qp,
                });
            }
            Ok(())
        }
        None => Ok(()),
    }
}

/// validate_end_height returns an error if the poll ends in the past
fn validate_end_height(end_height: Option<u64>, env: Env) -> Result<(), ContractError> {
    if end_height.is_some() && env.block.height >= end_height.unwrap() {
        Err(ContractError::PollCannotEndInPast {})
    } else {
        Ok(())
    }
}

/// create a new poll
pub fn create_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    quorum_percentage: Option<u8>,
    description: String,
    start_height: Option<u64>,
    end_height: Option<u64>,
) -> Result<Response, ContractError> {
    validate_quorum_percentage(quorum_percentage)?;
    validate_end_height(end_height, env.clone())?;
    validate_description(&description)?;

    let poll_id = new_uuid(&env, deps.storage, deps.api)?;

    let new_poll = Poll {
        creator: info.sender,
        status: PollStatus::InProgress,
        quorum_percentage,
        yes_votes: Uint128::zero(),
        no_votes: Uint128::zero(),
        voters: vec![],
        voter_info: vec![],
        end_height: end_height.unwrap_or(env.block.height + DEFAULT_END_HEIGHT_BLOCKS),
        start_height,
        description,
    };
    save_poll(deps.storage, &poll_id, &new_poll)?;

    let r = Response::new()
        .add_attribute("action", "create_poll")
        .add_attribute("creator", new_poll.creator)
        .add_attribute("poll_id", poll_id.to_string())
        .add_attribute(
            "quorum_percentage",
            quorum_percentage.unwrap_or(0).to_string(),
        )
        .add_attribute("end_height", new_poll.end_height.to_string())
        .add_attribute("start_height", start_height.unwrap_or(0).to_string())
        .set_data(to_json_binary(&CreatePollResponse { poll_id })?);
    Ok(r)
}

/*
 * Ends a poll. Only the creator of a given poll can end that poll.
 */
pub fn end_poll(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    poll_id: Uuid,
) -> Result<Response, ContractError> {
    let mut a_poll: Poll = load_poll(deps.storage, &poll_id)?;

    if a_poll.creator != info.sender {
        return Err(ContractError::PollNotCreator {
            creator: a_poll.creator.to_string(),
            sender: info.sender.to_string(),
        });
    }

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if let Some(start_height) = a_poll.start_height {
        if start_height > env.block.height {
            return Err(ContractError::PoolVotingPeriodNotStarted { start_height });
        }
    }

    if a_poll.end_height > env.block.height {
        return Err(ContractError::PollVotingPeriodNotExpired {
            expire_height: a_poll.end_height,
        });
    }

    let mut no = 0u128;
    let mut yes = 0u128;

    for voter in &a_poll.voter_info {
        if voter.vote == "yes" {
            yes += voter.weight.u128();
        } else {
            no += voter.weight.u128();
        }
    }
    let tallied_weight = yes + no;

    let mut rejected_reason = "";
    let mut passed = false;

    if tallied_weight > 0 {
        let state = load_config(deps.storage)?;

        let staked_weight = deps
            .querier
            .query_balance(&env.contract.address, &state.denom)
            .unwrap()
            .amount
            .u128();

        if staked_weight == 0 {
            return Err(ContractError::PollNoStake {});
        }

        let quorum = ((tallied_weight / staked_weight) * 100) as u8;
        if a_poll.quorum_percentage.is_some() && quorum < a_poll.quorum_percentage.unwrap() {
            // Quorum: More than quorum_percentage of the total staked tokens at the end of the voting
            // period need to have participated in the vote.
            rejected_reason = "Quorum not reached";
        } else if yes > tallied_weight / 2 {
            //Threshold: More than 50% of the tokens that participated in the vote
            // (after excluding “Abstain” votes) need to have voted in favor of the proposal (“Yes”).
            a_poll.status = PollStatus::Passed;
            passed = true;
        } else {
            rejected_reason = "Threshold not reached";
        }
    } else {
        rejected_reason = "Quorum not reached";
    }
    if !passed {
        a_poll.status = PollStatus::Rejected
    }
    save_poll(deps.storage, &poll_id, &a_poll)?;

    for voter in &a_poll.voters {
        unlock_tokens(deps.storage, voter, poll_id)?;
    }

    let attributes = vec![
        attr("action", "end_poll"),
        attr("poll_id", poll_id.to_string()),
        attr("rejected_reason", rejected_reason),
        attr("passed", passed.to_string()),
    ];

    Ok(Response::new().add_attributes(attributes))
}

// unlock voter's tokens in a given poll
fn unlock_tokens(
    storage: &mut dyn Storage,
    voter: &Addr,
    poll_id: Uuid,
) -> Result<Response, ContractError> {
    let mut token_manager: TokenManager = load_bank(storage, voter)?;

    // unlock entails removing the mapped poll_id, retaining the rest
    token_manager.locked_tokens.retain(|(k, _)| k != &poll_id);
    save_bank(storage, voter, &token_manager)?;
    Ok(Response::default())
}

// finds the largest locked amount in participated polls.
fn locked_amount(voter: &Addr, storage: &dyn Storage) -> Uint128 {
    let token_manager: TokenManager = load_bank(storage, voter).unwrap();
    token_manager
        .locked_tokens
        .iter()
        .map(|(_, v)| *v)
        .max()
        .unwrap_or_default()
}

fn has_voted(voter: &Addr, a_poll: &Poll) -> bool {
    a_poll.voters.iter().any(|i| i == voter)
}

pub fn cast_vote(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    poll_id: Uuid,
    vote: String,
    weight: Uint128,
) -> Result<Response, ContractError> {
    let mut a_poll: Poll = match load_poll(deps.storage, &poll_id) {
        Ok(poll) => poll,
        Err(_) => return Err(ContractError::PollNotExist {}),
    };

    if a_poll.status != PollStatus::InProgress {
        return Err(ContractError::PollNotInProgress {});
    }

    if has_voted(&info.sender, &a_poll) {
        return Err(ContractError::PollSenderVoted {});
    }

    let sender_key = info.sender;
    let mut token_manager: TokenManager =
        may_load_bank(deps.storage, &sender_key)?.unwrap_or_default();

    if token_manager.token_balance < weight {
        return Err(ContractError::PollInsufficientStake {});
    }
    token_manager.participated_polls.push(poll_id);
    token_manager.locked_tokens.push((poll_id, weight));
    save_bank(deps.storage, &sender_key, &token_manager)?;

    a_poll.voters.push(sender_key.clone());

    let voter_info = Voter { vote, weight };

    a_poll.voter_info.push(voter_info);
    save_poll(deps.storage, &poll_id, &a_poll)?;

    let attributes = vec![
        attr("action", "vote_casted"),
        attr("poll_id", poll_id.to_string()),
        attr("weight", weight.to_string()),
        attr("voter", &sender_key),
    ];

    Ok(Response::new().add_attributes(attributes))
}

fn send_tokens(to_address: &Addr, amount: Vec<Coin>, action: &str) -> Response {
    let attributes = vec![attr("action", action), attr("to", to_address.clone())];

    Response::new()
        .add_message(CosmosMsg::Bank(BankMsg::Send {
            to_address: to_address.to_string(),
            amount,
        }))
        .add_attributes(attributes)
}

pub fn make_uuid(deps: DepsMut, env: Env, _info: MessageInfo) -> Result<Response, ContractError> {
    let uuid = new_uuid(&env, deps.storage, deps.api)?;
    let r = Response::new()
        .add_attribute("action", "make_uuid")
        .add_attribute("uuid", uuid.to_string());
    Ok(r)
}

pub fn make_seq_id(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
) -> Result<Response, ContractError> {
    let seq_id: u64 = 0;

    let r = Response::new()
        .add_attribute("action", "make_seq_id")
        .add_attribute("uuid", seq_id.to_string());
    Ok(r)
}

#[entry_point]
pub fn query(deps: Deps, _env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::Config {} => to_json_binary(&load_config(deps.storage)?),
        QueryMsg::TokenStake { address } => {
            token_balance(deps, deps.api.addr_validate(address.as_str())?)
        }
        QueryMsg::Poll { poll_id } => query_poll(deps, poll_id),
    }
}

fn query_poll(deps: Deps, poll_id: Uuid) -> StdResult<Binary> {
    let poll: Poll = match may_load_poll(deps.storage, &poll_id)? {
        Some(poll) => poll,
        None => return Err(StdError::generic_err("Poll does not exist")),
    };

    let resp = PollResponse {
        creator: poll.creator.to_string(),
        status: poll.status,
        quorum_percentage: poll.quorum_percentage,
        end_height: Some(poll.end_height),
        start_height: poll.start_height,
        description: poll.description,
    };
    to_json_binary(&resp)
}

fn token_balance(deps: Deps, address: Addr) -> StdResult<Binary> {
    let token_manager = may_load_bank(deps.storage, &address)?.unwrap_or_default();

    let resp = TokenStakeResponse {
        token_balance: token_manager.token_balance,
    };

    to_json_binary(&resp)
}
