#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt;
use thiserror::Error;

use crate::errors::{RecoverPubkeyError, VerificationError};

/// Structured error type for init, execute and query.
///
/// This can be serialized and passed over the Wasm/VM boundary, which allows us to use structured
/// error types in e.g. integration tests. In that process backtraces are stripped off.
///
/// The prefix "Std" means "the standard error within the standard library". This is not the only
/// result/error type in cosmwasm-std.
///
/// When new cases are added, they should describe the problem rather than what was attempted (e.g.
/// InvalidBase64 is preferred over Base64DecodingErr). In the long run this allows us to get rid of
/// the duplication in "StdError::FooErr".
///
/// Checklist for adding a new error:
/// - Add enum case
/// - Add creator function in std_error_helpers.rs
#[derive(Error, Debug)]
pub enum StdError {
    #[error("Verification error: {source}")]
    VerificationErr {
        source: VerificationError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Recover pubkey error: {source}")]
    RecoverPubkeyErr {
        source: RecoverPubkeyError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    /// Whenever there is no specific error type available
    #[error("Generic error: {msg}")]
    GenericErr {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Invalid Base64 string: {msg}")]
    InvalidBase64 {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Invalid data size: expected={expected} actual={actual}")]
    InvalidDataSize {
        expected: u64,
        actual: u64,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    /// Whenever UTF-8 bytes cannot be decoded into a unicode string, e.g. in String::from_utf8 or str::from_utf8.
    #[error("Cannot decode UTF8 bytes into string: {msg}")]
    InvalidUtf8 {
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("{kind} not found")]
    NotFound {
        kind: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error parsing into type {target_type}: {msg}")]
    ParseErr {
        /// the target type that was attempted
        target_type: String,
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Error serializing type {source_type}: {msg}")]
    SerializeErr {
        /// the source type that was attempted
        source_type: String,
        msg: String,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Overflow: {source}")]
    Overflow {
        source: OverflowError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
    #[error("Divide by zero: {source}")]
    DivideByZero {
        source: DivideByZeroError,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl StdError {
    pub fn verification_err(source: VerificationError) -> Self {
        StdError::VerificationErr {
            source,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn recover_pubkey_err(source: RecoverPubkeyError) -> Self {
        StdError::RecoverPubkeyErr {
            source,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn generic_err<S: Into<String>>(msg: S) -> Self {
        StdError::GenericErr {
            msg: msg.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_base64<S: ToString>(msg: S) -> Self {
        StdError::InvalidBase64 {
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_data_size(expected: usize, actual: usize) -> Self {
        StdError::InvalidDataSize {
            // Cast is safe because usize is 32 or 64 bit large in all environments we support
            expected: expected as u64,
            actual: actual as u64,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn invalid_utf8<S: ToString>(msg: S) -> Self {
        StdError::InvalidUtf8 {
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn not_found<S: Into<String>>(kind: S) -> Self {
        StdError::NotFound {
            kind: kind.into(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn parse_err<T: Into<String>, M: ToString>(target: T, msg: M) -> Self {
        StdError::ParseErr {
            target_type: target.into(),
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn serialize_err<S: Into<String>, M: ToString>(source: S, msg: M) -> Self {
        StdError::SerializeErr {
            source_type: source.into(),
            msg: msg.to_string(),
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn overflow(source: OverflowError) -> Self {
        StdError::Overflow {
            source,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }

    pub fn divide_by_zero(source: DivideByZeroError) -> Self {
        StdError::DivideByZero {
            source,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl PartialEq<StdError> for StdError {
    fn eq(&self, rhs: &StdError) -> bool {
        match self {
            StdError::VerificationErr {
                source,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::VerificationErr {
                    source: rhs_source,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::RecoverPubkeyErr {
                source,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::RecoverPubkeyErr {
                    source: rhs_source,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::GenericErr {
                msg,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::GenericErr {
                    msg: rhs_msg,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::InvalidBase64 {
                msg,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::InvalidBase64 {
                    msg: rhs_msg,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::InvalidDataSize {
                expected,
                actual,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::InvalidDataSize {
                    expected: rhs_expected,
                    actual: rhs_actual,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    expected == rhs_expected && actual == rhs_actual
                } else {
                    false
                }
            }
            StdError::InvalidUtf8 {
                msg,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::InvalidUtf8 {
                    msg: rhs_msg,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::NotFound {
                kind,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::NotFound {
                    kind: rhs_kind,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    kind == rhs_kind
                } else {
                    false
                }
            }
            StdError::ParseErr {
                target_type,
                msg,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::ParseErr {
                    target_type: rhs_target_type,
                    msg: rhs_msg,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    target_type == rhs_target_type && msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::SerializeErr {
                source_type,
                msg,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::SerializeErr {
                    source_type: rhs_source_type,
                    msg: rhs_msg,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    source_type == rhs_source_type && msg == rhs_msg
                } else {
                    false
                }
            }
            StdError::Overflow {
                source,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::Overflow {
                    source: rhs_source,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
            StdError::DivideByZero {
                source,
                #[cfg(feature = "backtraces")]
                    backtrace: _,
            } => {
                if let StdError::DivideByZero {
                    source: rhs_source,
                    #[cfg(feature = "backtraces")]
                        backtrace: _,
                } = rhs
                {
                    source == rhs_source
                } else {
                    false
                }
            }
        }
    }
}

impl From<std::str::Utf8Error> for StdError {
    fn from(source: std::str::Utf8Error) -> Self {
        Self::invalid_utf8(source)
    }
}

impl From<std::string::FromUtf8Error> for StdError {
    fn from(source: std::string::FromUtf8Error) -> Self {
        Self::invalid_utf8(source)
    }
}

impl From<VerificationError> for StdError {
    fn from(source: VerificationError) -> Self {
        Self::verification_err(source)
    }
}

impl From<RecoverPubkeyError> for StdError {
    fn from(source: RecoverPubkeyError) -> Self {
        Self::recover_pubkey_err(source)
    }
}

impl From<OverflowError> for StdError {
    fn from(source: OverflowError) -> Self {
        Self::overflow(source)
    }
}

impl From<DivideByZeroError> for StdError {
    fn from(source: DivideByZeroError) -> Self {
        Self::divide_by_zero(source)
    }
}

/// The return type for init, execute and query. Since the error type cannot be serialized to JSON,
/// this is only available within the contract and its unit tests.
///
/// The prefix "Std" means "the standard result within the standard library". This is not the only
/// result/error type in cosmwasm-std.
pub type StdResult<T> = core::result::Result<T, StdError>;

#[derive(Error, Debug, PartialEq, Eq)]
pub enum OverflowOperation {
    Add,
    Sub,
    Mul,
    Pow,
}

impl fmt::Display for OverflowOperation {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self)
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Cannot {operation} with {operand1} and {operand2}")]
pub struct OverflowError {
    pub operation: OverflowOperation,
    pub operand1: String,
    pub operand2: String,
}

impl OverflowError {
    pub fn new<U: ToString>(operation: OverflowOperation, operand1: U, operand2: U) -> Self {
        Self {
            operation,
            operand1: operand1.to_string(),
            operand2: operand2.to_string(),
        }
    }
}

#[derive(Error, Debug, PartialEq, Eq)]
#[error("Cannot devide {operand} by zero")]
pub struct DivideByZeroError {
    pub operand: String,
}

impl DivideByZeroError {
    pub fn new<U: ToString>(operand: U) -> Self {
        Self {
            operand: operand.to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::str;

    // constructors

    // example of reporting contract errors with format!
    #[test]
    fn generic_err_owned() {
        let guess = 7;
        let error = StdError::generic_err(format!("{} is too low", guess));
        match error {
            StdError::GenericErr { msg, .. } => {
                assert_eq!(msg, String::from("7 is too low"));
            }
            e => panic!("unexpected error, {:?}", e),
        }
    }

    // example of reporting static contract errors
    #[test]
    fn generic_err_ref() {
        let error = StdError::generic_err("not implemented");
        match error {
            StdError::GenericErr { msg, .. } => assert_eq!(msg, "not implemented"),
            e => panic!("unexpected error, {:?}", e),
        }
    }

    #[test]
    fn invalid_base64_works_for_strings() {
        let error = StdError::invalid_base64("my text");
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_base64_works_for_errors() {
        let original = base64::DecodeError::InvalidLength;
        let error = StdError::invalid_base64(original);
        match error {
            StdError::InvalidBase64 { msg, .. } => {
                assert_eq!(msg, "Encoded text cannot have a 6-bit remainder.");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_data_size_works() {
        let error = StdError::invalid_data_size(31, 14);
        match error {
            StdError::InvalidDataSize {
                expected, actual, ..
            } => {
                assert_eq!(expected, 31);
                assert_eq!(actual, 14);
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_strings() {
        let error = StdError::invalid_utf8("my text");
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "my text");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn invalid_utf8_works_for_errors() {
        let original = String::from_utf8(vec![0x80]).unwrap_err();
        let error = StdError::invalid_utf8(original);
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 1 bytes from index 0");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn not_found_works() {
        let error = StdError::not_found("gold");
        match error {
            StdError::NotFound { kind, .. } => assert_eq!(kind, "gold"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn parse_err_works() {
        let error = StdError::parse_err("Book", "Missing field: title");
        match error {
            StdError::ParseErr {
                target_type, msg, ..
            } => {
                assert_eq!(target_type, "Book");
                assert_eq!(msg, "Missing field: title");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn serialize_err_works() {
        let error = StdError::serialize_err("Book", "Content too long");
        match error {
            StdError::SerializeErr {
                source_type, msg, ..
            } => {
                assert_eq!(source_type, "Book");
                assert_eq!(msg, "Content too long");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn underflow_works_for_u128() {
        let error =
            StdError::overflow(OverflowError::new(OverflowOperation::Sub, 123u128, 456u128));
        match error {
            StdError::Overflow {
                source:
                    OverflowError {
                        operation,
                        operand1,
                        operand2,
                    },
                ..
            } => {
                assert_eq!(operation, OverflowOperation::Sub);
                assert_eq!(operand1, "123");
                assert_eq!(operand2, "456");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn overflow_works_for_i64() {
        let error = StdError::overflow(OverflowError::new(OverflowOperation::Sub, 777i64, 1234i64));
        match error {
            StdError::Overflow {
                source:
                    OverflowError {
                        operation,
                        operand1,
                        operand2,
                    },
                ..
            } => {
                assert_eq!(operation, OverflowOperation::Sub);
                assert_eq!(operand1, "777");
                assert_eq!(operand2, "1234");
            }
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn divide_by_zero_works() {
        let error = StdError::divide_by_zero(DivideByZeroError::new(123u128));
        match error {
            StdError::DivideByZero {
                source: DivideByZeroError { operand },
                ..
            } => assert_eq!(operand, "123"),
            _ => panic!("expect different error"),
        }
    }

    #[test]
    fn implements_debug() {
        let error: StdError = StdError::from(OverflowError::new(OverflowOperation::Sub, 3, 5));
        let embedded = format!("Debug: {:?}", error);
        assert_eq!(
            embedded,
            r#"Debug: Overflow { source: OverflowError { operation: Sub, operand1: "3", operand2: "5" } }"#
        );
    }

    #[test]
    fn implements_display() {
        let error: StdError = StdError::from(OverflowError::new(OverflowOperation::Sub, 3, 5));
        let embedded = format!("Display: {}", error);
        assert_eq!(embedded, "Display: Overflow: Cannot Sub with 3 and 5");
    }

    #[test]
    fn implements_partial_eq() {
        let u1 = StdError::from(OverflowError::new(OverflowOperation::Sub, 3, 5));
        let u2 = StdError::from(OverflowError::new(OverflowOperation::Sub, 3, 5));
        let u3 = StdError::from(OverflowError::new(OverflowOperation::Sub, 3, 7));
        let s1 = StdError::serialize_err("Book", "Content too long");
        let s2 = StdError::serialize_err("Book", "Content too long");
        let s3 = StdError::serialize_err("Book", "Title too long");
        assert_eq!(u1, u2);
        assert_ne!(u1, u3);
        assert_ne!(u1, s1);
        assert_eq!(s1, s2);
        assert_ne!(s1, s3);
    }

    #[test]
    fn from_std_str_utf8error_works() {
        let error: StdError = str::from_utf8(b"Hello \xF0\x90\x80World")
            .unwrap_err()
            .into();
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }

    #[test]
    fn from_std_string_fromutf8error_works() {
        let error: StdError = String::from_utf8(b"Hello \xF0\x90\x80World".to_vec())
            .unwrap_err()
            .into();
        match error {
            StdError::InvalidUtf8 { msg, .. } => {
                assert_eq!(msg, "invalid utf-8 sequence of 3 bytes from index 6")
            }
            err => panic!("Unexpected error: {:?}", err),
        }
    }
}
