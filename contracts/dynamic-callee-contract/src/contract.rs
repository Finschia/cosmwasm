use cosmwasm_std::{
    callable_points, dynamic_link, entry_point, Addr, Contract, Deps, DepsMut, Env, MessageInfo,
    Response,
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

#[dynamic_link(Caller)]
trait ReEntrance: Contract {
    fn should_never_be_called(&self);
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
    fn do_nothing(_deps: Deps, _env: Env) {
        ()
    }

    #[callable_point]
    fn do_panic(_deps: Deps, _env: Env) {
        panic!();
    }

    #[callable_point]
    fn reentrancy(_deps: Deps, _env: Env, address: Addr) {
        let caller = Caller { address };
        caller.should_never_be_called()
    }
}
