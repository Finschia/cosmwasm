use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr};

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
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollectionMsg {
    Create {
        owner: HumanAddr,
        name: String,
        meta: String,
        base_img_uri: String,        
    },
}
