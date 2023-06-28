use cosmwasm_std::{from_slice, to_vec, Addr, Env};
use cosmwasm_vm::testing::{
    mock_env, read_data_from_mock_env, write_data_to_mock_env, Contract, MockApi,
    MockInstanceOptions, MockQuerier, MockStorage, MOCK_CONTRACT_ADDR,
};
use cosmwasm_vm::{Instance, VmError};
use dynamic_callee_contract::contract::ExampleStruct;
use std::collections::HashMap;
use wasmer::{FunctionType, Type};

static CONTRACT_CALLEE: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/dynamic_callee_contract.wasm");

fn required_exports() -> Vec<(String, FunctionType)> {
    vec![
        (
            String::from("pong"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_struct"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_tuple"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_tuple_takes_2_args"),
            ([Type::I32, Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (String::from("pong_env"), ([Type::I32], [Type::I32]).into()),
        (String::from("do_panic"), ([Type::I32], []).into()),
        (
            String::from("_get_callable_points_properties"),
            ([], [Type::I32]).into(),
        ),
        (
            String::from("caller_address"),
            ([Type::I32], [Type::I32]).into(),
        ),
        (String::from("reentrancy"), ([Type::I32], []).into()),
        (
            String::from("call_caller_address_of"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_stdresult"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
    ]
}

fn make_callee_instance() -> Instance<MockApi, MockStorage, MockQuerier> {
    let options = MockInstanceOptions::default();
    let api = MockApi::default();
    let querier = MockQuerier::new(&[]);
    let contract = Contract::from_code(CONTRACT_CALLEE, &options, None).unwrap();
    let instance = contract.generate_instance(api, querier, &options).unwrap();
    instance
        .env
        .set_serialized_env(&to_vec(&mock_env()).unwrap());

    instance
}

#[test]
fn callable_point_export_works() {
    let options = MockInstanceOptions::default();
    let contract = Contract::from_code(CONTRACT_CALLEE, &options, None).unwrap();

    let export_function_map: HashMap<_, _> = contract
        .module
        .exports()
        .functions()
        .map(|export| (export.name().to_string(), export.ty().clone()))
        .collect::<Vec<(String, FunctionType)>>()
        .into_iter()
        .collect();

    let required_exports = required_exports();
    for required_export in required_exports {
        match export_function_map.get(&required_export.0) {
            Some(exported_function) => {
                assert_eq!(*exported_function, required_export.1);
            }
            None => panic!("{} is not exported.", required_export.0),
        }
    }
}

#[test]
fn callable_point_pong_works() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let serialized_param = to_vec(&10u64).unwrap();
    let param_region_ptr = write_data_to_mock_env(&instance.env, &serialized_param).unwrap();

    let required_exports = required_exports();
    let export_index = 0;
    assert_eq!("pong".to_string(), required_exports[export_index].0);
    let call_result = instance
        .call_function("pong", &[env_region_ptr.into(), param_region_ptr.into()])
        .unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return =
        read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: u64 = from_slice(&serialized_return).unwrap();
    assert_eq!(result, 11u64);
}

#[test]
fn callable_point_pong_with_struct_works() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let serialized_param = to_vec(&ExampleStruct {
        str_field: String::from("hello"),
        u64_field: 100u64,
    })
    .unwrap();

    let param_region_ptr = write_data_to_mock_env(&instance.env, &serialized_param).unwrap();

    let required_exports = required_exports();
    let export_index = 1;
    assert_eq!(
        "pong_with_struct".to_string(),
        required_exports[export_index].0
    );
    let call_result = instance
        .call_function(
            "pong_with_struct",
            &[env_region_ptr.into(), param_region_ptr.into()],
        )
        .unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return =
        read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: ExampleStruct = from_slice(&serialized_return).unwrap();
    assert_eq!(result.str_field, String::from("hello world"));
    assert_eq!(result.u64_field, 101);
}

#[test]
fn callable_point_pong_with_tuple_works() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let serialized_param = to_vec(&(String::from("hello"), 41i32)).unwrap();
    let param_region_ptr = write_data_to_mock_env(&instance.env, &serialized_param).unwrap();

    let required_exports = required_exports();
    let export_index = 2;
    assert_eq!(
        "pong_with_tuple".to_string(),
        required_exports[export_index].0
    );
    let call_result = instance
        .call_function(
            "pong_with_tuple",
            &[env_region_ptr.into(), param_region_ptr.into()],
        )
        .unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return =
        read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: (String, i32) = from_slice(&serialized_return).unwrap();
    assert_eq!(result.0, String::from("hello world"));
    assert_eq!(result.1, 42);
}

#[test]
fn callable_point_pong_with_tuple_takes_2_args_works() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let serialized_param1 = to_vec(&String::from("hello")).unwrap();
    let param_region_ptr1 = write_data_to_mock_env(&instance.env, &serialized_param1).unwrap();

    let serialized_param2 = to_vec(&41i32).unwrap();
    let param_region_ptr2 = write_data_to_mock_env(&instance.env, &serialized_param2).unwrap();

    let required_exports = required_exports();
    let export_index = 3;
    assert_eq!(
        "pong_with_tuple_takes_2_args".to_string(),
        required_exports[export_index].0
    );
    let call_result = instance
        .call_function(
            "pong_with_tuple_takes_2_args",
            &[
                env_region_ptr.into(),
                param_region_ptr1.into(),
                param_region_ptr2.into(),
            ],
        )
        .unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return =
        read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: (String, i32) = from_slice(&serialized_return).unwrap();
    assert_eq!(result.0, String::from("hello world"));
    assert_eq!(result.1, 42);
}

#[test]
fn callable_point_pong_env_works() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let required_exports = required_exports();
    instance
        .env
        .set_serialized_env(&to_vec(&mock_env()).unwrap());
    let export_index = 4;
    assert_eq!("pong_env".to_string(), required_exports[export_index].0);
    let call_result = instance
        .call_function("pong_env", &[env_region_ptr.into()])
        .unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return =
        read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: Env = from_slice(&serialized_return).unwrap();
    assert_eq!(result.contract.address, Addr::unchecked(MOCK_CONTRACT_ADDR));
}

#[test]
fn callable_point_do_panic_raises_runtime_error() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let required_exports = required_exports();
    instance
        .env
        .set_serialized_env(&to_vec(&mock_env()).unwrap());
    let export_index = 5;
    assert_eq!("do_panic".to_string(), required_exports[export_index].0);
    let call_result = instance.call_function("do_panic", &[env_region_ptr.into()]);

    match call_result.unwrap_err() {
        VmError::RuntimeErr { msg, .. } => {
            // Because content in the latter part depends on the environment,
            // comparing whether the error begins with panic error or not.
            assert!(msg.starts_with("Wasmer runtime error: RuntimeError: unreachable"))
        }
        e => panic!("Unexpected error: {:?}", e),
    }
}

#[test]
fn callable_point_pong_with_stdresult_works() {
    let instance = make_callee_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let serialized_param = to_vec(&10u64).unwrap();
    let param_region_ptr = write_data_to_mock_env(&instance.env, &serialized_param).unwrap();

    let required_exports = required_exports();
    let export_index = 0;
    assert_eq!(
        "pong_with_stdresult".to_string(), 
        required_exports[export_index].0
    );
    let call_result = instance
        .call_function(
            "pong_with_stdresult", 
            &[env_region_ptr.into(), param_region_ptr.into()]
        )
        .unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return =
        read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: u64 = from_slice(&serialized_return).unwrap();
    assert_eq!(result, 110u64);
}
