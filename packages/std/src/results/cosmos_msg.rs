use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::addresses::HumanAddr;
use crate::binary::Binary;
use crate::coins::Coin;
use crate::errors::StdResult;
#[cfg(feature = "stargate")]
use crate::ibc::IbcMsg;
use crate::serde::to_binary;

use super::Empty;

#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
// See https://github.com/serde-rs/serde/issues/1296 why we cannot add De-Serialize trait bounds to T
pub enum CosmosMsg<T = Empty>
where
    T: Clone + fmt::Debug + PartialEq + JsonSchema,
{
    Bank(BankMsg),
    // by default we use RawMsg, but a contract can override that
    // to call into more app-specific code (whatever they define)
    Custom(T),
    Staking(StakingMsg),
    /// A Stargate message encoded the same way as a protobof [Any](https://github.com/protocolbuffers/protobuf/blob/master/src/google/protobuf/any.proto).
    /// This is the same structure as messages in `TxBody` from [ADR-020](https://github.com/cosmos/cosmos-sdk/blob/master/docs/architecture/adr-020-protobuf-transaction-encoding.md)
    #[cfg(feature = "stargate")]
    Stargate {
        type_url: String,
        value: Binary,
    },
    #[cfg(feature = "stargate")]
    Ibc(IbcMsg),
    Wasm(WasmMsg),
}

/// The message types of the bank module.
///
/// See https://github.com/line/lfb-sdk/blob/main/proto/lfb/bank/v1beta1/tx.proto.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum BankMsg {
    /// Sends native tokens from the contract to the given address.
    ///
    /// This is translated to a MsgSend in https://github.com/line/lfb-sdk/blob/main/proto/lfb/bank/v1beta1/tx.proto.
    /// `from_address` is automatically filled with the current contract's address.
    Send {
        to_address: HumanAddr,
        amount: Vec<Coin>,
    },
}

/// The message types of the staking module.
///
/// See https://github.com/line/lfb-sdk/blob/main/proto/lfb/staking/v1beta1/tx.proto.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum StakingMsg {
    /// This is translated to a MsgDelegate in https://github.com/line/lfb-sdk/blob/main/proto/lfb/staking/v1beta1/tx.proto.
    /// `delegator_address` is automatically filled with the current contract's address.
    Delegate { validator: HumanAddr, amount: Coin },
    /// This is translated to a MsgUndelegate in https://github.com/line/lfb-sdk/blob/main/proto/lfb/staking/v1beta1/tx.proto.
    /// `delegator_address` is automatically filled with the current contract's address.
    Undelegate { validator: HumanAddr, amount: Coin },
    /// This is translated to a MsgSetWithdrawAddress in https://github.com/line/lfb-sdk/blob/main/proto/lfb/distribution/v1beta1/tx.proto.
    /// followed by a MsgWithdrawDelegatorReward in https://github.com/line/lfb-sdk/blob/main/proto/lfb/distribution/v1beta1/tx.proto.
    /// `delegator_address` is automatically filled with the current contract's address.
    Withdraw {
        validator: HumanAddr,
        /// this is the "withdraw address", the one that should receive the rewards
        /// if None, then use delegator address
        recipient: Option<HumanAddr>,
    },
    /// This is translated to a MsgBeginRedelegate in https://github.com/line/lfb-sdk/blob/main/proto/lfb/staking/v1beta1/tx.proto.
    /// `delegator_address` is automatically filled with the current contract's address.
    Redelegate {
        src_validator: HumanAddr,
        dst_validator: HumanAddr,
        amount: Coin,
    },
}

/// The message types of the wasm module.
///
/// See https://github.com/line/lfb-sdk/blob/main/x/wasm/internal/types/tx.proto.
#[non_exhaustive]
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum WasmMsg {
    /// Dispatches a call to another contract at a known address (with known ABI).
    ///
    /// This is translated to a MsgExecuteContract in https://github.com/line/lfb-sdk/blob/main/x/wasm/internal/types/tx.proto.
    /// `sender` is automatically filled with the current contract's address.
    Execute {
        contract_addr: HumanAddr,
        /// msg is the json-encoded ExecuteMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
    },
    /// Instantiates a new contracts from previously uploaded Wasm code.
    ///
    /// This is translated to a MsgInstantiateContract in https://github.com/line/lfb-sdk/blob/main/x/wasm/internal/types/tx.proto.
    /// `sender` is automatically filled with the current contract's address.
    Instantiate {
        code_id: u64,
        /// msg is the JSON-encoded InstantiateMsg struct (as raw Binary)
        msg: Binary,
        send: Vec<Coin>,
        /// A human-readbale label for the contract
        label: String,
    },
    /// Migrates a given contracts to use new wasm code. Passes a MigrateMsg to allow us to
    /// customize behavior.
    ///
    /// Only the contract admin (as defined in wasmd), if any, is able to make this call.
    ///
    /// This is translated to a MsgMigrateContract in https://github.com/line/lfb-sdk/blob/main/x/wasm/internal/types/tx.proto.
    /// `sender` is automatically filled with the current contract's address.
    Migrate {
        contract_addr: HumanAddr,
        /// the code_id of the new logic to place in the given contract
        new_code_id: u64,
        /// msg is the json-encoded MigrateMsg struct that will be passed to the new code
        msg: Binary,
    },
}

/// Shortcut helper as the construction of WasmMsg::Instantiate can be quite verbose in contract code
pub fn wasm_instantiate<T>(
    code_id: u64,
    msg: &T,
    send: Vec<Coin>,
    label: String,
) -> StdResult<WasmMsg>
where
    T: Serialize,
{
    let payload = to_binary(msg)?;
    Ok(WasmMsg::Instantiate {
        code_id,
        msg: payload,
        send,
        label,
    })
}

/// Shortcut helper as the construction of WasmMsg::Instantiate can be quite verbose in contract code
pub fn wasm_execute<T, U>(contract_addr: T, msg: &U, send: Vec<Coin>) -> StdResult<WasmMsg>
where
    T: Into<HumanAddr>,
    U: Serialize,
{
    let payload = to_binary(msg)?;
    Ok(WasmMsg::Execute {
        contract_addr: contract_addr.into(),
        msg: payload,
        send,
    })
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<BankMsg> for CosmosMsg<T> {
    fn from(msg: BankMsg) -> Self {
        CosmosMsg::Bank(msg)
    }
}

#[cfg(feature = "staking")]
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<StakingMsg> for CosmosMsg<T> {
    fn from(msg: StakingMsg) -> Self {
        CosmosMsg::Staking(msg)
    }
}

impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<WasmMsg> for CosmosMsg<T> {
    fn from(msg: WasmMsg) -> Self {
        CosmosMsg::Wasm(msg)
    }
}

#[cfg(feature = "stargate")]
impl<T: Clone + fmt::Debug + PartialEq + JsonSchema> From<IbcMsg> for CosmosMsg<T> {
    fn from(msg: IbcMsg) -> Self {
        CosmosMsg::Ibc(msg)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::coins;

    #[test]
    fn from_bank_msg_works() {
        let to_address = HumanAddr::from("you");
        let amount = coins(1015, "earth");
        let bank = BankMsg::Send { to_address, amount };
        let msg: CosmosMsg = bank.clone().into();
        match msg {
            CosmosMsg::Bank(msg) => assert_eq!(bank, msg),
            _ => panic!("must encode in Bank variant"),
        }
    }
}
