use std::collections::HashMap;
use std::fmt;
use std::str;

use crate::backend::{BackendApi, Querier, Storage};
use crate::conversion::{ref_to_u32, to_u32};
use crate::environment::{process_gas_info, Environment};
use crate::errors::{CommunicationError, VmResult};
use crate::memory::{read_region, write_region};
use wasmer::{Exports, Function, FunctionType, ImportObject, Module, RuntimeError, Val};
use wasmer_types::ImportIndex;

pub type WasmerVal = Val;

pub struct FunctionMetadata {
    pub module_name: String,
    pub name: String,
    pub signature: FunctionType,
}

impl Clone for FunctionMetadata {
    fn clone(&self) -> Self {
        FunctionMetadata {
            module_name: self.module_name.clone(),
            name: self.name.clone(),
            signature: self.signature.clone(),
        }
    }
}

impl fmt::Display for FunctionMetadata {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        write!(
            fmt,
            "module_name:{}, name:{}, signature:{}",
            self.module_name, self.name, self.signature
        )
    }
}

impl FunctionMetadata {
    fn clone_and_drop_callee_addr_arg(&self) -> Self {
        let new_signature =
            FunctionType::new(&self.signature.params()[1..], self.signature.results());
        FunctionMetadata {
            module_name: self.module_name.clone(),
            name: self.name.clone(),
            signature: new_signature,
        }
    }
}

fn with_trace_dynamic_call<A, S, Q, C, R>(
    env: &Environment<A, S, Q>,
    callback: C,
) -> Result<R, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    C: FnOnce() -> Result<R, RuntimeError>,
{
    env.try_record_dynamic_call_trace()
        .map_err(|e| RuntimeError::new(e.to_string()))?;
    let res = callback();
    env.remove_latest_dynamic_call_trace();
    res
}

fn native_dynamic_link_trampoline<A: BackendApi, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    args: &[WasmerVal],
) -> Result<Vec<WasmerVal>, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    if args.is_empty() {
        return Err(RuntimeError::new(
            "No args are passed to trampoline. The first arg must be callee contract address.",
        ));
    };
    let address_region_ptr = ref_to_u32(&args[0])?;
    let raw_contract_addr = read_region(&env.memory(), address_region_ptr, 64)?;
    let contract_addr = str::from_utf8(&raw_contract_addr)
        .map_err(|_| RuntimeError::new("Invalid stored callee contract address"))?
        .trim_matches('"');
    let func_args = &args[1..];
    with_trace_dynamic_call(env, || {
        let func_info = env
            .with_callee_function_metadata(|func_info| {
                Ok(func_info.clone_and_drop_callee_addr_arg())
            })
            .unwrap();
        let (call_result, gas_info) =
            env.api
                .contract_call(env, contract_addr, &func_info, func_args);
        process_gas_info::<A, S, Q>(env, gas_info)?;
        match call_result {
            Ok(ret) => Ok(ret.to_vec()),
            Err(e) => Err(RuntimeError::new(format!(
                "func_info:{{{}}}, error:{}",
                func_info, e
            ))),
        }
    })
}

#[cfg(feature = "bench")]
pub fn native_dynamic_link_trampoline_for_bench<A: BackendApi, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    args: &[WasmerVal],
) -> Result<Vec<WasmerVal>, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    native_dynamic_link_trampoline(env, args)
}

pub fn dynamic_link<A: BackendApi, S: Storage, Q: Querier>(
    module: &Module,
    env: &Environment<A, S, Q>,
    imports: &mut ImportObject,
) where
    A: BackendApi + 'static, // 'static is needed here to allow copying API instances into closures
    S: Storage + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
    Q: Querier + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
{
    // Getting required imports to onther contracts
    let mut import_functions_by_module: HashMap<String, Vec<FunctionMetadata>> = HashMap::new();
    let module_info = module.artifact().module();
    for ((module_name, func_name, _), import_index) in module_info
        .imports
        .iter()
        .filter(|((module_name, _, _), _)| module_name != "env")
    {
        if let ImportIndex::Function(func_index) = import_index {
            let func_sig = module_info.signatures[module_info.functions[*func_index]].clone();
            //if compiled with '-s' option(symbol strapping), function_names is empty.
            //let func_symbol = module_info.function_names[func_index].clone();
            import_functions_by_module
                .entry(module_name.to_string())
                .or_insert_with(Vec::new)
                .push(FunctionMetadata {
                    module_name: module_name.to_string(),
                    name: func_name.to_string(),
                    signature: func_sig,
                });
        }
    }

    // link to gateway host function
    for module_name in import_functions_by_module.keys() {
        let mut module_exports = Exports::new();
        let func_infos = &import_functions_by_module[module_name];
        for func_metadata in func_infos {
            // make a new enviorment struct for pass the target function information
            let mut dynamic_env = env.clone();

            dynamic_env.set_callee_function_metadata(Some(func_metadata.clone()));
            module_exports.insert(
                func_metadata.name.clone(),
                Function::new_with_env(
                    module.store(),
                    func_metadata.signature.clone(),
                    dynamic_env,
                    native_dynamic_link_trampoline,
                ),
            );
        }

        imports.register(module_name.to_string(), module_exports);
    }
}

pub fn copy_region_vals_between_env<A, S, Q, A2, S2, Q2>(
    src_env: &Environment<A, S, Q>,
    dst_env: &Environment<A2, S2, Q2>,
    vals: &[WasmerVal],
    deallocation: bool,
) -> VmResult<Box<[WasmerVal]>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    A2: BackendApi + 'static,
    S2: Storage + 'static,
    Q2: Querier + 'static,
{
    let mut copied_region_ptrs = Vec::<WasmerVal>::with_capacity(vals.len());
    for val in vals {
        let val_region_ptr = ref_to_u32(val)?;
        let data = read_region(&src_env.memory(), val_region_ptr, u32::MAX as usize)?;
        if deallocation {
            src_env.call_function0("deallocate", &[val_region_ptr.into()])?;
        }

        let ret = dst_env.call_function1("allocate", &[to_u32(data.len())?.into()])?;
        let region_ptr = ref_to_u32(&ret)?;
        if region_ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }

        write_region(&dst_env.memory(), region_ptr, &data)?;
        copied_region_ptrs.push(region_ptr.into());
    }

    Ok(copied_region_ptrs.into_boxed_slice())
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, Addr, Empty};
    use std::cell::RefCell;
    use wasmer_types::Type;

    use crate::testing::{
        mock_env, mock_instance, read_data_from_mock_env, write_data_to_mock_env, MockApi,
        MockQuerier, MockStorage, INSTANCE_CACHE,
    };
    use crate::to_vec;
    use crate::VmError;

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    // prepared data
    const PADDING_DATA: &[u8] = b"deadbeef";
    const PASS_DATA1: &[u8] = b"data";

    const CALLEE_NAME_ADDR: &str = "callee";
    const CALLER_NAME_ADDR: &str = "caller";

    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    fn prepare_dynamic_call_data(
        callee_address: Option<Addr>,
        func_info: FunctionMetadata,
        caller_env: &mut Environment<MockApi, MockStorage, MockQuerier>,
    ) -> Option<u32> {
        let region_ptr = callee_address.map(|addr| {
            let data = to_vec(&addr).unwrap();
            write_data_to_mock_env(caller_env, &data).unwrap()
        });

        caller_env.set_callee_function_metadata(Some(func_info));

        let serialized_env = to_vec(&mock_env()).unwrap();
        caller_env.set_serialized_env(&serialized_env);

        let storage = MockStorage::new();
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(INIT_ADDR, &coins(INIT_AMOUNT, INIT_DENOM))]);
        caller_env.move_in(storage, querier);
        region_ptr
    }

    #[test]
    fn copy_single_region_works() {
        let src_instance = mock_instance(&CONTRACT, &[]);
        let dst_instance = mock_instance(&CONTRACT, &[]);

        let data_wasm_ptr = write_data_to_mock_env(&src_instance.env, PASS_DATA1).unwrap();
        let copy_result = copy_region_vals_between_env(
            &src_instance.env,
            &dst_instance.env,
            &[WasmerVal::I32(data_wasm_ptr as i32)],
            true,
        )
        .unwrap();
        assert_eq!(copy_result.len(), 1);

        let read_result =
            read_data_from_mock_env(&dst_instance.env, &copy_result[0], PASS_DATA1.len()).unwrap();
        assert_eq!(PASS_DATA1, read_result);

        // Even after deallocate, wasm region data remains.
        // However, This test is skipped as it is a matter of whether allocate and deallocate work as expected.
        // let read_deallocated_src_result = read_region(&src_env.memory(), data_wasm_ptr, PASS_DATA1.len());
        // assert!(matches!(
        //     read_deallocated_src_result,
        //     Err(VmError::CommunicationErr { .. })
        // ));
    }

    #[test]
    fn wrong_use_copied_region_fails() {
        let src_instance = mock_instance(&CONTRACT, &[]);
        let dst_instance = mock_instance(&CONTRACT, &[]);

        // If there is no padding data, it is difficult to compare because the same memory index falls apart.
        write_data_to_mock_env(&src_instance.env, PADDING_DATA).unwrap();

        let data_wasm_ptr = write_data_to_mock_env(&src_instance.env, PASS_DATA1).unwrap();
        let copy_result = copy_region_vals_between_env(
            &src_instance.env,
            &dst_instance.env,
            &[WasmerVal::I32(data_wasm_ptr as i32)],
            true,
        )
        .unwrap();
        assert_eq!(copy_result.len(), 1);

        let read_from_src_result =
            read_data_from_mock_env(&src_instance.env, &copy_result[0], PASS_DATA1.len());
        assert!(matches!(
            read_from_src_result,
            Err(VmError::CommunicationErr { .. })
        ));
    }

    fn init_cache_with_two_instances() {
        let callee_wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))
                (export "interface_version_5" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (type $t_succeed (func))
                (func $f_succeed (type $t_succeed) nop)
                (type $t_fail (func))
                (func $f_fail (type $t_fail) unreachable)
                (export "succeed" (func $f_succeed))
                (export "fail" (func $f_fail))
            )"#,
        )
        .unwrap();

        INSTANCE_CACHE.with(|lock| {
            let mut cache = lock.write().unwrap();
            cache.insert(
                CALLEE_NAME_ADDR.to_string(),
                RefCell::new(mock_instance(&callee_wasm, &[])),
            );
            cache.insert(
                CALLER_NAME_ADDR.to_string(),
                RefCell::new(mock_instance(&CONTRACT, &[])),
            );
        });
    }

    #[test]
    fn native_dynamic_link_trampoline_works() {
        init_cache_with_two_instances();

        INSTANCE_CACHE.with(|lock| {
            let cache = lock.read().unwrap();
            let caller_instance = cache.get(CALLER_NAME_ADDR).unwrap();
            let mut caller_env = &mut caller_instance.borrow_mut().env;
            let target_func_info = FunctionMetadata {
                module_name: CALLER_NAME_ADDR.to_string(),
                name: "succeed".to_string(),
                signature: ([Type::I32], []).into(),
            };
            let address_region = prepare_dynamic_call_data(
                Some(Addr::unchecked(CALLEE_NAME_ADDR)),
                target_func_info,
                &mut caller_env,
            )
            .unwrap();

            let result = native_dynamic_link_trampoline(
                &caller_env,
                &[WasmerVal::I32(address_region as i32)],
            )
            .unwrap();
            assert_eq!(result.len(), 0);
        });
    }

    #[test]
    fn native_dynamic_link_trampoline_do_not_specify_callee_address_fail() {
        init_cache_with_two_instances();

        INSTANCE_CACHE.with(|lock| {
            let cache = lock.read().unwrap();
            let caller_instance = cache.get(CALLER_NAME_ADDR).unwrap();
            let mut caller_env = &mut caller_instance.borrow_mut().env;
            let target_func_info = FunctionMetadata {
                module_name: CALLER_NAME_ADDR.to_string(),
                name: "foo".to_string(),
                signature: ([Type::I32], []).into(),
            };
            let none = prepare_dynamic_call_data(None, target_func_info, &mut caller_env);
            assert_eq!(none, None);

            let result = native_dynamic_link_trampoline(&caller_env, &[]);
            assert!(matches!(result, Err(RuntimeError { .. })));

            assert_eq!(
                result.err().unwrap().message(),
                "No args are passed to trampoline. The first arg must be callee contract address."
            );
        });
    }

    #[test]
    fn native_dynamic_link_trampoline_not_exist_callee_address_fails() {
        init_cache_with_two_instances();

        INSTANCE_CACHE.with(|lock| {
            let cache = lock.read().unwrap();
            let caller_instance = cache.get(CALLER_NAME_ADDR).unwrap();
            let mut caller_env = &mut caller_instance.borrow_mut().env;
            let target_func_info = FunctionMetadata {
                module_name: CALLER_NAME_ADDR.to_string(),
                name: "foo".to_string(),
                signature: ([Type::I32], []).into(),
            };
            let address_region = prepare_dynamic_call_data(
                Some(Addr::unchecked("invalid_address")),
                target_func_info,
                &mut caller_env,
            ).unwrap();

            let result = native_dynamic_link_trampoline(&caller_env, &[WasmerVal::I32(address_region as i32)]);
            assert!(matches!(
                result,
                Err(RuntimeError { .. })
            ));

            assert_eq!(result.err().unwrap().message(),
            "func_info:{module_name:caller, name:foo, signature:[] -> []}, error:Unknown error during call into backend: Some(\"cannot found contract\")"
            );
        });
    }
}
