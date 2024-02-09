use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::Addr;

#[cw_serde]
pub struct InstantiateMsg {
    pub callee_addr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Add { value: i32 },
    Sub { value: i32 },
    Mul { value: i32 },
    SubmsgReplyAdd { value: i32 },
    SubmsgReplySub { value: i32 },
    SubmsgReplyMul { value: i32 },
    LogQuery {},
    LogQueryDyn {},
}

#[cw_serde]
pub enum NumberExecuteMsg {
    Add { value: i32 },
    Sub { value: i32 },
    Mul { value: i32 },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(NumberResponse)]
    Add { value: i32 },
    #[returns(NumberResponse)]
    Sub { value: i32 },
    #[returns(NumberResponse)]
    Mul { value: i32 },
    #[returns(NumberResponse)]
    NumberDyn {},
    #[returns(NumberResponse)]
    Number {},
}

#[cw_serde]
pub struct NumberResponse {
    pub value: i32,
}
