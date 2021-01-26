use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::str::FromStr;

use cosmwasm_std::CustomQuery;

use crate::querier_collection::{CollectionQuery, CollectionQueryRoute};
use crate::querier_token::{TokenQuery, TokenQueryRoute};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Module {
    Tokenencode,
    Collectionencode,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct LinkQueryWrapper<R, D> {
    pub module: Module,
    pub query_data: QueryData<R, D>,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub struct QueryData<R, D> {
    pub route: R,
    pub data: D,
}

impl CustomQuery for LinkQueryWrapper<CollectionQueryRoute, CollectionQuery> {}

impl CustomQuery for LinkQueryWrapper<TokenQueryRoute, TokenQuery> {}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum Target {
    Mint,
    Burn,
    Supply,
}

impl FromStr for Target {
    type Err = &'static str;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "mint" => Ok(Target::Mint),
            "burn" => Ok(Target::Burn),
            "supply" => Ok(Target::Supply),
            _ => Err("Unknown target type"),
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Response<T> {
    #[serde(rename = "type")]
    pub key: String,
    pub value: T,
}
