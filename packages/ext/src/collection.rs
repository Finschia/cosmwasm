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
#[serde(tag = "type", content = "value")]
pub enum Token {
    #[serde(rename = "collection/FT")]
    FT(FungibleToken),
    #[serde(rename = "collection/NFT")]
    NFT(NonFungibleToken),
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
pub struct NonFungibleToken {
    pub contract_id: String,
    pub token_id: String,
    pub owner: HumanAddr,
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
#[serde(rename_all = "snake_case")]
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

#[cfg(test)]
mod tests {
    use super::*;

    use crate::query::Response;

    #[test]
    fn test_ft_and_nft_deserialize() {
        let json = r#"
        [
            {
                "type": "collection/FT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "0000000100000000",
                    "decimals": "18",
                    "mintable": true,
                    "name": "ft_test_name-1",
                    "meta": ""
                }
            },
            {
                "type": "collection/NFT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "1000000100000001",
                    "owner": "link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu",
                    "name": "nft-0",
                    "meta": ""
                }
            }
        ]
        "#;

        let res = serde_json::from_str::<Vec<Token>>(json);
        assert!(res.is_ok());
        let tokens = res.unwrap();
        let ft = tokens[0].clone();
        let nft = tokens[1].clone();

        assert_eq!(
            ft,
            Token::FT(FungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "0000000100000000".to_string(),
                name: "ft_test_name-1".to_string(),
                meta: "".to_string(),
                decimals: Uint128(18),
                mintable: true,
            })
        );
        assert_eq!(
            nft,
            Token::NFT(NonFungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "1000000100000001".to_string(),
                name: "nft-0".to_string(),
                meta: "".to_string(),
                owner: HumanAddr::from("link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu"),
            })
        );

        assert_eq!(
            ft,
            Token::FT(FungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "0000000100000000".to_string(),
                name: "ft_test_name-1".to_string(),
                meta: "".to_string(),
                decimals: Uint128(18),
                mintable: true,
            })
        );
        assert_eq!(
            nft,
            Token::NFT(NonFungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "1000000100000001".to_string(),
                name: "nft-0".to_string(),
                meta: "".to_string(),
                owner: HumanAddr::from("link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu"),
            })
        )
    }

    #[test]
    fn test_fts_deserialize() {
        let json = r#"
        [
            {
                "type": "collection/FT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "0000000100000000",
                    "decimals": "18",
                    "mintable": true,
                    "name": "ft_test_name-1",
                    "meta": ""
                }
            },
            {
                "type": "collection/FT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "0000000100000001",
                    "decimals": "8",
                    "mintable": false,
                    "name": "ft_test_name-2",
                    "meta": "meta"

                }
            }
        ]
        "#;

        let res = serde_json::from_str::<Vec<Token>>(json);
        assert!(res.is_ok());
        let tokens = res.unwrap();
        let ft1 = tokens[0].clone();
        let ft2 = tokens[1].clone();

        assert_eq!(
            ft1,
            Token::FT(FungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "0000000100000000".to_string(),
                name: "ft_test_name-1".to_string(),
                meta: "".to_string(),
                decimals: Uint128(18),
                mintable: true,
            })
        );
        assert_eq!(
            ft2,
            Token::FT(FungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "0000000100000001".to_string(),
                name: "ft_test_name-2".to_string(),
                meta: "meta".to_string(),
                decimals: Uint128(8),
                mintable: false,
            })
        )
    }

    #[test]
    fn test_nfts_deserialize() {
        let json = r#"
        [
            {
                "type": "collection/NFT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "1000000100000001",
                    "owner": "link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu",
                    "name": "nft-0",
                    "meta": ""
                }
            },
            {
                "type": "collection/NFT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "1000000100000002",
                    "owner": "link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu",
                    "name": "nft-1",
                    "meta": ""
                }
            }
        ]
        "#;

        let res = serde_json::from_str::<Vec<Token>>(json);
        assert!(res.is_ok());
        let tokens = res.unwrap();
        let nft1 = tokens[0].clone();
        let nft2 = tokens[1].clone();

        assert_eq!(
            nft1,
            Token::NFT(NonFungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "1000000100000001".to_string(),
                name: "nft-0".to_string(),
                meta: "".to_string(),
                owner: HumanAddr::from("link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu"),
            })
        );
        assert_eq!(
            nft2,
            Token::NFT(NonFungibleToken {
                contract_id: "9be17165".to_string(),
                token_id: "1000000100000002".to_string(),
                name: "nft-1".to_string(),
                meta: "".to_string(),
                owner: HumanAddr::from("link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu"),
            })
        )
    }

    #[test]
    fn test_invalid_token_deserialize() {
        let json = r#"
        [
            {
                "type": "collection/FT",
                "value": {
                    "contract_id": "9be17165",
                    "token_id": "0000000100000000",
                    "decimals": "18",
                    "mintable": true,
                    "name": "ft_test_name-1",
                    "meta": ""
                    "owner": "link18vd8fpwxzck93qlwghaj6arh4p7c5n89fvcmzu",
                }
            }
        ]
        "#;

        let res = serde_json::from_str::<Vec<Token>>(json);
        assert!(!res.is_ok());
    }
}
