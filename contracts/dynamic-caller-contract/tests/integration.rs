use cosmwasm_vm::testing::{mock_backend, Contract, MockInstanceOptions};
use std::collections::HashMap;
use wasmer_types::{FunctionType, Type};

static CONTRACT_CALLER: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/dynamic_caller_contract.wasm");

fn required_imports() -> Vec<(String, FunctionType)> {
    vec![
        (String::from("stub_pong"), ([Type::I32], [Type::I32]).into()),
        (
            String::from("stub_pong_with_struct"),
            ([Type::I32], [Type::I32]).into(),
        ),
    ]
}

#[test]
fn dynamic_link_import_works() {
    let options = MockInstanceOptions::default();
    let backend = mock_backend(&[]);
    let contract = Contract::from_code(CONTRACT_CALLER, backend, options).unwrap();

    let import_function_map: HashMap<_, _> = contract
        .module
        .imports()
        .functions()
        .map(|import| (import.name().to_string(), import.ty().clone()))
        .collect::<Vec<(String, FunctionType)>>()
        .into_iter()
        .collect();

    let required_imports = required_imports();
    for required_export in required_imports {
        match import_function_map.get(&required_export.0) {
            Some(exported_function) => {
                assert_eq!(*exported_function, required_export.1);
            }
            None => assert!(false),
        }
    }
}
