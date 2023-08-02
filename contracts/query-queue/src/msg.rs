use cosmwasm_schema::{cw_serde, QueryResponses};
#[cw_serde]
pub struct InstantiateMsg {
    // address to query
    pub queue_address: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    // change address to query
    ChangeAddress { queue_address: String },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(RawResponse)]
    Raw { key: u32 },
    #[returns(CountResponse)]
    Count {},
    #[returns(SumResponse)]
    Sum {},
    #[returns(ReducerResponse)]
    Reducer {},
    #[returns(ListResponse)]
    List {},
}

#[cw_serde]
pub struct RawResponse {
    pub item: Option<i32>,
}

#[cw_serde]
pub struct CountResponse {
    pub count: u32,
}

#[cw_serde]
pub struct SumResponse {
    pub sum: i32,
}

// the Vec contains pairs for every element in the queue
// (value of item i, sum of all elements where value > value[i])
#[cw_serde]
pub struct ReducerResponse {
    pub counters: Vec<(i32, i32)>,
}

#[cw_serde]
pub struct ListResponse {
    /// List an empty range, both bounded
    pub empty: Vec<u32>,
    /// List all IDs lower than 0x20
    pub early: Vec<u32>,
    /// List all IDs starting from 0x20
    pub late: Vec<u32>,
}

// Item of queue
#[cw_serde]
pub struct Item {
    pub value: i32,
}
