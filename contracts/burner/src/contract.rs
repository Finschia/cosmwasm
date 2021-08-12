use cosmwasm_std::{
    attr, entry_point, Binary, CosmosMsg, DepsMut, Env, MessageInfo, Order, Response, StdError, StdResult,
};
use lfb_sdk_proto::lfb::bank::v1beta1::MsgSend;

use crate::msg::{InstantiateMsg, MigrateMsg};

#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> StdResult<Response> {
    Err(StdError::generic_err(
        "You can only use this contract for migrations",
    ))
}

#[entry_point]
pub fn migrate(deps: DepsMut, env: Env, msg: MigrateMsg) -> StdResult<Response> {
    // delete all state
    let keys: Vec<_> = deps
        .storage
        .range(None, None, Order::Ascending)
        .map(|(k, _)| k)
        .collect();
    let count = keys.len();
    for k in keys {
        deps.storage.remove(&k);
    }

    // get balance and send all to recipient
    let balance = deps.querier.query_all_balances(env.contract.address.clone())?;
    let stargate_msg = MsgSend {
        from_address: env.contract.address.into(),
        to_address: msg.payout.clone().into(),
        amount: balance.iter().map(|s| s.into()).collect(),
    };
    let send = CosmosMsg::Stargate {
        type_url: "/lfb.bank.v1beta1.MsgSend".into(),
        value: Binary::encode_prost_message(&stargate_msg)?,
    };

    let data_msg = format!("burnt {} keys", count).into_bytes();

    Ok(Response {
        submessages: vec![],
        messages: vec![send.into()],
        attributes: vec![attr("action", "burn"), attr("payout", msg.payout)],
        data: Some(data_msg.into()),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{mock_dependencies, mock_env, mock_info, MOCK_CONTRACT_ADDR};
    use cosmwasm_std::{coins, StdError, Storage};

    #[test]
    fn instantiate_fails() {
        let mut deps = mock_dependencies(&[]);

        let msg = InstantiateMsg {};
        let info = mock_info("creator", &coins(1000, "earth"));
        // we can just call .unwrap() to assert this was a success
        let res = instantiate(deps.as_mut(), mock_env(), info, msg);
        match res.unwrap_err() {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, "You can only use this contract for migrations")
            }
            _ => panic!("expected migrate error message"),
        }
    }

    #[test]
    fn migrate_cleans_up_data() {
        let mut deps = mock_dependencies(&coins(123456, "gold"));

        // store some sample data
        deps.storage.set(b"foo", b"bar");
        deps.storage.set(b"key2", b"data2");
        deps.storage.set(b"key3", b"cool stuff");
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(3, cnt);

        // change the verifier via migrate
        let payout = String::from("someone else");
        let msg = MigrateMsg {
            payout: payout.clone(),
        };
        let res = migrate(deps.as_mut(), mock_env(), msg).unwrap();
        // check payout
        assert_eq!(1, res.messages.len());
        let msg = res.messages.get(0).expect("no message");
        let expected_stargate_msg = MsgSend {
            from_address: MOCK_CONTRACT_ADDR.into(),
            to_address: payout.into(),
            amount: coins(123456, "gold").iter().map(|s| s.into()).collect(),
        };
        assert_eq!(
            msg,
            &CosmosMsg::Stargate {
                type_url: "/lfb.bank.v1beta1.MsgSend".into(),
                value: Binary::encode_prost_message(&expected_stargate_msg).unwrap(),
            }
        );

        // check there is no data in storage
        let cnt = deps.storage.range(None, None, Order::Ascending).count();
        assert_eq!(0, cnt);
    }
}
