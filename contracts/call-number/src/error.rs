use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),
    #[error("Overflow Error")]
    Overflow,
    #[error("Cannot get callee address of the Storage")]
    CannotGetAddress,
}
