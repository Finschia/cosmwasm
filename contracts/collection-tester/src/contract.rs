use std::convert::TryFrom;
use std::str::FromStr;

use cosmwasm_std::{
    attr, to_binary, Binary, CosmosMsg, Deps, DepsMut, Env, HumanAddr, MessageInfo, Response,
    StdError, StdResult, Uint128,
};

use cosmwasm_ext::{
    Change, Coin, Collection, CollectionMsg, CollectionPerm, CollectionRoute,
    LinkCollectionQuerier, LinkMsgWrapper, MintNFTParam, Module, MsgData, Response as ExtResponse,
    Target,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, State};

pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    info: MessageInfo,
    _msg: InitMsg,
) -> StdResult<Response> {
    let state = State {
        owner: deps.api.canonical_address(&info.sender)?,
    };

    config(deps.storage).save(&state)?;

    Ok(Response {
        submessages: vec![],
        messages: vec![],
        attributes: vec![attr("action", "instantiate")],
        data: None,
    })
}

type CollectionExecuteResponse = Response<LinkMsgWrapper<CollectionRoute, CollectionMsg>>;
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: HandleMsg,
) -> StdResult<CollectionExecuteResponse> {
    match msg {
        HandleMsg::Create {
            owner,
            name,
            meta,
            base_img_uri,
        } => try_create(deps, env, info, owner, name, meta, base_img_uri),
        HandleMsg::IssueNft {
            owner,
            contract_id,
            name,
            meta,
        } => try_issue_nft(deps, env, info, owner, contract_id, name, meta),
        HandleMsg::IssueFt {
            owner,
            contract_id,
            to,
            name,
            meta,
            amount,
            mintable,
            decimals,
        } => try_issue_ft(
            deps,
            env,
            info,
            owner,
            contract_id,
            to,
            name,
            meta,
            amount,
            mintable,
            decimals,
        ),
        HandleMsg::MintNft {
            from,
            contract_id,
            to,
            token_types,
        } => try_mint_nft(deps, env, info, from, contract_id, to, token_types),
        HandleMsg::MintFt {
            from,
            contract_id,
            to,
            tokens,
        } => try_mint_ft(deps, env, info, from, contract_id, to, tokens),
        HandleMsg::BurnNft {
            from,
            contract_id,
            token_id,
        } => try_burn_nft(deps, env, info, from, contract_id, token_id),
        HandleMsg::BurnNftFrom {
            proxy,
            contract_id,
            from,
            token_ids,
        } => try_burn_nft_from(deps, env, info, proxy, contract_id, from, token_ids),
        HandleMsg::BurnFt {
            from,
            contract_id,
            amounts,
        } => try_burn_ft(deps, env, info, from, contract_id, amounts),
        HandleMsg::BurnFtFrom {
            proxy,
            contract_id,
            from,
            amounts,
        } => try_burn_ft_from(deps, env, info, proxy, contract_id, from, amounts),
        HandleMsg::TransferNft {
            from,
            contract_id,
            to,
            token_ids,
        } => try_transfer_nft(deps, env, info, from, contract_id, to, token_ids),
        HandleMsg::TransferNftFrom {
            proxy,
            contract_id,
            from,
            to,
            token_ids,
        } => try_transfer_nft_from(deps, env, info, proxy, contract_id, from, to, token_ids),
        HandleMsg::TransferFt {
            from,
            contract_id,
            to,
            tokens,
        } => try_transfer_ft(deps, env, info, from, contract_id, to, tokens),
        HandleMsg::TransferFtFrom {
            proxy,
            contract_id,
            from,
            to,
            tokens,
        } => try_transfer_ft_from(deps, env, info, proxy, contract_id, from, to, tokens),
        HandleMsg::Modify {
            owner,
            contract_id,
            token_type,
            token_index,
            key,
            value,
        } => try_modify(
            deps,
            env,
            info,
            owner,
            contract_id,
            token_type,
            token_index,
            key,
            value,
        ),
        HandleMsg::Approve {
            approver,
            contract_id,
            proxy,
        } => try_approve(deps, env, info, approver, contract_id, proxy),
        HandleMsg::Disapprove {
            approver,
            contract_id,
            proxy,
        } => try_disapprove(deps, env, info, approver, contract_id, proxy),
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
        HandleMsg::Attach {
            from,
            contract_id,
            to_token_id,
            token_id,
        } => try_attach(deps, env, info, from, contract_id, to_token_id, token_id),
        HandleMsg::Detach {
            from,
            contract_id,
            token_id,
        } => try_detach(deps, env, info, from, contract_id, token_id),
        HandleMsg::AttachFrom {
            proxy,
            contract_id,
            from,
            to_token_id,
            token_id,
        } => try_attach_from(
            deps,
            env,
            info,
            proxy,
            contract_id,
            from,
            to_token_id,
            token_id,
        ),
        HandleMsg::DetachFrom {
            proxy,
            contract_id,
            from,
            token_id,
        } => try_detach_from(deps, env, info, proxy, contract_id, from, token_id),
    }
}

pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCollection { contract_id } => query_collection(deps, env, contract_id),
        QueryMsg::GetBalance {
            contract_id,
            token_id,
            addr,
        } => query_balance(deps, env, contract_id, token_id, addr),
        QueryMsg::GetTokenType {
            contract_id,
            token_id,
        } => query_token_type(deps, env, contract_id, token_id),
        QueryMsg::GetTokenTypes { contract_id } => query_token_types(deps, env, contract_id),
        QueryMsg::GetToken {
            contract_id,
            token_id,
        } => query_token(deps, env, contract_id, token_id),
        QueryMsg::GetTokens { contract_id } => query_tokens(deps, env, contract_id),
        QueryMsg::GetNftCount {
            contract_id,
            token_id,
            target,
        } => query_nft_count(deps, env, contract_id, token_id, target),
        QueryMsg::GetTotal {
            contract_id,
            token_id,
            target,
        } => query_total(deps, env, contract_id, token_id, target),
        QueryMsg::GetRootOrParentOrChildren {
            contract_id,
            token_id,
            target,
        } => query_root_or_parent_or_children(deps, env, contract_id, token_id, target),
        QueryMsg::GetPerms { contract_id, addr } => query_perms(deps, env, contract_id, addr),
        QueryMsg::GetApproved {
            contract_id,
            proxy,
            approver,
        } => query_is_approved(deps, env, contract_id, proxy, approver),
        QueryMsg::GetApprovers { proxy, contract_id } => {
            query_approvers(deps, env, proxy, contract_id)
        }
    }
}

pub fn try_create(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: HumanAddr,
    name: String,
    meta: String,
    base_img_uri: String,
) -> StdResult<CollectionExecuteResponse> {
    // Some kind of logic.

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::Create,
                data: CollectionMsg::Create {
                    owner,
                    name,
                    meta,
                    base_img_uri,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg.into()],
        attributes: vec![attr("action", "create")],
        data: None,
    };
    Ok(res)
}

pub fn try_issue_nft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: HumanAddr,
    contract_id: String,
    name: String,
    meta: String,
) -> StdResult<CollectionExecuteResponse> {
    // Some kind of logic.

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::IssueNft,
                data: CollectionMsg::IssueNft {
                    owner,
                    contract_id,
                    name,
                    meta,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "issue_nft")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_issue_ft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    name: String,
    meta: String,
    amount: Uint128,
    mintable: bool,
    decimals: Uint128,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::IssueFt,
                data: CollectionMsg::IssueFt {
                    owner,
                    contract_id,
                    to,
                    name,
                    meta,
                    amount,
                    mintable,
                    decimals,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "issue_ft")],
        data: None,
    };
    Ok(res)
}

pub fn try_mint_nft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    token_types: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let mut params: Vec<MintNFTParam> = vec![];
    for (i, _) in token_types.iter().enumerate() {
        let mint_nft_param = MintNFTParam::new(
            "nft-".to_string() + &(i.to_string()),
            "".to_string(),
            token_types[i].clone(),
        );
        params.push(mint_nft_param)
    }

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::MintNft,
                data: CollectionMsg::MintNft {
                    from,
                    contract_id,
                    to,
                    params,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "mint_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_mint_ft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    tokens: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let mut amount: Vec<Coin> = vec![];
    tokens.iter().for_each(|token| {
        let v: Vec<&str> = (token).split(':').collect();
        let coin = Coin::new(v[1].to_string(), Uint128::try_from(v[0]).unwrap());
        amount.push(coin);
    });

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::MintFt,
                data: CollectionMsg::MintFt {
                    from,
                    contract_id,
                    to,
                    amount,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "mint_ft")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_nft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    token_id: String,
) -> StdResult<CollectionExecuteResponse> {
    let token_ids = vec![token_id];

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::BurnNft,
                data: CollectionMsg::BurnNft {
                    from,
                    contract_id,
                    token_ids,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "burn_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_nft_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    token_ids: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::BurnNftFrom,
                data: CollectionMsg::BurnNftFrom {
                    proxy,
                    contract_id,
                    from,
                    token_ids,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "burn_nft_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_ft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    tokens: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let mut amount: Vec<Coin> = vec![];
    tokens.iter().for_each(|token| {
        let v: Vec<&str> = (token).split(':').collect();
        let coin = Coin::new(v[1].to_string(), Uint128::try_from(v[0]).unwrap());
        amount.push(coin);
    });

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::BurnFt,
                data: CollectionMsg::BurnFt {
                    from,
                    contract_id,
                    amount,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "burn_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_ft_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    tokens: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let mut amount: Vec<Coin> = vec![];
    tokens.iter().for_each(|token| {
        let v: Vec<&str> = (token).split(':').collect();
        let coin = Coin::new(v[1].to_string(), Uint128::try_from(v[0]).unwrap());
        amount.push(coin);
    });

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::BurnFtFrom,
                data: CollectionMsg::BurnFtFrom {
                    proxy,
                    contract_id,
                    from,
                    amount,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "burn_nft_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_nft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    token_ids: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::TransferNft,
                data: CollectionMsg::TransferNft {
                    from,
                    contract_id,
                    to,
                    token_ids,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "transfer_nft")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_transfer_nft_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    to: HumanAddr,
    token_ids: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::TransferNftFrom,
                data: CollectionMsg::TransferNftFrom {
                    proxy,
                    contract_id,
                    from,
                    to,
                    token_ids,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "transfer_nft_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_ft(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    tokens: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let mut amount: Vec<Coin> = vec![];
    tokens.iter().for_each(|token| {
        let v: Vec<&str> = (token).split(':').collect();
        let coin = Coin::new(v[1].to_string(), Uint128::try_from(v[0]).unwrap());
        amount.push(coin);
    });

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::TransferFt,
                data: CollectionMsg::TransferFt {
                    from,
                    contract_id,
                    to,
                    amount,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "transfer_ft")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_transfer_ft_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    to: HumanAddr,
    tokens: Vec<String>,
) -> StdResult<CollectionExecuteResponse> {
    let mut amount: Vec<Coin> = vec![];
    tokens.iter().for_each(|token| {
        let v: Vec<&str> = (token).split(':').collect();
        let coin = Coin::new(v[1].to_string(), Uint128::try_from(v[0]).unwrap());
        amount.push(coin);
    });

    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::TransferFtFrom,
                data: CollectionMsg::TransferFtFrom {
                    proxy,
                    contract_id,
                    from,
                    to,
                    amount,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "transfer_ft_from")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_modify(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    owner: HumanAddr,
    contract_id: String,
    token_type: String,
    token_index: String,
    key: String,
    value: String,
) -> StdResult<CollectionExecuteResponse> {
    let change = Change::new(key, value);
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::Modify,
                data: CollectionMsg::Modify {
                    owner,
                    contract_id,
                    token_type,
                    token_index,
                    changes: vec![change],
                },
            },
        }
        .into();
    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "modify_collection")],
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
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::Approve,
                data: CollectionMsg::Approve {
                    approver,
                    contract_id,
                    proxy,
                },
            },
        }
        .into();
    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "approve")],
        data: None,
    };
    Ok(res)
}

pub fn try_disapprove(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    approver: HumanAddr,
    contract_id: String,
    proxy: HumanAddr,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::Disapprove,
                data: CollectionMsg::Disapprove {
                    approver,
                    contract_id,
                    proxy,
                },
            },
        }
        .into();
    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "approve")],
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
) -> StdResult<CollectionExecuteResponse> {
    let permission = CollectionPerm::from_str(&perm_str).unwrap();
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::GrantPerm,
                data: CollectionMsg::GrantPerm {
                    from,
                    contract_id,
                    to,
                    permission,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
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
) -> StdResult<CollectionExecuteResponse> {
    let permission = CollectionPerm::from_str(&perm_str).unwrap();
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::RevokePerm,
                data: CollectionMsg::RevokePerm {
                    from,
                    contract_id,
                    permission,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "revoke_perm")],
        data: None,
    };
    Ok(res)
}

pub fn try_attach(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    to_token_id: String,
    token_id: String,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::Attach,
                data: CollectionMsg::Attach {
                    from,
                    contract_id,
                    to_token_id,
                    token_id,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "attach")],
        data: None,
    };
    Ok(res)
}

pub fn try_detach(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    from: HumanAddr,
    contract_id: String,
    token_id: String,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::Detach,
                data: CollectionMsg::Detach {
                    from,
                    contract_id,
                    token_id,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "detach")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_attach_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    to_token_id: String,
    token_id: String,
) -> StdResult<Response<LinkMsgWrapper<CollectionRoute, CollectionMsg>>> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::AttachFrom,
                data: CollectionMsg::AttachFrom {
                    proxy,
                    contract_id,
                    from,
                    to_token_id,
                    token_id,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "attach_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_detach_from(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    token_id: String,
) -> StdResult<CollectionExecuteResponse> {
    let msg: CosmosMsg<LinkMsgWrapper<CollectionRoute, CollectionMsg>> =
        LinkMsgWrapper::<CollectionRoute, CollectionMsg> {
            module: Module::Collectionencode,
            msg_data: MsgData {
                route: CollectionRoute::DetachFrom,
                data: CollectionMsg::DetachFrom {
                    proxy,
                    contract_id,
                    from,
                    token_id,
                },
            },
        }
        .into();

    let res = Response {
        submessages: vec![],
        messages: vec![msg],
        attributes: vec![attr("action", "detach_from")],
        data: None,
    };
    Ok(res)
}

fn query_collection(deps: Deps, _env: Env, contract_id: String) -> StdResult<Binary> {
    let res = match LinkCollectionQuerier::new(deps.querier).query_collection(contract_id)? {
        Some(collection_response) => collection_response,
        None => return to_binary(&None::<Box<ExtResponse<Collection>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_balance(
    deps: Deps,
    _env: Env,
    contract_id: String,
    token_id: String,
    addr: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_balance(contract_id, token_id, addr)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_token_type(
    deps: Deps,
    _env: Env,
    contract_id: String,
    token_id: String,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_token_type(contract_id, token_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_token_types(deps: Deps, _env: Env, contract_id: String) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_token_types(contract_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_token(deps: Deps, _env: Env, contract_id: String, token_id: String) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_token(contract_id, token_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_tokens(deps: Deps, _env: Env, contract_id: String) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_tokens(contract_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_nft_count(
    deps: Deps,
    _env: Env,
    contract_id: String,
    token_id: String,
    target: String,
) -> StdResult<Binary> {
    let res = match &*target {
        "count" => LinkCollectionQuerier::new(deps.querier)
            .query_nft_count(contract_id, token_id)
            .unwrap(),
        "mint" => LinkCollectionQuerier::new(deps.querier)
            .query_nft_mint(contract_id, token_id)
            .unwrap(),
        "burn" => LinkCollectionQuerier::new(deps.querier)
            .query_nft_burn(contract_id, token_id)
            .unwrap(),
        _ => Uint128(0),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_total(
    deps: Deps,
    _env: Env,
    contract_id: String,
    token_id: String,
    target_str: String,
) -> StdResult<Binary> {
    let target = Target::from_str(&target_str).unwrap();
    if Target::Supply == target {
        let res = LinkCollectionQuerier::new(deps.querier)
            .query_supply(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else if Target::Mint == target {
        let res = LinkCollectionQuerier::new(deps.querier)
            .query_mint(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else {
        let res = LinkCollectionQuerier::new(deps.querier)
            .query_burn(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    }
}

fn query_root_or_parent_or_children(
    deps: Deps,
    _env: Env,
    contract_id: String,
    token_id: String,
    target: String,
) -> StdResult<Binary> {
    if target == "root" {
        let res = LinkCollectionQuerier::new(deps.querier)
            .query_root(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else if target == "parent" {
        let res = LinkCollectionQuerier::new(deps.querier)
            .query_parent(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else {
        let res = LinkCollectionQuerier::new(deps.querier)
            .query_children(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    }
}

fn query_perms(deps: Deps, _env: Env, contract_id: String, addr: HumanAddr) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_perm(contract_id, addr)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_is_approved(
    deps: Deps,
    _env: Env,
    contract_id: String,
    proxy: HumanAddr,
    approver: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(deps.querier)
        .query_is_approved(contract_id, proxy, approver)
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
    let res = match LinkCollectionQuerier::new(deps.querier).query_approvers(proxy, contract_id)? {
        Some(approvers) => approvers,
        None => return to_binary(&None::<Box<Vec<HumanAddr>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}
