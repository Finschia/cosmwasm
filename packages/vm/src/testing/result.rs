#![allow(dead_code)] // Disable unused code lint with the code used in https://github.com/line/cosmwasm-simulator.

use crate::VmError;

#[derive(Debug)]
pub enum TestingError {
    VmError(VmError),
    ContractError(String),
}

pub type TestingResult<T> = std::result::Result<T, TestingError>;

impl From<VmError> for TestingError {
    fn from(error: VmError) -> Self {
        TestingError::VmError(error)
    }
}
