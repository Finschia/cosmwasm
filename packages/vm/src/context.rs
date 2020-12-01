//! Internal details to be used by instance.rs only

use std::ffi::c_void;
use std::ptr::NonNull;

use wasmer_runtime_core::{
    typed_func::{Func, Wasm, WasmTypeList},
    vm::Ctx,
    Instance as WasmerInstance,
};

use crate::backend::{GasInfo, Querier, Storage};
use crate::backends::decrease_gas_left;
use crate::errors::{VmError, VmResult};

/** context data **/

#[derive(Clone, PartialEq, Debug, Default)]
pub struct GasState {
    /// Gas limit for the computation.
    pub gas_limit: u64,
    /// Tracking the gas used in the cosmos SDK, in cosmwasm units.
    #[allow(unused)]
    pub externally_used_gas: u64,
}

impl GasState {
    fn with_limit(gas_limit: u64) -> Self {
        Self {
            gas_limit,
            externally_used_gas: 0,
        }
    }

    #[allow(unused)]
    fn increase_externally_used_gas(&mut self, amount: u64) {
        self.externally_used_gas += amount;
    }

    pub(crate) fn set_gas_limit(&mut self, gas_limit: u64) {
        self.gas_limit = gas_limit;
    }

    /// Get the amount of gas units still left for the rest of the calculation.
    ///
    /// We need the amount of gas used in wasmer since it is not tracked inside this object.
    #[allow(unused)]
    fn get_gas_left(&self, wasmer_used_gas: u64) -> u64 {
        self.gas_limit
            .saturating_sub(self.externally_used_gas)
            .saturating_sub(wasmer_used_gas)
    }

    /// Get the amount of gas units used so far inside wasmer.
    ///
    /// We need the amount of gas left in wasmer since it is not tracked inside this object.
    #[allow(unused)]
    pub(crate) fn get_gas_used_in_wasmer(&self, wasmer_gas_left: u64) -> u64 {
        self.gas_limit
            .saturating_sub(self.externally_used_gas)
            .saturating_sub(wasmer_gas_left)
    }
}

struct ContextData<S: Storage, Q: Querier> {
    gas_state: GasState,
    storage: Option<S>,
    storage_readonly: bool,
    querier: Option<Q>,
    /// A non-owning link to the wasmer instance
    wasmer_instance: Option<NonNull<WasmerInstance>>,
}

pub fn setup_context<S: Storage, Q: Querier>(gas_limit: u64) -> (*mut c_void, fn(*mut c_void)) {
    (
        create_unmanaged_context_data::<S, Q>(gas_limit),
        destroy_unmanaged_context_data::<S, Q>,
    )
}

fn create_unmanaged_context_data<S: Storage, Q: Querier>(gas_limit: u64) -> *mut c_void {
    let data = ContextData::<S, Q> {
        gas_state: GasState::with_limit(gas_limit),
        storage: None,
        storage_readonly: true,
        querier: None,
        wasmer_instance: None,
    };
    let heap_data = Box::new(data); // move from stack to heap
    Box::into_raw(heap_data) as *mut c_void // give up ownership
}

fn destroy_unmanaged_context_data<S: Storage, Q: Querier>(ptr: *mut c_void) {
    if !ptr.is_null() {
        // obtain ownership and drop instance of ContextData when box gets out of scope
        let _dying = unsafe { Box::from_raw(ptr as *mut ContextData<S, Q>) };
    }
}

/// Get a mutable reference to the context's data. Ownership remains in the Context.
fn get_context_data_mut<'a, 'b, S: Storage, Q: Querier>(
    ctx: &'a mut Ctx,
) -> &'b mut ContextData<S, Q> {
    unsafe {
        let ptr = ctx.data as *mut ContextData<S, Q>;
        ptr.as_mut()
            .expect("The pointer ctx.data was null in get_context_data_mut; this is a bug.")
    }
}

fn get_context_data<'a, 'b, S: Storage, Q: Querier>(ctx: &'a Ctx) -> &'b ContextData<S, Q> {
    unsafe {
        let ptr = ctx.data as *mut ContextData<S, Q>;
        ptr.as_ref()
            .expect("The pointer ctx.data was null in get_context_data; this is a bug.")
    }
}

/// Creates a back reference from a contact to its partent instance
pub fn set_wasmer_instance<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    wasmer_instance: Option<NonNull<WasmerInstance>>,
) {
    let context_data = ctx.data as *mut ContextData<S, Q>;
    unsafe {
        (*context_data).wasmer_instance = wasmer_instance;
    }
}

/// Returns the original storage and querier as owned instances, and closes any remaining
/// iterators. This is meant to be called when recycling the instance.
pub(crate) fn move_out_of_context<S: Storage, Q: Querier>(
    source: &mut Ctx,
) -> (Option<S>, Option<Q>) {
    let b = get_context_data_mut::<S, Q>(source);
    (b.storage.take(), b.querier.take())
}

/// Moves owned instances of storage and querier into the context.
/// Should be followed by exactly one call to move_out_of_context when the instance is finished.
pub(crate) fn move_into_context<S: Storage, Q: Querier>(target: &mut Ctx, storage: S, querier: Q) {
    let b = get_context_data_mut::<S, Q>(target);
    b.storage = Some(storage);
    b.querier = Some(querier);
}

pub fn get_gas_state_mut<'a, 'b, S: Storage + 'b, Q: Querier + 'b>(
    ctx: &'a mut Ctx,
) -> &'b mut GasState {
    &mut get_context_data_mut::<S, Q>(ctx).gas_state
}

pub fn get_gas_state<'a, 'b, S: Storage + 'b, Q: Querier + 'b>(ctx: &'a Ctx) -> &'b GasState {
    &get_context_data::<S, Q>(ctx).gas_state
}

pub fn process_gas_info<S: Storage, Q: Querier>(ctx: &mut Ctx, info: GasInfo) -> VmResult<()> {
    decrease_gas_left(ctx, info.cost)?;
    account_for_externally_used_gas::<S, Q>(ctx, info.externally_used)?;
    Ok(())
}

/// Use this function to adjust the VM's gas limit when a call into the backend
/// reported there was externally metered gas used.
/// This does not increase the VM's gas usage but ensures the overall limit is not exceeded.
fn account_for_externally_used_gas<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    amount: u64,
) -> VmResult<()> {
    account_for_externally_used_gas_impl::<S, Q>(ctx, amount)
}

#[cfg(feature = "default-singlepass")]
fn account_for_externally_used_gas_impl<S: Storage, Q: Querier>(
    ctx: &mut Ctx,
    used_gas: u64,
) -> VmResult<()> {
    use crate::backends::{get_gas_left, set_gas_left};

    let ctx_data = get_context_data_mut::<S, Q>(ctx);
    if let Some(mut instance_ptr) = ctx_data.wasmer_instance {
        let instance = unsafe { instance_ptr.as_mut() };
        let gas_state = &mut ctx_data.gas_state;

        let wasmer_used_gas = gas_state.get_gas_used_in_wasmer(get_gas_left(instance.context()));

        gas_state.increase_externally_used_gas(used_gas);
        // These lines reduce the amount of gas available to wasmer
        // so it can not consume gas that was consumed externally.
        let new_limit = gas_state.get_gas_left(wasmer_used_gas);
        // This tells wasmer how much more gas it can consume from this point in time.
        set_gas_left(instance.context_mut(), new_limit);

        if gas_state.externally_used_gas + wasmer_used_gas > gas_state.gas_limit {
            Err(VmError::GasDepletion)
        } else {
            Ok(())
        }
    } else {
        Err(VmError::uninitialized_context_data("wasmer_instance"))
    }
}

#[cfg(feature = "default-cranelift")]
fn account_for_externally_used_gas_impl<S: Storage, Q: Querier>(
    _ctx: &mut Ctx,
    _used_gas: u64,
) -> VmResult<()> {
    Ok(())
}

/// Returns true iff the storage is set to readonly mode
pub fn is_storage_readonly<S: Storage, Q: Querier>(ctx: &Ctx) -> bool {
    let context_data = get_context_data::<S, Q>(ctx);
    context_data.storage_readonly
}

pub fn set_storage_readonly<S: Storage, Q: Querier>(ctx: &mut Ctx, new_value: bool) {
    let mut context_data = get_context_data_mut::<S, Q>(ctx);
    context_data.storage_readonly = new_value;
}

pub(crate) fn with_func_from_context<S, Q, Args, Rets, Callback, CallbackData>(
    ctx: &mut Ctx,
    name: &str,
    callback: Callback,
) -> VmResult<CallbackData>
where
    S: Storage,
    Q: Querier,
    Args: WasmTypeList,
    Rets: WasmTypeList,
    Callback: FnOnce(Func<Args, Rets, Wasm>) -> VmResult<CallbackData>,
{
    let ctx_data = get_context_data::<S, Q>(ctx);
    match ctx_data.wasmer_instance {
        Some(instance_ptr) => {
            let func = unsafe { instance_ptr.as_ref() }.exports.get(name)?;
            callback(func)
        }
        None => Err(VmError::uninitialized_context_data("wasmer_instance")),
    }
}

pub(crate) fn with_storage_from_context<'a, 'b, S: 'b, Q: 'b, F, T>(
    ctx: &'a mut Ctx,
    func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&'b mut S) -> VmResult<T>,
{
    let b = get_context_data_mut::<S, Q>(ctx);
    match b.storage.as_mut() {
        Some(data) => func(data),
        None => Err(VmError::uninitialized_context_data("storage")),
    }
}

pub(crate) fn with_querier_from_context<'a, 'b, S: 'b, Q: 'b, F, T>(
    ctx: &'a mut Ctx,
    func: F,
) -> VmResult<T>
where
    S: Storage,
    Q: Querier,
    F: FnOnce(&'b mut Q) -> VmResult<T>,
{
    let b = get_context_data_mut::<S, Q>(ctx);
    match b.querier.as_mut() {
        Some(querier) => func(querier),
        None => Err(VmError::uninitialized_context_data("querier")),
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::backend::Storage;
    use crate::backends::{compile, decrease_gas_left, set_gas_left};
    use crate::errors::VmError;
    use crate::testing::{MockQuerier, MockStorage};
    use cosmwasm_std::{
        coins, from_binary, to_vec, AllBalanceResponse, BankQuery, Empty, HumanAddr, QueryRequest,
    };
    use wasmer_runtime_core::{imports, typed_func::Func};

    static CONTRACT: &[u8] = include_bytes!("../testdata/contract.wasm");

    // shorthands for function generics below
    type MS = MockStorage;
    type MQ = MockQuerier;

    // prepared data
    const INIT_KEY: &[u8] = b"foo";
    const INIT_VALUE: &[u8] = b"bar";
    // this account has some coins
    const INIT_ADDR: &str = "someone";
    const INIT_AMOUNT: u128 = 500;
    const INIT_DENOM: &str = "TOKEN";

    const GAS_LIMIT: u64 = 5_000_000;
    const DEFAULT_QUERY_GAS_LIMIT: u64 = 300_000;

    fn make_instance() -> Box<WasmerInstance> {
        let module = compile(&CONTRACT).unwrap();
        // we need stubs for all required imports
        let import_obj = imports! {
            || { setup_context::<MockStorage, MockQuerier>(GAS_LIMIT) },
            "env" => {
                "db_read" => Func::new(|_a: u32| -> u32 { 0 }),
                "db_write" => Func::new(|_a: u32, _b: u32| {}),
                "db_remove" => Func::new(|_a: u32| {}),
                "db_scan" => Func::new(|_a: u32, _b: u32, _c: i32| -> u32 { 0 }),
                "db_next" => Func::new(|_a: u32| -> u32 { 0 }),
                "query_chain" => Func::new(|_a: u32| -> u32 { 0 }),
                "canonicalize_address" => Func::new(|_a: u32, _b: u32| -> u32 { 0 }),
                "humanize_address" => Func::new(|_a: u32, _b: u32| -> u32 { 0 }),
                "debug" => Func::new(|_a: u32| {}),
            },
        };
        let mut instance = Box::from(module.instantiate(&import_obj).unwrap());

        let instance_ptr = NonNull::from(instance.as_ref());
        set_wasmer_instance::<MS, MQ>(instance.context_mut(), Some(instance_ptr));

        instance
    }

    fn leave_default_data(ctx: &mut Ctx) {
        // create some mock data
        let mut storage = MockStorage::new();
        storage
            .set(INIT_KEY, INIT_VALUE)
            .0
            .expect("error setting value");
        let querier: MockQuerier<Empty> =
            MockQuerier::new(&[(&HumanAddr::from(INIT_ADDR), &coins(INIT_AMOUNT, INIT_DENOM))]);
        move_into_context(ctx, storage, querier);
    }

    #[test]
    fn leave_and_take_context_data() {
        // this creates an instance
        let mut instance = make_instance();
        let ctx = instance.context_mut();

        // empty data on start
        let (inits, initq) = move_out_of_context::<MS, MQ>(ctx);
        assert!(inits.is_none());
        assert!(initq.is_none());

        // store it on the instance
        leave_default_data(ctx);
        let (s, q) = move_out_of_context::<MS, MQ>(ctx);
        assert!(s.is_some());
        assert!(q.is_some());
        assert_eq!(
            s.unwrap().get(INIT_KEY).0.unwrap(),
            Some(INIT_VALUE.to_vec())
        );

        // now is empty again
        let (ends, endq) = move_out_of_context::<MS, MQ>(ctx);
        assert!(ends.is_none());
        assert!(endq.is_none());
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly() {
        let mut instance = make_instance();

        let gas_limit = 100;
        set_gas_left(instance.context_mut(), gas_limit);
        get_gas_state_mut::<MS, MQ>(instance.context_mut()).set_gas_limit(gas_limit);
        let context = instance.context_mut();

        // Consume all the Gas that we allocated
        account_for_externally_used_gas::<MS, MQ>(context, 70).unwrap();
        account_for_externally_used_gas::<MS, MQ>(context, 4).unwrap();
        account_for_externally_used_gas::<MS, MQ>(context, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(context, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(context, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    #[cfg(feature = "default-singlepass")]
    fn gas_tracking_works_correctly_with_gas_consumption_in_wasmer() {
        let mut instance = make_instance();

        let gas_limit = 100;
        set_gas_left(instance.context_mut(), gas_limit);
        get_gas_state_mut::<MS, MQ>(instance.context_mut()).set_gas_limit(gas_limit);
        let context = instance.context_mut();

        // Some gas was consumed externally
        account_for_externally_used_gas::<MS, MQ>(context, 50).unwrap();
        account_for_externally_used_gas::<MS, MQ>(context, 4).unwrap();

        // Consume 20 gas directly in wasmer
        decrease_gas_left(instance.context_mut(), 20).unwrap();

        let context = instance.context_mut();
        account_for_externally_used_gas::<MS, MQ>(context, 6).unwrap();
        account_for_externally_used_gas::<MS, MQ>(context, 20).unwrap();
        // Using one more unit of gas triggers a failure
        match account_for_externally_used_gas::<MS, MQ>(context, 1).unwrap_err() {
            VmError::GasDepletion => {}
            err => panic!("unexpected error: {:?}", err),
        }
    }

    #[test]
    fn is_storage_readonly_defaults_to_true() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), true);
    }

    #[test]
    fn set_storage_readonly_can_change_flag() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        // change
        set_storage_readonly::<MS, MQ>(ctx, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), false);

        // still false
        set_storage_readonly::<MS, MQ>(ctx, false);
        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), false);

        // change back
        set_storage_readonly::<MS, MQ>(ctx, true);
        assert_eq!(is_storage_readonly::<MS, MQ>(ctx), true);
    }

    #[test]
    fn with_func_from_context_works() {
        let mut instance = make_instance();
        leave_default_data(instance.context_mut());

        let ctx = instance.context_mut();
        let ptr = with_func_from_context::<MS, MQ, u32, u32, _, _>(ctx, "allocate", |alloc_func| {
            let ptr = alloc_func.call(10)?;
            Ok(ptr)
        })
        .unwrap();
        assert!(ptr > 0);
    }

    #[test]
    fn with_func_from_context_fails_for_missing_instance() {
        let mut instance = make_instance();
        leave_default_data(instance.context_mut());

        // Clear context's wasmer_instance
        set_wasmer_instance::<MS, MQ>(instance.context_mut(), None);

        let ctx = instance.context_mut();
        let res = with_func_from_context::<MS, MQ, u32, u32, _, ()>(ctx, "allocate", |_func| {
            panic!("unexpected callback call");
        });
        match res.unwrap_err() {
            VmError::UninitializedContextData { kind, .. } => assert_eq!(kind, "wasmer_instance"),
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_func_from_context_fails_for_missing_function() {
        let mut instance = make_instance();
        leave_default_data(instance.context_mut());

        let ctx = instance.context_mut();
        let res = with_func_from_context::<MS, MQ, u32, u32, _, ()>(ctx, "doesnt_exist", |_func| {
            panic!("unexpected callback call");
        });
        match res.unwrap_err() {
            VmError::ResolveErr { msg, .. } => {
                assert_eq!(
                    msg,
                    "Wasmer resolve error: ExportNotFound { name: \"doesnt_exist\" }"
                );
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn with_storage_from_context_set_get() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let val = with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            Ok(store.get(INIT_KEY).0.expect("error getting value"))
        })
        .unwrap();
        assert_eq!(val, Some(INIT_VALUE.to_vec()));

        let set_key: &[u8] = b"more";
        let set_value: &[u8] = b"data";

        with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            store
                .set(set_key, set_value)
                .0
                .expect("error setting value");
            Ok(())
        })
        .unwrap();

        with_storage_from_context::<MS, MQ, _, _>(ctx, |store| {
            assert_eq!(store.get(INIT_KEY).0.unwrap(), Some(INIT_VALUE.to_vec()));
            assert_eq!(store.get(set_key).0.unwrap(), Some(set_value.to_vec()));
            Ok(())
        })
        .unwrap();
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_storage_from_context_handles_panics() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        with_storage_from_context::<MS, MQ, _, ()>(ctx, |_store| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }

    #[test]
    fn with_querier_from_context_works() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        let res = with_querier_from_context::<MS, MQ, _, _>(ctx, |querier| {
            let req: QueryRequest<Empty> = QueryRequest::Bank(BankQuery::AllBalances {
                address: HumanAddr::from(INIT_ADDR),
            });
            let (result, _gas_info) =
                querier.query_raw(&to_vec(&req).unwrap(), DEFAULT_QUERY_GAS_LIMIT);
            Ok(result.unwrap())
        })
        .unwrap()
        .unwrap()
        .unwrap();
        let balance: AllBalanceResponse = from_binary(&res).unwrap();

        assert_eq!(balance.amount, coins(INIT_AMOUNT, INIT_DENOM));
    }

    #[test]
    #[should_panic(expected = "A panic occurred in the callback.")]
    fn with_querier_from_context_handles_panics() {
        let mut instance = make_instance();
        let ctx = instance.context_mut();
        leave_default_data(ctx);

        with_querier_from_context::<MS, MQ, _, ()>(ctx, |_querier| {
            panic!("A panic occurred in the callback.")
        })
        .unwrap();
    }
}
