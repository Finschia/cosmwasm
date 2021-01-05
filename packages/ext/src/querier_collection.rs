use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Querier, StdResult, Uint128};

use crate::collection::{Collection, CollectionPerm, Token, TokenType};
use crate::query::{LinkQueryWrapper, Module, QueryData, Response};

pub struct LinkCollectionQuerier<'a, Q: Querier> {
    querier: &'a Q,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollectionQueryRoute {
    Collections,
    Balance,
    Tokens,
    Supply,
    Perms,
    Tokentypes,
    Nftcount,
    Nftmint,
    Nftburn,
    Mint,
    Burn,
    Total,
    Root,
    Parent,
    Children,
    Approved,
    Approver,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum CollectionQuery {
    CollectionParam {
        contract_id: String,
    },
    BalanceParam {
        contract_id: String,
        token_id: String,
        addr: HumanAddr,
    },
    TokentypesParam {
        contract_id: String,
        token_id: String,
    },
    TokensParam {
        contract_id: String,
        token_id: String,
    },
    TotalParam {
        contract_id: String,
        token_id: String,
    },
    PermParam {
        contract_id: String,
        address: HumanAddr,
    },
    ParentParam {
        contract_id: String,
        token_id: String,
    },
    RootParam {
        contract_id: String,
        token_id: String,
    },
    ChildrenParam {
        contract_id: String,
        token_id: String,
    },
    IsApprovedParam {
        contract_id: String,
        proxy: HumanAddr,
        approver: HumanAddr,
    },
    ApproversParam {
        contract_id: String,
        proxy: HumanAddr,
    },
}

impl<'a, Q: Querier> LinkCollectionQuerier<'a, Q> {
    pub fn new(querier: &'a Q) -> Self {
        LinkCollectionQuerier { querier }
    }

    pub fn query_collection(&self, contract_id: String) -> StdResult<Option<Response<Collection>>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Collections,
                data: CollectionQuery::CollectionParam { contract_id },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_balance(
        &self,
        contract_id: String,
        token_id: String,
        addr: HumanAddr,
    ) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Balance,
                data: CollectionQuery::BalanceParam {
                    contract_id,
                    token_id,
                    addr,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_token_type(
        &self,
        contract_id: String,
        token_id: String,
    ) -> StdResult<Response<TokenType>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Tokentypes,
                data: CollectionQuery::TokentypesParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_token_types(&self, contract_id: String) -> StdResult<Vec<Response<TokenType>>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Tokentypes,
                data: CollectionQuery::TokentypesParam {
                    contract_id,
                    token_id: "".to_string(),
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_token(&self, contract_id: String, token_id: String) -> StdResult<Token> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Tokens,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_tokens(&self, contract_id: String) -> StdResult<Vec<Token>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Tokens,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id: "".to_string(),
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_nft_count(&self, contract_id: String, token_id: String) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Nftcount,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_nft_mint(&self, contract_id: String, token_id: String) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Nftmint,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_nft_burn(&self, contract_id: String, token_id: String) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Nftburn,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_supply(&self, contract_id: String, token_id: String) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Supply,
                data: CollectionQuery::TotalParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_mint(&self, contract_id: String, token_id: String) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Mint,
                data: CollectionQuery::TotalParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_burn(&self, contract_id: String, token_id: String) -> StdResult<Uint128> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Burn,
                data: CollectionQuery::TotalParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_parent(&self, contract_id: String, token_id: String) -> StdResult<Token> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Parent,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_root(&self, contract_id: String, token_id: String) -> StdResult<Token> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Root,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_children(&self, contract_id: String, token_id: String) -> StdResult<Vec<Token>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Children,
                data: CollectionQuery::TokensParam {
                    contract_id,
                    token_id,
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
    ) -> StdResult<Option<Vec<CollectionPerm>>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Perms,
                data: CollectionQuery::PermParam {
                    contract_id,
                    address,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_is_approved(
        &self,
        contract_id: String,
        proxy: HumanAddr,
        approver: HumanAddr,
    ) -> StdResult<bool> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Approved,
                data: CollectionQuery::IsApprovedParam {
                    contract_id,
                    proxy,
                    approver,
                },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }

    pub fn query_approvers(
        &self,
        proxy: HumanAddr,
        contract_id: String,
    ) -> StdResult<Option<Vec<HumanAddr>>> {
        let request = LinkQueryWrapper::<CollectionQueryRoute, CollectionQuery> {
            module: Module::Collectionencode,
            query_data: QueryData {
                route: CollectionQueryRoute::Approver,
                data: CollectionQuery::ApproversParam { proxy, contract_id },
            },
        };

        let res = self.querier.custom_query(&request.into())?;
        Ok(res)
    }
}
