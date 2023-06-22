use cosmos_sdk_proto::prost::{DecodeError, EncodeError};
use cosmwasm_std::StdError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ContractError {
    #[error("{0}")]
    Std(#[from] StdError),

    #[error("{0}")]
    EncodeError(#[from] EncodeError),

    #[error("Semver parsing error: {0}")]
    SemVer(String),

    #[error("Decode Error: {0}. This is likely from a failing stargate query.")]
    Decode(#[from] DecodeError),

    #[error("No pending rewards for {0}")]
    NoPendingRewards(String),

    #[error("Target Not Implemented")]
    NotImplemented {},

    #[error("Unauthorized")]
    Unauthorized {},

    #[error("{0} is not a valid delegator/granter address.")]
    InvalidGranterAddress(String),

    #[error("{0} is not a valid grantee address.")]
    InvalidGranteeAddress(String),

    #[error("{0} is not a valid withdraw share address.")]
    InvalidWithdrawShareAddress(String),

    #[error("Falied to query pending rewards.")]
    QueryPendingRewardsFailure,

    #[error("No active grant for {0}")]
    NoActiveGrant(String),
}

impl From<semver::Error> for ContractError {
    fn from(err: semver::Error) -> Self {
        Self::SemVer(err.to_string())
    }
}
