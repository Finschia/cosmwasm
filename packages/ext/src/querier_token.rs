use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Querier, StdResult, Uint128};

use crate::query::{LinkQueryWrapper, Module, QueryData, Response, Target};
use crate::token::{Token, TokenPerm};

pub struct LinkTokenQuerier<'a, Q: Querier> {
    querier: &'a Q,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenQueryRoute {
    Tokens,
    Balance,
    Supply,
    Perms,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenQuery {
    QueryTokenParam {
        contract_id: String,
    },
    QueryBalanceParam {
        contract_id: String,
        address: HumanAddr,
    },
    QueryTotalParam {
        contract_id: String,
        target: Target,
    },
    QueryPermParam {
        contract_id: String,
        address: HumanAddr,
    },
}

impl<'a, Q: Querier> LinkTokenQuerier<'a, Q> {
    pub fn new(querier: &'a Q) -> Self {
        LinkTokenQuerier { querier }
    }

    pub fn query_token(&self, contract_id: String) -> StdResult<Option<Response<Token>>> {
        let request = LinkQueryWrapper::<TokenQueryRoute, TokenQuery> {
            module: Module::Tokenencode,
            query_data: QueryData {
                route: TokenQueryRoute::Tokens,
                data: TokenQuery::QueryTokenParam { contract_id },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_balance(&self, contract_id: String, address: HumanAddr) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<TokenQueryRoute, TokenQuery> {
            module: Module::Tokenencode,
            query_data: QueryData {
                route: TokenQueryRoute::Balance,
                data: TokenQuery::QueryBalanceParam {
                    contract_id,
                    address,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_supply(&self, contract_id: String, target: Target) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<TokenQueryRoute, TokenQuery> {
            module: Module::Tokenencode,
            query_data: QueryData {
                route: TokenQueryRoute::Supply,
                data: TokenQuery::QueryTotalParam {
                    contract_id,
                    target,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_perm(
        &self,
        contract_id: String,
        address: HumanAddr,
    ) -> StdResult<Option<Vec<TokenPerm>>> {
        let request = LinkQueryWrapper::<TokenQueryRoute, TokenQuery> {
            module: Module::Tokenencode,
            query_data: QueryData {
                route: TokenQueryRoute::Perms,
                data: TokenQuery::QueryPermParam {
                    contract_id,
                    address,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }
}
