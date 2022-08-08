use criterion::{criterion_group, criterion_main, Criterion, PlottingBackend};
use std::time::Duration;
use tempfile::TempDir;

use cosmwasm_std::{coins, to_vec, Addr, Empty};
use cosmwasm_vm::testing::{
    mock_backend, mock_env, mock_info, mock_instance, mock_instance_options,
    mock_instance_with_options, write_data_to_mock_env, MockApi, MockInstanceOptions, MockQuerier,
    MockStorage, INSTANCE_CACHE,
};
use cosmwasm_vm::{
    call_execute, call_instantiate, features_from_csv, native_dynamic_link_trampoline_for_bench,
    read_region, ref_to_u32, to_u32, write_region, Backend, BackendApi, BackendError,
    BackendResult, Cache, CacheOptions, Checksum, Environment, FunctionMetadata, GasInfo, Instance,
    InstanceOptions, Querier, Size, Storage, WasmerVal,
};
use std::cell::RefCell;
use wasmer_types::Type;

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 400_000;
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
    print_debug: false,
};

// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");

// For Dynamic Call
const CALLEE_NAME_ADDR: &str = "callee";
const CALLER_NAME_ADDR: &str = "caller";

// DummyApi is Api with dummy `call_contract` which does nothing
#[derive(Copy, Clone)]
struct DummyApi {}

impl BackendApi for DummyApi {
    fn canonical_address(&self, _human: &str) -> BackendResult<Vec<u8>> {
        (
            Err(BackendError::unknown("not implemented")),
            GasInfo::with_cost(0),
        )
    }

    fn human_address(&self, _canonical: &[u8]) -> BackendResult<String> {
        (
            Err(BackendError::unknown("not implemented")),
            GasInfo::with_cost(0),
        )
    }

    fn contract_call<A, S, Q>(
        &self,
        _caller_env: &Environment<A, S, Q>,
        _contract_addr: &str,
        _func_info: &FunctionMetadata,
        _args: &[WasmerVal],
    ) -> BackendResult<Box<[WasmerVal]>>
    where
        A: BackendApi + 'static,
        S: Storage + 'static,
        Q: Querier + 'static,
    {
        (Ok(Box::from([])), GasInfo::with_cost(0))
    }
}

fn bench_instance(c: &mut Criterion) {
    let mut group = c.benchmark_group("Instance");

    group.bench_function("compile and instantiate", |b| {
        b.iter(|| {
            let backend = mock_backend(&[]);
            let (instance_options, memory_limit) = mock_instance_options();
            let _instance =
                Instance::from_code(CONTRACT, backend, instance_options, memory_limit).unwrap();
        });
    });

    group.bench_function("execute init", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: 500_000_000_000,
            ..DEFAULT_INSTANCE_OPTIONS
        };
        let mut instance =
            Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        b.iter(|| {
            let info = mock_info("creator", &coins(1000, "earth"));
            let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
            let contract_result =
                call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        });
    });

    group.bench_function("execute execute", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: 500_000_000_000,
            ..DEFAULT_INSTANCE_OPTIONS
        };
        let mut instance =
            Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        assert!(contract_result.into_result().is_ok());

        b.iter(|| {
            let info = mock_info("verifies", &coins(15, "earth"));
            let msg = br#"{"release":{}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
        });
    });

    group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cache");

    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        supported_features: features_from_csv("staking"),
        memory_cache_size: MEMORY_CACHE_SIZE,
        instance_memory_limit: DEFAULT_MEMORY_LIMIT,
    };

    group.bench_function("save wasm", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };

        b.iter(|| {
            let result = cache.save_wasm(CONTRACT);
            assert!(result.is_ok());
        });
    });

    group.bench_function("load wasm", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let result = cache.load_wasm(&checksum);
            assert!(result.is_ok());
        });
    });

    group.bench_function("analyze", |b| {
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let result = cache.analyze(&checksum);
            assert!(result.is_ok());
        });
    });

    group.bench_function("instantiate from fs", |b| {
        let non_memcache = CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            supported_features: features_from_csv("staking"),
            memory_cache_size: Size(0),
            instance_memory_limit: DEFAULT_MEMORY_LIMIT,
        };
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(non_memcache).unwrap() };
        let checksum = cache.save_wasm(CONTRACT).unwrap();

        b.iter(|| {
            let _ = cache
                .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_fs_cache >= 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.bench_function("instantiate from memory", |b| {
        let checksum = Checksum::generate(CONTRACT);
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        // Load into memory
        cache
            .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
            .unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let _ = cache
                .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_pinned_memory_cache, 0);
            assert!(cache.stats().hits_memory_cache >= 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.bench_function("instantiate from pinned memory", |b| {
        let checksum = Checksum::generate(CONTRACT);
        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options.clone()).unwrap() };
        // Load into pinned memory
        cache.pin(&checksum).unwrap();

        b.iter(|| {
            let backend = mock_backend(&[]);
            let _ = cache
                .get_instance(&checksum, backend, DEFAULT_INSTANCE_OPTIONS)
                .unwrap();
            assert_eq!(cache.stats().hits_memory_cache, 0);
            assert!(cache.stats().hits_pinned_memory_cache >= 1);
            assert_eq!(cache.stats().hits_fs_cache, 1);
            assert_eq!(cache.stats().misses, 0);
        });
    });

    group.finish();
}

fn prepare_dynamic_call_data<A: BackendApi>(
    callee_address: Addr,
    func_info: FunctionMetadata,
    caller_env: &mut Environment<A, MockStorage, MockQuerier>,
) -> u32 {
    let data = to_vec(&callee_address).unwrap();
    let region_ptr = write_data_to_mock_env(caller_env, &data).unwrap();

    caller_env.set_callee_function_metadata(Some(func_info));

    let serialized_env = to_vec(&mock_env()).unwrap();
    caller_env.set_serialized_env(&serialized_env);

    let storage = MockStorage::new();
    let querier: MockQuerier<Empty> = MockQuerier::new(&[]);
    caller_env.move_in(storage, querier);
    region_ptr
}

fn bench_dynamic_link(c: &mut Criterion) {
    let mut group = c.benchmark_group("DynamicLink");

    group.bench_function(
        "native_dynamic_link_trampoline with dummy contract_call",
        |b| {
            let backend = Backend {
                api: DummyApi {},
                storage: MockStorage::default(),
                querier: MockQuerier::new(&[]),
            };
            let mock_options = MockInstanceOptions::default();
            let options = InstanceOptions {
                gas_limit: mock_options.gas_limit,
                print_debug: mock_options.print_debug,
            };
            let instance =
                Instance::from_code(CONTRACT, backend, options, mock_options.memory_limit).unwrap();
            let mut dummy_env = instance.env;
            let callee_address = Addr::unchecked(CALLEE_NAME_ADDR);
            let target_func_info = FunctionMetadata {
                module_name: CALLEE_NAME_ADDR.to_string(),
                name: "foo".to_string(),
                signature: ([Type::I32], []).into(),
            };

            let address_region =
                prepare_dynamic_call_data(callee_address, target_func_info, &mut dummy_env);

            b.iter(|| {
                let _ = native_dynamic_link_trampoline_for_bench(
                    &dummy_env,
                    &[WasmerVal::I32(address_region as i32)],
                )
                .unwrap();
            })
        },
    );

    group.bench_function(
        "native_dynamic_link_trampoline with mock cache environment",
        |b| {
            let callee_wasm = wat::parse_str(
                r#"(module
                (memory 3)
                (export "memory" (memory 0))
                (export "interface_version_5" (func 0))
                (export "instantiate" (func 0))
                (export "allocate" (func 0))
                (export "deallocate" (func 0))
                (type (func))
                (func (type 0) nop)
                (export "foo" (func 0))
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
                    RefCell::new(mock_instance_with_options(
                        &CONTRACT,
                        MockInstanceOptions {
                            gas_limit: 500_000_000_000,
                            ..MockInstanceOptions::default()
                        },
                    )),
                );
            });

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
                    Addr::unchecked(CALLEE_NAME_ADDR),
                    target_func_info,
                    &mut caller_env,
                );

                b.iter(|| {
                    let _ = native_dynamic_link_trampoline_for_bench(
                        &caller_env,
                        &[WasmerVal::I32(address_region as i32)],
                    )
                    .unwrap();
                })
            });
        },
    );

    group.finish()
}

fn bench_copy_region(c: &mut Criterion) {
    let mut group = c.benchmark_group("CopyRegion");

    for i in 0..=3 {
        let length = 10_i32.pow(i);
        group.bench_function(format!("read region (length == {})", length), |b| {
            let data: Vec<u8> = (0..length).map(|x| (x % 255) as u8).collect();
            assert_eq!(data.len(), length as usize);
            let instance = mock_instance(&CONTRACT, &[]);
            let ret = instance
                .env
                .call_function1("allocate", &[to_u32(data.len()).unwrap().into()])
                .unwrap();
            let region_ptr = ref_to_u32(&ret).unwrap();
            write_region(&instance.env.memory(), region_ptr, &data).unwrap();
            let got_data =
                read_region(&instance.env.memory(), region_ptr, u32::MAX as usize).unwrap();
            assert_eq!(data, got_data);
            b.iter(|| {
                let _ = read_region(&instance.env.memory(), region_ptr, u32::MAX as usize);
            })
        });

        group.bench_function(format!("write region (length == {})", length), |b| {
            let data: Vec<u8> = (0..length).map(|x| (x % 255) as u8).collect();
            assert_eq!(data.len(), length as usize);
            let instance = mock_instance(&CONTRACT, &[]);
            let ret = instance
                .env
                .call_function1("allocate", &[to_u32(data.len()).unwrap().into()])
                .unwrap();
            let region_ptr = ref_to_u32(&ret).unwrap();
            b.iter(|| {
                write_region(&instance.env.memory(), region_ptr, &data).unwrap();
            })
        });
    }

    group.finish();
}

fn make_config() -> Criterion {
    Criterion::default()
        .plotting_backend(PlottingBackend::Plotters)
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
}

criterion_group!(
    name = instance;
    config = make_config();
    targets = bench_instance
);
criterion_group!(
    name = cache;
    config = make_config();
    targets = bench_cache
);
criterion_group!(
    name = dynamic_link;
    config = make_config();
    targets = bench_dynamic_link
);
criterion_group!(
    name = copy_region;
    config = make_config();
    targets = bench_copy_region
);
//criterion_main!(instance, cache, dynamic_link, copy_region);
criterion_main!(copy_region);
