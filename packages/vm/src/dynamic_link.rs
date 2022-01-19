use std::collections::HashMap;
use std::fmt;
use std::str;

use crate::backend::{BackendApi, Querier, Storage};
use crate::environment::{process_gas_info, Environment};
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

fn native_dynamic_link_trampoline<A: BackendApi, S: Storage, Q: Querier>(
    env: &Environment<A, S, Q>,
    args: &[WasmerVal],
) -> Result<Vec<WasmerVal>, RuntimeError>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
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
        Err(e) => return Err(RuntimeError::new("Invalid stored callee contract address")),
    };

    let (call_result, gas_info) =
        env.api
            .contract_call(contract_addr, &func_info, args, env.get_gas_left());
    process_gas_info::<A, S, Q>(env, gas_info)?;
    match call_result {
        Ok(ret) => Ok(ret.iter().map(|v| v.clone()).collect()),
        Err(e) => Err(RuntimeError::new(format!(
            "func_info:{{{}}}, error:{}",
            func_info, e
        ))),
    }
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
        match import_index {
            ImportIndex::Function(func_index) => {
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
            _ => {}
        }
    }

    // link to gateway host function
    for module_name in import_functions_by_module.keys() {
        let mut module_exports = Exports::new();
        let func_infos = &import_functions_by_module[module_name];
        for func_metadata in func_infos {
            // make a new enviorment struct for pass the target function information
            let dynamic_env = env.clone();
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
