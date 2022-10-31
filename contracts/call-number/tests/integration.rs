use cosmwasm_vm::testing::{Contract, MockInstanceOptions};
use std::collections::HashMap;
use wasmer::{FunctionType, Type};

static CONTRACT_CALLER: &[u8] =
    include_bytes!("../target/wasm32-unknown-unknown/release/call_number.wasm");

fn required_imports() -> Vec<(String, String, FunctionType)> {
    let module_name = "dynamiclinked_NumberContract";
    vec![
        (
            String::from("add"),
            module_name.to_string(),
            ([Type::I32, Type::I32], []).into(),
        ),
        (
            String::from("sub"),
            module_name.to_string(),
            ([Type::I32, Type::I32], []).into(),
        ),
        (
            String::from("mul"),
            module_name.to_string(),
            ([Type::I32, Type::I32], []).into(),
        ),
        (
            String::from("number"),
            module_name.to_string(),
            ([Type::I32], [Type::I32]).into(),
        ),
    ]
}

#[test]
fn dynamic_link_import_works() {
    let options = MockInstanceOptions::default();
    let contract = Contract::from_code(CONTRACT_CALLER, &options, None).unwrap();

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
