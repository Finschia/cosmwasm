# Changelog


## [[v1.1.9+0.7.0-dynamiclink2](https://github.com/line/cosmwasm/compare/v1.1.9-0.7.0-dynamiclink1...v1.1.9+0.7.0-dynamiclink2)] - 2023-06-30

### Changes

* Set version 1.1.9+0.7.0-dynamiclink2 ([#315](https://github.com/line/cosmwasm/pull/315))

### Features

* StdResult (de)serializer ([#313](https://github.com/line/cosmwasm/pull/313))
* add a query to get own address via callee's get caller address … ([#307](https://github.com/line/cosmwasm/pull/307))

### Fixes

* delete outdated serde json ([#312](https://github.com/line/cosmwasm/pull/312))


## [[v1.1.9-0.7.0-dynamiclink1](https://github.com/line/cosmwasm/compare/v1.1.9-0.7.0...v1.1.9-0.7.0-dynamiclink1)] - 2023-06-12

### Changes


### Ci

* fix benchmarkings and make CI benchmark each PR ([#294](https://github.com/line/cosmwasm/pull/294))
* add lack clippy tests for some contracts and fix contracts according to them ([#285](https://github.com/line/cosmwasm/pull/285))
* add Documentation tests ([#262](https://github.com/line/cosmwasm/pull/262))
* add setup process of dynamic link benchmarking to avoid runing out of gas ([#250](https://github.com/line/cosmwasm/pull/250))
* fix dynamic_link ci for benchmarking following [#209](https://github.com/line/cosmwasm/issues/209) ([#237](https://github.com/line/cosmwasm/pull/237))
* add tests for package derive ([#230](https://github.com/line/cosmwasm/pull/230))

### Code Refactoring

* remove unused INSTNACE_CACHE ([#296](https://github.com/line/cosmwasm/pull/296))

### Docs

* fix README ([#235](https://github.com/line/cosmwasm/pull/235))

### Features

* add get_caller_addr to deps.api ([#304](https://github.com/line/cosmwasm/pull/304))
* check property function for dynamic linked callee contract ([#301](https://github.com/line/cosmwasm/pull/301))
* switch how to do dynamic link and validate interface to solve rust/Go pointer sharing issue ([#283](https://github.com/line/cosmwasm/pull/283))
* add set_callee_permission ([#278](https://github.com/line/cosmwasm/pull/278))
* implement set_dynamic_callstack and call_function to Instance ([#272](https://github.com/line/cosmwasm/pull/272))
* improve events contract to be usable via dynamic link ([#274](https://github.com/line/cosmwasm/pull/274))
* Add EventManager to Context Data in Instance's Environment ([#266](https://github.com/line/cosmwasm/pull/266))
* force the dynamic linked functions' module name to start with "dynamiclinked_" ([#248](https://github.com/line/cosmwasm/pull/248))
* add interface validation method for dynamic link functions ([#245](https://github.com/line/cosmwasm/pull/245))
* pass env as the second arg of dynamic linked callee function ([#234](https://github.com/line/cosmwasm/pull/234))
* limit the regions length using dynamic link ([#240](https://github.com/line/cosmwasm/pull/240))
* make callable point takes deps as the first arg ([#233](https://github.com/line/cosmwasm/pull/233))
* add do_panic as callable_point ([#236](https://github.com/line/cosmwasm/pull/236))
* change error message ([#232](https://github.com/line/cosmwasm/pull/232))
* make available user defined mock for dynamic link ([#229](https://github.com/line/cosmwasm/pull/229))
* improve dynamic link contracts' tests and add them to github workflow ([#221](https://github.com/line/cosmwasm/pull/221))
* add number contracts ([#222](https://github.com/line/cosmwasm/pull/222))
* improve how to specify the callee contract address ([#215](https://github.com/line/cosmwasm/pull/215))
* enable dynamic callee returns tuple typed value ([#195](https://github.com/line/cosmwasm/pull/195))
* add abort when using the unsupported ABI. ([#187](https://github.com/line/cosmwasm/pull/187))
* add dynamic callstack for prevent the re-entrancy attack ([#178](https://github.com/line/cosmwasm/pull/178))
* add global api for to get env,deps in dynamic call ([#182](https://github.com/line/cosmwasm/pull/182))
* add import&export macro and serialized data copy pass ([#157](https://github.com/line/cosmwasm/pull/157))
* direct contract call with dynamic link ([#153](https://github.com/line/cosmwasm/pull/153))

### Fixes

* change the type of argument of "validate_interface" to serialized binary ([#289](https://github.com/line/cosmwasm/pull/289))
* serde wasm32 target ([#288](https://github.com/line/cosmwasm/pull/288))
* Remove unused dynamiclink features ([#286](https://github.com/line/cosmwasm/pull/286))
* Fix events contract ([#276](https://github.com/line/cosmwasm/pull/276))
* remake functions to read/write from/to env region for dynamic link ([#259](https://github.com/line/cosmwasm/pull/259))
* max address_length on interface validation ([#254](https://github.com/line/cosmwasm/pull/254))
* revert type of Unknown msg ([#251](https://github.com/line/cosmwasm/pull/251))
* fix vm bench ([#246](https://github.com/line/cosmwasm/pull/246))
* fix local test error ([#239](https://github.com/line/cosmwasm/pull/239))
* rename generated functions from dynamic_link and callable_point macro ([#226](https://github.com/line/cosmwasm/pull/226))
* relax the limit of exporting targets of global env ([#220](https://github.com/line/cosmwasm/pull/220))
* correct path ([#217](https://github.com/line/cosmwasm/pull/217))
* limit exporting GlobalEnv only to wasm32 target ([#212](https://github.com/line/cosmwasm/pull/212))
* fix a error message for re-entrancy ([#200](https://github.com/line/cosmwasm/pull/200))

### Improvements

* Add benchmarks for estimate dynamic link gas cost ([#224](https://github.com/line/cosmwasm/pull/224))


## [[v1.1.9-0.7.0](https://github.com/line/cosmwasm/compare/v1.0.0-0.6.0...v1.1.9-0.7.0)] - 2023-04-27

### Ci

* renew how to check the WASM in release ([#298](https://github.com/line/cosmwasm/pull/298))
* update rust-optimizer version used in release ci ([#293](https://github.com/line/cosmwasm/pull/293))
* add ci tests for package derive ([#231](https://github.com/line/cosmwasm/pull/231))
* fix broken workflow for benchmarking ([#209](https://github.com/line/cosmwasm/pull/209))
* fix broken ci tests.yml ([#210](https://github.com/line/cosmwasm/pull/210))

### Docs

* replace line with finschia in docs, comments, and scripts related docs ([#290](https://github.com/line/cosmwasm/pull/290))
* fix dead links ([#253](https://github.com/line/cosmwasm/pull/253))

### Features

* add codeowners file ([#275](https://github.com/line/cosmwasm/pull/275))

### Fixes

* release action ends with success without renew tag ([#206](https://github.com/line/cosmwasm/pull/206))
* re create README.md ([#201](https://github.com/line/cosmwasm/pull/201))
* fix keys of action caches for contracts ([#204](https://github.com/line/cosmwasm/pull/204))
* add voting_with_uuid to README and tests in actions ([#205](https://github.com/line/cosmwasm/pull/205))
* update query-queue to version 1.0.0 ([#203](https://github.com/line/cosmwasm/pull/203))


## [[v1.0.0-0.6.0](https://github.com/line/cosmwasm/compare/v0.16.3-0.5.1...v1.0.0-0.6.0)] - 2022-06-03

### Ci

* fix github release action bug ([#199](https://github.com/line/cosmwasm/pull/199))

### Features

* add integration tests for query-queue ([#197](https://github.com/line/cosmwasm/pull/197))
* add a contract query-queue ([#183](https://github.com/line/cosmwasm/pull/183))
* add memory_limit arg to some Contract's functions ([#179](https://github.com/line/cosmwasm/pull/179))
* simplify vm::testing::Contract ([#181](https://github.com/line/cosmwasm/pull/181))
* cherry pick upstream's commits about vm querier mock ([#180](https://github.com/line/cosmwasm/pull/180))

### Fixes

* make query-queue's entry points using entry_points ([#198](https://github.com/line/cosmwasm/pull/198))
* fix author of query-queue contract ([#196](https://github.com/line/cosmwasm/pull/196))
* modify the chglog template to filter commits and correct PRs' URL ([#170](https://github.com/line/cosmwasm/pull/170))
* fix release.yml to trigger release only when PR is merged ([#172](https://github.com/line/cosmwasm/pull/172))


## [[0.16.3-0.5.1](https://github.com/line/cosmwasm/compare/v0.16.3-0.5.0...0.16.3-0.5.1)] - 2022-03-03

### Fixes

* downgrade wasmer version 2.0.0 ([#166](https://github.com/line/cosmwasm/issues/166))
* copy with cargo.lock in release CD ([#168](https://github.com/line/cosmwasm/pull/168))

## [[0.16.3-0.5.0](https://github.com/line/cosmwasm/compare/v0.14.0-0.4.0...0.16.3-0.5.0)] - 2022-03-02

### Features

* Add derive macro "IntoEvent" ([#161](https://github.com/line/cosmwasm/pull/161))
* merge original version 0.16.3 ([#148](https://github.com/line/cosmwasm/pull/148))
* support Uuid type and sha1_calculate API ([#145](https://github.com/line/cosmwasm/pull/145))
* Add release automation config ([#108](https://github.com/line/cosmwasm/pull/108))

### Fixes

* exclude floaty from release checking ([#165](https://github.com/line/cosmwasm/pull/165))
* update rust version in release.yml ([#163](https://github.com/line/cosmwasm/pull/163))
  - change the MSRV (Minimum Supported Rust Version) to 1.57.0
* export vm::testing::contract::Contract ([#147](https://github.com/line/cosmwasm/pull/147))
* export DivideByZeroError to pub ([#140](https://github.com/line/cosmwasm/pull/140))

## [[v0.14.0-0.4.0](https://github.com/line/cosmwasm/compare/v0.14.0-0.3.0...v0.14.0-0.4.0)] - 2021-06-28

### Changes

* Update upstream Cosmwasm/cosmwasm version to 0.14.0 ([#122](https://github.com/line/cosmwasm/pull/122))
  - Please refer [CHANGELOG_OF_COSMWASM_v0.14.0](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0/CHANGELOG.md)

### Removes
* Remove cosmwasm-ext ([#117](https://github.com/line/cosmwasm/pull/117))

### Chores

* ci: Migrate CI to Actions ([#104](https://github.com/line/cosmwasm/pull/104))
* docs: Renew CHANGELOG form ([#111](https://github.com/line/cosmwasm/pull/111))
* docs: Add copyright to license and notice ([#113](https://github.com/line/cosmwasm/pull/113))
* docs: Add contributing and CoC ([#114](https://github.com/line/cosmwasm/pull/114))
* docs: Remove docs and add links to original docs ([#115](https://github.com/line/cosmwasm/pull/115))
* Modify package description ([#114](https://github.com/line/cosmwasm/pull/114))
* ci: Fix shellcheck failure ([#119](https://github.com/line/cosmwasm/pull/119))
* docs: Change ref links to ours ([#120](https://github.com/line/cosmwasm/pull/120))
* Merge develop into main ([#124](https://github.com/line/cosmwasm/pull/124))
* Update pull request template ([#125](https://github.com/line/cosmwasm/pull/125))
* Rename lbm to lfb ([#126](https://github.com/line/cosmwasm/pull/126))
* Replace links in source ([#127](https://github.com/line/cosmwasm/pull/127))

## [[v0.14.0-0.3.0](https://github.com/line/cosmwasm/compare/v0.12.0-0.2.0...v0.14.0-0.3.0)] - 2021-04-02

### Changes

* Update upstream Cosmwasm/cosmwasm version to 0.14.0-beta1 ([#86](https://github.com/line/cosmwasm/issues/86))
  - Please refer [CHANGELOG_OF_COSMWASM_v0.14.0-beta1](https://github.com/CosmWasm/cosmwasm/blob/v0.14.0-beta1/CHANGELOG.md)


## [[v0.12.0-0.2.0](https://github.com/line/cosmwasm/compare/v0.12.0-0.1.0...v0.12.0-0.2.0)] - 2021-03-29

### Features

* Add vm::testing::MockApi::new_with_gas_cost to specify how much gas api consume ([#89](https://github.com/line/cosmwasm/issues/89))
* Add vm::testing::Contract for integration test using more actual flow ([#87](https://github.com/line/cosmwasm/issues/87))


## [v0.12.0-0.1.0] - 2021-01-27

### Features

* Add approve, burn_from and transfer_from for token/collection ([#29](https://github.com/line/cosmwasm/issues/29))
* Add features to use collection module in ext and add tests for it ([#8](https://github.com/line/cosmwasm/issues/8))
* Add features to use token module in ext and add tests for it ([#7](https://github.com/line/cosmwasm/issues/7))
* Add the ext package and tests for it. It is a wrapper to use token and collection module from contracts ([#6](https://github.com/line/cosmwasm/issues/6))

### Fixes

* Fix bugs on ext packages and refine it ([#35](https://github.com/line/cosmwasm/issues/35))

## [cosmwasm v0.12.0] - 2021-01-05
Initial code is based on the cosmwasm v0.12.0

* (cosmwasm) [v0.12.0](https://github.com/CosmWasm/cosmwasm/releases/tag/v0.12.0).

Please refer [CHANGELOG_OF_COSMWASM_v0.12.0](https://github.com/CosmWasm/cosmwasm/releases?after=v0.12.0)
