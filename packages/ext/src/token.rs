use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use cosmwasm_std::Uint128;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Token {
    pub contract_id: String,
    pub name: String,
    pub symbol: String,
    pub meta: String,
    pub img_uri: String,
    pub decimals: Uint128,
    pub mintable: bool,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename = "permission")]
#[serde(rename_all = "snake_case")]
pub enum TokenPerm {
    Mint,
    Burn,
    Modify,
}

impl FromStr for TokenPerm {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mint" => Ok(TokenPerm::Mint),
            "burn" => Ok(TokenPerm::Burn),
            "modify" => Ok(TokenPerm::Modify),
            _ => Err("Unknown permission type"),
        }
    }
}
