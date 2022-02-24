# CosmWasm VM

_Forked from [CosmWasm/cosmwasm-vm](https://github.com/CosmWasm/cosmwasm/tree/main/packages/vm)_

This is an abstraction layer around the wasmer VM to expose just what we need to
run cosmwasm contracts in a high-level manner. This is intended both for
efficient writing of unit tests, as well as a public API to run contracts in eg.
go-cosmwasm. As such it includes all glue code needed for typical actions, like
fs caching.

## Compatibility

A VM can support one or more contract-VM interface versions. The interface
version is communicated by the contract via a Wasm import. This is the current
compatibility list:

| cosmwasm-vm | Supported interface versions | cosmwasm-std |
| ----------- | ---------------------------- | ------------ |
| 0.14        | `interface_version_5`        | 0.14         |
| 0.13        | `cosmwasm_vm_version_4`      | 0.11-0.13    |
| 0.12        | `cosmwasm_vm_version_4`      | 0.11-0.13    |
| 0.11        | `cosmwasm_vm_version_4`      | 0.11-0.13    |
| 0.10        | `cosmwasm_vm_version_3`      | 0.10         |
| 0.9         | `cosmwasm_vm_version_2`      | 0.9          |
| 0.8         | `cosmwasm_vm_version_1`      | 0.8          |

## Setup

There are demo files in `testdata/*.wasm`. Those are compiled and optimized
versions of
[contracts/hackatom](https://github.com/line/cosmwasm/tree/main/contracts/hackatom)
and
[contracts/staking](https://github.com/line/cosmwasm/tree/main/contracts/staking)
run through [rust-optimizer](https://github.com/CosmWasm/rust-optimizer).

To rebuild the test contracts, go to the repo root and do

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_hackatom",target=/code/contracts/hackatom/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.11.0 ./contracts/hackatom \
  && cp artifacts/hackatom.wasm packages/vm/testdata/hackatom_0.14.wasm

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect",target=/code/contracts/ibc-reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.11.0 ./contracts/ibc-reflect \
  && cp artifacts/ibc_reflect.wasm packages/vm/testdata/ibc_reflect_0.14.wasm
```

## Testing

By default, this repository is built and tested with the singlepass backend. You
can enable the `cranelift` feature to override the default backend with
Cranelift

```sh
cd packages/vm
cargo test --features iterator
cargo test --features cranelift,iterator
```

## Benchmarking

Using Singlepass:

```
cd packages/vm
cargo bench --no-default-features
```

Using Cranelift:

```
cd packages/vm
cargo bench --no-default-features --features cranelift
```

## Tools

`module_size` and `module_size.sh`

Memory profiling of compiled modules. `module_size.sh` executes `module_size`,
and uses valgrind's memory profiling tool (massif) to compute the amount of heap
memory used by a compiled module.

```
cd packages/vm
RUSTFLAGS="-g" cargo build --release --example module_size
./examples/module_size.sh ./testdata/hackatom.wasm
```

## License

This package is part of the line/cosmwasm repository, licensed under the Apache
License 2.0 (see [NOTICE](https://github.com/line/cosmwasm/blob/main/NOTICE)
and [LICENSE](https://github.com/line/cosmwasm/blob/main/LICENSE)).
