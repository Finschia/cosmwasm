use std::collections::HashSet;
use std::ptr::NonNull;

use wasmer::{Exports, Function, ImportObject, Instance as WasmerInstance, Module, Val};

use crate::backend::{Backend, BackendApi, Querier, Storage};
use crate::conversion::{ref_to_u32, to_u32};
use crate::environment::Environment;
use crate::errors::{CommunicationError, VmError, VmResult};
use crate::features::required_features_from_wasmer_instance;
use crate::imports::{
    native_addr_canonicalize, native_addr_humanize, native_addr_validate, native_db_read,
    native_db_remove, native_db_write, native_debug, native_ed25519_batch_verify,
    native_ed25519_verify, native_query_chain, native_secp256k1_recover_pubkey,
    native_secp256k1_verify,
};
#[cfg(feature = "iterator")]
use crate::imports::{native_db_next, native_db_scan};
use crate::memory::{read_region, write_region};
use crate::size::Size;
use crate::wasm_backend::compile;

#[derive(Copy, Clone, Debug)]
pub struct GasReport {
    /// The original limit the instance was created with
    pub limit: u64,
    /// The remaining gas that can be spend
    pub remaining: u64,
    /// The amount of gas that was spend and metered externally in operations triggered by this instance
    pub used_externally: u64,
    /// The amount of gas that was spend and metered internally (i.e. by executing Wasm and calling
    /// API methods which are not metered externally)
    pub used_internally: u64,
}

#[derive(Copy, Clone, Debug)]
pub struct InstanceOptions {
    pub gas_limit: u64,
    pub print_debug: bool,
}

pub struct Instance<A: BackendApi, S: Storage, Q: Querier> {
    /// We put this instance in a box to maintain a constant memory address for the entire
    /// lifetime of the instance in the cache. This is needed e.g. when linking the wasmer
    /// instance to a context. See also https://github.com/CosmWasm/cosmwasm/pull/245.
    ///
    /// This instance should only be accessed via the Environment, which provides safe access.
    _inner: Box<WasmerInstance>,
    env: Environment<A, S, Q>,
    pub required_features: HashSet<String>,
}

impl<A, S, Q> Instance<A, S, Q>
where
    A: BackendApi + 'static, // 'static is needed here to allow copying API instances into closures
    S: Storage + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
    Q: Querier + 'static, // 'static is needed here to allow using this in an Environment that is cloned into closures
{
    /// This is the only Instance constructor that can be called from outside of cosmwasm-vm,
    /// e.g. in test code that needs a customized variant of cosmwasm_vm::testing::mock_instance*.
    pub fn from_code(
        code: &[u8],
        backend: Backend<A, S, Q>,
        options: InstanceOptions,
        memory_limit: Option<Size>,
    ) -> VmResult<Self> {
        let module = compile(code, memory_limit)?;
        Instance::from_module(&module, backend, options.gas_limit, options.print_debug)
    }

    pub(crate) fn from_module(
        module: &Module,
        backend: Backend<A, S, Q>,
        gas_limit: u64,
        print_debug: bool,
    ) -> VmResult<Self> {
        let store = module.store();

        let env = Environment::new(backend.api, gas_limit, print_debug);

        let mut import_obj = ImportObject::new();
        let mut env_imports = Exports::new();

        // Reads the database entry at the given key into the the value.
        // Returns 0 if key does not exist and pointer to result region otherwise.
        // Ownership of the key pointer is not transferred to the host.
        // Ownership of the value pointer is transferred to the contract.
        env_imports.insert(
            "db_read",
            Function::new_native_with_env(store, env.clone(), native_db_read),
        );

        // Writes the given value into the database entry at the given key.
        // Ownership of both input and output pointer is not transferred to the host.
        env_imports.insert(
            "db_write",
            Function::new_native_with_env(store, env.clone(), native_db_write),
        );

        // Removes the value at the given key. Different than writing &[] as future
        // scans will not find this key.
        // At the moment it is not possible to differentiate between a key that existed before and one that did not exist (https://github.com/CosmWasm/cosmwasm/issues/290).
        // Ownership of both key pointer is not transferred to the host.
        env_imports.insert(
            "db_remove",
            Function::new_native_with_env(store, env.clone(), native_db_remove),
        );

        // Reads human address from source_ptr and checks if it is valid.
        // Returns 0 on if the input is valid. Returns a non-zero memory location to a Region containing an UTF-8 encoded error string for invalid inputs.
        // Ownership of the input pointer is not transferred to the host.
        env_imports.insert(
            "addr_validate",
            Function::new_native_with_env(store, env.clone(), native_addr_validate),
        );

        // Reads human address from source_ptr and writes canonicalized representation to destination_ptr.
        // A prepared and sufficiently large memory Region is expected at destination_ptr that points to pre-allocated memory.
        // Returns 0 on success. Returns a non-zero memory location to a Region containing an UTF-8 encoded error string for invalid inputs.
        // Ownership of both input and output pointer is not transferred to the host.
        env_imports.insert(
            "addr_canonicalize",
            Function::new_native_with_env(store, env.clone(), native_addr_canonicalize),
        );

        // Reads canonical address from source_ptr and writes humanized representation to destination_ptr.
        // A prepared and sufficiently large memory Region is expected at destination_ptr that points to pre-allocated memory.
        // Returns 0 on success. Returns a non-zero memory location to a Region containing an UTF-8 encoded error string for invalid inputs.
        // Ownership of both input and output pointer is not transferred to the host.
        env_imports.insert(
            "addr_humanize",
            Function::new_native_with_env(store, env.clone(), native_addr_humanize),
        );

        // Verifies message hashes against a signature with a public key, using the secp256k1 ECDSA parametrization.
        // Returns 0 on verification success, 1 on verification failure, and values greater than 1 in case of error.
        // Ownership of input pointers is not transferred to the host.
        env_imports.insert(
            "secp256k1_verify",
            Function::new_native_with_env(store, env.clone(), native_secp256k1_verify),
        );

        env_imports.insert(
            "secp256k1_recover_pubkey",
            Function::new_native_with_env(store, env.clone(), native_secp256k1_recover_pubkey),
        );

        // Verifies a message against a signature with a public key, using the ed25519 EdDSA scheme.
        // Returns 0 on verification success, 1 on verification failure, and values greater than 1 in case of error.
        // Ownership of input pointers is not transferred to the host.
        env_imports.insert(
            "ed25519_verify",
            Function::new_native_with_env(store, env.clone(), native_ed25519_verify),
        );

        // Verifies a batch of messages against a batch of signatures with a batch of public keys,
        // using the ed25519 EdDSA scheme.
        // Returns 0 on verification success (all batches verify correctly), 1 on verification failure, and values
        // greater than 1 in case of error.
        // Ownership of input pointers is not transferred to the host.
        env_imports.insert(
            "ed25519_batch_verify",
            Function::new_native_with_env(store, env.clone(), native_ed25519_batch_verify),
        );

        // Allows the contract to emit debug logs that the host can either process or ignore.
        // This is never written to chain.
        // Takes a pointer argument of a memory region that must contain an UTF-8 encoded string.
        // Ownership of both input and output pointer is not transferred to the host.
        env_imports.insert(
            "debug",
            Function::new_native_with_env(store, env.clone(), native_debug),
        );

        env_imports.insert(
            "query_chain",
            Function::new_native_with_env(store, env.clone(), native_query_chain),
        );

        // Creates an iterator that will go from start to end.
        // If start_ptr == 0, the start is unbounded.
        // If end_ptr == 0, the end is unbounded.
        // Order is defined in cosmwasm_std::Order and may be 1 (ascending) or 2 (descending). All other values result in an error.
        // Ownership of both start and end pointer is not transferred to the host.
        // Returns an iterator ID.
        #[cfg(feature = "iterator")]
        env_imports.insert(
            "db_scan",
            Function::new_native_with_env(store, env.clone(), native_db_scan),
        );

        // Get next element of iterator with ID `iterator_id`.
        // Creates a region containing both key and value and returns its address.
        // Ownership of the result region is transferred to the contract.
        // The KV region uses the format value || key || keylen, where keylen is a fixed size big endian u32 value.
        // An empty key (i.e. KV region ends with \0\0\0\0) means no more element, no matter what the value is.
        #[cfg(feature = "iterator")]
        env_imports.insert(
            "db_next",
            Function::new_native_with_env(store, env.clone(), native_db_next),
        );

        import_obj.register("env", env_imports);

        let wasmer_instance = Box::from(WasmerInstance::new(module, &import_obj).map_err(
            |original| {
                VmError::instantiation_err(format!("Error instantiating module: {:?}", original))
            },
        )?);

        let required_features = required_features_from_wasmer_instance(wasmer_instance.as_ref());
        let instance_ptr = NonNull::from(wasmer_instance.as_ref());
        env.set_wasmer_instance(Some(instance_ptr));
        env.set_gas_left(gas_limit);
        env.move_in(backend.storage, backend.querier);
        let instance = Instance {
            _inner: wasmer_instance,
            env,
            required_features,
        };
        Ok(instance)
    }

    pub fn api(&self) -> &A {
        &self.env.api
    }

    /// Decomposes this instance into its components.
    /// External dependencies are returned for reuse, the rest is dropped.
    pub fn recycle(self) -> Option<Backend<A, S, Q>> {
        if let (Some(storage), Some(querier)) = self.env.move_out() {
            let api = self.env.api;
            Some(Backend {
                api,
                storage,
                querier,
            })
        } else {
            None
        }
    }

    /// Returns the size of the default memory in pages.
    /// This provides a rough idea of the peak memory consumption. Note that
    /// Wasm memory always grows in 64 KiB steps (pages) and can never shrink
    /// (https://github.com/WebAssembly/design/issues/1300#issuecomment-573867836).
    pub fn memory_pages(&self) -> usize {
        self.env.memory().size().0 as _
    }

    /// Returns the currently remaining gas.
    pub fn get_gas_left(&self) -> u64 {
        self.env.get_gas_left()
    }

    /// Creates and returns a gas report.
    /// This is a snapshot and multiple reports can be created during the lifetime of
    /// an instance.
    pub fn create_gas_report(&self) -> GasReport {
        let state = self.env.with_gas_state(|gas_state| gas_state.clone());
        let gas_left = self.env.get_gas_left();
        GasReport {
            limit: state.gas_limit,
            remaining: gas_left,
            used_externally: state.externally_used_gas,
            // If externally_used_gas exceeds the gas limit, this will return 0.
            // no matter how much gas was used internally. But then we error with out of gas
            // anyways, and it does not matter much anymore where gas was spend.
            used_internally: state
                .gas_limit
                .saturating_sub(state.externally_used_gas)
                .saturating_sub(gas_left),
        }
    }

    /// Sets the readonly storage flag on this instance. Since one instance can be used
    /// for multiple calls in integration tests, this should be set to the desired value
    /// right before every call.
    pub fn set_storage_readonly(&mut self, new_value: bool) {
        self.env.set_storage_readonly(new_value);
    }

    pub fn with_storage<F: FnOnce(&mut S) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        self.env.with_storage_from_context::<F, T>(func)
    }

    pub fn with_querier<F: FnOnce(&mut Q) -> VmResult<T>, T>(&mut self, func: F) -> VmResult<T> {
        self.env.with_querier_from_context::<F, T>(func)
    }

    /// Requests memory allocation by the instance and returns a pointer
    /// in the Wasm address space to the created Region object.
    pub(crate) fn allocate(&mut self, size: usize) -> VmResult<u32> {
        let ret = self.call_function1("allocate", &[to_u32(size)?.into()])?;
        let ptr = ref_to_u32(&ret)?;
        if ptr == 0 {
            return Err(CommunicationError::zero_address().into());
        }
        Ok(ptr)
    }

    // deallocate frees memory in the instance and that was either previously
    // allocated by us, or a pointer from a return value after we copy it into rust.
    // we need to clean up the wasm-side buffers to avoid memory leaks
    pub(crate) fn deallocate(&mut self, ptr: u32) -> VmResult<()> {
        self.call_function0("deallocate", &[ptr.into()])?;
        Ok(())
    }

    /// Copies all data described by the Region at the given pointer from Wasm to the caller.
    pub(crate) fn read_memory(&self, region_ptr: u32, max_length: usize) -> VmResult<Vec<u8>> {
        read_region(&self.env.memory(), region_ptr, max_length)
    }

    /// Copies data to the memory region that was created before using allocate.
    pub(crate) fn write_memory(&mut self, region_ptr: u32, data: &[u8]) -> VmResult<()> {
        write_region(&self.env.memory(), region_ptr, data)?;
        Ok(())
    }

    /// Calls a function exported by the instance.
    /// The function is expected to return no value. Otherwise this calls errors.
    pub(crate) fn call_function0(&self, name: &str, args: &[Val]) -> VmResult<()> {
        self.env.call_function0(name, args)
    }

    /// Calls a function exported by the instance.
    /// The function is expected to return one value. Otherwise this calls errors.
    pub(crate) fn call_function1(&self, name: &str, args: &[Val]) -> VmResult<Val> {
        self.env.call_function1(name, args)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::backend::Storage;
    use crate::call_instantiate;
    use crate::errors::VmError;
    use crate::testing::{
        mock_backend, mock_env, mock_info, mock_instance, mock_instance_options,
        mock_instance_with_balances, mock_instance_with_failing_api, mock_instance_with_gas_limit,
        mock_instance_with_options, MockInstanceOptions,
    };
    use cosmwasm_std::{
        coin, coins, from_binary, AllBalanceResponse, BalanceResponse, BankQuery, Empty,
        QueryRequest,
    };

    const KIB: usize = 1024;
    const MIB: usize = 1024 * 1024;
    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;
    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    #[test]
    fn required_features_works() {
        let backend = mock_backend(&[]);
        let (instance_options, memory_limit) = mock_instance_options();
        let instance =
            Instance::from_code(CONTRACT, backend, instance_options, memory_limit).unwrap();
        assert_eq!(instance.required_features.len(), 0);
    }

    #[test]
    fn required_features_works_for_many_exports() {
        let wasm = wat::parse_str(
            r#"(module
            (type (func))
            (func (type 0) nop)
            (export "requires_water" (func 0))
            (export "requires_" (func 0))
            (export "requires_nutrients" (func 0))
            (export "require_milk" (func 0))
            (export "REQUIRES_air" (func 0))
            (export "requires_sun" (func 0))
            )"#,
        )
        .unwrap();

        let backend = mock_backend(&[]);
        let (instance_options, memory_limit) = mock_instance_options();
        let instance = Instance::from_code(&wasm, backend, instance_options, memory_limit).unwrap();
        assert_eq!(instance.required_features.len(), 3);
        assert!(instance.required_features.contains("nutrients"));
        assert!(instance.required_features.contains("sun"));
        assert!(instance.required_features.contains("water"));
    }

    #[test]
    fn call_function0_works() {
        let instance = mock_instance(&CONTRACT, &[]);

        instance
            .call_function0("interface_version_5", &[])
            .expect("error calling function");
    }

    #[test]
    fn call_function1_works() {
        let instance = mock_instance(&CONTRACT, &[]);

        // can call function few times
        let result = instance
            .call_function1("allocate", &[0u32.into()])
            .expect("error calling allocate");
        assert_ne!(result.unwrap_i32(), 0);

        let result = instance
            .call_function1("allocate", &[1u32.into()])
            .expect("error calling allocate");
        assert_ne!(result.unwrap_i32(), 0);

        let result = instance
            .call_function1("allocate", &[33u32.into()])
            .expect("error calling allocate");
        assert_ne!(result.unwrap_i32(), 0);
    }

    #[test]
    fn allocate_deallocate_works() {
        let mut instance = mock_instance_with_options(
            &CONTRACT,
            MockInstanceOptions {
                memory_limit: Some(Size::mebi(500)),
                ..Default::default()
            },
        );

        let sizes: Vec<usize> = vec![
            0,
            4,
            40,
            400,
            4 * KIB,
            40 * KIB,
            400 * KIB,
            4 * MIB,
            40 * MIB,
            400 * MIB,
        ];
        for size in sizes.into_iter() {
            let region_ptr = instance.allocate(size).expect("error allocating");
            instance.deallocate(region_ptr).expect("error deallocating");
        }
    }

    #[test]
    fn write_and_read_memory_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        let sizes: Vec<usize> = vec![
            0,
            4,
            40,
            400,
            4 * KIB,
            40 * KIB,
            400 * KIB,
            4 * MIB,
            // disabled for performance reasons, but pass as well
            // 40 * MIB,
            // 400 * MIB,
        ];
        for size in sizes.into_iter() {
            let region_ptr = instance.allocate(size).expect("error allocating");
            let original = vec![170u8; size];
            instance
                .write_memory(region_ptr, &original)
                .expect("error writing");
            let data = instance
                .read_memory(region_ptr, size)
                .expect("error reading");
            assert_eq!(data, original);
            instance.deallocate(region_ptr).expect("error deallocating");
        }
    }

    #[test]
    fn errors_in_imports() {
        // set up an instance that will experience an error in an import
        let error_message = "Api failed intentionally";
        let mut instance = mock_instance_with_failing_api(&CONTRACT, &[], error_message);
        let init_result = call_instantiate::<_, _, _, serde_json::Value>(
            &mut instance,
            &mock_env(),
            &mock_info("someone", &[]),
            b"{\"verifier\": \"some1\", \"beneficiary\": \"some2\"}",
        );

        match init_result.unwrap_err() {
            VmError::RuntimeErr { msg, .. } => assert!(msg.contains(error_message)),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn read_memory_errors_when_when_length_is_too_long() {
        let length = 6;
        let max_length = 5;
        let mut instance = mock_instance(&CONTRACT, &[]);

        // Allocate sets length to 0. Write some data to increase length.
        let region_ptr = instance.allocate(length).expect("error allocating");
        let data = vec![170u8; length];
        instance
            .write_memory(region_ptr, &data)
            .expect("error writing");

        let result = instance.read_memory(region_ptr, max_length);
        match result.unwrap_err() {
            VmError::CommunicationErr {
                source:
                    CommunicationError::RegionLengthTooBig {
                        length, max_length, ..
                    },
                ..
            } => {
                assert_eq!(length, 6);
                assert_eq!(max_length, 5);
            }
            err => panic!("unexpected error: {:?}", err),
        };

        instance.deallocate(region_ptr).expect("error deallocating");
    }

    #[test]
    fn memory_pages_returns_min_memory_size_by_default() {
        // min: 0 pages, max: none
        let wasm = wat::parse_str(
            r#"(module
                (memory 0)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_5" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
            )"#,
        )
        .unwrap();
        let instance = mock_instance(&wasm, &[]);
        assert_eq!(instance.memory_pages(), 0);

        // min: 3 pages, max: none
        let wasm = wat::parse_str(
            r#"(module
                (memory 3)
                (export "memory" (memory 0))

                (type (func))
                (func (type 0) nop)
                (export "interface_version_5" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
            )"#,
        )
        .unwrap();
        let instance = mock_instance(&wasm, &[]);
        assert_eq!(instance.memory_pages(), 3);
    }

    #[test]
    fn memory_pages_grows_with_usage() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        assert_eq!(instance.memory_pages(), 17);

        // 100 KiB require two more pages
        let region_ptr = instance.allocate(100 * 1024).expect("error allocating");

        assert_eq!(instance.memory_pages(), 19);

        // Deallocating does not shrink memory
        instance.deallocate(region_ptr).expect("error deallocating");
        assert_eq!(instance.memory_pages(), 19);
    }

    #[test]
    fn get_gas_left_works() {
        let instance = mock_instance_with_gas_limit(&CONTRACT, 123321);
        let orig_gas = instance.get_gas_left();
        assert_eq!(orig_gas, 123321);
    }

    #[test]
    fn create_gas_report_works() {
        const LIMIT: u64 = 7_000_000;
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, LIMIT);

        let report1 = instance.create_gas_report();
        assert_eq!(report1.used_externally, 0);
        assert_eq!(report1.used_internally, 0);
        assert_eq!(report1.limit, LIMIT);
        assert_eq!(report1.remaining, LIMIT);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let report2 = instance.create_gas_report();
        assert_eq!(report2.used_externally, 73);
        assert_eq!(report2.used_internally, 38804);
        assert_eq!(report2.limit, LIMIT);
        assert_eq!(
            report2.remaining,
            LIMIT - report2.used_externally - report2.used_internally
        );
    }

    #[test]
    fn set_storage_readonly_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        assert_eq!(instance.env.is_storage_readonly(), true);

        instance.set_storage_readonly(false);
        assert_eq!(instance.env.is_storage_readonly(), false);

        instance.set_storage_readonly(false);
        assert_eq!(instance.env.is_storage_readonly(), false);

        instance.set_storage_readonly(true);
        assert_eq!(instance.env.is_storage_readonly(), true);
    }

    #[test]
    fn with_storage_works() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // initial check
        instance
            .with_storage(|store| {
                assert!(store.get(b"foo").0.unwrap().is_none());
                Ok(())
            })
            .unwrap();

        // write some data
        instance
            .with_storage(|store| {
                store.set(b"foo", b"bar").0.unwrap();
                Ok(())
            })
            .unwrap();

        // read some data
        instance
            .with_storage(|store| {
                assert_eq!(store.get(b"foo").0.unwrap(), Some(b"bar".to_vec()));
                Ok(())
            })
            .unwrap();
    }

    #[test]
    #[should_panic]
    fn with_storage_safe_for_panic() {
        // this should fail with the assertion, but not cause a double-free crash (issue #59)
        let mut instance = mock_instance(&CONTRACT, &[]);
        instance
            .with_storage::<_, ()>(|_store| panic!("trigger failure"))
            .unwrap();
    }

    #[test]
    fn with_querier_works_readonly() {
        let rich_addr = String::from("foobar");
        let rich_balance = vec![coin(10000, "gold"), coin(8000, "silver")];
        let mut instance = mock_instance_with_balances(&CONTRACT, &[(&rich_addr, &rich_balance)]);

        // query one
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::Balance {
                            address: rich_addr.clone(),
                            denom: "silver".to_string(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let BalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.amount.u128(), 8000);
                assert_eq!(amount.denom, "silver");
                Ok(())
            })
            .unwrap();

        // query all
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::AllBalances {
                            address: rich_addr.clone(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let AllBalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.len(), 2);
                assert_eq!(amount[0].amount.u128(), 10000);
                assert_eq!(amount[0].denom, "gold");
                assert_eq!(amount[1].amount.u128(), 8000);
                assert_eq!(amount[1].denom, "silver");

                Ok(())
            })
            .unwrap();
    }

    /// This is needed for writing intagration tests in which the balance of a contract changes over time
    #[test]
    fn with_querier_allows_updating_balances() {
        let rich_addr = String::from("foobar");
        let rich_balance1 = vec![coin(10000, "gold"), coin(500, "silver")];
        let rich_balance2 = vec![coin(10000, "gold"), coin(8000, "silver")];
        let mut instance = mock_instance_with_balances(&CONTRACT, &[(&rich_addr, &rich_balance1)]);

        // Get initial state
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::Balance {
                            address: rich_addr.clone(),
                            denom: "silver".to_string(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let BalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.amount.u128(), 500);
                Ok(())
            })
            .unwrap();

        // Update balance
        instance
            .with_querier(|querier| {
                querier.update_balance(&rich_addr, rich_balance2);
                Ok(())
            })
            .unwrap();

        // Get updated state
        instance
            .with_querier(|querier| {
                let response = querier
                    .query::<Empty>(
                        &QueryRequest::Bank(BankQuery::Balance {
                            address: rich_addr.clone(),
                            denom: "silver".to_string(),
                        }),
                        DEFAULT_QUERY_GAS_LIMIT,
                    )
                    .0
                    .unwrap()
                    .unwrap()
                    .unwrap();
                let BalanceResponse { amount } = from_binary(&response).unwrap();
                assert_eq!(amount.amount.u128(), 8000);
                Ok(())
            })
            .unwrap();
    }
}

#[cfg(test)]
mod singlepass_tests {
    use cosmwasm_std::{coins, Empty};

    use crate::calls::{call_execute, call_instantiate, call_query};
    use crate::testing::{mock_env, mock_info, mock_instance, mock_instance_with_gas_limit};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

    #[test]
    fn contract_deducts_gas_init() {
        let mut instance = mock_instance(&CONTRACT, &[]);
        let orig_gas = instance.get_gas_left();

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let init_used = orig_gas - instance.get_gas_left();
        assert_eq!(init_used, 38877);
    }

    #[test]
    fn contract_deducts_gas_execute() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_execute = instance.get_gas_left();
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        let execute_used = gas_before_execute - instance.get_gas_left();
        assert_eq!(execute_used, 157981);
    }

    #[test]
    fn contract_enforces_gas_limit() {
        let mut instance = mock_instance_with_gas_limit(&CONTRACT, 20_000);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let res = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg);
        assert!(res.is_err());
    }

    #[test]
    fn query_works_with_gas_metering() {
        let mut instance = mock_instance(&CONTRACT, &[]);

        // init contract
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let _res = call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // run contract - just sanity check - results validate in contract unit tests
        let gas_before_query = instance.get_gas_left();
        // we need to encode the key in base64
        let msg = br#"{"verifier":{}}"#;
        let res = call_query(&mut instance, &mock_env(), msg).unwrap();
        let answer = res.unwrap();
        assert_eq!(answer.as_slice(), b"{\"verifier\":\"verifies\"}");

        let query_used = gas_before_query - instance.get_gas_left();
        assert_eq!(query_used, 28999);
    }
}
