use cosmwasm_std::to_json_vec;
use cosmwasm_vm::testing::{
    call_function, get_fe_mut, mock_env, Contract, MockApi, MockInstanceOptions, MockQuerier,
    MockStorage,
};
use cosmwasm_vm::{write_value_to_env, Instance};
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
    let env = to_json_vec(&mock_env()).unwrap();
    let api = MockApi::default();
    let querier = MockQuerier::new(&[]);
    let contract = Contract::from_code(CONTRACT, &env, &options, None).unwrap();
    let instance = contract.generate_instance(api, querier, &options).unwrap();

    instance
}

#[test]
fn callable_point_export_works() {
    let options = MockInstanceOptions::default();
    let env = to_json_vec(&mock_env()).unwrap();
    let contract = Contract::from_code(CONTRACT, &env, &options, None).unwrap();

    let export_function_map: HashMap<_, _> = contract
        .module()
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
    let mut instance = make_instance();
    let mut fe = get_fe_mut(&mut instance);
    let (vm_env, mut vm_store) = fe.data_and_store_mut();
    let env = to_json_vec(&mock_env()).unwrap();
    let env_region_ptr = write_value_to_env(&vm_env, &mut vm_store, &env).unwrap();

    let required_exports = required_exports();
    let export_index = 0;
    assert_eq!("succeed".to_string(), required_exports[export_index].0);

    // check succeed
    call_function(&mut instance, "succeed", &[env_region_ptr.into()]).unwrap();
}

#[test]
fn callable_point_succeed_readonly_works() {
    let mut instance = make_instance();
    let mut fe = get_fe_mut(&mut instance);
    let (vm_env, mut vm_store) = fe.data_and_store_mut();
    let env = to_json_vec(&mock_env()).unwrap();
    let env_region_ptr = write_value_to_env(&vm_env, &mut vm_store, &env).unwrap();

    let required_exports = required_exports();
    let export_index = 1;
    assert_eq!(
        "succeed_readonly".to_string(),
        required_exports[export_index].0
    );

    // check succeed_readonly
    call_function(&mut instance, "succeed_readonly", &[env_region_ptr.into()]).unwrap();
}

#[test]
fn callable_fail_fails() {
    let mut instance = make_instance();
    let mut fe = get_fe_mut(&mut instance);
    let (vm_env, mut vm_store) = fe.data_and_store_mut();
    let env = to_json_vec(&mock_env()).unwrap();
    let env_region_ptr = write_value_to_env(&vm_env, &mut vm_store, &env).unwrap();

    let required_exports = required_exports();
    let export_index = 2;
    assert_eq!("fail".to_string(), required_exports[export_index].0);

    // check unreachable
    let call_result = call_function(&mut instance, "fail", &[env_region_ptr.into()]).unwrap_err();
    assert!(call_result
        .to_string()
        .contains("RuntimeError: unreachable"))
}
