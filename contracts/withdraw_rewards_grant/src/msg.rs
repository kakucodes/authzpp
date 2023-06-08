use cosmwasm_schema::{cw_serde, QueryResponses};
use cosmwasm_std::{Addr, Coin, Decimal, Timestamp};

#[cw_serde]
pub struct InstantiateMsg {}

#[cw_serde]
pub struct MigrateMsg {}

pub type ActiveGrantsByGranteeResponse = Vec<GrantQueryResponse>;

pub type ActiveGrantsByGranterResponse = Option<GrantQueryResponse>;

#[cw_serde]
#[derive(QueryResponses)]
pub enum QueryMsg {
    #[returns(VersionResponse)]
    Version {},

    /// Returns the grant information for the given granter.
    /// Will either return the active grant info or nothing if there is no active grant.
    #[returns(ActiveGrantsByGranterResponse)]
    ActiveGrantsByGranter(String),

    /// Returns the grant information for the given grantee.
    /// Will return a list of all grants that the grantee has access to.
    #[returns(ActiveGrantsByGranteeResponse)]
    ActiveGrantsByGrantee(String),

    /// Returns the amounts that the granter and withdraw address will receive if the execute function is called
    #[returns(SimulateExecuteResponse)]
    SimulateExecute(ExecuteSettings),
}

#[cw_serde]
pub struct SimulateExecuteResponse {
    /// rewards that the granter will receive
    pub granter_rewards: Vec<Coin>,
    /// rewards that the withdraw/fee address will receive
    pub withdraw_address_rewards: Vec<Coin>,
}

#[cw_serde]
pub struct GrantQueryResponse {
    pub granter: Addr,
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

    /// Withdraws a user's rewards to the address that was granted access to them and the remainder to the grantee's address
    Execute(ExecuteSettings),
}

#[cw_serde]
pub struct ExecuteSettings {
    /// address to withdraw the rewards for
    pub granter: String,
    /// the percentage of rewards to be withdrawn to the grantee. if none is specified, the max is used
    pub percentage: Option<Decimal>,
}

#[cw_serde]
pub struct AllowedWithdrawlSettings {
    /// the address that is allowed to execute the withdraw function
    pub grantee: String,
    /// address to withdraw portion of rewards to
    pub withdraw_fee_address: String,
    /// percentage of rewards that can be withdrawn to the given address
    pub max_fee_percentage: Decimal,
    /// expiration date of the grant as a unix timestamp
    pub expiration: Timestamp,
}
