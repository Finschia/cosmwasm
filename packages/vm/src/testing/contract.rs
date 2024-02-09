use wasmer::{Engine, Module, Store};

use crate::backend::{Backend, Storage};
use crate::compatibility::check_wasm;
use crate::instance::Instance;
use crate::size::Size;
use crate::wasm_backend::{compile, make_compiling_engine};

use super::instance::MockInstanceOptions;
use super::mock::MockApi;
use super::querier::MockQuerier;
use super::result::{TestingError, TestingResult};
use super::storage::MockStorage;

/// This is Contract type for testing.
///
/// the engine and module need to correspond one-to-one.
/// See: https://github.com/CosmWasm/cosmwasm/pull/1753
#[derive(Clone)]
pub struct Contract {
    engine: Engine,
    module: Module,
    storage: MockStorage,
    serialized_env: Vec<u8>,
}

/// representing a contract in integration test
///
/// This enables tests to instantiate a new instance every time,
/// they test call_(instantiate/execute/query/migrate),
/// similar to the actual behavior of wasmd.
/// This is like Cache but it is for single contract and cannot save data in disk.
impl Contract {
    pub fn from_code(
        wasm: &[u8],
        serialized_env: &[u8],
        options: &MockInstanceOptions,
        memory_limit: Option<Size>,
    ) -> TestingResult<Self> {
        check_wasm(wasm, &options.available_capabilities)?;
        let engine = make_compiling_engine(memory_limit);
        let module = compile(&engine, wasm)?;
        let storage = MockStorage::new();
        let contract = Self {
            engine,
            module,
            storage,
            serialized_env: serialized_env.to_vec(),
        };
        Ok(contract)
    }

    /// change the wasm code for testing migrate
    ///
    /// call this before `generate_instance` for testing `call_migrate`.
    ///
    /// the engine and module need to correspond one-to-one,
    /// and with changes in WASM, both the engine and module are being updated.
    pub fn change_wasm(
        &mut self,
        wasm: &[u8],
        options: &MockInstanceOptions,
        memory_limit: Option<Size>,
    ) -> TestingResult<()> {
        check_wasm(wasm, &options.available_capabilities)?;
        let engine = make_compiling_engine(memory_limit);
        let module = compile(&engine, wasm)?;
        self.engine = engine;
        self.module = module;
        Ok(())
    }

    /// generate instance for testing
    pub fn generate_instance(
        &self,
        api: MockApi,
        querier: MockQuerier,
        options: &MockInstanceOptions,
    ) -> TestingResult<Instance<MockApi, MockStorage, MockQuerier>> {
        let storage = self.storage.clone();
        let backend = Backend {
            api,
            storage,
            querier,
        };
        let store = Store::new(self.engine.clone());
        let mut instance = Instance::from_module(
            store,
            &self.module,
            backend,
            options.gas_limit,
            options.print_debug,
            None,
            None,
        )?;
        instance.set_serialized_env(&self.serialized_env);
        Ok(instance)
    }

    /// update storage via instance recycling
    pub fn update_storage(
        &mut self,
        instance: Instance<MockApi, MockStorage, MockQuerier>,
    ) -> TestingResult<()> {
        let backend = instance.recycle().ok_or_else(|| {
            TestingError::ContractError(
                "Cannot recycle the instance with cosmwasm_vm::Instance::recycle".to_string(),
            )
        })?;
        self.storage = backend.storage;
        Ok(())
    }

    /// get value from storage
    pub fn raw_get(&self, key: &[u8]) -> Option<Vec<u8>> {
        self.storage.get(key).0.unwrap()
    }

    /// get clone module
    pub fn module(&self) -> Module {
        self.module.clone()
    }
}

#[cfg(test)]
#[cfg(feature = "iterator")]
mod test {
    use super::*;
    use crate::calls::{call_execute, call_instantiate, call_migrate, call_query};
    use crate::testing::{mock_env, mock_info, MockInstanceOptions};
    use cosmwasm_std::{to_json_vec, QueryResponse, Response};

    static CONTRACT_WITHOUT_MIGRATE: &[u8] =
        include_bytes!("../../testdata/queue_1.4.0_without_migrate.wasm");
    static CONTRACT_WITH_MIGRATE: &[u8] =
        include_bytes!("../../testdata/queue_1.4.0_with_migrate.wasm");

    #[test]
    fn test_sanity_integration_test_flow() {
        let options = MockInstanceOptions::default();
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);

        // common env/info
        let env = mock_env();
        let info = mock_info("sender", &[]);

        let mut contract = Contract::from_code(
            CONTRACT_WITHOUT_MIGRATE,
            &to_json_vec(&env).unwrap(),
            &options,
            None,
        )
        .unwrap();

        // init
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{}".as_bytes();
        let _: Response = call_instantiate(&mut instance, &env, &info, msg)
            .unwrap()
            .into_result()
            .unwrap();
        contract.update_storage(instance).unwrap();

        // query and confirm the queue is empty
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{\"count\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"count\":0}".as_bytes());
        contract.update_storage(instance).unwrap();

        // handle and enqueue 42
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{\"enqueue\": {\"value\": 42}}".as_bytes();
        let _: Response = call_execute(&mut instance, &env, &info, msg)
            .unwrap()
            .into_result()
            .unwrap();
        contract.update_storage(instance).unwrap();

        // query and confirm the length of the queue is 1
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{\"count\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"count\":1}".as_bytes());
        contract.update_storage(instance).unwrap();

        // query and confirm the sum of the queue is 42
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{\"sum\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"sum\":42}".as_bytes());
        contract.update_storage(instance).unwrap();

        // change the code and migrate
        contract
            .change_wasm(CONTRACT_WITH_MIGRATE, &options, None)
            .unwrap();
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{}".as_bytes();
        let _: Response = call_migrate(&mut instance, &env, msg)
            .unwrap()
            .into_result()
            .unwrap();
        contract.update_storage(instance).unwrap();

        // query and check the length of the queue is 3
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{\"count\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"count\":3}".as_bytes());
        contract.update_storage(instance).unwrap();

        // query and check the sum of the queue is 303
        let api = MockApi::default();
        let querier = MockQuerier::new(&[]);
        let mut instance = contract.generate_instance(api, querier, &options).unwrap();
        let msg = "{\"sum\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"sum\":303}".as_bytes());
        contract.update_storage(instance).unwrap();
    }
}
