use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Issue {
        owner: HumanAddr,
        to: HumanAddr,
        name: String,
        symbol: String,
        img_uri: String,
        meta: String,
        amount: Uint128,
        mintable: bool,
        decimals: Uint128,
    },
    Transfer {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        amount: Uint128,
    },
    Mint {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        amount: Uint128,
    },
    Burn {
        from: HumanAddr,
        contract_id: String,
        amount: Uint128,
    },
    GrantPerm {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        permission: String,
    },
    RevokePerm {
        from: HumanAddr,
        contract_id: String,
        permission: String,
    },
    Modify {
        owner: HumanAddr,
        contract_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    GetToken {
        contract_id: String,
    },
    GetBalance {
        contract_id: String,
        address: HumanAddr,
    },
    GetTotal {
        contract_id: String,
        target: String,
    },
    GetPerm {
        contract_id: String,
        address: HumanAddr,
    },
}
