#[cfg(feature = "backtraces")]
use std::backtrace::Backtrace;
use std::fmt::Debug;
use thiserror::Error;

#[cfg(not(target_arch = "wasm32"))]
use cosmwasm_crypto::CryptoError;

#[derive(Error, Debug)]
pub enum HashCalculationError {
    #[error("Inputs are larger than supported")]
    InputsTooLarger,
    #[error("Input is longer than supported")]
    InputTooLonger,
    #[error("Unknown error: {error_code}")]
    UnknownErr {
        error_code: u32,
        #[cfg(feature = "backtraces")]
        backtrace: Backtrace,
    },
}

impl HashCalculationError {
    pub fn unknown_err(error_code: u32) -> Self {
        HashCalculationError::UnknownErr {
            error_code,
            #[cfg(feature = "backtraces")]
            backtrace: Backtrace::capture(),
        }
    }
}

impl PartialEq<HashCalculationError> for HashCalculationError {
    fn eq(&self, rhs: &HashCalculationError) -> bool {
        match self {
            HashCalculationError::InputsTooLarger => {
                matches!(rhs, HashCalculationError::InputsTooLarger)
            }
            HashCalculationError::InputTooLonger => {
                matches!(rhs, HashCalculationError::InputTooLonger)
            }
            HashCalculationError::UnknownErr { error_code, .. } => {
                if let HashCalculationError::UnknownErr {
                    error_code: rhs_error_code,
                    ..
                } = rhs
                {
                    error_code == rhs_error_code
                } else {
                    false
                }
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl From<CryptoError> for HashCalculationError {
    fn from(original: CryptoError) -> Self {
        match original {
            CryptoError::InputsTooLarger { .. } => HashCalculationError::InputsTooLarger,
            CryptoError::InputTooLong { .. } => HashCalculationError::InputTooLonger,
            CryptoError::InvalidHashFormat { .. }
            | CryptoError::InvalidPubkeyFormat { .. }
            | CryptoError::InvalidSignatureFormat { .. }
            | CryptoError::GenericErr { .. }
            | CryptoError::InvalidRecoveryParam { .. }
            | CryptoError::BatchErr { .. } => panic!("Conversion not supported"),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    // constructors
    #[test]
    fn unknown_err_works() {
        let error = HashCalculationError::unknown_err(123);
        match error {
            HashCalculationError::UnknownErr { error_code, .. } => assert_eq!(error_code, 123),
            _ => panic!("wrong error type!"),
        }
    }
}
