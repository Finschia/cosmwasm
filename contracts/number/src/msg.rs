use cosmwasm_schema::{cw_serde, QueryResponses};

#[cw_serde]
pub struct InstantiateMsg {
    pub value: i32,
}

#[cw_serde]
pub enum ExecuteMsg {
    Add { value: i32 },
    Sub { value: i32 },
    Mul { value: i32 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NumberResponse)]
    Number {},
}

#[cw_serde]
pub struct NumberResponse {
    pub value: i32,
}
