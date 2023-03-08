use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Overflow Error")]
    Overflow,
    #[error("Storage does not have the number")]
    StorageError,
}
