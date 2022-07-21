use cosmwasm_vm::testing::{mock_backend, Contract, MockInstanceOptions};
use std::collections::HashMap;
use wasmer_types::{FunctionType, Type};

static CONTRACT_CALLER: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/dynamic_caller_contract.wasm");

fn required_imports() -> Vec<(String, String, FunctionType)> {
    vec![
        (
            String::from("stub_pong"),
            String::from("CalleeContract"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("stub_pong_with_struct"),
            String::from("CalleeContract"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("stub_pong_with_tuple"),
            String::from("CalleeContract"),
            ([Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("stub_pong_with_tuple_takes_2_args"),
            String::from("CalleeContract"),
            ([Type::I32, Type::I32, Type::I32], [Type::I32]).into(),
        ),
        (
            String::from("stub_pong_env"),
            String::from("CalleeContract"),
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
