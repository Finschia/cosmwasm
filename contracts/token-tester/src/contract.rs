use cosmwasm_std::{
    log, Api, Binary, CosmosMsg, Env, Extern, HandleResponse, HandleResult, HumanAddr,
    InitResponse, Querier, StdResult, Storage, Uint128,
};

use cosmwasm_ext::{LinkMsgWrapper, Module, MsgData, TokenMsg, TokenRoute};

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
    }
}

pub fn query<S: Storage, A: Api, Q: Querier>(
    deps: &Extern<S, A, Q>,
    msg: QueryMsg,
) -> StdResult<Binary> {
    match msg {
        QueryMsg::GetToken { contract_id } => query_token(deps, contract_id),
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

fn query_token<S: Storage, A: Api, Q: Querier>(
    _deps: &Extern<S, A, Q>,
    _contract_id: String,
) -> StdResult<Binary> {
    unimplemented!()
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
