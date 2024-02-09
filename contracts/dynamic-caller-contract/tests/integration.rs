use cosmwasm_std::to_json_vec;
use cosmwasm_vm::testing::{mock_env, Contract, MockInstanceOptions};
use std::collections::HashMap;
use wasmer::{FunctionType, Type};

static CONTRACT_CALLER: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/dynamic_caller_contract.wasm");

fn required_imports() -> Vec<(String, String, FunctionType)> {
    let module_name = "dynamiclinked_CalleeContract";
    vec![
        (
            String::from("pong"),
            module_name.to_string(),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_struct"),
            module_name.to_string(),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_tuple"),
            module_name.to_string(),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_tuple_takes_2_args"),
            module_name.to_string(),
            ([Type::I32, Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_env"),
            module_name.to_string(),
            ([Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("do_nothing"),
            module_name.to_string(),
            ([Type::I32], []).into(),
        ),
        (
            String::from("do_panic"),
            module_name.to_string(),
            ([Type::I32], []).into(),
        ),
        (
            String::from("reentrancy"),
            module_name.to_string(),
            ([Type::I32], []).into(),
        ),
        (
            String::from("caller_address"),
            module_name.to_string(),
            ([Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("call_caller_address_of"),
            module_name.to_string(),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_stdresult"),
            module_name.to_string(),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("pong_with_stdresult_err"),
            module_name.to_string(),
            ([Type::I32], [Type::I32]).into(),
        ),
    ]
}

fn required_exports() -> Vec<(String, FunctionType)> {
    vec![(
        String::from("_get_callable_points_properties"),
        ([], [Type::I32]).into(),
    )]
}

#[test]
fn dynamic_link_import_works() {
    let options = MockInstanceOptions::default();
    let env = to_json_vec(&mock_env()).unwrap();
    let contract = Contract::from_code(CONTRACT_CALLER, &env, &options, None).unwrap();

    let import_function_map: HashMap<_, _> = contract
        .module()
        .imports()
        .functions()
        .map(|import| {
            (
                import.name().to_string(),
                (import.module().to_string(), import.ty().clone()),
            )
        })
        .collect::<Vec<(String, (String, FunctionType))>>()
        .into_iter()
        .collect();

    let required_imports = required_imports();
    for required_import in required_imports {
        match import_function_map.get(&required_import.0) {
            Some(imported_function) => {
                let module_name = &imported_function.0;
                let function_type = &imported_function.1;
                assert_eq!(*module_name, required_import.1);
                assert_eq!(*function_type, required_import.2);
            }
            None => panic!("{} is not imported.", required_import.0),
        }
    }
}

#[test]
fn callable_point_export_works() {
    let options = MockInstanceOptions::default();
    let env = to_json_vec(&mock_env()).unwrap();
    let contract = Contract::from_code(CONTRACT_CALLER, &env, &options, None).unwrap();

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
