use std::str::FromStr;

use cosmwasm_std::{
    log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, Querier, StdResult, Storage, Uint128,
};

use cosmwasm_ext::{
    Change, LinkMsgWrapper, LinkTokenQuerier, Module, MsgData, Response, Target, Token, TokenMsg,
    TokenPerm, TokenRoute,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

pub fn init<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    _msg: InitMsg,
) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&env.message.sender)?,
    };

    config(&mut deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle<S: Storage, A: Api, Q: Querier>(
    deps: &mut Extern<S, A, Q>,
    env: Env,
    msg: HandleMsg,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    match msg {
        HandleMsg::Issue {
            owner,
            to,
            name,
            symbol,
            img_uri,
            meta,
            amount,
            mintable,
            decimals,
        } => try_issue(
            deps, env, owner, to, name, symbol, img_uri, meta, amount, mintable, decimals,
        ),
        HandleMsg::Transfer {
            from,
            contract_id,
            to,
            amount,
        } => try_transfer(deps, env, from, contract_id, to, amount),
        HandleMsg::TransferFrom {
            proxy,
            from,
            contract_id,
            to,
            amount,
        } => try_transfer_from(deps, env, proxy, from, contract_id, to, amount),
        HandleMsg::Mint {
            from,
            contract_id,
            to,
            amount,
        } => try_mint(deps, env, from, contract_id, to, amount),
        HandleMsg::Burn {
            from,
            contract_id,
            amount,
        } => try_burn(deps, env, from, contract_id, amount),
        HandleMsg::BurnFrom {
            proxy,
            from,
            contract_id,
            amount,
        } => try_burn_from(deps, env, proxy, from, contract_id, amount),
        HandleMsg::GrantPerm {
            from,
            contract_id,
            to,
            permission,
        } => try_grant_perm(deps, env, from, contract_id, to, permission),
        HandleMsg::RevokePerm {
            from,
            contract_id,
            permission,
        } => try_revoke_perm(deps, env, from, contract_id, permission),
        HandleMsg::Modify {
            owner,
            contract_id,
            key,
            value,
        } => try_modify(deps, env, owner, contract_id, key, value),
        HandleMsg::Approve {
            approver,
            contract_id,
            proxy,
        } => try_approve(deps, env, approver, contract_id, proxy),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetToken { contract_id } => query_token(deps, contract_id),
        QueryMsg::GetBalance {
            contract_id,
            address,
        } => query_balance(deps, contract_id, address),
        QueryMsg::GetTotal {
            contract_id,
            target,
        } => query_total(deps, contract_id, target),
        QueryMsg::GetPerm {
            contract_id,
            address,
        } => query_perm(deps, contract_id, address),
        QueryMsg::GetIsApproved {
            proxy,
            contract_id,
            approver,
        } => query_is_approved(deps, proxy, contract_id, approver),
        QueryMsg::GetApprovers { proxy, contract_id } => query_approvers(deps, proxy, contract_id),
    }
}

#[allow(clippy::too_many_arguments)]
pub fn try_issue<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    owner: HumanAddr,
    to: HumanAddr,
    name: String,
    symbol: String,
    img_uri: String,
    meta: String,
    amount: Uint128,
    mintable: bool,
    decimals: Uint128,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> =
        LinkMsgWrapper::<TokenRoute, TokenMsg> {
            module: Module::Tokenencode,
            msg_data: MsgData {
                route: TokenRoute::Issue,
                data: TokenMsg::Issue {
                    owner,
                    to,
                    name,
                    symbol,
                    img_uri,
                    meta,
                    amount,
                    mintable,
                    decimals,
                },
            },
        }
        .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "issue")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    amount: Uint128,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    // Some kind of logic.

    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::Transfer,
            data: TokenMsg::Transfer {
                from,
                contract_id,
                to,
                amount,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "transfer")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    amount: Uint128,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    // Some kind of logic.

    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::TransferFrom,
            data: TokenMsg::TransferFrom {
                proxy,
                from,
                contract_id,
                to,
                amount,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "transfer_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_mint<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    amount: Uint128,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::Mint,
            data: TokenMsg::Mint {
                from,
                contract_id,
                to,
                amount,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "mint")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    amount: Uint128,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::Burn,
            data: TokenMsg::Burn {
                from,
                contract_id,
                amount,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "burn")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    from: HumanAddr,
    contract_id: String,
    amount: Uint128,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::BurnFrom,
            data: TokenMsg::BurnFrom {
                proxy,
                from,
                contract_id,
                amount,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "burn_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_grant_perm<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    perm_str: String,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let permission = TokenPerm::from_str(&perm_str).unwrap();
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::GrantPerm,
            data: TokenMsg::GrantPerm {
                from,
                contract_id,
                to,
                permission,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "grant_perm")],
        data: None,
    };
    Ok(res)
}

pub fn try_revoke_perm<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    perm_str: String,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let permission = TokenPerm::from_str(&perm_str).unwrap();
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::RevokePerm,
            data: TokenMsg::RevokePerm {
                from,
                contract_id,
                permission,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "revoke_perm")],
        data: None,
    };
    Ok(res)
}

pub fn try_modify<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    owner: HumanAddr,
    contract_id: String,
    key: String,
    value: String,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let change = Change::new(key, value);
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::Modify,
            data: TokenMsg::Modify {
                owner,
                contract_id,
                changes: vec![change],
            },
        },
    }
    .into();
    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "modify")],
        data: None,
    };
    Ok(res)
}

pub fn try_approve<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    approver: HumanAddr,
    contract_id: String,
    proxy: HumanAddr,
) -> HandleResult<LinkMsgWrapper<TokenRoute, TokenMsg>> {
    let msg: CosmosMsg<LinkMsgWrapper<TokenRoute, TokenMsg>> = LinkMsgWrapper {
        module: Module::Tokenencode,
        msg_data: MsgData {
            route: TokenRoute::Approve,
            data: TokenMsg::Approve {
                approver,
                contract_id,
                proxy,
            },
        },
    }
    .into();

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "approve")],
        data: None,
    };
    Ok(res)
}

fn query_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
) -> StdResult<Binary> {
    let res = match LinkTokenQuerier::new(&deps.querier).query_token(contract_id)? {
        Some(token_response) => token_response,
        None => return to_binary(&None::<Box<Response<Token>>>),
    };

    let out = to_binary(&res)?;
    Ok(out)
}

fn query_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    address: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkTokenQuerier::new(&deps.querier)
        .query_balance(contract_id, address)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_total<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    target_str: String,
) -> StdResult<Binary> {
    let target = Target::from_str(&target_str).unwrap();
    if Target::Supply == target {
        let res = LinkTokenQuerier::new(&deps.querier)
            .query_supply(contract_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else if Target::Mint == target {
        let res = LinkTokenQuerier::new(&deps.querier)
            .query_mint(contract_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else {
        let res = LinkTokenQuerier::new(&deps.querier)
            .query_burn(contract_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    }
}

fn query_perm<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    address: HumanAddr,
) -> StdResult<Binary> {
    let res = match LinkTokenQuerier::new(&deps.querier).query_perm(contract_id, address)? {
        Some(permissions) => permissions,
        None => return to_binary(&None::<Box<Vec<TokenPerm>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_is_approved<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proxy: HumanAddr,
    contract_id: String,
    approver: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkTokenQuerier::new(&deps.querier)
        .query_is_approved(proxy, contract_id, approver)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_approvers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proxy: HumanAddr,
    contract_id: String,
) -> StdResult<Binary> {
    let res = match LinkTokenQuerier::new(&deps.querier).query_approvers(proxy, contract_id)? {
        Some(approvers) => approvers,
        None => return to_binary(&None::<Box<Vec<HumanAddr>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn _query_owner<S: Storage, A: Api, Q: Querier>(deps: &Extern<S, A, Q>) -> StdResult<HumanAddr> {
    let state = config_read(&deps.storage).load()?;
    Ok(deps.api.human_address(&state.owner)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, MockApi, MockQuerier, MockStorage};
    use cosmwasm_std::{coins, Env};

    fn create_contract(owner: String) -> (Extern<MockStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies(20, &coins(1000, "cony"));
        let env = mock_env(owner, &coins(1000, "cony"));
        let res = init(&mut deps, env.clone(), InitMsg {}).unwrap();
        assert_eq!(0, res.messages.len());
        (deps, env)
    }

    #[test]
    fn init_contract() {
        let addr = "creator";

        let (deps, _) = create_contract(addr.to_string());
        let value = _query_owner(&deps).unwrap();
        assert_eq!("creator", value.as_str());
    }
}
