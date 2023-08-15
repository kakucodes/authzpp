use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Coin, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

pub type ActiveGrantsResponse = Vec<AllowlistSendSettings>;

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

    /// Returns the grant information for the given grant.
    #[returns(Option<AllowlistSendSettings>)]
    Grant { granter: String, receiver: String },
    // /// Returns the amounts that the delegator and taxation address will receive if the execute function is called
    // #[returns(SimulateExecuteResponse)]
    // SimulateExecute(ExecuteSettings),

    #[returns(Vec<GrantSpec>)]
    QueryRequiredGrants {}
}

// #[cw_serde]
// pub struct SimulateExecuteResponse {
//     /// rewards that the granter will receive
//     pub delegator_rewards: Vec<Coin>,
//     /// rewards that the taxation address will receive
//     pub taxation_address_rewards: Vec<Coin>,
// }

#[cw_serde]
pub struct VersionResponse {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Creates a new grant that allows sending of tokens to an allow list of addresses
    Grant(AllowlistSendSettings),

    /// Revokes an existing grant for the griven send_to_address so that it can no longer be used
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
pub struct AllowlistSendSettings {
    /// the address that authorized use of their funds to the given address
    pub granter: String,
    /// the address that is allowed to execute the send on behalf of the granter
    pub grantee: String,
    /// address to recieve the tokens from the gran the given percentage of rewards to
    pub receiver: String,
    /// expiration date of the grant as a timestamp
    pub expiration: Timestamp,
}
