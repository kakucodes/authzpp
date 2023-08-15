use cosmwasm_schema::cw_serde;
use cosmwasm_std::{Addr, Coin};

#[cw_serde]
pub struct GrantSpec {
    pub grant_type: GrantType,
    pub granter: Addr,
    pub grantee: Addr,
}

#[cw_serde]
pub enum GrantType {
    GenericAuthorization {
        msg: String,
    },
    SendAuthorization {
        spend_limit: Option<Coin>,
    },
    StakeAuthorization {
        max_tokens: Option<Coin>,
        authorization_type: StakeAuthorizationType,
        validators: Option<StakeAuthorizationPolicy>,
    },
    ContractExecutionAuthorization {
        contract_addr: Addr,
        /// Limit defines execution limits that are enforced and updated when the grant
        /// is applied. When the limit lapsed the grant is removed.
        limit: Option<u64>,
        /// Filter define more fine-grained control on the message payload passed
        /// to the contract in the operation. When no filter applies on execution, the
        /// operation is prohibited.
        filter: Option<String>,
    },
    Authzpp {
        contract_addr: Addr,
        grant_type: AuthzppGrantType,
    }, // TransferAuthorization,
}

#[cw_serde]
pub enum AuthzppGrantType {
    WithdrawTax {},
    AllowlistSend { receiver: Addr },
    DenomAllowlistSend { allowed_denoms: Vec<String> },
}

#[cw_serde]
pub enum StakeAuthorizationType {
    /// AUTHORIZATION_TYPE_UNSPECIFIED specifies an unknown authorization type
    Unspecified = 0,
    /// AUTHORIZATION_TYPE_DELEGATE defines an authorization type for Msg/Delegate
    Delegate = 1,
    /// AUTHORIZATION_TYPE_UNDELEGATE defines an authorization type for Msg/Undelegate
    Undelegate = 2,
    /// AUTHORIZATION_TYPE_REDELEGATE defines an authorization type for Msg/BeginRedelegate
    Redelegate = 3,
}

/// Validators defines list of validator addresses.
#[cw_serde]
pub struct StakeAuthorizationValidators {
    pub address: Vec<String>,
}

#[cw_serde]
pub enum StakeAuthorizationPolicy {
    /// allow_list specifies list of validator addresses to whom grantee can delegate tokens on behalf of granter's
    /// account.
    AllowList(StakeAuthorizationValidators),
    /// deny_list specifies list of validator addresses to whom grantee can not delegate tokens.
    DenyList(StakeAuthorizationValidators),
}

#[cw_serde]
pub enum ContractExecutionAuthorizationLimit {
    MaxCallsLimit {
        /// Remaining number that is decremented on each execution
        remaining: u64,
    },
    MaxFundsLimit {
        /// Amounts is the maximal amount of tokens transferable to the contract.
        amounts: Vec<Coin>,
    },
    CombinedLimit {
        /// Remaining number that is decremented on each execution
        calls_remaining: u64,
        /// Amounts is the maximal amount of tokens transferable to the contract.
        amounts: Vec<Coin>,
    },
}

#[cw_serde]
pub enum ContractExecutionAuthorizationFilter {
    /// AllowAllMessagesFilter is a wildcard to allow any type of contract payload
    /// message.
    AllowAllMessagesFilter,
    /// AcceptedMessageKeysFilter accept only the specific contract message keys in
    /// the json object to be executed.
    AcceptedMessageKeysFilter {
        /// Keys is a list of keys.
        keys: Vec<String>,
    },
    /// AcceptedMessagesFilter accept only the specific raw contract messages to be
    /// executed.
    AcceptedMessagesFilter {
        /// Messages is a list of raw messages.
        messages: Vec<Vec<u8>>,
    },
}