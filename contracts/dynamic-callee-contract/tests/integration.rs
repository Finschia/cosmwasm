use std::collections::HashMap;
use wasmer_types::{Type, FunctionType};
use cosmwasm_std::{to_vec, from_slice};
use cosmwasm_vm::testing::{Contract, mock_backend, MockInstanceOptions, write_data_to_mock_env, read_data_from_mock_env};
use dynamic_callee_contract::contract::ExampleStruct;

static CONTRACT_CALLEE: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/dynamic_callee_contract.wasm");


fn required_exports() -> Vec<(String, FunctionType)> {
    vec![(String::from("stub_pong"), ([Type::I32], [Type::I32]).into()),
    (String::from("stub_pong_with_struct"), ([Type::I32], [Type::I32]).into()),]
}


#[test]
fn callable_point_export_works() {
    let options = MockInstanceOptions::default();
    let backend = mock_backend(&[]);
    let contract = Contract::from_code(CONTRACT_CALLEE, backend, options).unwrap();
    
    let export_function_map: HashMap<_, _> = contract.module.exports().functions().map(|export|{
        (export.name().to_string(), export.ty().clone())
    }).collect::<Vec<(String, FunctionType)>>()
    .into_iter()
    .collect();
    
    let required_exports = required_exports();
    for required_export in required_exports {
        match export_function_map.get(&required_export.0) {
            Some(exported_function) => {
                assert_eq!(*exported_function, required_export.1);
            },
            None => assert!(false),
        }  
    }
}

#[test]
fn callable_point_pong_works() {
    let options = MockInstanceOptions::default();
    let backend = mock_backend(&[]);
    let mut contract = Contract::from_code(CONTRACT_CALLEE, backend, options).unwrap();
    let instance = contract.generate_instance().unwrap();
    

    let serialized_param = to_vec(&10u64).unwrap();
    let param_region_ptr = write_data_to_mock_env(&instance.env, &serialized_param).unwrap();

    let required_exports = required_exports();
    let call_result = instance.call_function_strict(&required_exports[0].1, "stub_pong", &[param_region_ptr.into()]).unwrap();
    assert_eq!(call_result.len(), 1);
    
    let serialized_return = read_data_from_mock_env(&instance.env, &call_result[0], usize::MAX).unwrap();
    let result: u64 = from_slice(&serialized_return).unwrap();
    assert_eq!(result, 11u64);
}

#[test]
fn callable_point_pong_with_struct_works() {
    let options = MockInstanceOptions::default();
    let backend = mock_backend(&[]);
    let mut contract = Contract::from_code(CONTRACT_CALLEE, backend, options).unwrap();
    let instance = contract.generate_instance().unwrap();
    
    let serialized_param = to_vec(&ExampleStruct{
        str_field: String::from("hello"),
        u64_field: 100u64,
    }).unwrap();
    let param_region_ptr = write_data_to_mock_env(&instance.env, &serialized_param).unwrap();

    let required_exports = required_exports();
    let call_result = instance.call_function_strict(&required_exports[1].1, "stub_pong_with_struct", &[param_region_ptr.into()]).unwrap();
    assert_eq!(call_result.len(), 1);

    let serialized_return = read_data_from_mock_env(&instance.env, &call_result[0], u32::MAX as usize).unwrap();
    let result: ExampleStruct = from_slice(&serialized_return).unwrap();
    assert_eq!(result.str_field, String::from("hello world"));
    assert_eq!(result.u64_field, 101);
}