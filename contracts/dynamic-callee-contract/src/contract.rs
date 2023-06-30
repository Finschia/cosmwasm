use cosmwasm_std::{
    callable_points, dynamic_link, entry_point, Addr, Contract, Deps, DepsMut, Env, MessageInfo,
    Response, StdError, StdResult,
};
use serde::{Deserialize, Serialize};

use crate::error::ContractError;
use crate::msg::{ExecuteMsg, InstantiateMsg};

// Note, you can use StdResult in some functions where you do not
// make use of the custom errors
#[entry_point]
pub fn instantiate(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

// And declare a custom Error variant for the ones where you will want to make use of it
#[entry_point]
pub fn execute(
    _deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[derive(Serialize, Deserialize)]
pub struct ExampleStruct {
    pub str_field: String,
    pub u64_field: u64,
}

#[derive(Contract)]
struct Caller {
    address: Addr,
}

#[derive(Contract)]
struct Callee {
    address: Addr,
}

#[dynamic_link(Caller)]
trait Parent: Contract {
    fn should_never_be_called(&self);
}

#[dynamic_link(Callee)]
trait Child: Contract {
    fn caller_address(&self) -> Addr;
}

#[callable_points]
mod callable_points {
    use super::*;

    #[callable_point]
    fn pong(_deps: Deps, _env: Env, x: u64) -> u64 {
        x + 1
    }

    #[callable_point]
    fn pong_with_struct(_deps: Deps, _env: Env, example: ExampleStruct) -> ExampleStruct {
        ExampleStruct {
            str_field: example.str_field + " world",
            u64_field: example.u64_field + 1,
        }
    }

    #[callable_point]
    fn pong_with_tuple(_deps: Deps, _env: Env, input: (String, i32)) -> (String, i32) {
        (input.0 + " world", input.1 + 1)
    }

    #[callable_point]
    fn pong_with_tuple_takes_2_args(
        _deps: Deps,
        _env: Env,
        input1: String,
        input2: i32,
    ) -> (String, i32) {
        (input1 + " world", input2 + 1)
    }

    #[callable_point]
    fn pong_env(_deps: Deps, env: Env) -> Env {
        env
    }

    #[callable_point]
    fn do_nothing(_deps: Deps, _env: Env) {}

    #[callable_point]
    fn do_panic(_deps: Deps, _env: Env) {
        panic!();
    }

    #[callable_point]
    fn reentrancy(deps: Deps, _env: Env) {
        let address = deps.api.get_caller_addr().unwrap();
        let caller = Caller { address };
        caller.should_never_be_called()
    }

    #[callable_point]
    fn caller_address(deps: Deps, _env: Env) -> Addr {
        deps.api.get_caller_addr().unwrap()
    }

    #[callable_point]
    fn call_caller_address_of(_deps: Deps, _env: Env, address: Addr) -> Addr {
        let callee = Callee { address };
        callee.caller_address()
    }

    #[callable_point]
    fn pong_with_stdresult(_deps: Deps, _env: Env, x: u64) -> StdResult<u64> {
        Ok(x + 100)
    }

    #[callable_point]
    fn pong_with_stdresult_err(_deps: Deps, _env: Env) -> StdResult<u64> {
        Err(StdError::generic_err("pong_with_stdresult_err"))
    }
}
