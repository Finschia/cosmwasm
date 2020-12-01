use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fmt;

/// This is the final result type that is created and serialized in a contract for
/// every init/handle/migrate call. The VM then deserializes this type to distinguish
/// between successful and failed executions.
///
/// We use a custom type here instead of Rust's Result because we want to be able to
/// define the serialization, which is a public interface. Every language that compiles
/// to Wasm and runs in the ComsWasm VM needs to create the same JSON representation.
///
/// # Examples
///
/// Success:
///
/// ```
/// # use cosmwasm_std::{to_vec, ContractResult, HandleResponse};
/// let response: HandleResponse = HandleResponse::default();
/// let result: ContractResult<HandleResponse> = ContractResult::Ok(response);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"ok":{"messages":[],"attributes":[],"data":null}}"#.to_vec());
/// ```
///
/// Failure:
///
/// ```
/// # use cosmwasm_std::{to_vec, ContractResult, HandleResponse};
/// let error_msg = String::from("Something went wrong");
/// let result: ContractResult<HandleResponse> = ContractResult::Err(error_msg);
/// assert_eq!(to_vec(&result).unwrap(), br#"{"error":"Something went wrong"}"#.to_vec());
/// ```
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq, JsonSchema)]
#[serde(rename_all = "snake_case")]
pub enum ContractResult<S> {
    Ok(S),
    /// An error type that every custom error created by contract developers can be converted to.
    /// This could potientially have more structure, but String is the easiest.
    #[serde(rename = "error")]
    Err(String),
}

// Implementations here mimic the Result API and should be implemented via a conversion to Result
// to ensure API consistency
impl<S> ContractResult<S> {
    /// Converts a `ContractResult<S>` to a `Result<S, String>` as a convenient way
    /// to access the full Result API.
    pub fn into_result(self) -> Result<S, String> {
        Result::<S, String>::from(self)
    }

    pub fn unwrap(self) -> S {
        self.into_result().unwrap()
    }
}

impl<S: fmt::Debug> ContractResult<S> {
    pub fn unwrap_err(self) -> String {
        self.into_result().unwrap_err()
    }
}

impl<S, E: ToString> From<Result<S, E>> for ContractResult<S> {
    fn from(original: Result<S, E>) -> ContractResult<S> {
        match original {
            Ok(value) => ContractResult::Ok(value),
            Err(err) => ContractResult::Err(err.to_string()),
        }
    }
}

impl<S> From<ContractResult<S>> for Result<S, String> {
    fn from(original: ContractResult<S>) -> Result<S, String> {
        match original {
            ContractResult::Ok(value) => Ok(value),
            ContractResult::Err(err) => Err(err),
        }
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::{from_slice, to_vec, HandleResponse, HandleResult, StdError, StdResult};

    #[test]
    fn contract_result_serialization_works() {
        let result = ContractResult::Ok(12);
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":12}");

        let result = ContractResult::Ok("foo");
        assert_eq!(&to_vec(&result).unwrap(), b"{\"ok\":\"foo\"}");

        let result: ContractResult<HandleResponse> = ContractResult::Ok(HandleResponse::default());
        assert_eq!(
            to_vec(&result).unwrap(),
            br#"{"ok":{"messages":[],"attributes":[],"data":null}}"#.to_vec()
        );

        let result: ContractResult<HandleResponse> = ContractResult::Err("broken".to_string());
        assert_eq!(&to_vec(&result).unwrap(), b"{\"error\":\"broken\"}");
    }

    #[test]
    fn contract_result_deserialization_works() {
        let result: ContractResult<u64> = from_slice(br#"{"ok":12}"#).unwrap();
        assert_eq!(result, ContractResult::Ok(12));

        let result: ContractResult<String> = from_slice(br#"{"ok":"foo"}"#).unwrap();
        assert_eq!(result, ContractResult::Ok("foo".to_string()));

        let result: ContractResult<HandleResponse> =
            from_slice(br#"{"ok":{"messages":[],"attributes":[],"data":null}}"#).unwrap();
        assert_eq!(result, ContractResult::Ok(HandleResponse::default()));

        let result: ContractResult<HandleResponse> = from_slice(br#"{"error":"broken"}"#).unwrap();
        assert_eq!(result, ContractResult::Err("broken".to_string()));

        // ignores whitespace
        let result: ContractResult<u64> = from_slice(b" {\n\t  \"ok\": 5898\n}  ").unwrap();
        assert_eq!(result, ContractResult::Ok(5898));

        // fails for additional attributes
        let parse: StdResult<ContractResult<u64>> = from_slice(br#"{"unrelated":321,"ok":4554}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<ContractResult<u64>> = from_slice(br#"{"ok":4554,"unrelated":321}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
        let parse: StdResult<ContractResult<u64>> =
            from_slice(br#"{"ok":4554,"error":"What's up now?"}"#);
        match parse.unwrap_err() {
            StdError::ParseErr { .. } => {}
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn can_convert_from_core_result() {
        let original: HandleResult = Ok(HandleResponse::default());
        let converted: ContractResult<HandleResponse> = original.into();
        assert_eq!(converted, ContractResult::Ok(HandleResponse::default()));

        let original: HandleResult = Err(StdError::generic_err("broken"));
        let converted: ContractResult<HandleResponse> = original.into();
        assert_eq!(
            converted,
            ContractResult::Err("Generic error: broken".to_string())
        );
    }

    #[test]
    fn can_convert_to_core_result() {
        let original = ContractResult::Ok(HandleResponse::default());
        let converted: Result<HandleResponse, String> = original.into();
        assert_eq!(converted, Ok(HandleResponse::default()));

        let original = ContractResult::Err("went wrong".to_string());
        let converted: Result<HandleResponse, String> = original.into();
        assert_eq!(converted, Err("went wrong".to_string()));
    }
}
