use cosmwasm_schema::{cw_serde, QueryResponses};
#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_std::Binary;
use cosmwasm_std::{Addr, Uint128};

#[cw_serde]
pub struct InstantiateMsg {
    pub callee_addr: Addr,
}

#[cw_serde]
pub enum ExecuteMsg {
    Ping { ping_num: Uint128 },
    TryReEntrancy {},
    DoPanic {},
    ValidateInterface {},
    ValidateInterfaceErr {},
    CallCallerAddressOf { target: Addr },
}

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(Binary)]
    GetOwnAddressViaCalleesGetCallerAddress {},
}
