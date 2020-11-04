use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Collection {
    pub contract_id: String,
    pub name: String,
    pub meta: String,
    pub base_img_uri: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct NonFungibleToken {
    pub contract_id: String,
    pub token_id: String,
    pub owner: HumanAddr,
    pub name: String,
    pub meta: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct FungibleToken {
    pub contract_id: String,
    pub token_id: String,
    pub decimals: Uint128,
    pub mintable: bool,
    pub name: String,
    pub meta: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct TokenType {
    pub contract_id: String,
    pub token_type: String,
    pub name: String,
    pub meta: String,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Coin {
    pub token_id: String,
    pub amount: Uint128,
}

impl Coin {
    pub fn new(token_id: String, amount: Uint128) -> Self {
        Coin { token_id, amount }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct MintNFTParam {
    pub name: String,
    pub meta: String,
    pub token_type: String,
}

impl MintNFTParam {
    pub fn new(name: String, meta: String, token_type: String) -> Self {
        MintNFTParam {
            name,
            meta,
            token_type,
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename = "permission")]
pub enum CollectionPerm {
    Mint,
    Burn,
    Issue,
    Modify,
}

impl FromStr for CollectionPerm {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mint" => Ok(CollectionPerm::Mint),
            "burn" => Ok(CollectionPerm::Burn),
            "issue" => Ok(CollectionPerm::Issue),
            "modify" => Ok(CollectionPerm::Modify),
            _ => Err("Unknown permission type"),
        }
    }
}
