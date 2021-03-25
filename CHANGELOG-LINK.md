# CHANGELOG LINK
This is CHANGELOG after this repository was forked from CosmWasm/cosmwasm.

## 0.12.0-0.2.0
### Add
- Add vm::testing::Contract for integration test using more actual flow (#87)
- Add vm::testing::MockApi::new_with_gas_cost to specify how much gas api consume (#89)

## 0.12.0-0.1.0
### Add
- Add the ext package and tests for it. It is a wrapper to use token and collection module from contracts. (#6)
- Add features to use token module in ext and add tests for it (#7)
- Add features to use collection module in ext and add tests for it (#8)
- Add approve, burn_from and transfer_from for token/collection (#29)
- Add semantic.yml for CI (#42)

### Change
- Update upstream CosmWasm/cosmwasm version to 0.12 (#39)

### Fixes
- Fix bugs on ext packages and refine it (#35)
- Fix contract/Readme.md (#37)
- Fix README.md for our repository (#49, #56)
