use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};

use crate::collection::{Coin, CollectionPerm, MintNFTParam};
use crate::msg::Change;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollectionRoute {
    Create,
    IssueNft,
    IssueFt,
    MintNft,
    MintFt,
    BurnNft,
    BurnNftFrom,
    BurnFt,
    BurnFtFrom,
    TransferNft,
    TransferNftFrom,
    TransferFt,
    TransferFtFrom,
    Approve,
    Disapprove,
    Attach,
    Detach,
    AttachFrom,
    DetachFrom,
    GrantPerm,
    RevokePerm,
    Modify,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum CollectionMsg {
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
        params: Vec<MintNFTParam>,
    },
    MintFt {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        amount: Vec<Coin>,
    },
    BurnNft {
        from: HumanAddr,
        contract_id: String,
        token_ids: Vec<String>,
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
        amount: Vec<Coin>,
    },
    BurnFtFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        amount: Vec<Coin>,
    },
    TransferNft {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        token_ids: Vec<String>,
    },
    TransferNftFrom {
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
        amount: Vec<Coin>,
    },
    TransferFtFrom {
        proxy: HumanAddr,
        contract_id: String,
        from: HumanAddr,
        to: HumanAddr,
        amount: Vec<Coin>,
    },
    GrantPerm {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        permission: CollectionPerm,
    },
    RevokePerm {
        from: HumanAddr,
        contract_id: String,
        permission: CollectionPerm,
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
    Modify {
        owner: HumanAddr,
        contract_id: String,
        token_type: String,
        token_index: String,
        changes: Vec<Change>,
    },
}
