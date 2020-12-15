use std::convert::TryFrom;
use std::str::FromStr;

use cosmwasm_std::{
    log, to_binary, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, Querier, StdResult, Storage, Uint128,
};

use cosmwasm_ext::{
    Change, Coin, Collection, CollectionMsg, CollectionPerm, CollectionRoute,
    LinkCollectionQuerier, LinkMsgWrapper, MintNFTParam, Module, MsgData, Response, Target,
};

use crate::msg::{HandleMsg, InitMsg, QueryMsg};
use crate::state::{config, State};

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
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
    match msg {
        HandleMsg::Create {
            owner,
            name,
            meta,
            base_img_uri,
        } => try_create(deps, env, owner, name, meta, base_img_uri),
        HandleMsg::IssueNft {
            owner,
            contract_id,
            name,
            meta,
        } => try_issue_nft(deps, env, owner, contract_id, name, meta),
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
        } => try_mint_nft(deps, env, from, contract_id, to, token_types),
        HandleMsg::MintFt {
            from,
            contract_id,
            to,
            tokens,
        } => try_mint_ft(deps, env, from, contract_id, to, tokens),
        HandleMsg::BurnNft {
            from,
            contract_id,
            token_id,
        } => try_burn_nft(deps, env, from, contract_id, token_id),
        HandleMsg::BurnNftFrom {
            proxy,
            contract_id,
            from,
            token_ids,
        } => try_burn_nft_from(deps, env, proxy, contract_id, from, token_ids),
        HandleMsg::BurnFt {
            from,
            contract_id,
            amounts,
        } => try_burn_ft(deps, env, from, contract_id, amounts),
        HandleMsg::BurnFtFrom {
            proxy,
            contract_id,
            from,
            amounts,
        } => try_burn_ft_from(deps, env, proxy, contract_id, from, amounts),
        HandleMsg::TransferNft {
            from,
            contract_id,
            to,
            token_ids,
        } => try_transfer_nft(deps, env, from, contract_id, to, token_ids),
        HandleMsg::TransferNftFrom {
            proxy,
            contract_id,
            from,
            to,
            token_ids,
        } => try_transfer_nft_from(deps, env, proxy, contract_id, from, to, token_ids),
        HandleMsg::TransferFt {
            from,
            contract_id,
            to,
            tokens,
        } => try_transfer_ft(deps, env, from, contract_id, to, tokens),
        HandleMsg::TransferFtFrom {
            proxy,
            contract_id,
            from,
            to,
            tokens,
        } => try_transfer_ft_from(deps, env, proxy, contract_id, from, to, tokens),
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
        } => try_approve(deps, env, approver, contract_id, proxy),
        HandleMsg::Disapprove {
            approver,
            contract_id,
            proxy,
        } => try_disapprove(deps, env, approver, contract_id, proxy),
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
        HandleMsg::Attach {
            from,
            contract_id,
            to_token_id,
            token_id,
        } => try_attach(deps, env, from, contract_id, to_token_id, token_id),
        HandleMsg::Detach {
            from,
            contract_id,
            token_id,
        } => try_detach(deps, env, from, contract_id, token_id),
        HandleMsg::AttachFrom {
            proxy,
            contract_id,
            from,
            to_token_id,
            token_id,
        } => try_attach_from(deps, env, proxy, contract_id, from, to_token_id, token_id),
        HandleMsg::DetachFrom {
            proxy,
            contract_id,
            from,
            token_id,
        } => try_detach_from(deps, env, proxy, contract_id, from, token_id),
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetCollection { contract_id } => query_collection(deps, contract_id),
        QueryMsg::GetBalance {
            contract_id,
            token_id,
            addr,
        } => query_balance(deps, contract_id, token_id, addr),
        QueryMsg::GetTokenType {
            contract_id,
            token_id,
        } => query_token_type(deps, contract_id, token_id),
        QueryMsg::GetTokenTypes { contract_id } => query_token_types(deps, contract_id),
        QueryMsg::GetToken {
            contract_id,
            token_id,
        } => query_token(deps, contract_id, token_id),
        QueryMsg::GetTokens { contract_id } => query_tokens(deps, contract_id),
        QueryMsg::GetNft {
            contract_id,
            token_id,
            target,
        } => query_nft(deps, contract_id, token_id, target),
        QueryMsg::GetTotal {
            contract_id,
            token_id,
            target,
        } => query_total(deps, contract_id, token_id, target),
        QueryMsg::GetRootOrParentOrChildren {
            contract_id,
            token_id,
            target,
        } => query_root_or_parent_or_children(deps, contract_id, token_id, target),
        QueryMsg::GetPerms { contract_id, addr } => query_perms(deps, contract_id, addr),
        QueryMsg::GetApproved {
            contract_id,
            proxy,
            approver,
        } => query_is_approved(deps, contract_id, proxy, approver),
        QueryMsg::GetApprovers { proxy, contract_id } => query_approvers(deps, proxy, contract_id),
    }
}

pub fn try_create<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    owner: HumanAddr,
    name: String,
    meta: String,
    base_img_uri: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "create")],
        data: None,
    };
    Ok(res)
}

pub fn try_issue_nft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    owner: HumanAddr,
    contract_id: String,
    name: String,
    meta: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "issue_nft")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_issue_ft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    owner: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    name: String,
    meta: String,
    amount: Uint128,
    mintable: bool,
    decimals: Uint128,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "issue_ft")],
        data: None,
    };
    Ok(res)
}

pub fn try_mint_nft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    token_types: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "mint_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_mint_ft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    tokens: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "mint_ft")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_nft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    token_id: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "burn_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_nft_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    token_ids: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "burn_nft_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_ft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    tokens: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "burn_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_burn_ft_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    tokens: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "burn_nft_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_nft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    token_ids: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "transfer_nft")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_nft_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    to: HumanAddr,
    token_ids: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "transfer_nft_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_ft<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to: HumanAddr,
    tokens: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "transfer_ft")],
        data: None,
    };
    Ok(res)
}

pub fn try_transfer_ft_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    to: HumanAddr,
    tokens: Vec<String>,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "transfer_ft_from")],
        data: None,
    };
    Ok(res)
}

#[allow(clippy::too_many_arguments)]
pub fn try_modify<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    owner: HumanAddr,
    contract_id: String,
    token_type: String,
    token_index: String,
    key: String,
    value: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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
    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "modify_collection")],
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
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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
    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "approve")],
        data: None,
    };
    Ok(res)
}

pub fn try_disapprove<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    approver: HumanAddr,
    contract_id: String,
    proxy: HumanAddr,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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
    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "approve")],
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
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "revoke_perm")],
        data: None,
    };
    Ok(res)
}

pub fn try_attach<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    to_token_id: String,
    token_id: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "attach")],
        data: None,
    };
    Ok(res)
}

pub fn try_detach<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    from: HumanAddr,
    contract_id: String,
    token_id: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "detach")],
        data: None,
    };
    Ok(res)
}

pub fn try_attach_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    to_token_id: String,
    token_id: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "attach_from")],
        data: None,
    };
    Ok(res)
}

pub fn try_detach_from<S: Storage, A: Api, Q: Querier>(
    _deps: &mut Extern<S, A, Q>,
    _env: Env,
    proxy: HumanAddr,
    contract_id: String,
    from: HumanAddr,
    token_id: String,
) -> HandleResult<LinkMsgWrapper<CollectionRoute, CollectionMsg>> {
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

    let res = HandleResponse {
        messages: vec![msg],
        log: vec![log("action", "detach_from")],
        data: None,
    };
    Ok(res)
}

fn query_collection<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
) -> StdResult<Binary> {
    let res = match LinkCollectionQuerier::new(&deps.querier).query_collection(contract_id)? {
        Some(collection_response) => collection_response,
        None => return to_binary(&None::<Box<Response<Collection>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_balance<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    token_id: String,
    addr: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_balance(contract_id, token_id, addr)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_token_type<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    token_id: String,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_token_type(contract_id, token_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_token_types<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_token_types(contract_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_token<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    token_id: String,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_token(contract_id, token_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_tokens<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_tokens(contract_id)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_nft<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    token_id: String,
    target: String,
) -> StdResult<Binary> {
    let res = match &*target {
        "count" => LinkCollectionQuerier::new(&deps.querier)
            .query_nft_count(contract_id, token_id)
            .unwrap(),
        "mint" => LinkCollectionQuerier::new(&deps.querier)
            .query_nft_mint(contract_id, token_id)
            .unwrap(),
        "burn" => LinkCollectionQuerier::new(&deps.querier)
            .query_nft_burn(contract_id, token_id)
            .unwrap(),
        _ => Uint128(0),
    };
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_total<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    token_id: String,
    target_str: String,
) -> StdResult<Binary> {
    let target = Target::from_str(&target_str).unwrap();
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_supply(contract_id, token_id, target)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_root_or_parent_or_children<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    token_id: String,
    target: String,
) -> StdResult<Binary> {
    if target == "root" {
        let res = LinkCollectionQuerier::new(&deps.querier)
            .query_root(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else if target == "parent" {
        let res = LinkCollectionQuerier::new(&deps.querier)
            .query_parent(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    } else {
        let res = LinkCollectionQuerier::new(&deps.querier)
            .query_children(contract_id, token_id)
            .unwrap();
        let out = to_binary(&res)?;
        Ok(out)
    }
}

fn query_perms<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    addr: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_perm(contract_id, addr)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_is_approved<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    contract_id: String,
    proxy: HumanAddr,
    approver: HumanAddr,
) -> StdResult<Binary> {
    let res = LinkCollectionQuerier::new(&deps.querier)
        .query_is_approved(contract_id, proxy, approver)
        .unwrap();
    let out = to_binary(&res)?;
    Ok(out)
}

fn query_approvers<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    proxy: HumanAddr,
    contract_id: String,
) -> StdResult<Binary> {
    let res = match LinkCollectionQuerier::new(&deps.querier).query_approvers(proxy, contract_id)? {
        Some(approvers) => approvers,
        None => return to_binary(&None::<Box<Vec<HumanAddr>>>),
    };
    let out = to_binary(&res)?;
    Ok(out)
}
