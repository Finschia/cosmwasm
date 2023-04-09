use cosmwasm_std::to_vec;
use cosmwasm_vm::testing::{
    mock_env, write_data_to_mock_env, Contract, MockApi, MockInstanceOptions, MockQuerier,
    MockStorage,
};
use cosmwasm_vm::Instance;
use std::collections::HashMap;
use wasmer::{FunctionType, Type};

static CONTRACT: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/simple_callee.wasm");

fn required_exports() -> Vec<(String, FunctionType)> {
    vec![
        (String::from("succeed"), ([Type::I32], []).into()),
        (String::from("succeed_readonly"), ([Type::I32], []).into()),
        (String::from("fail"), ([Type::I32], []).into()),
        (
            String::from("_get_callable_points_properties"),
            ([], [Type::I32]).into(),
        ),
    ]
}

fn make_instance() -> Instance<MockApi, MockStorage, MockQuerier> {
    let options = MockInstanceOptions::default();
    let api = MockApi::default();
    let querier = MockQuerier::new(&[]);
    let contract = Contract::from_code(CONTRACT, &options, None).unwrap();
    let instance = contract.generate_instance(api, querier, &options).unwrap();
    instance
        .env
        .set_serialized_env(&to_vec(&mock_env()).unwrap());

    instance
}

#[test]
fn callable_point_export_works() {
    let options = MockInstanceOptions::default();
    let contract = Contract::from_code(CONTRACT, &options, None).unwrap();

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
fn callable_point_succeed_works() {
    let instance = make_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let required_exports = required_exports();
    let export_index = 0;
    assert_eq!("succeed".to_string(), required_exports[export_index].0);

    // check succeed
    instance
        .call_function_strict(
            &required_exports[export_index].1,
            "succeed",
            &[env_region_ptr.into()],
        )
        .unwrap();
}

#[test]
fn callable_point_succeed_readonly_works() {
    let instance = make_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let required_exports = required_exports();
    let export_index = 1;
    assert_eq!(
        "succeed_readonly".to_string(),
        required_exports[export_index].0
    );

    // check succeed_readonly
    instance
        .call_function_strict(
            &required_exports[export_index].1,
            "succeed_readonly",
            &[env_region_ptr.into()],
        )
        .unwrap();
}

#[test]
fn callable_fail_fails() {
    let instance = make_instance();
    let env = to_vec(&mock_env()).unwrap();
    let env_region_ptr = write_data_to_mock_env(&instance.env, &env).unwrap();

    let required_exports = required_exports();
    let export_index = 2;
    assert_eq!("fail".to_string(), required_exports[export_index].0);

    // check unreachable
    let call_result = instance
        .call_function_strict(
            &required_exports[export_index].1,
            "fail",
            &[env_region_ptr.into()],
        )
        .unwrap_err();
    assert!(call_result
        .to_string()
        .contains("RuntimeError: unreachable"))
}
