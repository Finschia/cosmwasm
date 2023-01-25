use serde::de::DeserializeOwned;
use wasmer::Val;

use cosmwasm_std::{ContractResult, CustomMsg, Env, MessageInfo, QueryResponse, Reply, Response};
#[cfg(feature = "stargate")]
use cosmwasm_std::{
    Ibc3ChannelOpenResponse, IbcBasicResponse, IbcChannelCloseMsg, IbcChannelConnectMsg,
    IbcChannelOpenMsg, IbcPacketAckMsg, IbcPacketReceiveMsg, IbcPacketTimeoutMsg,
    IbcReceiveResponse,
};

use crate::backend::{BackendApi, Querier, Storage};
use crate::conversion::ref_to_u32;
use crate::errors::{VmError, VmResult};
use crate::instance::Instance;
use crate::serde::{from_slice, to_vec};

/// The limits in here protect the host from allocating an unreasonable amount of memory
/// and copying an unreasonable amount of data.
///
/// A JSON deserializer would want to set the limit to a much smaller value because
/// deserializing JSON is more expensive. As a consequence, any sane contract should hit
/// the deserializer limit before the read limit.
mod read_limits {
    /// A mibi (mega binary)
    const MI: usize = 1024 * 1024;
    /// Max length (in bytes) of the result data from an instantiate call.
    pub const RESULT_INSTANTIATE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from an execute call.
    pub const RESULT_EXECUTE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a migrate call.
    pub const RESULT_MIGRATE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a sudo call.
    pub const RESULT_SUDO: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a reply call.
    pub const RESULT_REPLY: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a query call.
    pub const RESULT_QUERY: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_channel_open call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_OPEN: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_channel_connect call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CONNECT: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_channel_close call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CLOSE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_packet_receive call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_RECEIVE: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_packet_ack call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_ACK: usize = 64 * MI;
    /// Max length (in bytes) of the result data from a ibc_packet_timeout call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_TIMEOUT: usize = 64 * MI;
}

/// The limits for the JSON deserialization.
///
/// Those limits are not used when the Rust JSON deserializer is bypassed by using the
/// public `call_*_raw` functions directly.
mod deserialization_limits {
    /// A kibi (kilo binary)
    const KI: usize = 1024;
    /// Max length (in bytes) of the result data from an instantiate call.
    pub const RESULT_INSTANTIATE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from an execute call.
    pub const RESULT_EXECUTE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a migrate call.
    pub const RESULT_MIGRATE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a sudo call.
    pub const RESULT_SUDO: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a reply call.
    pub const RESULT_REPLY: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a query call.
    pub const RESULT_QUERY: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_channel_open call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_OPEN: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_channel_connect call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CONNECT: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_channel_close call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_CHANNEL_CLOSE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_packet_receive call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_RECEIVE: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_packet_ack call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_ACK: usize = 256 * KI;
    /// Max length (in bytes) of the result data from a ibc_packet_timeout call.
    #[cfg(feature = "stargate")]
    pub const RESULT_IBC_PACKET_TIMEOUT: usize = 256 * KI;
}

pub fn call_instantiate<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_instantiate_raw(instance, &env, &info, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_INSTANTIATE)?;
    Ok(result)
}

pub fn call_execute<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    info: &MessageInfo,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let info = to_vec(info)?;
    let data = call_execute_raw(instance, &env, &info, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_EXECUTE)?;
    Ok(result)
}

pub fn call_migrate<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let data = call_migrate_raw(instance, &env, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_MIGRATE)?;
    Ok(result)
}

pub fn call_sudo<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let data = call_sudo_raw(instance, &env, msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_SUDO)?;
    Ok(result)
}

pub fn call_reply<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &Reply,
) -> VmResult<ContractResult<Response<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_reply_raw(instance, &env, &msg)?;
    let result: ContractResult<Response<U>> =
        from_slice(&data, deserialization_limits::RESULT_REPLY)?;
    Ok(result)
}

pub fn call_query<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &[u8],
) -> VmResult<ContractResult<QueryResponse>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let env = to_vec(env)?;
    let data = call_query_raw(instance, &env, msg)?;
    let result: ContractResult<QueryResponse> =
        from_slice(&data, deserialization_limits::RESULT_QUERY)?;
    // Ensure query response is valid JSON
    if let ContractResult::Ok(binary_response) = &result {
        serde_json::from_slice::<serde_json::Value>(binary_response.as_slice()).map_err(|e| {
            VmError::generic_err(format!("Query response must be valid JSON. {}", e))
        })?;
    }

    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_open<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcChannelOpenMsg,
) -> VmResult<ContractResult<Option<Ibc3ChannelOpenResponse>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_channel_open_raw(instance, &env, &msg)?;
    let result: ContractResult<Option<Ibc3ChannelOpenResponse>> =
        from_slice(&data, deserialization_limits::RESULT_IBC_CHANNEL_OPEN)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_connect<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcChannelConnectMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_channel_connect_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_CHANNEL_CONNECT)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_close<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcChannelCloseMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_channel_close_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_CHANNEL_CLOSE)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_receive<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcPacketReceiveMsg,
) -> VmResult<ContractResult<IbcReceiveResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_packet_receive_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_RECEIVE)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_ack<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcPacketAckMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_packet_ack_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_ACK)?;
    Ok(result)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_timeout<A, S, Q, U>(
    instance: &mut Instance<A, S, Q>,
    env: &Env,
    msg: &IbcPacketTimeoutMsg,
) -> VmResult<ContractResult<IbcBasicResponse<U>>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
    U: DeserializeOwned + CustomMsg,
{
    let env = to_vec(env)?;
    let msg = to_vec(msg)?;
    let data = call_ibc_packet_timeout_raw(instance, &env, &msg)?;
    let result = from_slice(&data, deserialization_limits::RESULT_IBC_PACKET_TIMEOUT)?;
    Ok(result)
}

/// Calls Wasm export "instantiate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_instantiate_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.env.set_serialized_env(env);
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "instantiate",
        &[env, info, msg],
        read_limits::RESULT_INSTANTIATE,
    )
}

/// Calls Wasm export "execute" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_execute_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    info: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.env.set_serialized_env(env);
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "execute",
        &[env, info, msg],
        read_limits::RESULT_EXECUTE,
    )
}

/// Calls Wasm export "migrate" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_migrate_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.env.set_serialized_env(env);
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "migrate",
        &[env, msg],
        read_limits::RESULT_MIGRATE,
    )
}

/// Calls Wasm export "sudo" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_sudo_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.env.set_serialized_env(env);
    instance.set_storage_readonly(false);
    call_raw(instance, "sudo", &[env, msg], read_limits::RESULT_SUDO)
}

/// Calls Wasm export "reply" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_reply_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.env.set_serialized_env(env);
    instance.set_storage_readonly(false);
    call_raw(instance, "reply", &[env, msg], read_limits::RESULT_REPLY)
}

/// Calls Wasm export "query" and returns raw data from the contract.
/// The result is length limited to prevent abuse but otherwise unchecked.
pub fn call_query_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.env.set_serialized_env(env);
    instance.set_storage_readonly(true);
    call_raw(instance, "query", &[env, msg], read_limits::RESULT_QUERY)
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_open_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_open",
        &[env, msg],
        read_limits::RESULT_IBC_CHANNEL_OPEN,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_connect_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_connect",
        &[env, msg],
        read_limits::RESULT_IBC_CHANNEL_CONNECT,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_channel_close_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_channel_close",
        &[env, msg],
        read_limits::RESULT_IBC_CHANNEL_CLOSE,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_receive_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_receive",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_RECEIVE,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_ack_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_ack",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_ACK,
    )
}

#[cfg(feature = "stargate")]
pub fn call_ibc_packet_timeout_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    env: &[u8],
    msg: &[u8],
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    instance.set_storage_readonly(false);
    call_raw(
        instance,
        "ibc_packet_timeout",
        &[env, msg],
        read_limits::RESULT_IBC_PACKET_TIMEOUT,
    )
}

/// Calls a function with the given arguments.
/// The exported function must return exactly one result (an offset to the result Region).
pub(crate) fn call_raw<A, S, Q>(
    instance: &mut Instance<A, S, Q>,
    name: &str,
    args: &[&[u8]],
    result_max_length: usize,
) -> VmResult<Vec<u8>>
where
    A: BackendApi + 'static,
    S: Storage + 'static,
    Q: Querier + 'static,
{
    let mut arg_region_ptrs = Vec::<Val>::with_capacity(args.len());
    for arg in args {
        let region_ptr = instance.allocate(arg.len())?;
        instance.write_memory(region_ptr, arg)?;
        arg_region_ptrs.push(region_ptr.into());
    }
    let result = instance.call_function1(name, &arg_region_ptrs)?;
    let res_region_ptr = ref_to_u32(&result)?;
    let data = instance.read_memory(res_region_ptr, result_max_length)?;
    // free return value in wasm (arguments were freed in wasm code)
    instance.deallocate(res_region_ptr)?;
    Ok(data)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::{mock_env, mock_info, mock_instance};
    use cosmwasm_std::{coins, Empty};

    static CONTRACT: &[u8] = include_bytes!("../testdata/hackatom.wasm");
    static EVENTS_CONTRACT: &[u8] = include_bytes!("../testdata/events.wasm");

    #[test]
    fn call_instantiate_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_execute_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // execute
        let info = mock_info("verifies", &coins(15, "earth"));
        let msg = br#"{"release":{}}"#;
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();
    }

    #[test]
    fn call_migrate_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // change the verifier via migrate
        let msg = br#"{"verifier": "someone else"}"#;
        let _res = call_migrate::<_, _, _, Empty>(&mut instance, &mock_env(), msg);

        // query the new_verifier with verifier
        let msg = br#"{"verifier":{}}"#;
        let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
        let query_response = contract_result.unwrap();
        assert_eq!(
            query_response.as_slice(),
            b"{\"verifier\":\"someone else\"}"
        );
    }

    #[test]
    fn call_query_works() {
        let mut instance = mock_instance(CONTRACT, &[]);

        // init
        let info = mock_info("creator", &coins(1000, "earth"));
        let msg = br#"{"verifier": "verifies", "beneficiary": "benefits"}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // query
        let msg = br#"{"verifier":{}}"#;
        let contract_result = call_query(&mut instance, &mock_env(), msg).unwrap();
        let query_response = contract_result.unwrap();
        assert_eq!(query_response.as_slice(), b"{\"verifier\":\"verifies\"}");
    }

    #[test]
    fn add_event_works() {
        let mut instance = mock_instance(EVENTS_CONTRACT, &[]);

        // init
        let info = mock_info("creator", &[]);
        let msg = br#"{}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // execute
        let ty = "type";
        let key = "foo";
        let value = "Alice";
        let msg = format!(
            r#"{{"event":{{"type":"{}","attributes":[{{"key":"{}","value":"{}"}}]}}}}"#,
            ty, key, value
        );
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        let (events, attributes) = instance.get_events_attributes();
        assert_eq!(events.len(), 1);
        assert_eq!(events[0].ty, ty);
        assert_eq!(events[0].attributes.len(), 1);
        assert_eq!(events[0].attributes[0].key, key);
        assert_eq!(events[0].attributes[0].value, value);
        assert_eq!(attributes.len(), 0);
    }

    #[test]
    fn add_events_works() {
        let mut instance = mock_instance(EVENTS_CONTRACT, &[]);

        // init
        let info = mock_info("creator", &[]);
        let msg = br#"{}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // execute
        let ty1 = "type1";
        let key1 = "foo";
        let value1 = "Alice";
        let ty2 = "type2";
        let key2 = "bar";
        let value2 = "Bob";
        let msg = format!(
            r#"{{"events":{{"events":[{{"type":"{}", "attributes":[{{"key":"{}","value":"{}"}}]}},{{"type":"{}", "attributes":[{{"key":"{}","value":"{}"}}]}}]}}}}"#,
            ty1, key1, value1, ty2, key2, value2
        );
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        let (events, attributes) = instance.get_events_attributes();
        assert_eq!(events.len(), 2);
        assert_eq!(events[0].ty, ty1);
        assert_eq!(events[0].attributes.len(), 1);
        assert_eq!(events[0].attributes[0].key, key1);
        assert_eq!(events[0].attributes[0].value, value1);
        assert_eq!(events[1].ty, ty2);
        assert_eq!(events[1].attributes.len(), 1);
        assert_eq!(events[1].attributes[0].key, key2);
        assert_eq!(events[1].attributes[0].value, value2);
        assert_eq!(attributes.len(), 0);
    }

    #[test]
    fn add_attribute_works() {
        let mut instance = mock_instance(EVENTS_CONTRACT, &[]);

        // init
        let info = mock_info("creator", &[]);
        let msg = br#"{}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // execute
        let key = "foo";
        let value = "Alice";
        let msg = format!(r#"{{"attribute":{{"key":"{}","value":"{}"}}}}"#, key, value);
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        let (events, attributes) = instance.get_events_attributes();
        assert_eq!(events.len(), 0);
        assert_eq!(attributes.len(), 1);
        assert_eq!(attributes[0].key, key);
        assert_eq!(attributes[0].value, value);
    }

    #[test]
    fn add_attributes_works() {
        let mut instance = mock_instance(EVENTS_CONTRACT, &[]);

        // init
        let info = mock_info("creator", &[]);
        let msg = br#"{}"#;
        call_instantiate::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg)
            .unwrap()
            .unwrap();

        // execute
        let key1 = "foo";
        let value1 = "Alice";
        let key2 = "bar";
        let value2 = "Bob";
        let msg = format!(
            r#"{{"attributes":{{"attributes":[{{"key":"{}","value":"{}"}},{{"key":"{}","value":"{}"}}]}}}}"#,
            key1, value1, key2, value2
        );
        call_execute::<_, _, _, Empty>(&mut instance, &mock_env(), &info, msg.as_bytes())
            .unwrap()
            .unwrap();

        let (events, attributes) = instance.get_events_attributes();
        assert_eq!(events.len(), 0);
        assert_eq!(attributes.len(), 2);
        assert_eq!(attributes[0].key, key1);
        assert_eq!(attributes[0].value, value1);
        assert_eq!(attributes[1].key, key2);
        assert_eq!(attributes[1].value, value2);
    }

    #[cfg(feature = "stargate")]
    mod ibc {
        use super::*;
        use crate::calls::{call_instantiate, call_reply};
        use crate::testing::{
            mock_env, mock_info, mock_instance, MockApi, MockQuerier, MockStorage,
        };
        use cosmwasm_std::testing::{
            mock_ibc_channel_close_init, mock_ibc_channel_connect_ack, mock_ibc_channel_open_init,
            mock_ibc_packet_ack, mock_ibc_packet_recv, mock_ibc_packet_timeout, mock_wasmd_attr,
        };
        use cosmwasm_std::{
            Empty, Event, IbcAcknowledgement, IbcOrder, Reply, ReplyOn, SubMsgResponse,
            SubMsgResult,
        };
        static CONTRACT: &[u8] = include_bytes!("../testdata/ibc_reflect.wasm");
        const IBC_VERSION: &str = "ibc-reflect-v1";
        fn setup(
            instance: &mut Instance<MockApi, MockStorage, MockQuerier>,
            channel_id: &str,
            account: &str,
        ) {
            // init
            let info = mock_info("creator", &[]);
            let msg = br#"{"reflect_code_id":77}"#;
            call_instantiate::<_, _, _, Empty>(instance, &mock_env(), &info, msg)
                .unwrap()
                .unwrap();
            // first we try to open with a valid handshake
            let handshake_open =
                mock_ibc_channel_open_init(channel_id, IbcOrder::Ordered, IBC_VERSION);
            call_ibc_channel_open(instance, &mock_env(), &handshake_open)
                .unwrap()
                .unwrap();
            // then we connect (with counter-party version set)
            let handshake_connect =
                mock_ibc_channel_connect_ack(channel_id, IbcOrder::Ordered, IBC_VERSION);
            let res: IbcBasicResponse = call_ibc_channel_connect::<_, _, _, Empty>(
                instance,
                &mock_env(),
                &handshake_connect,
            )
            .unwrap()
            .unwrap();
            assert_eq!(1, res.messages.len());
            assert_eq!(
                res.events,
                [Event::new("ibc").add_attribute("channel", "connect")]
            );
            assert_eq!(ReplyOn::Success, res.messages[0].reply_on);
            let id = res.messages[0].id;
            let event = Event::new("instantiate").add_attributes(vec![
                // We have to force this one to avoid the debug assertion against _
                mock_wasmd_attr("_contract_address", account),
            ]);
            // which creates a reflect account. here we get the callback
            let response = Reply {
                id,
                result: SubMsgResult::Ok(SubMsgResponse {
                    events: vec![event],
                    data: None,
                }),
            };
            call_reply::<_, _, _, Empty>(instance, &mock_env(), &response).unwrap();
        }
        const CHANNEL_ID: &str = "channel-123";
        const ACCOUNT: &str = "account-456";
        #[test]
        fn call_ibc_channel_open_and_connect_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
        }
        #[test]
        fn call_ibc_channel_close_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let handshake_close =
                mock_ibc_channel_close_init(CHANNEL_ID, IbcOrder::Ordered, IBC_VERSION);
            call_ibc_channel_close::<_, _, _, Empty>(&mut instance, &mock_env(), &handshake_close)
                .unwrap()
                .unwrap();
        }
        #[test]
        fn call_ibc_packet_ack_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let ack = IbcAcknowledgement::new(br#"{}"#);
            let msg = mock_ibc_packet_ack(CHANNEL_ID, br#"{}"#, ack).unwrap();
            call_ibc_packet_ack::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
        }
        #[test]
        fn call_ibc_packet_timeout_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let msg = mock_ibc_packet_timeout(CHANNEL_ID, br#"{}"#).unwrap();
            call_ibc_packet_timeout::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
        }
        #[test]
        fn call_ibc_packet_receive_works() {
            let mut instance = mock_instance(CONTRACT, &[]);
            setup(&mut instance, CHANNEL_ID, ACCOUNT);
            let who_am_i = br#"{"who_am_i":{}}"#;
            let msg = mock_ibc_packet_recv(CHANNEL_ID, who_am_i).unwrap();
            call_ibc_packet_receive::<_, _, _, Empty>(&mut instance, &mock_env(), &msg)
                .unwrap()
                .unwrap();
        }
    }
}
