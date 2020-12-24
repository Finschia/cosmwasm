use schemars::JsonSchema;
use serde::de::{Deserialize as deDeserialize, Deserializer, MapAccess, Visitor};
use serde::{Deserialize, Serialize};
use std::fmt;

use std::str::FromStr;

use cosmwasm_std::{HumanAddr, Uint128};

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
pub struct Collection {
    pub contract_id: String,
    pub name: String,
    pub meta: String,
    pub base_img_uri: String,
}

#[derive(Serialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(untagged)]
pub enum Token {
    FT(FungibleToken),
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

const FIELDS: &[&str] = &[
    "contract_id",
    "token_id",
    "name",
    "meta",
    "decimals",
    "mintable",
    "owner",
];

#[derive(Serialize)]
#[serde(rename_all = "snake_case")]
enum Field {
    ContractId,
    TokenId,
    Name,
    Meta,
    Decimals,
    Mintable,
    Owner,
}

struct FieldVisitor;

impl<'de> Visitor<'de> for FieldVisitor {
    type Value = Field;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str(&format!(
            "`{}` or `{}` or `{}` or `{}` or `{}` or `{}` or `{}`",
            FIELDS[0], FIELDS[1], FIELDS[2], FIELDS[3], FIELDS[4], FIELDS[5], FIELDS[6]
        ))
    }

    fn visit_str<E>(self, v: &str) -> Result<Self::Value, E>
    where
        E: serde::de::Error,
    {
        match v {
            "contract_id" => Ok(Field::ContractId),
            "token_id" => Ok(Field::TokenId),
            "name" => Ok(Field::Name),
            "meta" => Ok(Field::Meta),
            "decimals" => Ok(Field::Decimals),
            "mintable" => Ok(Field::Mintable),
            "owner" => Ok(Field::Owner),
            _ => Err(serde::de::Error::unknown_field(v, FIELDS)),
        }
    }
}

impl<'de> deDeserialize<'de> for Field {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_identifier(FieldVisitor)
    }
}

struct TokenVisitor;

impl<'de> Visitor<'de> for TokenVisitor {
    type Value = Token;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("struct Token")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, <A as MapAccess<'de>>::Error>
    where
        A: MapAccess<'de>,
    {
        let mut contract_id = None;
        let mut token_id = None;
        let mut name = None;
        let mut meta = None;
        let mut decimals_str = None;
        let mut mintable = None;
        let mut owner = None;

        while let Some(key) = map.next_key::<Field>()? {
            match key {
                Field::ContractId => {
                    if contract_id.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[0]));
                    }
                    contract_id = Some(map.next_value::<String>()?);
                }
                Field::TokenId => {
                    if token_id.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[1]));
                    }
                    token_id = Some(map.next_value::<String>()?);
                }
                Field::Name => {
                    if name.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[2]));
                    }
                    name = Some(map.next_value::<String>()?);
                }
                Field::Meta => {
                    if meta.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[3]));
                    }
                    meta = Some(map.next_value::<String>()?);
                }
                Field::Decimals => {
                    if decimals_str.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[4]));
                    }
                    decimals_str = Some(map.next_value::<String>()?);
                }
                Field::Mintable => {
                    if mintable.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[5]));
                    }
                    mintable = Some(map.next_value::<bool>()?);
                }
                Field::Owner => {
                    if owner.is_some() {
                        return Err(serde::de::Error::duplicate_field(FIELDS[6]));
                    }
                    owner = Some(map.next_value::<String>()?);
                }
            }
        }

        Ok(
            match (
                contract_id,
                token_id,
                name,
                meta,
                decimals_str,
                mintable,
                owner,
            ) {
                (
                    Some(contract_id),
                    Some(token_id),
                    Some(name),
                    Some(meta),
                    Some(decimals_str),
                    Some(mintable),
                    None,
                ) => {
                    let decimals = (&decimals_str)
                        .parse::<u128>()
                        .unwrap_or_else(|e| panic!(e));
                    Token::FT(FungibleToken {
                        contract_id,
                        token_id,
                        name,
                        meta,
                        decimals: Uint128::from(decimals),
                        mintable,
                    })
                }
                (
                    Some(contract_id),
                    Some(token_id),
                    Some(name),
                    Some(meta),
                    None,
                    None,
                    Some(owner),
                ) => Token::NFT(NonFungibleToken {
                    contract_id,
                    token_id,
                    name,
                    meta,
                    owner: HumanAddr::from(owner),
                }),
                _ => panic!("unexpected token type"),
            },
        )
    }
}

impl<'de> deDeserialize<'de> for Token {
    fn deserialize<D>(deserializer: D) -> Result<Self, <D as Deserializer<'de>>::Error>
    where
        D: Deserializer<'de>,
    {
        deserializer.deserialize_struct("Token", FIELDS, TokenVisitor)
    }
}
