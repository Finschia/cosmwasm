use std::collections::HashMap;
use std::fmt;

use crate::backend::{BackendApi, Querier, Storage};
use crate::conversion::ref_to_u32;
use crate::environment::{process_gas_info, Environment};
use crate::errors::{CommunicationError, VmError, VmResult};
use crate::imports::write_to_contract;
use crate::instance::Instance;
use crate::memory::read_region;
use serde::{Deserialize, Serialize};
use wasmer::{
    ExportType, Exports, Function, FunctionType, ImportObject, Module, RuntimeError, Val,
};
use wasmer_types::ImportIndex;

use cosmwasm_std::{from_slice, Addr};

// The length of the address is 63 characters for strings and 65 characters with "" for []byte. Thus, 65<64*2 is used.
const MAX_ADDRESS_LENGTH: usize = 64 * 2;
// enough big value for copy interface. This is less than crate::calls::read_limits::XXX
const MAX_INTERFACE_REGIONS_LENGTH: usize = 1024 * 1024;
const MAX_PROPERTIES_REGIONS_LENGTH: usize = 1024 * 1024;
const GET_PROPERTY_FUNCTION: &str = "_get_callable_points_properties";

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
    let contract_addr_binary = read_region(&env.memory(), address_region_ptr, MAX_ADDRESS_LENGTH)?;
    let contract_addr: Addr = from_slice(&contract_addr_binary)
        .map_err(|_| RuntimeError::new("Invalid callee contract address"))?;
    let func_args = &args[1..];
    with_trace_dynamic_call(env, || {
        let func_info = env
            .with_callee_function_metadata(|func_info| {
                Ok(func_info.clone_and_drop_callee_addr_arg())
            })
            .unwrap();
        let (call_result, gas_info) =
            env.api
                .contract_call(env, contract_addr.as_str(), &func_info, func_args);
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
        .filter(|((module_name, _, _), _)| module_name.starts_with("dynamiclinked_"))
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

// returns copied region ptrs and size of regions
pub fn read_region_vals_from_env<A, S, Q>(
    env: &Environment<A, S, Q>,
    ptrs: &[WasmerVal],
    limit_length: usize,
    deallocation: bool,
) -> VmResult<Vec<Vec<u8>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let mut datas = Vec::<Vec<u8>>::with_capacity(ptrs.len());
    let mut max_regions_len = limit_length;
    for ptr in ptrs {
        let val_region_ptr = ref_to_u32(ptr)?;
        let data =
            read_region(&env.memory(), val_region_ptr, max_regions_len).map_err(|e| match e {
                VmError::CommunicationErr {
                    source: CommunicationError::RegionLengthTooBig { .. },
                    #[cfg(feature = "backtraces")]
                    backtrace,
                } => VmError::CommunicationErr {
                    source: CommunicationError::exceeds_limit_length_copy_regions(limit_length),
                    #[cfg(feature = "backtraces")]
                    backtrace,
                },
                _ => e,
            })?;
        max_regions_len -= data.len();
        datas.push(data);

        if deallocation {
            env.call_function0("deallocate", &[val_region_ptr.into()])?;
        };
    }

    Ok(datas)
}

pub fn write_value_to_env<A, S, Q>(env: &Environment<A, S, Q>, value: &[u8]) -> VmResult<WasmerVal>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    Ok(write_to_contract(env, value)?.into())
}

pub fn native_validate_dynamic_link_interface<A: BackendApi, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    address: u32,
    interface: u32,
) -> Result<u32, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let contract_addr_raw = read_region(&env.memory(), address, MAX_ADDRESS_LENGTH)?;
    let contract_addr: Addr = from_slice(&contract_addr_raw)
        .map_err(|_| RuntimeError::new("Invalid contract address to validate interface"))?;
    let expected_interface_binary =
        read_region(&env.memory(), interface, MAX_INTERFACE_REGIONS_LENGTH)?;
    let expected_interface: Vec<ExportType<FunctionType>> = from_slice(&expected_interface_binary)
        .map_err(|_| RuntimeError::new("Invalid expected interface"))?;
    let (module_result, gas_info) = env.api.get_wasmer_module(contract_addr.as_str());
    process_gas_info::<A, S, Q>(env, gas_info)?;
    let module = module_result.map_err(|_| RuntimeError::new("Cannot get module"))?;
    let mut exported_fns: HashMap<String, FunctionType> = HashMap::new();
    for f in module.exports().functions() {
        exported_fns.insert(f.name().to_string(), f.ty().clone());
    }

    // No gas fee for comparison now
    let mut err_msg = "The following functions are not implemented: ".to_string();
    let mut is_err = false;
    for expected_fn in expected_interface.iter() {
        // if not expected
        if !exported_fns
            .get(expected_fn.name())
            .map_or(false, |t| t == expected_fn.ty())
        {
            if is_err {
                err_msg.push_str(", ");
            };
            err_msg.push_str(&format!("{}: {}", expected_fn.name(), expected_fn.ty()));
            is_err = true;
        }
    }

    if is_err {
        // not expected
        Ok(write_to_contract::<A, S, Q>(env, err_msg.as_bytes())?)
    } else {
        // as expected
        Ok(0)
    }
}

// CalleeProperty represents property about the function of callee
#[derive(Serialize, Deserialize)]
struct CalleeProperty {
    is_read_only: bool,
}

pub fn set_callee_permission<A, S, Q>(
    callee_instance: &mut Instance<A, S, Q>,
    callable_point: &str,
    is_readonly_context: bool,
) -> VmResult<()>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    callee_instance.set_storage_readonly(true);
    let ret = callee_instance.call_function(GET_PROPERTY_FUNCTION, &[])?;

    let ret_datas = read_region_vals_from_env(
        &callee_instance.env,
        &ret,
        MAX_PROPERTIES_REGIONS_LENGTH,
        true,
    )?;
    if ret_datas.len() != 1 {
        Err(VmError::dynamic_call_err(format!(
            "{} returns no or more than 1 values. It should returns just 1 value.",
            GET_PROPERTY_FUNCTION
        )))?
    };

    let properties: HashMap<String, CalleeProperty> = serde_json::from_slice(&ret_datas[0])
        .map_err(|e| VmError::dynamic_call_err(e.to_string()))?;

    let property = properties.get(callable_point).ok_or_else(|| {
        VmError::dynamic_call_err(format!(
            "callee function properties has not key:{}",
            callable_point
        ))
    })?;

    if is_readonly_context && !property.is_read_only {
        // An error occurs because read-only permission cannot be inherited from read-write permission
        Err(VmError::dynamic_call_err(
            "a read-write callable point is called in read-only context.",
        ))?
    };

    Ok(callee_instance.set_storage_readonly(property.is_read_only))
}

#[cfg(test)]
mod tests {
    use super::*;
    use cosmwasm_std::{coins, Addr, Empty};
    use std::cell::RefCell;
    use wasmer_types::Type;

    use crate::testing::{
        mock_backend, mock_env, mock_instance, write_data_to_mock_env, MockApi, MockQuerier,
        MockStorage, INSTANCE_CACHE,
    };
    use crate::{to_vec, Instance, InstanceOptions};

    const MAX_REGIONS_LENGTH: usize = 1024 * 1024;
    const HIGH_GAS_LIMIT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
        gas_limit: 20_000_000_000_000_000,
        print_debug: false,
    };

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CONTRACT_CALLEE: &[u8] = include_bytes!("../testdata/simple_callee.wasm");

    // prepared data
    const PASS_DATA: &[u8] = b"data";

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
    fn read_single_region_works() {
        let instance = mock_instance(&CONTRACT, &[]);

        let data_wasm_ptr = write_data_to_mock_env(&instance.env, PASS_DATA).unwrap();
        let datas = read_region_vals_from_env(
            &instance.env,
            &[WasmerVal::I32(data_wasm_ptr as i32)],
            MAX_REGIONS_LENGTH,
            true,
        )
        .unwrap();
        assert_eq!(datas.len(), 1);
        assert_eq!(datas[0], PASS_DATA);

        // check deallocated
        match read_region_vals_from_env(
            &instance.env,
            &[WasmerVal::I32(data_wasm_ptr as i32)],
            MAX_REGIONS_LENGTH,
            true,
        ) {
            Ok(datas) => assert_ne!(datas[0], PASS_DATA),
            Err(VmError::RuntimeErr { msg, .. }) => {
                assert!(msg.contains("out of bounds memory access"))
            }
            Err(e) => panic!("Unexpected Error: {}", e),
        }
    }

    #[test]
    fn trying_read_too_large_region_fails() {
        let src_instance = mock_instance(&CONTRACT, &[]);

        let big_data_1 = [0_u8; MAX_REGIONS_LENGTH - 42 + 1];
        let big_data_2 = [1_u8; 42];

        let data_ptr1 = write_data_to_mock_env(&src_instance.env, &big_data_1).unwrap();
        let data_ptr2 = write_data_to_mock_env(&src_instance.env, &big_data_2).unwrap();
        let read_result = read_region_vals_from_env(
            &src_instance.env,
            &[
                WasmerVal::I32(data_ptr1 as i32),
                WasmerVal::I32(data_ptr2 as i32),
            ],
            MAX_REGIONS_LENGTH,
            true,
        );
        assert!(matches!(
            read_result.unwrap_err(),
            VmError::CommunicationErr {
                source: CommunicationError::ExceedsLimitLengthCopyRegions {
                    max_length: MAX_REGIONS_LENGTH
                },
                ..
            }
        ))
    }

    fn init_cache_with_two_instances() {
        INSTANCE_CACHE.with(|lock| {
            let mut cache = lock.write().unwrap();
            cache.insert(
                CALLEE_NAME_ADDR.to_string(),
                RefCell::new(
                    Instance::from_code(
                        CONTRACT_CALLEE,
                        mock_backend(&[]),
                        HIGH_GAS_LIMIT_INSTANCE_OPTIONS,
                        None,
                    )
                    .unwrap(),
                ),
            );
            cache.insert(
                CALLER_NAME_ADDR.to_string(),
                RefCell::new(
                    Instance::from_code(
                        CONTRACT,
                        mock_backend(&[]),
                        HIGH_GAS_LIMIT_INSTANCE_OPTIONS,
                        None,
                    )
                    .unwrap(),
                ),
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
                result.unwrap_err().message(),
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

            assert_eq!(result.unwrap_err().message(),
            "func_info:{module_name:caller, name:foo, signature:[] -> []}, error:Error in dynamic link: \"cannot found contract\""
            );
        });
    }

    #[test]
    fn dynamic_link_callee_contract_fails() {
        init_cache_with_two_instances();

        INSTANCE_CACHE.with(|lock| {
            let cache = lock.read().unwrap();
            let caller_instance = cache.get(CALLER_NAME_ADDR).unwrap();
            let mut caller_env = &mut caller_instance.borrow_mut().env;
            let target_func_info = FunctionMetadata {
                module_name: CALLER_NAME_ADDR.to_string(),
                name: "fail".to_string(),
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
            );
            assert!(matches!(result, Err(RuntimeError { .. })));
            assert!(result.unwrap_err().message().starts_with("func_info:{module_name:caller, name:fail, signature:[] -> []}, error:Error in dynamic link: \"Error executing Wasm: Wasmer runtime error: RuntimeError: unreachable"));
        });
    }

    #[test]
    fn set_callee_permission_works_readwrite() {
        let mut instance = mock_instance(&CONTRACT_CALLEE, &[]);
        set_callee_permission(&mut instance, "succeed", false).unwrap();
        assert!(!instance.env.is_storage_readonly())
    }

    #[test]
    fn set_callee_permission_works_readonly() {
        let mut instance = mock_instance(&CONTRACT_CALLEE, &[]);
        set_callee_permission(&mut instance, "succeed_readonly", true).unwrap();
        assert!(instance.env.is_storage_readonly())
    }

    #[test]
    #[should_panic(expected = "a read-write callable point is called in read-only context.")]
    fn set_callee_permission_fails() {
        let mut instance = mock_instance(&CONTRACT_CALLEE, &[]);
        set_callee_permission(&mut instance, "succeed", true).unwrap();
    }
}
