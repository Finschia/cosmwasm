# Example contracts

Those contracts are made for development purpose only. For more realistic
example contracts, see
[cosmwasm-examples](https://github.com/CosmWasm/cosmwasm-examples).

## Optimized builds

Those development contracts are used for testing in other repos, e.g. in
[wasmvm](https://github.com/line/wasmvm/tree/main/api/testdata) or
[cosmjs](https://github.com/cosmos/cosmjs/tree/main/scripts/wasmd/contracts).

They are [built and deployed](https://github.com/line/cosmwasm/releases) by
the CI for every release tag. In case you need to build them manually for some
reason, use the following commands:

```sh
docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_burner",target=/code/contracts/burner/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/burner

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_crypto_verify",target=/code/contracts/crypto-verify/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/crypto-verify

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_floaty",target=/code/contracts/floaty/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/floaty

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_hackatom",target=/code/contracts/hackatom/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/hackatom

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect",target=/code/contracts/ibc-reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/ibc-reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_ibc_reflect_send",target=/code/contracts/ibc-reflect-send/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/ibc-reflect-send

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_queue",target=/code/contracts/queue/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/queue

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_reflect",target=/code/contracts/reflect/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/reflect

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_staking",target=/code/contracts/staking/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/staking

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_voting_with_uuid",target=/code/contracts/voting-with-uuid/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/voting-with-uuid

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_dynamic_callee_contract",target=/code/contracts/dynamic-callee-contract/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/dynamic-callee-contract

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_dynamic_caller_contract",target=/code/contracts/dynamic-caller-contract/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/dynamic-caller-contract

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_number",target=/code/contracts/number/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/number

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_call_number",target=/code/contracts/call-number/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/call-number

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_simple_callee",target=/code/contracts/simple-callee/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/simple-callee

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_events",target=/code/contracts/events/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/events

docker run --rm -v "$(pwd)":/code \
  --mount type=volume,source="devcontract_cache_intermediate_number",target=/code/contracts/intermediate-number/target \
  --mount type=volume,source=registry_cache,target=/usr/local/cargo/registry \
  cosmwasm/rust-optimizer:0.12.9 ./contracts/intermediate-number 
```

## Entry points

The development contracts in this folder contain a variety of different entry
points in order to demonstrate and test the flexibility we have.

| Contract                | Macro                                         | Has `query` | Has `migrate` |
| ----------------------- | --------------------------------------------- | ----------- | ------------- |
| burner                  | `#[entry_point]`                              | no          | yes           |
| hackatom                | [`create_entry_points_with_migration!`][cepm] | yes         | yes           |
| ibc-reflect             | `#[entry_point]`                              | yes         | no            |
| queue                   | mixed<sup>1</sup>                             | yes         | yes           |
| reflect                 | [`create_entry_points!`][cep]                 | yes         | no            |
| staking                 | `#[entry_point]`                              | yes         | no            |
| voting-with-uuid        | `#[entry_point]`                              | yes         | no            |
| dynamic_callee_contract | `#[entry_point]`                              | no          | no            |
| dynamic_caller_contract | `#[entry_point]`                              | no          | no            |
| number                  | `#[entry_point]`                              | yes         | no            |
| call-number             | `#[entry_point]`                              | yes         | no            |
| simple-callee           | `#[entry_point]`                              | no          | no            |
| events                  | `#[entry_point]`                              | no          | no            |
| intermediate-number     | `#[entry_point]`                              | yes         | no            |


<sup>1</sup> Because we can. Don't try this at home.

[cepm]:
  https://docs.rs/cosmwasm-std/0.13.0/cosmwasm_std/macro.create_entry_points_with_migration.html
[cep]:
  https://docs.rs/cosmwasm-std/0.13.0/cosmwasm_std/macro.create_entry_points.html
