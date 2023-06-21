use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

pub type ActiveGrantsResponse = Vec<AllowedDenomsSendSettings>;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VersionResponse)]
    Version {},

    /// Returns the active grants for the granter.
    #[returns(ActiveGrantsResponse)]
    ActiveGrantsByGranter(String),

    /// Returns the grants for a given grantee.
    /// Will return a list of all grants that the grantee has access to.
    #[returns(ActiveGrantsResponse)]
    ActiveGrantsByGrantee(String),

    #[returns(Option<AllowedDenomsSendSettings>)]
    Grant { granter: String, grantee: String },
    // /// Returns the amounts that the delegator and taxation address will receive if the execute function is called
    // #[returns(SimulateExecuteResponse)]
    // SimulateExecute(ExecuteSettings),
}

#[cw_serde]
pub struct VersionResponse {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Creates a new grant that allows sending of tokens to an allow list of addresses
    Grant(AllowedDenomsSendSettings),

    /// Revokes an existing grant for the sender
    Revoke(String),

    /// Sends tokens to a given address if the grantee is allowed to do so
    Execute(ExecuteSettings),

    /// Processes the execute function without broadcasting the transaction.
    /// This will update state and return the amount that would be sent to the given address.
    /// Only expected to be used by a composition contract.
    ProcessExecuteWithoutBroadcast(ExecuteSettings),

    /// Prunes expired grants from state
    /// This function should be called periodically to clean up free up contract space and
    PruneExpiredGrants(),
}

#[cw_serde]
pub struct ExecuteSettings {
    /// wallet/address that has granted access to their funds
    pub granter: String,
    /// the address that is allowed to execute the send on behalf of the granter
    pub grantee: String,
    /// the tokens to send
    pub amount: Vec<Coin>,
    /// address to recieve the tokens from the granter
    pub receiver: String,
}

#[cw_serde]
pub struct AllowedDenomsSendSettings {
    /// the address that is allowed to execute the send on behalf of the granter
    pub grantee: String,
    /// allowed denoms
    pub allowed_denoms: Vec<String>,
    /// expiration date of the grant as a timestamp
    pub expiration: Timestamp,
}
