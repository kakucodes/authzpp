use std::vec;

use cosmwasm_std::{coin, coins, Addr, Timestamp};

use crate::{
    grantable_trait::dedupe_grant_reqs,
    grants::{
        AuthorizationType, ContractExecutionAuthorizationFilter,
        ContractExecutionAuthorizationLimit, ContractExecutionSetting, GrantRequirement,
        StakeAuthorizationPolicy, StakeAuthorizationType, StakeAuthorizationValidators,
    },
};

#[test]
pub fn dedupe_basic_grants() {
    let granter1 = Addr::unchecked("granter1");
    // let granter2 = Addr::unchecked("granter2");
    let grantee1 = Addr::unchecked("grantee1");
    let grantee2 = Addr::unchecked("grantee2");
    let validator1 = Addr::unchecked("validator1");
    let validator2 = Addr::unchecked("validator2");

    assert_eq!(
        dedupe_grant_reqs(vec![
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::StakeAuthorization {
                    max_tokens: None,
                    authorization_type: StakeAuthorizationType::Delegate,
                    validators: None
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::StakeAuthorization {
                    max_tokens: None,
                    authorization_type: StakeAuthorizationType::Delegate,
                    validators: None
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            }
        ]),
        vec![GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::StakeAuthorization {
                max_tokens: None,
                authorization_type: StakeAuthorizationType::Delegate,
                validators: None
            },
            granter: granter1.clone(),
            grantee: grantee1.clone(),
            expiration: Timestamp::from_seconds(0)
        }]
    );

    assert_eq!(
        dedupe_grant_reqs(vec![
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::StakeAuthorization {
                    max_tokens: None,
                    authorization_type: StakeAuthorizationType::Delegate,
                    validators: Some(StakeAuthorizationPolicy::AllowList(
                        StakeAuthorizationValidators {
                            address: vec![validator1.to_string()]
                        }
                    ))
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::StakeAuthorization {
                    max_tokens: None,
                    authorization_type: StakeAuthorizationType::Delegate,
                    validators: Some(StakeAuthorizationPolicy::AllowList(
                        StakeAuthorizationValidators {
                            address: vec![validator2.to_string()]
                        }
                    ))
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            }
        ]),
        vec![GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::StakeAuthorization {
                max_tokens: None,
                authorization_type: StakeAuthorizationType::Delegate,
                validators: Some(StakeAuthorizationPolicy::AllowList(
                    StakeAuthorizationValidators {
                        address: vec![validator1.to_string(), validator2.to_string()]
                    }
                ))
            },
            granter: granter1.clone(),
            grantee: grantee1.clone(),
            expiration: Timestamp::from_seconds(0)
        }]
    );

    // Test concatenation of send authorizations
    assert_eq!(
        dedupe_grant_reqs(vec![
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: None,
                    allow_list: Some(vec![grantee1.clone(), validator1.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: None,
                    allow_list: Some(vec![grantee2.clone(), validator1.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: None,
                    allow_list: Some(vec![validator2.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee2.clone(),
                expiration: Timestamp::from_seconds(0)
            }
        ]),
        vec![
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: None,
                    allow_list: Some(vec![grantee1.clone(), validator1.clone(), grantee2.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: None,
                    allow_list: Some(vec![validator2.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee2.clone(),
                expiration: Timestamp::from_seconds(0)
            }
        ]
    );
}

#[test]
pub fn dedupe_contract_auth_grants() {
    let user1 = Addr::unchecked("user1");
    // let granter1 = Addr::unchecked("granter1");
    // let granter2 = Addr::unchecked("granter2");
    let grantee1 = Addr::unchecked("grantee1");
    // let grantee2 = Addr::unchecked("grantee2");
    let contract1 = Addr::unchecked("contract1");
    // let contract2 = Addr::unchecked("contract2");

    // Test concatenation of contract execution authorizations
    assert_eq!(
        dedupe_grant_reqs(vec![
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::ContractExecutionAuthorization(vec![
                    ContractExecutionSetting {
                        contract_addr: contract1.clone(),
                        limit: ContractExecutionAuthorizationLimit::MaxCallsLimit {
                            remaining: u64::MAX
                        },
                        filter: ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter {
                            keys: vec!["key1".to_string()]
                        }
                    }
                ]),
                granter: user1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::ContractExecutionAuthorization(vec![
                    ContractExecutionSetting {
                        contract_addr: contract1.clone(),
                        limit: ContractExecutionAuthorizationLimit::MaxCallsLimit {
                            remaining: u64::MAX
                        },
                        filter: ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter {
                            keys: vec!["key2".to_string()]
                        }
                    }
                ]),
                granter: user1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
        ]),
        vec![GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::ContractExecutionAuthorization(vec![
                ContractExecutionSetting {
                    contract_addr: contract1.clone(),
                    limit: ContractExecutionAuthorizationLimit::MaxCallsLimit {
                        remaining: u64::MAX
                    },
                    filter: ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter {
                        keys: vec!["key2".to_string()]
                    }
                },
                ContractExecutionSetting {
                    contract_addr: contract1.clone(),
                    limit: ContractExecutionAuthorizationLimit::MaxCallsLimit {
                        remaining: u64::MAX
                    },
                    filter: ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter {
                        keys: vec!["key1".to_string()]
                    }
                },
            ]),
            granter: user1.clone(),
            grantee: grantee1.clone(),
            expiration: Timestamp::from_seconds(0)
        },]
    );
}

#[test]
pub fn dedupe_send_auth_grants() {
    // let user1 = Addr::unchecked("user1");
    let granter1 = Addr::unchecked("granter1");
    // let granter2 = Addr::unchecked("granter2");
    let grantee1 = Addr::unchecked("grantee1");
    // let grantee2 = Addr::unchecked("grantee2");
    let receiver1 = Addr::unchecked("receiver1");
    let receiver2 = Addr::unchecked("receiver2");

    // Test concatenation of send authorizations
    assert_eq!(
        dedupe_grant_reqs(vec![
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: Some(vec![coin(100, "ubtc"), coin(200, "aeth")]),
                    allow_list: Some(vec![receiver1.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
            GrantRequirement::GrantSpec {
                grant_type: AuthorizationType::SendAuthorization {
                    spend_limit: Some(coins(200, "ubtc")),
                    allow_list: Some(vec![receiver2.clone()])
                },
                granter: granter1.clone(),
                grantee: grantee1.clone(),
                expiration: Timestamp::from_seconds(0)
            },
        ]),
        vec![GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::SendAuthorization {
                spend_limit: Some(vec![coin(200, "aeth"), coin(300, "ubtc"),]),
                allow_list: Some(vec![receiver1.clone(), receiver2.clone()])
            },
            granter: granter1.clone(),
            grantee: grantee1.clone(),
            expiration: Timestamp::from_seconds(0)
        },],
        "send auths should be concatenated"
    );
}
