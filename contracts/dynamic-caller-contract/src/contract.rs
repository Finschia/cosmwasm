use cosmwasm_std::{
    callable_point, dynamic_link, entry_point, from_slice, to_vec, Addr, Contract, DepsMut, Env,
    MessageInfo, Response, Uint128,
};
use serde::{Deserialize, Serialize};
use std::fmt;

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

#[derive(Serialize, Deserialize)]
pub struct ExampleStruct {
    pub str_field: String,
    pub u64_field: u64,
}
impl fmt::Display for ExampleStruct {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{} {}", self.str_field, self.u64_field)
    }
}

#[derive(Contract)]
struct CalleeContract {
    address: Addr,
}

#[dynamic_link(CalleeContract)]
trait Callee: Contract {
    fn pong(&self, ping_num: u64) -> u64;
    fn pong_with_struct(&self, example: ExampleStruct) -> ExampleStruct;
    fn pong_with_tuple(&self, input: (String, i32)) -> (String, i32);
    fn pong_with_tuple_takes_2_args(&self, input1: String, input2: i32) -> (String, i32);
    fn pong_env(&self) -> Env;
    fn reentrancy(&self, addr: Addr);
}

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    deps.storage
        .set(b"dynamic_callee_contract", &to_vec(&msg.callee_addr)?);

    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    deps: DepsMut,
    env: Env,
    _info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Ping { ping_num } => try_ping(deps, ping_num),
        ExecuteMsg::TryReEntrancy {} => try_re_entrancy(deps, env),
    }
}

pub fn try_ping(deps: DepsMut, ping_num: Uint128) -> Result<Response, ContractError> {
    let address = from_slice(&deps.storage.get(b"dynamic_callee_contract").unwrap())?;
    let contract = CalleeContract { address };
    let pong_ret = contract.pong(ping_num.u128() as u64);
    let struct_ret = contract.pong_with_struct(ExampleStruct {
        str_field: String::from("hello"),
        u64_field: 100u64,
    });
    let tuple_ret = contract.pong_with_tuple((String::from("hello"), 41));
    let tuple_ret2 = contract.pong_with_tuple_takes_2_args(String::from("hello"), 41);

    let mut res = Response::default();
    res.add_attribute("returned_pong", pong_ret.to_string());
    res.add_attribute("returned_pong_with_struct", struct_ret.to_string());
    res.add_attribute(
        "returned_pong_with_tuple",
        format!("({}, {})", tuple_ret.0, tuple_ret.1),
    );
    res.add_attribute(
        "returned_pong_with_tuple_takes_2_args",
        format!("({}, {})", tuple_ret2.0, tuple_ret2.1),
    );
    res.add_attribute(
        "returned_contract_address",
        contract.pong_env().contract.address.to_string(),
    );
    Ok(res)
}

pub fn try_re_entrancy(deps: DepsMut, env: Env) -> Result<Response, ContractError> {
    // It will be tried to call the should_never_be_called function below.
    // But, should be blocked by VM host side normally because it's a reentrancy case.
    let address = from_slice(&deps.storage.get(b"dynamic_callee_contract").unwrap())?;
    let contract = CalleeContract { address };
    contract.reentrancy(env.contract.address);
    Ok(Response::default())
}

#[callable_point]
fn should_never_be_called() {}
