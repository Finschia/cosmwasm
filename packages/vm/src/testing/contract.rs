use wasmer::Module;

use crate::backend::Backend;
use crate::compatibility::check_wasm;
use crate::instance::Instance;
use crate::wasm_backend::compile;

use super::instance::MockInstanceOptions;
use super::mock::MockApi;
use super::querier::MockQuerier;
use super::result::{TestingError, TestingResult};
use super::storage::MockStorage;

pub struct Contract<'a> {
    module: Module,
    backend: Option<Backend<MockApi, MockStorage, MockQuerier>>,
    options: MockInstanceOptions<'a>,
}

const ERR_GENERATE_INSTANCE_TWICE: &str = "generate_instance is called twice without called recycle. After generate_instance in called, Contract::recycle needs to be called before generate_instance is called the next time.";
const ERR_RECYCLE_BEFORE_GENERATE_INSTANCE: &str = "recycle_instance is called before generate_instance. The parameter instance of the recycle_instance should be created with Contract::generate_instance of the same Contract.";

/// representing a contract in integration test
///
/// This enables tests instantiate a new instance every time testing call_(instantiate/execute/query/migrate) like actual wasmd's behavior.
/// This is like Cache but it is for single contract and cannot save data in disk.
impl<'a> Contract<'a> {
    pub fn from_code(
        wasm: &[u8],
        backend: Backend<MockApi, MockStorage, MockQuerier>,
        options: MockInstanceOptions<'a>,
    ) -> TestingResult<Self> {
        check_wasm(wasm, &options.supported_features)?;
        let module = compile(wasm, None)?;
        let backend = Some(backend);
        let contract = Self {
            module,
            backend,
            options,
        };
        Ok(contract)
    }

    /// change the wasm code for testing migrate
    ///
    /// call this before `generate_instance` for testing `call_migrate`.
    pub fn change_wasm(&mut self, wasm: &[u8]) -> TestingResult<()> {
        check_wasm(wasm, &self.options.supported_features)?;
        let module = compile(wasm, None)?;
        self.module = module;
        Ok(())
    }

    /// generate instance for testing
    ///
    /// once this is called, result instance needs to be recycled by Contract::recycle_instance to generate new instance next time.
    pub fn generate_instance(
        &mut self,
    ) -> TestingResult<Instance<MockApi, MockStorage, MockQuerier>> {
        let backend = self
            .backend
            .take()
            .ok_or_else(|| TestingError::ContractError(ERR_GENERATE_INSTANCE_TWICE.to_string()))?;
        let instance = Instance::from_module(
            &self.module,
            backend,
            self.options.gas_limit,
            self.options.print_debug,
        )?;
        Ok(instance)
    }

    /// recycle passed instance and take the ownership of the backend
    ///
    /// instance of a contract must be singleton and this is for recycling the instance.
    pub fn recycle_instance(
        &mut self,
        instance: Instance<MockApi, MockStorage, MockQuerier>,
    ) -> TestingResult<()> {
        if self.backend.is_some() {
            return Err(TestingError::ContractError(
                ERR_RECYCLE_BEFORE_GENERATE_INSTANCE.to_string(),
            ));
        };
        let backend = instance.recycle().ok_or_else(|| {
            TestingError::ContractError(
                "Cannot recycle the instance with cosmwasm_vm::Instance::recycle".to_string(),
            )
        })?;
        self.backend = Some(backend);
        Ok(())
    }

    /// change options
    pub fn change_options(&mut self, options: MockInstanceOptions<'a>) {
        self.options = options
    }
}

#[cfg(test)]
#[cfg(feature = "iterator")]
mod test {
    use super::*;
    use crate::calls::{call_execute, call_instantiate, call_migrate, call_query};
    use crate::testing::{mock_backend, mock_env, mock_info, mock_instance, MockInstanceOptions};
    use cosmwasm_std::{HandleResponse, HumanAddr, InitResponse, MigrateResponse, QueryResponse};

    static CONTRACT_WITHOUT_MIGRATE: &[u8] =
        include_bytes!("../../testdata/queue_0.14_without_migrate.wasm");
    static CONTRACT_WITH_MIGRATE: &[u8] =
        include_bytes!("../../testdata/queue_0.14_with_migrate.wasm");

    #[test]
    fn test_sanity_integration_test_flow() {
        let options = MockInstanceOptions::default();
        let backend = mock_backend(&[]);
        let mut contract = Contract::from_code(CONTRACT_WITHOUT_MIGRATE, backend, options).unwrap();

        // common env/info
        let env = mock_env();
        let info = mock_info(HumanAddr::from("sender"), &[]);

        // init
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{}".as_bytes();
        let _: InitResponse = call_instantiate(&mut instance, &env, &info, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        let _ = contract.recycle_instance(instance).unwrap();

        // query and confirm the queue is empty
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{\"count\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"count\":0}".as_bytes());
        let _ = contract.recycle_instance(instance).unwrap();

        // handle and enqueue 42
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{\"enqueue\": {\"value\": 42}}".as_bytes();
        let _: HandleResponse = call_execute(&mut instance, &env, &info, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        let _ = contract.recycle_instance(instance).unwrap();

        // query and confirm the length of the queue is 1
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{\"count\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"count\":1}".as_bytes());
        let _ = contract.recycle_instance(instance).unwrap();

        // query and confirm the sum of the queue is 42
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{\"sum\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"sum\":42}".as_bytes());
        let _ = contract.recycle_instance(instance).unwrap();

        // change the code and migrate
        contract.change_wasm(CONTRACT_WITH_MIGRATE).unwrap();
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{}".as_bytes();
        let _: MigrateResponse = call_migrate(&mut instance, &env, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        let _ = contract.recycle_instance(instance).unwrap();

        // query and check the length of the queue is 3
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{\"count\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"count\":3}".as_bytes());
        let _ = contract.recycle_instance(instance).unwrap();

        // query and check the sum of the queue is 303
        let mut instance = contract.generate_instance().unwrap();
        let msg = "{\"sum\": {}}".as_bytes();
        let res: QueryResponse = call_query(&mut instance, &env, &msg)
            .unwrap()
            .into_result()
            .unwrap();
        assert_eq!(res, "{\"sum\":303}".as_bytes());
        let _ = contract.recycle_instance(instance).unwrap();
    }

    #[test]
    #[should_panic(expected = "generate_instance is called twice")]
    fn test_err_call_generate_instance_twice() {
        let options = MockInstanceOptions::default();
        let backend = mock_backend(&[]);
        let mut contract = Contract::from_code(CONTRACT_WITHOUT_MIGRATE, backend, options).unwrap();

        // generate_instance
        let _instance = contract.generate_instance().unwrap();

        // should panic when call generate_instance before recycle
        contract.generate_instance().unwrap();
    }

    #[test]
    #[should_panic(expected = "recycle_instance is called before generate_instance")]
    fn test_err_call_recycle_before_generate_instance() {
        let options = MockInstanceOptions::default();
        let backend = mock_backend(&[]);
        let mut contract = Contract::from_code(CONTRACT_WITHOUT_MIGRATE, backend, options).unwrap();

        // make a dummy instance
        let dummy_instance = mock_instance(CONTRACT_WITHOUT_MIGRATE, &[]);

        // should panic when call recycle before generate_instance
        contract.recycle_instance(dummy_instance).unwrap();
    }
}
