use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::QueryRequest;

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

impl Into<QueryRequest<LinkQueryWrapper<TokenQueryRoute, TokenQuery>>>
    for LinkQueryWrapper<TokenQueryRoute, TokenQuery>
{
    fn into(self) -> QueryRequest<LinkQueryWrapper<TokenQueryRoute, TokenQuery>> {
        QueryRequest::Custom(self)
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Response<T> {
    #[serde(rename = "type")]
    pub key: String,
    pub value: T,
}
