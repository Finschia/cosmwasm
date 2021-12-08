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
