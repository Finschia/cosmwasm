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
    with_trace_dynamic_call(env, || {
        let func_info = env
            .with_callee_function_metadata(|func_info| Ok(func_info.clone()))
            .unwrap();

        let (store_result, gas_info) = env.with_storage_from_context::<_, _>(|store| {
            Ok(store.get(func_info.module_name.as_bytes()))
        })?;
        process_gas_info::<A, S, Q>(env, gas_info)?;
        let raw_contract_addr = match store_result.unwrap() {
            Some(raw_contract_addr) => raw_contract_addr,
            None => {
                return Err(RuntimeError::new(
                    "cannot found the callee contract address in the storage",
                ))
            }
        };
        let contract_addr = match str::from_utf8(&raw_contract_addr) {
            Ok(contract_addr) => contract_addr.trim_matches('"'),
            Err(_) => return Err(RuntimeError::new("Invalid stored callee contract address")),
        };

        let (call_result, gas_info) =
            env.api
                .contract_call(env, contract_addr, &func_info, args);
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
    use std::ptr::NonNull;
    use wasmer::{imports, Function, Instance as WasmerInstance};

    use crate::size::Size;
    use crate::testing::{
        mock_env, read_data_from_mock_env, write_data_to_mock_env, MockApi, MockQuerier,
        MockStorage,
    };
    use crate::to_vec;
    use crate::wasm_backend::compile;
    use crate::VmError;

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    // prepared data
    const PADDING_DATA: &[u8] = b"deadbeef";
    const PASS_DATA1: &[u8] = b"data";

    const TESTING_GAS_LIMIT: u64 = 500_000;
    const TESTING_MEMORY_LIMIT: Option<Size> = Some(Size::mebi(16));

    fn make_instance(
        api: MockApi,
    ) -> (
        Environment<MockApi, MockStorage, MockQuerier>,
        Box<WasmerInstance>,
    ) {
        let gas_limit = TESTING_GAS_LIMIT;
        let env = Environment::new(api, gas_limit, false);

        let module = compile(&CONTRACT, TESTING_MEMORY_LIMIT).unwrap();
        let store = module.store();
        // we need stubs for all required imports
        let import_obj = imports! {
            "env" => {
                "db_read" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "db_write" => Function::new_native(&store, |_a: u32, _b: u32| {}),
                "db_remove" => Function::new_native(&store, |_a: u32| {}),
                "db_scan" => Function::new_native(&store, |_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "query_chain" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "addr_validate" => Function::new_native(&store, |_a: u32| -> u32 { 0 }),
                "addr_canonicalize" => Function::new_native(&store, |_a: u32, _b: u32| -> u32 { 0 }),
                "addr_humanize" => Function::new_native(&store, |_a: u32, _b: u32| -> u32 { 0 }),
                "secp256k1_verify" => Function::new_native(&store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "secp256k1_recover_pubkey" => Function::new_native(&store, |_a: u32, _b: u32, _c: u32| -> u64 { 0 }),
                "ed25519_verify" => Function::new_native(&store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "ed25519_batch_verify" => Function::new_native(&store, |_a: u32, _b: u32, _c: u32| -> u32 { 0 }),
                "sha1_calculate" => Function::new_native(&store, |_a: u32| -> u64 { 0 }),
                "debug" => Function::new_native(&store, |_a: u32| {}),
            },
        };
        let instance = Box::from(WasmerInstance::new(&module, &import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));
        env.set_gas_left(gas_limit);

        let serialized_env = to_vec(&mock_env()).unwrap();
        env.set_serialized_env(&serialized_env);

        (env, instance)
    }

    #[test]
    fn copy_single_region_works() {
        let api = MockApi::default();
        let (src_env, _src_instance) = make_instance(api);
        let (dst_env, _dst_instance) = make_instance(api);

        let data_wasm_ptr = write_data_to_mock_env(&src_env, PASS_DATA1).unwrap();
        let copy_result = copy_region_vals_between_env(
            &src_env,
            &dst_env,
            &[WasmerVal::I32(data_wasm_ptr as i32)],
            true,
        )
        .unwrap();
        assert_eq!(copy_result.len(), 1);

        let read_result =
            read_data_from_mock_env(&dst_env, &copy_result[0], PASS_DATA1.len()).unwrap();
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
        let api = MockApi::default();
        let (src_env, _src_instance) = make_instance(api);
        let (dst_env, _dst_instance) = make_instance(api);

        // If there is no padding data, it is difficult to compare because the same memory index falls apart.
        write_data_to_mock_env(&src_env, PADDING_DATA).unwrap();

        let data_wasm_ptr = write_data_to_mock_env(&src_env, PASS_DATA1).unwrap();
        let copy_result = copy_region_vals_between_env(
            &src_env,
            &dst_env,
            &[WasmerVal::I32(data_wasm_ptr as i32)],
            true,
        )
        .unwrap();
        assert_eq!(copy_result.len(), 1);

        let read_from_src_result =
            read_data_from_mock_env(&src_env, &copy_result[0], PASS_DATA1.len());
        assert!(matches!(
            read_from_src_result,
            Err(VmError::CommunicationErr { .. })
        ));
    }
}
