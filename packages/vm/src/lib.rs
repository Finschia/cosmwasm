#![cfg_attr(feature = "backtraces", feature(error_generic_member_access))]
#![cfg_attr(feature = "backtraces", feature(provide_any))]

mod backend;
mod cache;
mod calls;
mod capabilities;
mod checksum;
mod compatibility;
mod conversion;
mod dynamic_link;
mod environment;
mod errors;
mod filesystem;
mod imports;
mod instance;
mod limited;
mod memory;
mod modules;
mod parsed_wasm;
mod sections;
mod serde;
mod size;
mod static_analysis;
pub mod testing;
mod wasm_backend;

pub use crate::backend::{
    Backend, BackendApi, BackendError, BackendResult, GasInfo, Querier, Storage,
};
pub use crate::cache::{AnalysisReport, Cache, CacheOptions, Metrics, Stats};
pub use crate::calls::{
    call_execute, call_execute_raw, call_instantiate, call_instantiate_raw, call_migrate,
    call_migrate_raw, call_query, call_query_raw, call_reply, call_reply_raw, call_sudo,
    call_sudo_raw,
};
#[cfg(feature = "stargate")]
pub use crate::calls::{
    call_ibc_channel_close, call_ibc_channel_close_raw, call_ibc_channel_connect,
    call_ibc_channel_connect_raw, call_ibc_channel_open, call_ibc_channel_open_raw,
    call_ibc_packet_ack, call_ibc_packet_ack_raw, call_ibc_packet_receive,
    call_ibc_packet_receive_raw, call_ibc_packet_timeout, call_ibc_packet_timeout_raw,
};
pub use crate::capabilities::capabilities_from_csv;
pub use crate::checksum::Checksum;
#[cfg(feature = "bench")]
pub use crate::conversion::{ref_to_u32, to_u32};
#[cfg(feature = "bench")]
pub use crate::dynamic_link::native_dynamic_link_trampoline_for_bench;
pub use crate::dynamic_link::{
    dynamic_link, read_region_vals, set_callee_permission, write_value_to_env, FunctionMetadata,
};
pub use crate::environment::Environment;
pub use crate::errors::{
    CommunicationError, CommunicationResult, RegionValidationError, RegionValidationResult,
    VmError, VmResult,
};
pub use crate::instance::{DebugInfo, GasReport, Instance, InstanceOptions};
#[cfg(feature = "bench")]
pub use crate::memory::{read_region, write_region};
pub use crate::serde::{from_slice, to_vec};
pub use crate::size::Size;

#[doc(hidden)]
pub mod internals {
    //! We use the internals module for exporting types that are only
    //! intended to be used in internal crates / utils.
    //! Please don't use any of these types directly, as
    //! they might change frequently or be removed in the future.

    pub use crate::compatibility::check_wasm;
    pub use crate::instance::instance_from_module;
    pub use crate::wasm_backend::{compile, make_compiling_engine, make_runtime_engine};
}
