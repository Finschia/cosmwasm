use cosmwasm_std::{
    callable_point, dynamic_link, entry_point, from_slice, to_vec, Addr, Contract, Deps, DepsMut,
    Env, MessageInfo, Response, Uint128,
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

#[dynamic_link(CalleeContract, user_defined_mock = true)]
trait Callee: Contract {
    fn pong(&self, ping_num: u64) -> u64;
    fn pong_with_struct(&self, example: ExampleStruct) -> ExampleStruct;
    fn pong_with_tuple(&self, input: (String, i32)) -> (String, i32);
    fn pong_with_tuple_takes_2_args(&self, input1: String, input2: i32) -> (String, i32);
    fn pong_env(&self) -> Env;
    fn reentrancy(&self, addr: Addr);
    fn do_panic(&self);
}

#[cfg(not(target_arch = "wasm32"))]
impl Callee for CalleeContract {
    fn pong(&self, ping_num: u64) -> u64 {
        ping_num + 1
    }

    fn pong_with_struct(&self, example: ExampleStruct) -> ExampleStruct {
        ExampleStruct {
            str_field: example.str_field + " world",
            u64_field: example.u64_field + 1,
        }
    }

    fn pong_with_tuple(&self, input: (String, i32)) -> (String, i32) {
        (input.0 + " world", input.1 + 1)
    }

    fn pong_with_tuple_takes_2_args(&self, input1: String, input2: i32) -> (String, i32) {
        (input1 + " world", input2 + 1)
    }

    fn pong_env(&self) -> Env {
        cosmwasm_std::testing::mock_env()
    }

    fn reentrancy(&self, _addr: Addr) {
        panic!()
    }

    fn do_panic(&self) {
        panic!()
    }
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
        ExecuteMsg::DoPanic {} => try_do_panic(deps, env),
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

pub fn try_do_panic(deps: DepsMut, _env: Env) -> Result<Response, ContractError> {
    let address = from_slice(&deps.storage.get(b"dynamic_callee_contract").unwrap())?;
    let contract = CalleeContract { address };
    contract.do_panic();
    Ok(Response::default())
}

#[callable_point]
fn should_never_be_called(_deps: Deps) {}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::testing::{
        mock_dependencies, mock_env, mock_info, MockApi, MockQuerier, MockStorage,
    };
    use cosmwasm_std::OwnedDeps;

    fn create_contract() -> (OwnedDeps<MockStorage, MockApi, MockQuerier>, MessageInfo) {
        let mut deps = mock_dependencies(&[]);
        let info = mock_info("creator", &[]);
        let res = instantiate(
            deps.as_mut(),
            mock_env(),
            info.clone(),
            InstantiateMsg {
                callee_addr: Addr::unchecked("callee"),
            },
        )
        .unwrap();
        assert_eq!(0, res.messages.len());
        (deps, info)
    }

    #[test]
    fn test_ping_works() {
        let (mut deps, info) = create_contract();
        let res = execute(
            deps.as_mut(),
            mock_env(),
            info,
            ExecuteMsg::Ping {
                ping_num: Uint128(41),
            },
        )
        .unwrap();
        assert_eq!(5, res.attributes.len());

        // returned pong
        assert_eq!("returned_pong", res.attributes[0].key);
        assert_eq!("42", res.attributes[0].value);

        // returned pong with struct
        let expected = ExampleStruct {
            str_field: "hello world".to_string(),
            u64_field: 101,
        };
        assert_eq!("returned_pong_with_struct", res.attributes[1].key);
        assert_eq!(expected.to_string(), res.attributes[1].value);

        // returned_pong_with_tuple
        assert_eq!("returned_pong_with_tuple", res.attributes[2].key);
        assert_eq!("(hello world, 42)", res.attributes[2].value);

        // returned_pong_with_tuple_takes_2_args
        assert_eq!(
            "returned_pong_with_tuple_takes_2_args",
            res.attributes[3].key
        );
        assert_eq!("(hello world, 42)", res.attributes[3].value);

        // returned_contract_address
        assert_eq!("returned_contract_address", res.attributes[4].key);
        assert_eq!(
            cosmwasm_std::testing::mock_env()
                .contract
                .address
                .to_string(),
            res.attributes[4].value
        );
    }
}
