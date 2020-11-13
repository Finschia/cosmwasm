use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

use cosmwasm_std::{HumanAddr, Uint128};

use crate::msg::Change;
use crate::token::TokenPerm;

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum TokenRoute {
    Issue,
    Transfer,
    TransferFrom,
    Mint,
    Burn,
    BurnFrom,
    GrantPerm,
    RevokePerm,
    Modify,
    Approve,
}

#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
#[serde(untagged)]
pub enum TokenMsg {
    Issue {
        owner: HumanAddr,
        to: HumanAddr,
        name: String,
        symbol: String,
        img_uri: String,
        meta: String,
        amount: Uint128,
        mintable: bool,
        decimals: Uint128,
    },
    Transfer {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        amount: Uint128,
    },
    TransferFrom {
        proxy: HumanAddr,
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        amount: Uint128,
    },
    Mint {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        amount: Uint128,
    },
    Burn {
        from: HumanAddr,
        contract_id: String,
        amount: Uint128,
    },
    BurnFrom {
        proxy: HumanAddr,
        from: HumanAddr,
        contract_id: String,
        amount: Uint128,
    },
    GrantPerm {
        from: HumanAddr,
        contract_id: String,
        to: HumanAddr,
        permission: TokenPerm,
    },
    RevokePerm {
        from: HumanAddr,
        contract_id: String,
        permission: TokenPerm,
    },
    Modify {
        owner: HumanAddr,
        contract_id: String,
        changes: Vec<Change>,
    },
    Approve {
        approver: HumanAddr,
        contract_id: String,
        proxy: HumanAddr,
    },
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    #[allow(irrefutable_let_patterns)]
    fn new_msg_issue() {
        let addr1 = HumanAddr::from("addr1");
        let addr2 = HumanAddr::from("addr2");

        let msg_issue = TokenMsg::Issue {
            owner: addr1.clone(),
            to: addr2.clone(),
            name: "test_token".to_string(),
            symbol: "TT1".to_string(),
            img_uri: "".to_string(),
            meta: "".to_string(),
            amount: Uint128(100),
            mintable: true,
            decimals: Uint128(18),
        };

        if let TokenMsg::Issue {
            owner,
            to,
            name,
            symbol,
            img_uri,
            meta,
            amount,
            mintable,
            decimals,
        } = msg_issue
        {
            assert_eq!(owner, addr1);
            assert_eq!(to, addr2);
            assert_eq!(name, "test_token".to_string());
            assert_eq!(symbol, "TT1".to_string());
            assert_eq!(img_uri, "".to_string());
            assert_eq!(meta, "".to_string());
            assert_eq!(amount, Uint128(100));
            assert_eq!(mintable, true);
            assert_eq!(decimals, Uint128(18));
        }
    }
}
