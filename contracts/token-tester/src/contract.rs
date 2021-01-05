use std::str::FromStr;

use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, HandleResponse, HandleResult,
    HumanAddr, InitResponse, MessageInfo, StdResult, Uint128,
};

use cosmwasm_ext::{
    Change, LinkMsgWrapper, LinkTokenQuerier, Module, MsgData, Response, Target, Token, TokenMsg,
    TokenPerm, TokenRoute,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, config_read, State};

pub fn init(deps: DepsMut, _env: Env, info: MessageInfo, _msg: InitMsg) -> StdResult<InitResponse> {
    let state = State {
        owner: deps.api.canonical_address(&info.sender)?,
    };

    config(deps.storage).save(&state)?;

    Ok(InitResponse::default())
}

pub fn handle(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
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
            deps, env, info, owner, to, name, symbol, img_uri, meta, amount, mintable, decimals,
        ),
        HandleMsg::Transfer {
            from,
            contract_id,
            to,
            amount,
        } => try_transfer(deps, env, info, from, contract_id, to, amount),
        HandleMsg::TransferFrom {
            proxy,
            from,
            contract_id,
            to,
            amount,
        } => try_transfer_from(deps, env, info, proxy, from, contract_id, to, amount),
        HandleMsg::Mint {
            from,
            contract_id,
            to,
            amount,
        } => try_mint(deps, env, info, from, contract_id, to, amount),
        HandleMsg::Burn {
            from,
            contract_id,
            amount,
        } => try_burn(deps, env, info, from, contract_id, amount),
        HandleMsg::BurnFrom {
            proxy,
            from,
            contract_id,
            amount,
        } => try_burn_from(deps, env, info, proxy, from, contract_id, amount),
        HandleMsg::GrantPerm {
            from,
            contract_id,
            to,
            permission,
        } => try_grant_perm(deps, env, info, from, contract_id, to, permission),
        HandleMsg::RevokePerm {
            from,
            contract_id,
            permission,
        } => try_revoke_perm(deps, env, info, from, contract_id, permission),
        HandleMsg::Modify {
            owner,
            contract_id,
            key,
            value,
        } => try_modify(deps, env, info, owner, contract_id, key, value),
        HandleMsg::Approve {
            approver,
            contract_id,
            proxy,
        } => try_approve(deps, env, info, approver, contract_id, proxy),
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetToken { contract_id } => query_token(deps, env, contract_id),
        QueryMsg::GetBalance {
            contract_id,
            address,
        } => query_balance(deps, env, contract_id, address),
        QueryMsg::GetTotal {
            contract_id,
            target,
        } => query_total(deps, env, contract_id, target),
        QueryMsg::GetPerm {
            contract_id,
            address,
        } => query_perm(deps, env, contract_id, address),
        QueryMsg::GetIsApproved {
            proxy,
            contract_id,
            approver,
        } => query_is_approved(deps, env, proxy, contract_id, approver),
        QueryMsg::GetApprovers { proxy, contract_id } => {
            query_approvers(deps, env, proxy, contract_id)
        }
    }
}

#[allow(clippy::too_many_arguments)]
pub fn try_issue(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "issue")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "transfer")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_transfer_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "transfer_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_mint(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "mint")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "burn")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "burn_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_grant_perm(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "grant_perm")],
        data: None,
    };
    Ok(res)
}

pub fn try_revoke_perm(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "revoke_perm")],
        data: None,
    };
    Ok(res)
}

pub fn try_modify(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "modify")],
        data: None,
    };
    Ok(res)
}

pub fn try_approve(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
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
        attributes: vec![attr("action", "approve")],
        data: None,
    };
    Ok(res)
}

fn query_token(deps: Deps, _env: Env, contract_id: String) -> StdResult<Binary> {
    let res = match LinkTokenQuerier::new(deps.querier).query_token(contract_id)? {
        Some(token_response) => token_response,
        None => return to_binary(&None::<Box<Response<Token>>>),
    };

    let out = to_binary(&res)?;
    Ok(out)
}

fn query_balance(
    deps: Deps,
    _env: Env,
    contract_id: String,
    address: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkTokenQuerier::new(deps.querier)
        .query_balance(contract_id, address)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_total(
    deps: Deps,
    _env: Env,
    contract_id: String,
    target_str: String,
) -> StdResult<Binary> {
    let target = Target::from_str(&target_str).unwrap();
    if Target::Supply == target {
        let res = LinkTokenQuerier::new(deps.querier)
            .query_supply(contract_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else if Target::Mint == target {
        let res = LinkTokenQuerier::new(deps.querier)
            .query_mint(contract_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else {
        let res = LinkTokenQuerier::new(deps.querier)
            .query_burn(contract_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    }
}

fn query_perm(deps: Deps, _env: Env, contract_id: String, address: HumanAddr) -> StdResult<Binary> {
    let res = match LinkTokenQuerier::new(deps.querier).query_perm(contract_id, address)? {
        Some(permissions) => permissions,
        None => return to_binary(&None::<Box<Vec<TokenPerm>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_is_approved(
    deps: Deps,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    approver: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkTokenQuerier::new(deps.querier)
        .query_is_approved(proxy, contract_id, approver)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_approvers(
    deps: Deps,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
) -> StdResult<Binary> {
    let res = match LinkTokenQuerier::new(deps.querier).query_approvers(proxy, contract_id)? {
        Some(approvers) => approvers,
        None => return to_binary(&None::<Box<Vec<HumanAddr>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn _query_owner(deps: Deps, _env: Env) -> StdResult<HumanAddr> {
    let state = config_read(deps.storage).load()?;
    Ok(deps.api.human_address(&state.owner)?)
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::{coins, Env, OwnedDeps};

    fn create_contract(owner: String) -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, Env) {
        let mut deps = mock_dependencies(&coins(1000, "cony"));
        let env = mock_env();
        let info = mock_info(owner, &coins(1000, "cony"));
        let res = init(deps.as_mut(), env.clone(), info.clone(), InitMsg {}).unwrap();
        assert_eq!(0, res.messages.len());
        (deps, env)
    }

    #[test]
    fn init_contract() {
        let addr = "creator";

        let (deps, _) = create_contract(addr.to_string());
        let env = mock_env();
        let value = _query_owner(deps.as_ref(), env).unwrap();
        assert_eq!(addr, value.as_str());
    }
}
