# Changelog


## [[0.16.3-0.5.1](https://github.com/Finschia/cosmwasm/compare/v0.16.3-0.5.0...0.16.3-0.5.1)] - 2022-03-03

### Fixes

* downgrade wasmer version 2.0.0 ([#166](https://github.com/Finschia/cosmwasm/issues/166))
* copy with cargo.lock in release CD ([#168](https://github.com/Finschia/cosmwasm/pull/168))

## [[0.16.3-0.5.0](https://github.com/Finschia/cosmwasm/compare/v0.14.0-0.4.0...0.16.3-0.5.0)] - 2022-03-02

### Features

* Add derive macro "IntoEvent" ([#161](https://github.com/Finschia/cosmwasm/pull/161))
* merge original version 0.16.3 ([#148](https://github.com/Finschia/cosmwasm/pull/148))
* support Uuid type and sha1_calculate API ([#145](https://github.com/Finschia/cosmwasm/pull/145))
* Add release automation config ([#108](https://github.com/Finschia/cosmwasm/pull/108))

### Fixes

* exclude floaty from release checking ([#165](https://github.com/Finschia/cosmwasm/pull/165))
* update rust version in release.yml ([#163](https://github.com/Finschia/cosmwasm/pull/163))
  - change the MSRV (Minimum Supported Rust Version) to 1.57.0
* export vm::testing::contract::Contract ([#147](https://github.com/Finschia/cosmwasm/pull/147))
* export DivideByZeroError to pub ([#140](https://github.com/Finschia/cosmwasm/pull/140))

## [[v0.14.0-0.4.0](https://github.com/Finschia/cosmwasm/compare/v0.14.0-0.3.0...v0.14.0-0.4.0)] - 2021-06-28

### Changes

* Update upstream Cosmwasm/cosmwasm version to 0.14.0 ([#122](https://github.com/Finschia/cosmwasm/pull/122))
  - Please refer [CHANGELOG_OF_COSMWASM_v0.14.0](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0/CHANGELOG.md)

### Removes
* Remove cosmwasm-ext ([#117](https://github.com/Finschia/cosmwasm/pull/117))

### Chores

* ci: Migrate CI to Actions ([#104](https://github.com/Finschia/cosmwasm/pull/104))
* docs: Renew CHANGELOG form ([#111](https://github.com/Finschia/cosmwasm/pull/111))
* docs: Add copyright to license and notice ([#113](https://github.com/Finschia/cosmwasm/pull/113))
* docs: Add contributing and CoC ([#114](https://github.com/Finschia/cosmwasm/pull/114))
* docs: Remove docs and add links to original docs ([#115](https://github.com/Finschia/cosmwasm/pull/115))
* Modify package description ([#114](https://github.com/Finschia/cosmwasm/pull/114))
* ci: Fix shellcheck failure ([#119](https://github.com/Finschia/cosmwasm/pull/119))
* docs: Change ref links to ours ([#120](https://github.com/Finschia/cosmwasm/pull/120))
* Merge develop into main ([#124](https://github.com/Finschia/cosmwasm/pull/124))
* Update pull request template ([#125](https://github.com/Finschia/cosmwasm/pull/125))
* Rename lbm to lfb ([#126](https://github.com/Finschia/cosmwasm/pull/126))
* Replace links in source ([#127](https://github.com/Finschia/cosmwasm/pull/127))

## [[v0.14.0-0.3.0](https://github.com/Finschia/cosmwasm/compare/v0.12.0-0.2.0...v0.14.0-0.3.0)] - 2021-04-02

### Changes

* Update upstream Cosmwasm/cosmwasm version to 0.14.0-beta1 ([#86](https://github.com/Finschia/cosmwasm/issues/86))
  - Please refer [CHANGELOG_OF_COSMWASM_v0.14.0-beta1](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/CHANGELOG.md)


## [[v0.12.0-0.2.0](https://github.com/Finschia/cosmwasm/compare/v0.12.0-0.1.0...v0.12.0-0.2.0)] - 2021-03-29

### Features

* Add vm::testing::MockApi::new_with_gas_cost to specify how much gas api consume ([#89](https://github.com/Finschia/cosmwasm/issues/89))
* Add vm::testing::Contract for integration test using more actual flow ([#87](https://github.com/Finschia/cosmwasm/issues/87))


## [v0.12.0-0.1.0] - 2021-01-27

### Features

* Add approve, burn_from and transfer_from for token/collection ([#29](https://github.com/Finschia/cosmwasm/issues/29))
* Add features to use collection module in ext and add tests for it ([#8](https://github.com/Finschia/cosmwasm/issues/8))
* Add features to use token module in ext and add tests for it ([#7](https://github.com/Finschia/cosmwasm/issues/7))
* Add the ext package and tests for it. It is a wrapper to use token and collection module from contracts ([#6](https://github.com/Finschia/cosmwasm/issues/6))

### Fixes

* Fix bugs on ext packages and refine it ([#35](https://github.com/Finschia/cosmwasm/issues/35))

## [cosmwasm v0.12.0] - 2021-01-05
Initial code is based on the cosmwasm v0.12.0

* (cosmwasm) [v0.12.0](https://github.com/CosmWasm/cosmwasm/releases/tag/v0.12.0).

Please refer [CHANGELOG_OF_COSMWASM_v0.12.0](https://github.com/CosmWasm/cosmwasm/releases?after=v0.12.0)
