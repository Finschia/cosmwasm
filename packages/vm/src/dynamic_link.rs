use std::collections::HashMap;
use std::fmt;

use crate::backend::{BackendApi, Querier, Storage};
use crate::conversion::ref_to_u32;
use crate::environment::{process_gas_info, Environment};
use crate::errors::{CommunicationError, VmError, VmResult};
use crate::imports::write_to_contract;
use crate::instance::Instance;
use crate::memory::read_region;
use crate::serde::from_slice;
use serde::{Deserialize, Serialize};
use wasmer::{
    AsStoreMut, Exports, ExternType, Function, FunctionEnv, FunctionEnvMut, FunctionType, Imports,
    Module, RuntimeError, Store, Value,
};

use cosmwasm_std::{Addr, Binary};

// The length of the address is 63 characters for strings and 65 characters with "" for []byte. Thus, 65<64*2 is used.
const MAX_ADDRESS_LENGTH: usize = 64 * 2;
// enough big value for copy interface. This is less than crate::calls::read_limits::XXX
const MAX_INTERFACE_REGIONS_LENGTH: usize = 1024 * 1024;
const MAX_PROPERTIES_REGIONS_LENGTH: usize = 1024 * 1024;
const MAX_REGIONS_LENGTH_INPUT: usize = 64 * 1024 * 1024;
const GET_PROPERTY_FUNCTION: &str = "_get_callable_points_properties";

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

fn native_dynamic_link_trampoline<
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
>(
    mut fe: FunctionEnvMut<Environment<A, S, Q>>,
    args: &[Value],
) -> Result<Vec<Value>, RuntimeError>
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
    let (env, mut store) = fe.data_and_store_mut();
    let address_region_ptr = ref_to_u32(&args[0])?;
    let contract_addr_binary =
        read_region(&env.memory(&store), address_region_ptr, MAX_ADDRESS_LENGTH)?;
    let contract_addr: Addr = from_slice(&contract_addr_binary, MAX_ADDRESS_LENGTH)
        .map_err(|_| RuntimeError::new("Invalid callee contract address"))?;
    let mut args_data: Vec<Binary> = vec![];
    for arg in &args[1..] {
        let arg_ptr = ref_to_u32(arg)?;
        let arg_data = read_region(&env.memory(&store), arg_ptr, MAX_REGIONS_LENGTH_INPUT)?;
        args_data.push(Binary(arg_data))
    }
    let args_binary = serde_json::to_vec(&args_data).map_err(|e| {
        RuntimeError::new(format!(
            "Error during serializing args for a callable point: {}",
            e
        ))
    })?;
    let func_info = env.with_callee_function_metadata(|func_info| {
        Ok(func_info.clone_and_drop_callee_addr_arg())
    })?;
    let callstack = env.get_dynamic_callstack()?;
    let callstack_binary = serde_json::to_vec(&callstack).map_err(|e| {
        RuntimeError::new(format!(
            "Error during serializing callstack of callable points: {}",
            e
        ))
    })?;
    let (call_result, gas_info) = env.api.call_callable_point(
        contract_addr.as_str(),
        &func_info.name,
        &args_binary,
        env.is_storage_readonly(),
        &callstack_binary,
        env.get_gas_left(&mut store),
    );
    process_gas_info::<A, S, Q>(env, &mut store, gas_info)?;
    match call_result {
        Ok(ret) => match serde_json::from_slice::<Option<Binary>>(&ret).map_err(|e| {
            RuntimeError::new(format!(
                r#"Error during deserializing result of callable point "{}" of "{}": {}"#,
                func_info.name, contract_addr, e
            ))
        })? {
            Some(v) => Ok(vec![write_value_to_env::<A, S, Q>(env, &mut store, &v)?]),
            None => Ok(vec![]),
        },
        Err(e) => Err(RuntimeError::new(format!(
            r#"Error during calling callable point "{}" of "{}" (func_info:{{{}}}): error:{}"#,
            func_info.name, contract_addr, func_info, e,
        ))),
    }
}

#[cfg(feature = "bench")]
pub fn native_dynamic_link_trampoline_for_bench<
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
>(
    fe: FunctionEnvMut<Environment<A, S, Q>>,
    args: &[Value],
) -> Result<Vec<Value>, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    native_dynamic_link_trampoline(fe, args)
}

fn into_functiontype(name: &str, ty: &ExternType) -> VmResult<FunctionType> {
    match ty {
        ExternType::Function(ft) => Ok(ft.clone()),
        _ => Err(VmError::invalid_dynamic_link_import(name)),
    }
}

pub fn dynamic_link<A: BackendApi, S: Storage, Q: Querier>(
    store: &mut Store,
    fe: &FunctionEnv<Environment<A, S, Q>>,
    module: &Module,
    imports: &mut Imports,
) -> VmResult<()>
where
    A: BackendApi + 'static, // 'static is needed here to allow copying API instances into closures
    S: Storage + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
    Q: Querier + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
{
    // Getting required imports to onther contracts
    let mut import_functions_by_module: HashMap<String, Vec<FunctionMetadata>> = HashMap::new();
    for import in module
        .imports()
        .filter(|import| import.module().starts_with("dynamiclinked_"))
    {
        //if compiled with '-s' option(symbol strapping), function_names is empty.
        import_functions_by_module
            .entry(import.module().to_string())
            .or_default()
            .push(FunctionMetadata {
                module_name: import.module().to_string(),
                name: import.name().to_string(),
                signature: into_functiontype(import.name(), import.ty())?,
            })
    }

    // link to gateway host function
    for module_name in import_functions_by_module.keys() {
        let mut module_exports = Exports::new();
        let func_infos = &import_functions_by_module[module_name];
        for func_metadata in func_infos {
            // make a new enviorment struct for pass the target function information
            let dynamic_fe = fe.clone();

            fe.as_mut(store)
                .set_callee_function_metadata(Some(func_metadata.clone()));
            module_exports.insert(
                func_metadata.name.clone(),
                Function::new_with_env(
                    store,
                    &dynamic_fe,
                    func_metadata.signature.clone(),
                    native_dynamic_link_trampoline,
                ),
            );
        }

        imports.register_namespace(module_name, module_exports);
    }

    Ok(())
}

// returns copied region ptrs and size of regions
pub fn read_region_vals<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    ptrs: &[Value],
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
        let data = instance
            .read_memory(val_region_ptr, max_regions_len)
            .map_err(|e| match e {
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
            instance.deallocate(val_region_ptr)?;
        };
    }

    Ok(datas)
}

pub fn write_value_to_env<A, S, Q>(
    env: &Environment<A, S, Q>,
    store: &mut impl AsStoreMut,
    value: &[u8],
) -> VmResult<Value>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    Ok(write_to_contract(env, store, value)?.into())
}

pub fn native_validate_dynamic_link_interface<
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
>(
    mut fe: FunctionEnvMut<Environment<A, S, Q>>,
    address: u32,
    interface: u32,
) -> Result<u32, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let (env, mut store) = fe.data_and_store_mut();

    let contract_addr_raw = read_region(&env.memory(&store), address, MAX_ADDRESS_LENGTH)?;
    let contract_addr: Addr = from_slice(&contract_addr_raw, MAX_ADDRESS_LENGTH)
        .map_err(|_| RuntimeError::new("Invalid contract address to validate interface"))?;
    let expected_interface_binary =
        read_region(&env.memory(&store), interface, MAX_INTERFACE_REGIONS_LENGTH)?;
    let (result_data, gas_info) = env
        .api
        .validate_dynamic_link_interface(contract_addr.as_str(), &expected_interface_binary);
    process_gas_info::<A, S, Q>(env, &mut store, gas_info)?;
    let result_data = result_data.map_err(|e| {
        RuntimeError::new(format!(
            "Error during calling validate_dynamic_link_interface: {}",
            e
        ))
    })?;
    let result: Option<String> = serde_json::from_slice(&result_data).map_err(|e| {
        RuntimeError::new(format!(
            "Error during deserializing the result of validate_dynamic_link_interface: {}",
            e
        ))
    })?;
    match result {
        Some(err_msg) => Ok(write_to_contract::<A, S, Q>(
            env,
            &mut store,
            err_msg.as_bytes(),
        )?),
        None => Ok(0),
    }
}

// CalleeProperty represents property about the function of callee
#[derive(Serialize, Deserialize)]
struct CalleeProperty {
    is_read_only: bool,
}

// This sets callee instance read/write permission according to
// GET_PROPERTY_FUNCTION in callee instance.
// This checks callee instance does not take write permission in readonly context.
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

    let ret_datas = read_region_vals(callee_instance, &ret, MAX_PROPERTIES_REGIONS_LENGTH, true)?;
    if ret_datas.len() != 1 {
        return Err(VmError::dynamic_call_err(format!(
            "{} returns no or more than 1 values. It should returns just 1 value.",
            GET_PROPERTY_FUNCTION
        )));
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
        return Err(VmError::dynamic_call_err(
            "a read-write callable point is called in read-only context.",
        ));
    };

    callee_instance.set_storage_readonly(property.is_read_only);
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    use crate::testing::mock_instance;

    const MAX_REGIONS_LENGTH: usize = 1024 * 1024;

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static CONTRACT_CALLEE: &[u8] = include_bytes!("../testdata/simple_callee.wasm");

    // prepared data
    const PASS_DATA: &[u8] = b"data";

    #[test]
    fn read_single_region_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        let region_ptr = instance.allocate(PASS_DATA.len()).unwrap();
        instance.write_memory(region_ptr, PASS_DATA).unwrap();

        let datas = read_region_vals(
            &mut instance,
            &[Value::I32(region_ptr as i32)],
            MAX_REGIONS_LENGTH,
            true,
        )
        .unwrap();
        assert_eq!(datas.len(), 1);
        assert_eq!(datas[0], PASS_DATA);

        // check deallocated
        match read_region_vals(
            &mut instance,
            &[Value::I32(region_ptr as i32)],
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
        let mut src_instance = mock_instance(CONTRACT, &[]);

        let big_data_1 = [0_u8; MAX_REGIONS_LENGTH - 42 + 1];
        let big_data_2 = [1_u8; 42];

        let region_ptr1 = src_instance.allocate(big_data_1.len()).unwrap();
        src_instance.write_memory(region_ptr1, &big_data_1).unwrap();
        let region_ptr2 = src_instance.allocate(big_data_2.len()).unwrap();
        src_instance.write_memory(region_ptr2, &big_data_2).unwrap();

        let read_result = read_region_vals(
            &mut src_instance,
            &[
                Value::I32(region_ptr1 as i32),
                Value::I32(region_ptr2 as i32),
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

    #[test]
    fn set_callee_permission_works_readwrite() {
        let mut instance = mock_instance(CONTRACT_CALLEE, &[]);
        set_callee_permission(&mut instance, "succeed", false).unwrap();
        assert!(!instance.is_storage_readonly())
    }

    #[test]
    fn set_callee_permission_works_readonly() {
        let mut instance = mock_instance(CONTRACT_CALLEE, &[]);
        set_callee_permission(&mut instance, "succeed_readonly", true).unwrap();
        assert!(instance.is_storage_readonly())
    }

    #[test]
    #[should_panic(expected = "a read-write callable point is called in read-only context.")]
    fn set_callee_permission_fails() {
        let mut instance = mock_instance(CONTRACT_CALLEE, &[]);
        set_callee_permission(&mut instance, "succeed", true).unwrap();
    }
}
