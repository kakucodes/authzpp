use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coins, Addr, Binary, Coin, Timestamp};
// use withdraw_rewards_tax_grant::msg::GrantsSpecData as WithdrawTaxGrantsSpecData;

#[cw_serde]
#[derive(Eq)]
pub enum GrantRequirement {
    GrantSpec {
        grant_type: AuthorizationType,
        granter: Addr,
        grantee: Addr,
        expiration: Timestamp,
    },
    ContractExec {
        contract_addr: Addr,
        msg: Binary,
        sender: Addr,
    },
}

impl From<GrantRequirement> for RevokeRequirement {
    fn from(val: GrantRequirement) -> Self {
        match val {
            GrantRequirement::GrantSpec {
                grant_type,
                granter,
                grantee,
                ..
            } => RevokeRequirement::RevokeSpec {
                grant_type,
                granter,
                grantee,
            },
            GrantRequirement::ContractExec {
                contract_addr,
                msg,
                sender,
            } => RevokeRequirement::ContractExec {
                contract_addr,
                msg,
                sender,
            },
        }
    }
}

#[cw_serde]
#[derive(Eq)]
pub enum RevokeRequirement {
    RevokeSpec {
        grant_type: AuthorizationType,
        granter: Addr,
        grantee: Addr,
    },
    ContractExec {
        contract_addr: Addr,
        msg: Binary,
        sender: Addr,
    },
}

#[cw_serde]
#[derive(Eq)]
pub enum AuthorizationType {
    GenericAuthorization {
        msg: String,
    },
    SendAuthorization {
        spend_limit: Option<Vec<Coin>>,
        allow_list: Option<Vec<Addr>>,
    },
    StakeAuthorization {
        max_tokens: Option<Coin>,
        authorization_type: StakeAuthorizationType,
        validators: Option<StakeAuthorizationPolicy>,
    },
    ContractExecutionAuthorization(Vec<ContractExecutionSetting>),
    TransferAuthorization(Vec<TransferAuthorizationSetting>),
}

#[cw_serde]
#[derive(Eq, Default)]
pub struct TransferAuthorizationSetting {
    source_port: String,
    source_channel: String,
    // spend limitation on the channel
    spend_limit: Vec<Coin>,
    // allow list of receivers, an empty allow list permits any receiver address
    allow_list: Vec<String>,
}

#[cw_serde]
#[derive(Eq)]
pub struct ContractExecutionSetting {
    pub contract_addr: Addr,
    /// Limit defines execution limits that are enforced and updated when the grant
    /// is applied. When the limit lapsed the grant is removed.
    pub limit: ContractExecutionAuthorizationLimit,
    /// Filter define more fine-grained control on the message payload passed
    /// to the contract in the operation. When no filter applies on execution, the
    /// operation is prohibited.
    pub filter: ContractExecutionAuthorizationFilter,
}

#[cw_serde]
#[derive(Eq)]
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
#[derive(Eq)]
pub struct StakeAuthorizationValidators {
    pub address: Vec<String>,
}

#[cw_serde]
#[derive(Eq)]
pub enum StakeAuthorizationPolicy {
    /// allow_list specifies list of validator addresses to whom grantee can delegate tokens on behalf of granter's
    /// account.
    AllowList(StakeAuthorizationValidators),
    /// deny_list specifies list of validator addresses to whom grantee can not delegate tokens.
    DenyList(StakeAuthorizationValidators),
}

#[cw_serde]
#[derive(Eq)]
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
impl Default for ContractExecutionAuthorizationLimit {
    fn default() -> Self {
        ContractExecutionAuthorizationLimit::MaxCallsLimit {
            remaining: u64::MAX,
        }
    }
}
impl ContractExecutionAuthorizationLimit {
    pub fn single_fund_limit(denom: impl Into<String>) -> Self {
        self::ContractExecutionAuthorizationLimit::MaxFundsLimit {
            amounts: coins(u128::MAX, denom),
        }
    }
}

#[cw_serde]
#[derive(Default, Eq)]
pub enum ContractExecutionAuthorizationFilter {
    /// AllowAllMessagesFilter is a wildcard to allow any type of contract payload
    /// message.
    #[default]
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
        messages: Vec<Binary>,
    },
}
