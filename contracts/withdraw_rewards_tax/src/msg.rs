use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

pub type ActiveGrantsByGranteeResponse = Vec<GrantQueryResponse>;

pub type ActiveGrantsByDelegatorResponse = Option<GrantQueryResponse>;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VersionResponse)]
    Version {},

    /// Returns the grant information for the given delegator.
    /// Will either return the active grant info or nothing if there is no active grant.
    #[returns(ActiveGrantsByDelegatorResponse)]
    ActiveGrantsByDelegator(String),

    /// Returns the grant information for the given grantee.
    /// Will return a list of all grants that the grantee has access to.
    #[returns(ActiveGrantsByGranteeResponse)]
    ActiveGrantsByGrantee(String),

    /// Returns the amounts that the delegator and taxation address will receive if the execute function is called
    #[returns(SimulateExecuteResponse)]
    SimulateExecute(ExecuteSettings),
}

#[cw_serde]
pub struct SimulateExecuteResponse {
    /// rewards that the granter will receive
    pub delegator_rewards: Vec<Coin>,
    /// rewards that the taxation address will receive
    pub taxation_address_rewards: Vec<Coin>,
}

#[cw_serde]
pub struct GrantQueryResponse {
    pub delegator_addr: Addr,
    /// grant settings
    pub allowed_withdrawls: AllowedWithdrawlSettings,
}

#[cw_serde]
pub struct VersionResponse {
    pub version: String,
}

#[cw_serde]
pub enum ExecuteMsg {
    /// Creates a new grant that allows portions of one's staking rewards to be claimed by other addresses
    Grant(AllowedWithdrawlSettings),

    /// Revokes an existing grant so that it can no longer be used
    Revoke(),

    /// Withdraws a user's rewards while sending the granted address a, specified, portion of the rewards
    Execute(ExecuteSettings),

    /// Prunes expired grants from state
    /// This function should be called periodically to clean up free up contract space and
    PruneExpiredGrants(),
}

#[cw_serde]
pub struct ExecuteSettings {
    /// originating delegator address to withdraw the rewards for
    pub delegator: String,
    /// the percentage of rewards to be shared. if none is specified, the max is used
    pub percentage: Option<Decimal>,
}

#[cw_serde]
pub struct AllowedWithdrawlSettings {
    /// the address that is allowed to execute the withdraw function
    pub grantee: String,
    /// address to withdraw the given percentage of rewards to
    pub taxation_address: String,
    /// percentage of rewards that can be withdrawn to the given address
    pub max_fee_percentage: Decimal,
    /// expiration date of the grant as a unix timestamp
    pub expiration: Timestamp,
}
