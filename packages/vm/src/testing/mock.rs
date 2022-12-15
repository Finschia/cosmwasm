use cosmwasm_std::testing::{digit_sum, riffle_shuffle};
use cosmwasm_std::{
    Addr, BlockInfo, Coin, ContractInfo, Env, MessageInfo, Timestamp, TransactionInfo,
};
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::RwLock;
use std::thread_local;
use wasmer::Module;

use super::querier::MockQuerier;
use super::storage::MockStorage;
use crate::environment::Environment;
use crate::instance::Instance;
use crate::{
    copy_region_vals_between_env, Backend, BackendApi, BackendError, BackendResult, GasInfo,
    Querier, Storage,
};
use crate::{FunctionMetadata, WasmerVal};

pub const MOCK_CONTRACT_ADDR: &str = "cosmos2contract";
const DEFAULT_GAS_COST_HUMANIZE: u64 = 44;
const DEFAULT_GAS_COST_CANONICALIZE: u64 = 55;
const MAX_REGIONS_LENGTH: usize = 64 * 1024 * 1024;

/// All external requirements that can be injected for unit tests.
/// It sets the given balance for the contract itself, nothing else
pub fn mock_backend(contract_balance: &[Coin]) -> Backend<MockApi, MockStorage, MockQuerier> {
    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: MockQuerier::new(&[(MOCK_CONTRACT_ADDR, contract_balance)]),
    }
}

/// Initializes the querier along with the mock_dependencies.
/// Sets all balances provided (yoy must explicitly set contract balance if desired)
pub fn mock_backend_with_balances(
    balances: &[(&str, &[Coin])],
) -> Backend<MockApi, MockStorage, MockQuerier> {
    Backend {
        api: MockApi::default(),
        storage: MockStorage::default(),
        querier: MockQuerier::new(balances),
    }
}

type MockInstance = Instance<MockApi, MockStorage, MockQuerier>;
thread_local! {
    // INSTANCE_CACHE and MODULE_CACHE are intended to replace wasmvm's cache layer in the mock.
    // Unlike wasmvm, you have to initialize them yourself in the place where you test the dynamic call.
    pub static INSTANCE_CACHE: RwLock<HashMap<String, RefCell<MockInstance>>> = RwLock::new(HashMap::new());
    pub static MODULE_CACHE: RwLock<HashMap<String, RefCell<Module>>> = RwLock::new(HashMap::new());
}
/// Length of canonical addresses created with this API. Contracts should not make any assumtions
/// what this value is.
/// The value here must be restorable with `SHUFFLES_ENCODE` + `SHUFFLES_DECODE` in-shuffles.
const CANONICAL_LENGTH: usize = 54;

const SHUFFLES_ENCODE: usize = 18;
const SHUFFLES_DECODE: usize = 2;

/// Zero-pads all human addresses to make them fit the canonical_length and
/// trims off zeros for the reverse operation.
/// This is not really smart, but allows us to see a difference (and consistent length for canonical adddresses).
#[derive(Copy, Clone)]
pub struct MockApi {
    /// Length of canonical addresses created with this API. Contracts should not make any assumtions
    /// what this value is.
    canonical_length: usize,
    /// `canonicalize_cost` is consumed gas value when the contract all api `canonical_address`
    canonicalize_cost: u64,
    /// `humanize_cost` is consumed gas value when the contract all api `human_address`
    humanize_cost: u64,
    /// When set, all calls to the API fail with BackendError::Unknown containing this message
    backend_error: Option<&'static str>,
}

impl MockApi {
    /// create a `MockApi` instance with specified gas costs to call api
    pub fn new_with_gas_cost(canonicalize_cost: u64, humanize_cost: u64) -> Self {
        MockApi {
            canonicalize_cost,
            humanize_cost,
            ..MockApi::default()
        }
    }

    /// Read-only getter for `canonical_length`, which must not be changed by the caller.
    pub fn canonical_length(&self) -> usize {
        self.canonical_length
    }

    /// Read-only getter for `canonical_length`, which must not be changed by the caller.
    pub fn canonicalize_cost(&self) -> u64 {
        self.canonicalize_cost
    }

    /// Read-only getter for `canonical_length`, which must not be changed by the caller.
    pub fn humanize_cost(&self) -> u64 {
        self.humanize_cost
    }

    pub fn new_failing(backend_error: &'static str) -> Self {
        MockApi {
            backend_error: Some(backend_error),
            ..MockApi::default()
        }
    }
}

impl Default for MockApi {
    fn default() -> Self {
        MockApi {
            canonical_length: CANONICAL_LENGTH,
            canonicalize_cost: DEFAULT_GAS_COST_CANONICALIZE,
            humanize_cost: DEFAULT_GAS_COST_HUMANIZE,
            backend_error: None,
        }
    }
}

impl BackendApi for MockApi {
    fn canonical_address(&self, input: &str) -> BackendResult<Vec<u8>> {
        // mimicks formats like hex or bech32 where different casings are valid for one address
        let normalized = input.to_lowercase();

        let gas_info = GasInfo::with_cost(self.canonicalize_cost);

        if let Some(backend_error) = self.backend_error {
            return (Err(BackendError::unknown(backend_error)), gas_info);
        }

        // Dummy input validation. This is more sophisticated for formats like bech32, where format and checksum are validated.
        if normalized.len() < 3 {
            return (
                Err(BackendError::user_err(
                    "Invalid input: human address too short",
                )),
                gas_info,
            );
        }
        if normalized.len() > self.canonical_length {
            return (
                Err(BackendError::user_err(
                    "Invalid input: human address too long",
                )),
                gas_info,
            );
        }

        let mut out = Vec::from(normalized);
        // pad to canonical length with NULL bytes
        out.resize(self.canonical_length, 0x00);
        // content-dependent rotate followed by shuffle to destroy
        // the most obvious structure (https://github.com/CosmWasm/cosmwasm/issues/552)
        let rotate_by = digit_sum(&out) % self.canonical_length;
        out.rotate_left(rotate_by);
        for _ in 0..SHUFFLES_ENCODE {
            out = riffle_shuffle(&out);
        }
        (Ok(out), gas_info)
    }

    fn human_address(&self, canonical: &[u8]) -> BackendResult<String> {
        let gas_info = GasInfo::with_cost(self.humanize_cost);

        if let Some(backend_error) = self.backend_error {
            return (Err(BackendError::unknown(backend_error)), gas_info);
        }

        if canonical.len() != self.canonical_length {
            return (
                Err(BackendError::user_err(
                    "Invalid input: canonical address length not correct",
                )),
                gas_info,
            );
        }

        let mut tmp: Vec<u8> = canonical.into();
        // Shuffle two more times which restored the original value (24 elements are back to original after 20 rounds)
        for _ in 0..SHUFFLES_DECODE {
            tmp = riffle_shuffle(&tmp);
        }
        // Rotate back
        let rotate_by = digit_sum(&tmp) % self.canonical_length;
        tmp.rotate_right(rotate_by);
        // Remove NULL bytes (i.e. the padding)
        let trimmed = tmp.into_iter().filter(|&x| x != 0x00).collect();

        let result = match String::from_utf8(trimmed) {
            Ok(human) => Ok(human),
            Err(err) => Err(err.into()),
        };
        (result, gas_info)
    }

    fn contract_call<A, S, Q>(
        &self,
        caller_env: &Environment<A, S, Q>,
        contract_addr: &str,
        func_info: &FunctionMetadata,
        args: &[WasmerVal],
    ) -> BackendResult<Box<[WasmerVal]>>
    where
        A: BackendApi + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
    {
        let mut gas_info = GasInfo::new(0, 0);
        INSTANCE_CACHE.with(|lock| {
            let cache = lock.read().unwrap();
            match cache.get(contract_addr) {
                Some(callee_instance_cell) => {
                    let callee_instance = callee_instance_cell.borrow_mut();

                    let arg_region_ptrs = copy_region_vals_between_env(
                        caller_env,
                        &callee_instance.env,
                        args,
                        MAX_REGIONS_LENGTH,
                        false,
                    )
                    .unwrap();
                    let call_ret = match callee_instance.call_function_strict(
                        &func_info.signature,
                        &func_info.name,
                        &arg_region_ptrs,
                    ) {
                        Ok(rets) => Ok(copy_region_vals_between_env(
                            &callee_instance.env,
                            caller_env,
                            &rets,
                            MAX_REGIONS_LENGTH,
                            true,
                        )
                        .unwrap()),
                        Err(e) => Err(BackendError::dynamic_link_err(e.to_string())),
                    };
                    gas_info.cost += callee_instance.create_gas_report().used_internally;
                    (call_ret, gas_info)
                }
                None => (
                    Err(BackendError::dynamic_link_err("cannot found contract")),
                    gas_info,
                ),
            }
        })
    }

    fn get_wasmer_module(&self, contract_addr: &str) -> BackendResult<Module> {
        let gas_info = GasInfo::new(0, 0);
        MODULE_CACHE.with(|lock| {
            let cache = lock.read().unwrap();
            match cache.get(contract_addr) {
                Some(module) => (Ok(module.borrow().clone()), gas_info),
                None => (
                    Err(BackendError::dynamic_link_err("cannot found checksum")),
                    gas_info,
                ),
            }
        })
    }
}

/// Returns a default enviroment with height, time, chain_id, and contract address
/// You can submit as is to most contracts, or modify height/time if you want to
/// test for expiration.
///
/// This is intended for use in test code only.
pub fn mock_env() -> Env {
    Env {
        block: BlockInfo {
            height: 12_345,
            time: Timestamp::from_nanos(1_571_797_419_879_305_533),
            chain_id: "cosmos-testnet-14002".to_string(),
        },
        transaction: Some(TransactionInfo { index: 3 }),
        contract: ContractInfo {
            address: Addr::unchecked(MOCK_CONTRACT_ADDR),
        },
    }
}

/// Just set sender and sent funds for the message. The essential for
/// This is intended for use in test code only.
pub fn mock_info(sender: &str, funds: &[Coin]) -> MessageInfo {
    MessageInfo {
        sender: Addr::unchecked(sender),
        funds: funds.to_vec(),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::BackendError;
    use cosmwasm_std::coins;

    #[test]
    fn mock_info_works() {
        let info = mock_info("my name", &coins(100, "atom"));
        assert_eq!(
            info,
            MessageInfo {
                sender: Addr::unchecked("my name"),
                funds: vec![Coin {
                    amount: 100u128.into(),
                    denom: "atom".into(),
                }]
            }
        );
    }

    #[test]
    fn canonical_address_works() {
        let api = MockApi::default();

        api.canonical_address("foobar123").0.unwrap();

        // is case insensitive
        let data1 = api.canonical_address("foo123").0.unwrap();
        let data2 = api.canonical_address("FOO123").0.unwrap();
        assert_eq!(data1, data2);
    }

    #[test]
    fn canonicalize_and_humanize_restores_original() {
        let api = MockApi::default();

        // simple
        let original = "shorty";
        let canonical = api.canonical_address(original).0.unwrap();
        let (recovered, _gas_cost) = api.human_address(&canonical);
        assert_eq!(recovered.unwrap(), original);

        // normalizes input
        let original = String::from("CosmWasmChef");
        let canonical = api.canonical_address(&original).0.unwrap();
        let recovered = api.human_address(&canonical).0.unwrap();
        assert_eq!(recovered, "cosmwasmchef");
    }

    #[test]
    fn human_address_input_length() {
        let api = MockApi::default();
        let input = vec![61; 11];
        let (result, _gas_info) = api.human_address(&input);
        match result.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn canonical_address_min_input_length() {
        let api = MockApi::default();
        let human = "1";
        match api.canonical_address(human).0.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn canonical_address_max_input_length() {
        let api = MockApi::default();
        let human = "longer-than-the-address-length-supported-by-this-api-longer-than-54";
        match api.canonical_address(human).0.unwrap_err() {
            BackendError::UserErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn test_default_gas_cost() {
        let api = MockApi::default();

        let original = "alice";
        let (canonical_res, gas_cost) = api.canonical_address(original);
        assert_eq!(gas_cost.cost, DEFAULT_GAS_COST_CANONICALIZE);
        assert_eq!(gas_cost.externally_used, 0);
        let canonical = canonical_res.unwrap();
        let (_recovered, gas_cost) = api.human_address(&canonical);
        assert_eq!(gas_cost.cost, DEFAULT_GAS_COST_HUMANIZE);
        assert_eq!(gas_cost.externally_used, 0);
    }

    #[test]
    fn test_specified_gas_cost() {
        let canonicalize_cost: u64 = 42;
        let humanize_cost: u64 = 101010;
        assert_ne!(canonicalize_cost, DEFAULT_GAS_COST_CANONICALIZE);
        assert_ne!(humanize_cost, DEFAULT_GAS_COST_HUMANIZE);
        let api = MockApi::new_with_gas_cost(canonicalize_cost, humanize_cost);

        let original = "bob";
        let (canonical_res, gas_cost) = api.canonical_address(original);
        assert_eq!(gas_cost.cost, canonicalize_cost);
        assert_eq!(gas_cost.externally_used, 0);
        let canonical = canonical_res.unwrap();
        let (_recovered, gas_cost) = api.human_address(&canonical);
        assert_eq!(gas_cost.cost, humanize_cost);
        assert_eq!(gas_cost.externally_used, 0);
    }
}
