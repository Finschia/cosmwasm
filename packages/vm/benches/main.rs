use criterion::{black_box, criterion_group, criterion_main, Criterion};

use rand::Rng;
use std::sync::Arc;
use std::thread;
use std::time::{Duration, SystemTime};
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
use wasmer::Module;
use wasmer_types::Type;

// Instance
const DEFAULT_MEMORY_LIMIT: Size = Size::mebi(64);
const DEFAULT_GAS_LIMIT: u64 = 1_000_000_000_000; // ~1ms
const DEFAULT_INSTANCE_OPTIONS: InstanceOptions = InstanceOptions {
    gas_limit: DEFAULT_GAS_LIMIT,
    print_debug: false,
};
const HIGH_GAS_LIMIT: u64 = 20_000_000_000_000_000; // ~20s, allows many calls on one instance

// Cache
const MEMORY_CACHE_SIZE: Size = Size::mebi(200);

// Multi-threaded get_instance benchmark
const INSTANTIATION_THREADS: usize = 128;
const CONTRACTS: u64 = 10;

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

    fn get_wasmer_module(&self, _contract_addr: &str) -> BackendResult<Module> {
        (
            Err(BackendError::unknown("not implemented")),
            GasInfo::with_cost(0),
        )
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
            gas_limit: HIGH_GAS_LIMIT,
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

    group.bench_function("execute execute (release)", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
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

    group.bench_function("execute execute (argon2)", |b| {
        let backend = mock_backend(&[]);
        let much_gas: InstanceOptions = InstanceOptions {
            gas_limit: HIGH_GAS_LIMIT,
            ..DEFAULT_INSTANCE_OPTIONS
        };
        let mut instance =
            Instance::from_code(CONTRACT, backend, much_gas, Some(DEFAULT_MEMORY_LIMIT)).unwrap();

        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        let contract_result =
            call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
        assert!(contract_result.into_result().is_ok());

        let mut gas_used = 0;
        b.iter(|| {
            let gas_before = instance.get_gas_left();
            let info = mock_info("hasher", &[]);
            let msg = br#"{"argon2":{"mem_cost":256,"time_cost":3}}"#;
            let contract_result =
                call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg).unwrap();
            assert!(contract_result.into_result().is_ok());
            gas_used = gas_before - instance.get_gas_left();
        });
        println!("Gas used: {}", gas_used);
    });

    group.finish();
}

fn bench_cache(c: &mut Criterion) {
    let mut group = c.benchmark_group("Cache");

    let options = CacheOptions {
        base_dir: TempDir::new().unwrap().into_path(),
        supported_features: features_from_csv("iterator,staking"),
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
            supported_features: features_from_csv("iterator,staking"),
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
                            // enough gas for bench iterations
                            gas_limit: 500_000_000_000_000_000,
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

pub fn bench_instance_threads(c: &mut Criterion) {
    c.bench_function("multi-threaded get_instance", |b| {
        let options = CacheOptions {
            base_dir: TempDir::new().unwrap().into_path(),
            supported_features: features_from_csv("iterator,staking"),
            memory_cache_size: MEMORY_CACHE_SIZE,
            instance_memory_limit: DEFAULT_MEMORY_LIMIT,
        };

        let cache: Cache<MockApi, MockStorage, MockQuerier> =
            unsafe { Cache::new(options).unwrap() };
        let cache = Arc::new(cache);

        // Find sub-sequence helper
        fn find_subsequence(haystack: &[u8], needle: &[u8]) -> Option<usize> {
            haystack
                .windows(needle.len())
                .position(|window| window == needle)
        }

        // Offset to the i32.const (0x41) 15731626 (0xf00baa) (unsigned leb128 encoded) instruction
        // data we want to replace
        let query_int_data = b"\x41\xaa\x97\xc0\x07";
        let offset = find_subsequence(CONTRACT, query_int_data).unwrap() + 1;

        let mut leb128_buf = [0; 4];
        let mut contract = CONTRACT.to_vec();

        let mut random_checksum = || {
            let mut writable = &mut leb128_buf[..];

            // Generates a random number in the range of a 4-byte unsigned leb128 encoded number
            let r = rand::thread_rng().gen_range(2097152..2097152 + CONTRACTS);

            leb128::write::unsigned(&mut writable, r).expect("Should write number");

            // Splice data in contract
            contract.splice(offset..offset + leb128_buf.len(), leb128_buf);

            cache.save_wasm(contract.as_slice()).unwrap()
            // let checksum = cache.save_wasm(contract.as_slice()).unwrap();
            // Preload into memory
            // cache
            //     .get_instance(&checksum, mock_backend(&[]), DEFAULT_INSTANCE_OPTIONS)
            //     .unwrap();
            // checksum
        };

        b.iter_custom(|iters| {
            let mut res = Duration::from_secs(0);
            for _ in 0..iters {
                let mut durations: Vec<_> = (0..INSTANTIATION_THREADS)
                    .map(|_id| {
                        let cache = Arc::clone(&cache);
                        let checksum = random_checksum();

                        thread::spawn(move || {
                            let checksum = checksum;
                            // Perform measurement internally
                            let t = SystemTime::now();
                            black_box(
                                cache
                                    .get_instance(
                                        &checksum,
                                        mock_backend(&[]),
                                        DEFAULT_INSTANCE_OPTIONS,
                                    )
                                    .unwrap(),
                            );
                            t.elapsed().unwrap()
                        })
                    })
                    .collect::<Vec<_>>()
                    .into_iter()
                    .map(|handle| handle.join().unwrap())
                    .collect(); // join threads, collect durations

                // Calculate median thread duration
                durations.sort_unstable();
                res += durations[durations.len() / 2];
            }
            res
        });
    });
}

fn make_config() -> Criterion {
    Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(10, 0))
        .sample_size(12)
        .configure_from_args()
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

criterion_group!(
    name = multi_threaded_instance;
    config = Criterion::default()
        .without_plots()
        .measurement_time(Duration::new(16, 0))
        .sample_size(10)
        .configure_from_args();
    targets = bench_instance_threads
);
criterion_main!(
    instance,
    cache,
    dynamic_link,
    copy_region,
    multi_threaded_instance
);
