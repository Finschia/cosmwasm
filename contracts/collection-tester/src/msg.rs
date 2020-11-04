use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct InitMsg {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum HandleMsg {
    Create {
        owner: HumanAddr,
        name: String,
        meta: String,
        base_img_uri: String,
    },
    IssueNft {
        owner: HumanAddr,
        contract_id: String,
        name: String,
        meta: String,
    },
    IssueFt {
        owner: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        name: String,
        meta: String,
        amount: Uint128,
        mintable: bool,
        decimals: Uint128,
    },
    MintNft {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        token_types: Vec<String>,
    },
    MintFt {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        tokens: Vec<String>,
    },
    BurnNft {
        from: HumanAddr,
        contract_id: String,
        token_id: String,
    },
    BurnNftFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        token_ids: Vec<String>,
    },
    BurnFt {
        from: HumanAddr,
        contract_id: String,
        amounts: Vec<String>,
    },
    BurnFtFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        amounts: Vec<String>,
    },
    TransferNFT {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        token_ids: Vec<String>,
    },
    TransferNFTFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        to: HumanAddr,
        token_ids: Vec<String>,
    },
    TransferFt {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        tokens: Vec<String>,
    },
    TransferFTFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        to: HumanAddr,
        tokens: Vec<String>,
    },
    Modify {
        owner: HumanAddr,
        contract_id: String,
        token_type: String,
        token_index: String,
        key: String,
        value: String,
    },
    Approve {
        approver: HumanAddr,
        contract_id: String,
        proxy: HumanAddr,
    },
    Disapprove {
        approver: HumanAddr,
        contract_id: String,
        proxy: HumanAddr,
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
    Attach {
        from: HumanAddr,
        contract_id: String,
        to_token_id: String,
        token_id: String,
    },
    Detach {
        from: HumanAddr,
        contract_id: String,
        token_id: String,
    },
    AttachFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        to_token_id: String,
        token_id: String,
    },
    DetachFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        token_id: String,
    },
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum QueryMsg {
    // GetCount returns the current count as a json-encoded number
    GetCollection {
        contract_id: String,
    },
    GetBalance {
        contract_id: String,
        token_id: String,
        addr: HumanAddr,
    },
    GetTokenType {
        contract_id: String,
        token_id: String,
    },
    GetTokenTypes {
        contract_id: String,
    },
    GetToken {
        contract_id: String,
        token_id: String,
    },
    GetTokens {
        contract_id: String,
    },
    GetNft {
        contract_id: String,
        token_id: String,
        target: String,
    },
    GetTotal {
        contract_id: String,
        token_id: String,
        target: String,
    },
    GetRootOrParentOrChildren {
        contract_id: String,
        token_id: String,
        target: String,
    },
    GetPerms {
        contract_id: String,
        addr: HumanAddr,
    },
    GetApproved {
        contract_id: String,
        proxy: HumanAddr,
        approver: HumanAddr,
    },
}
