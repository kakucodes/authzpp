use cosmwasm_schema::cw_serde;
use cosmwasm_std::{coins, Addr, Binary, Coin, Timestamp};
use serde::Serialize;

#[cfg(feature = "wasm")]
use tsify::Tsify;

#[cfg(feature = "wasm")]
use wasm_bindgen::prelude::*;

#[cw_serde]
#[derive(Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
                grant_type: grant_type.msg_type_url(),
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

impl GrantRequirement {
    pub fn default_contract_exec_auth(
        base: GrantBase,
        contract_addr: Addr,
        keys: Vec<impl Into<String>>,
        limit_denom: Option<&str>,
    ) -> Self {
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::ContractExecutionAuthorization(vec![
                ContractExecutionSetting {
                    contract_addr,
                    limit: limit_denom.map_or(
                        ContractExecutionAuthorizationLimit::default(),
                        |limit_denom| {
                            ContractExecutionAuthorizationLimit::single_fund_limit(limit_denom)
                        },
                    ),
                    filter: ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter {
                        keys: keys.into_iter().map(|k| k.into()).collect(),
                    },
                },
            ]),
            granter: base.granter,
            grantee: base.grantee,
            expiration: base.expiration,
        }
    }

    pub fn contract_exec_messages_auth<T>(
        base: GrantBase,
        contract_addr: Addr,
        messages: Vec<T>,
        limit_denom: Option<&str>,
    ) -> Self
    where
        T: Serialize + Sized,
    {
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::ContractExecutionAuthorization(vec![
                ContractExecutionSetting {
                    contract_addr,
                    limit: limit_denom.map_or(
                        ContractExecutionAuthorizationLimit::default(),
                        |limit_denom| {
                            ContractExecutionAuthorizationLimit::single_fund_limit(limit_denom)
                        },
                    ),
                    filter: ContractExecutionAuthorizationFilter::AcceptedMessagesFilter {
                        messages: messages
                            .iter()
                            // normally unwrap would be a nogo but it should be alright here since this is only used for queries
                            .map(|m| cosmwasm_std::to_binary(m).unwrap())
                            .collect(),
                    },
                },
            ]),
            granter: base.granter,
            grantee: base.grantee,
            expiration: base.expiration,
        }
    }

    pub fn delegation_authorization(
        base: GrantBase,
        // the list of validators to whitelist
        validator_addresses: Option<String>,
        max_tokens: Option<Coin>,
    ) -> Self {
        let (max_token_amount, max_token_denom) = max_tokens
            .map(|c| (Some(c.amount.u128()), Some(c.denom)))
            .unwrap_or((None, None));
        gen_delegation_authorization(base, validator_addresses, max_token_amount, max_token_denom)
    }
}

// #[cfg_attr(feature = "wasm", wasm_bindgen)]
pub fn gen_delegation_authorization(
    base: GrantBase,
    // the list of validators to whitelist
    validator_addresses: Option<String>,
    max_token_amount: Option<u128>,
    max_token_denom: Option<String>,
) -> GrantRequirement {
    GrantRequirement::GrantSpec {
        grant_type: AuthorizationType::StakeAuthorization {
            max_tokens: if let (Some(amount), Some(denom)) = (max_token_amount, max_token_denom) {
                Some(Coin {
                    amount: amount.into(),
                    denom,
                })
            } else {
                None
            },
            authorization_type: StakeAuthorizationType::Delegate,
            validators: validator_addresses.map(|address| {
                StakeAuthorizationPolicy::AllowList(StakeAuthorizationValidators {
                    address: vec![address],
                })
            }),
        },
        granter: base.granter,
        grantee: base.grantee,
        expiration: base.expiration,
    }
}

#[cw_serde]
#[derive(Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum RevokeRequirement {
    RevokeSpec {
        grant_type: String,
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
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
impl AuthorizationType {
    pub fn msg_type_url(&self) -> String {
        match self {
            AuthorizationType::GenericAuthorization { msg } => msg.to_string(),
            AuthorizationType::SendAuthorization { .. } => {
                "/cosmos.bank.v1beta1.MsgSend".to_string()
            }
            AuthorizationType::StakeAuthorization { .. } => {
                "/cosmos.staking.v1beta1.MsgDelegate".to_string()
            }
            AuthorizationType::ContractExecutionAuthorization { .. } => {
                "/cosmwasm.wasm.v1.MsgExecuteContract".to_string()
            }
            AuthorizationType::TransferAuthorization { .. } => {
                "/ibc.applications.transfer.v1.MsgTransfer".to_string()
            }
        }
    }
}

#[cw_serde]
#[derive(Eq, Default)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct StakeAuthorizationValidators {
    pub address: Vec<String>,
}

#[cw_serde]
#[derive(Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub enum StakeAuthorizationPolicy {
    /// allow_list specifies list of validator addresses to whom grantee can delegate tokens on behalf of granter's
    /// account.
    AllowList(StakeAuthorizationValidators),
    /// deny_list specifies list of validator addresses to whom grantee can not delegate tokens.
    DenyList(StakeAuthorizationValidators),
}

#[cw_serde]
#[derive(Eq)]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
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

#[cw_serde]
#[cfg_attr(feature = "wasm", derive(Tsify))]
#[cfg_attr(feature = "wasm", tsify(into_wasm_abi, from_wasm_abi))]
pub struct GrantBase {
    pub granter: Addr,
    pub grantee: Addr,
    pub expiration: Timestamp,
}
