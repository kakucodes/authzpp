use crate::grants::{
    AuthorizationType, ContractExecutionAuthorizationFilter, ContractExecutionAuthorizationLimit,
    ContractExecutionSetting, GrantRequirement, RevokeRequirement, StakeAuthorizationPolicy,
    StakeAuthorizationType, StakeAuthorizationValidators,
};
use cosmwasm_std::{Addr, Coin, StdResult, Timestamp};
use itertools::Itertools;
use std::u64;

pub trait Grantable {
    type GrantSettings;

    fn query_grants(
        grant: GrantStructure<Self::GrantSettings>,
        current_timestamp: Timestamp,
    ) -> StdResult<Vec<GrantRequirement>>;

    fn query_revokes(
        grant: GrantStructure<Self::GrantSettings>,
    ) -> StdResult<Vec<RevokeRequirement>>;
}

#[derive(Clone, Debug, PartialEq)]
pub struct GrantStructure<T> {
    pub granter: Addr,
    pub grantee: Addr,
    pub expiration: Timestamp,
    pub grant_contract: Addr,
    pub grant_data: T,
}

type StakeAuthKey = (Addr, Addr, StakeAuthorizationType);

pub fn dedupe_grant_reqs(grants: Vec<GrantRequirement>) -> Vec<GrantRequirement> {
    let mut stake_authorizations = vec![];
    let mut generic_authorizations = vec![];
    let mut send_authorizations = vec![];
    let mut contract_execute_authorizations = vec![];
    let mut transfer_authorizations = vec![];
    let mut contract_executions = vec![];

    grants.into_iter().for_each(|grant| match grant {
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::StakeAuthorization { .. },
            ..
        } => {
            // only add if it's unique
            if !stake_authorizations.contains(&grant) {
                stake_authorizations.push(grant)
            }
        }
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::GenericAuthorization { .. },
            ..
        } => {
            // only add if it's unique
            if !generic_authorizations.contains(&grant) {
                generic_authorizations.push(grant)
            }
        }
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::SendAuthorization { .. },
            ..
        } => {
            // only add if it's unique
            if !send_authorizations.contains(&grant) {
                send_authorizations.push(grant)
            }
        }
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::ContractExecutionAuthorization(_),
            ..
        } => {
            // only add if it's unique
            if !contract_execute_authorizations.contains(&grant) {
                contract_execute_authorizations.push(grant)
            }
        }
        GrantRequirement::GrantSpec {
            grant_type: AuthorizationType::TransferAuthorization { .. },
            ..
        } => {
            // TODO: update transfer logic
            if !transfer_authorizations.contains(&grant) {
                transfer_authorizations.push(grant)
            }
        }
        GrantRequirement::ContractExec { .. } => contract_executions.push(grant),
    });

    // these are only stake authorizations
    // we need to concatenate them with the other stake authorizations when they are
    // aimed at the same grant (granter, grantee, stake authorization type)
    // TODO: this is dumb and should use a hashmap instead
    let stake_authorizations = stake_authorizations
        .iter()
        .fold(
            Vec::<(StakeAuthKey, (AuthorizationType, Timestamp))>::new(),
            |mut all_stake_grants, stake_grant| {
                if let GrantRequirement::GrantSpec {
                    grant_type:
                        AuthorizationType::StakeAuthorization {
                            max_tokens,
                            authorization_type,
                            validators,
                        },
                    granter,
                    grantee,
                    expiration,
                } = stake_grant.clone()
                {
                    let matching_index = all_stake_grants.iter().find_position(|(key, _)| {
                        key.clone().eq(&(
                            granter.clone(),
                            grantee.clone(),
                            authorization_type.clone(),
                        ))
                    });
                    if let Some((i, _)) = matching_index {
                        // if the grant already exists, we need to update the max_tokens and validators

                        // take whichever expiration is later
                        let new_expiration = all_stake_grants[i].1 .1.max(expiration);

                        let new_validators = combine_stake_auth_policies(
                            if let AuthorizationType::StakeAuthorization {
                                validators: a_validators,
                                ..
                            } = all_stake_grants[i].1 .0.clone()
                            {
                                a_validators
                            } else {
                                None
                            },
                            validators,
                        );

                        all_stake_grants[i].1 = (
                            AuthorizationType::StakeAuthorization {
                                // TODO: more intelligent max_tokens merging
                                // for now just set no max
                                max_tokens: None,
                                authorization_type,
                                validators: new_validators,
                            },
                            // expiration: new_expiration,
                            new_expiration,
                        );
                    } else {
                        // if the grant doesn't exist, we need to add it
                        all_stake_grants.push((
                            (granter.clone(), grantee.clone(), authorization_type.clone()),
                            (
                                AuthorizationType::StakeAuthorization {
                                    max_tokens,
                                    authorization_type,
                                    validators,
                                },
                                expiration,
                            ),
                        ));
                    }
                }

                all_stake_grants
            },
        )
        .iter()
        .map(
            |((granter, grantee, ..), (grant_type, expiration))| GrantRequirement::GrantSpec {
                grant_type: grant_type.clone(),
                granter: granter.clone(),
                grantee: grantee.clone(),
                expiration: *expiration,
            },
        )
        .collect::<Vec<GrantRequirement>>();

    // these are only send authorizations
    // we need to concatenate them with the other send authorizations when they are
    // aimed at the same grant (granter, grantee)
    // TODO: this is dumb and should use a hashmap instead
    let send_authorizations = send_authorizations
        .iter()
        .fold(
            Vec::<((Addr, Addr), (AuthorizationType, Timestamp))>::new(),
            |mut all_send_grants, send_grant| {
                if let GrantRequirement::GrantSpec {
                    grant_type:
                        AuthorizationType::SendAuthorization {
                            spend_limit: b_send_limit,
                            allow_list: b_allow_list,
                        },
                    granter,
                    grantee,
                    expiration,
                } = send_grant.clone()
                {
                    let matching_index = all_send_grants.iter().find_position(|(key, _)| {
                        key.clone().eq(&(granter.clone(), grantee.clone()))
                    });
                    if let Some((i, _)) = matching_index {
                        // if the grant already exists, we need to update the max_tokens and validators

                        // take whichever expiration is later
                        let new_expiration = all_send_grants[i].1 .1.max(expiration);

                        let new_grant_type = if let AuthorizationType::SendAuthorization {
                            spend_limit: a_spend_limit,
                            allow_list: a_allow_list,
                        } = all_send_grants[i].1 .0.clone()
                        {
                            combine_send_auths(
                                a_spend_limit,
                                a_allow_list,
                                b_send_limit,
                                b_allow_list,
                            )
                        } else {
                            panic!("This should never happen")
                        };

                        all_send_grants[i].1 = (
                            new_grant_type,
                            // expiration: new_expiration,
                            new_expiration,
                        );
                    } else {
                        // if the grant doesn't exist, we need to add it
                        all_send_grants.push((
                            (granter.clone(), grantee.clone()),
                            (
                                AuthorizationType::SendAuthorization {
                                    spend_limit: b_send_limit,
                                    allow_list: b_allow_list,
                                },
                                expiration,
                            ),
                        ));
                    }
                }

                all_send_grants
            },
        )
        .iter()
        .map(
            |((granter, grantee), (grant_type, expiration))| GrantRequirement::GrantSpec {
                grant_type: grant_type.clone(),
                granter: granter.clone(),
                grantee: grantee.clone(),
                expiration: *expiration,
            },
        )
        .collect::<Vec<GrantRequirement>>();

    // these are only contract execute authorizations
    // we need to concatenate them with the other contract execute authorizations when they are
    // aimed at the same grant (granter, grantee)
    // let contract_execute_authorizations =
    let contract_execute_authorizations = contract_execute_authorizations
        .iter()
        .fold(
            Vec::<((Addr, Addr), (AuthorizationType, Timestamp))>::new(),
            |mut all_send_grants, send_grant| {
                if let GrantRequirement::GrantSpec {
                    grant_type:
                        AuthorizationType::ContractExecutionAuthorization(
                            additional_contract_execution_settings,
                        ),
                    granter,
                    grantee,
                    expiration: additional_timestamp,
                } = send_grant.clone()
                {
                    let matching_index = all_send_grants
                        .iter()
                        .find_position(|(key, _)| key.0.eq(&granter) && key.1.eq(&grantee)); //key.clone().eq(&(granter.clone(), grantee.clone())));
                    if let Some((
                        i,
                        (
                            _,
                            (
                                AuthorizationType::ContractExecutionAuthorization(
                                    existing_contract_execution_settings,
                                ),
                                existing_timestamp,
                            ),
                        ),
                    )) = matching_index
                    {
                        // if the grant already exists, we need to update the max_tokens and validators

                        // take whichever expiration is later
                        let new_expiration = (*existing_timestamp).max(additional_timestamp);

                        all_send_grants[i].1 = (
                            AuthorizationType::ContractExecutionAuthorization(
                                [
                                    additional_contract_execution_settings,
                                    existing_contract_execution_settings.clone(),
                                ]
                                .concat(),
                            ),
                            // expiration: new_expiration,
                            new_expiration,
                        );
                    } else {
                        // if the grant doesn't exist, we need to add it
                        all_send_grants.push((
                            (granter.clone(), grantee.clone()),
                            (
                                AuthorizationType::ContractExecutionAuthorization(
                                    additional_contract_execution_settings,
                                ),
                                additional_timestamp,
                            ),
                        ));
                    }
                }

                all_send_grants
            },
        )
        .into_iter()
        .map(
            |((granter, grantee), (grant_type, expiration))| GrantRequirement::GrantSpec {
                grant_type,
                granter,
                grantee,
                expiration,
            },
        )
        .collect::<Vec<GrantRequirement>>();

    [
        contract_execute_authorizations,
        generic_authorizations,
        send_authorizations,
        stake_authorizations,
        contract_executions,
        transfer_authorizations,
    ]
    .concat()
}

fn combine_send_auths(
    a_spend_limit: Option<Vec<Coin>>,
    a_allow_list: Option<Vec<Addr>>,
    b_spend_limit: Option<Vec<Coin>>,
    b_allow_list: Option<Vec<Addr>>,
) -> AuthorizationType {
    println!(
        "a_allow_list: {:?}, b_allow_list: {:?}",
        a_allow_list, b_allow_list,
    );
    let spend_limit = match (a_spend_limit, b_spend_limit) {
        (Some(a), Some(b)) => Some([a, b].concat()),
        // if one has a spend limit but not the other than the combined send auth can't have a spend limit
        _ => None,
    };

    let allow_list = match (a_allow_list, b_allow_list) {
        (Some(a), Some(b)) => Some([a, b].concat().iter().unique().cloned().collect()),
        // if one has an allowlist and the other doesnt, then the combined send auth can't have an allowlist
        _ => None,
    };

    AuthorizationType::SendAuthorization {
        spend_limit,
        allow_list,
    }
}

fn combine_stake_auth_policies(
    a: Option<StakeAuthorizationPolicy>,
    b: Option<StakeAuthorizationPolicy>,
) -> Option<StakeAuthorizationPolicy> {
    match (a, b) {
        (
            Some(StakeAuthorizationPolicy::AllowList(StakeAuthorizationValidators {
                address: a_allow_list,
            })),
            Some(StakeAuthorizationPolicy::AllowList(StakeAuthorizationValidators {
                address: b_allow_list,
            })),
        ) => Some(StakeAuthorizationPolicy::AllowList(
            StakeAuthorizationValidators {
                address: [a_allow_list, b_allow_list]
                    .concat()
                    .into_iter()
                    .unique()
                    .collect(),
            },
        )),
        (
            Some(StakeAuthorizationPolicy::DenyList(StakeAuthorizationValidators {
                address: a_deny_list,
            })),
            Some(StakeAuthorizationPolicy::DenyList(StakeAuthorizationValidators {
                address: b_deny_list,
            })),
        ) => Some(StakeAuthorizationPolicy::DenyList(
            StakeAuthorizationValidators {
                address: [a_deny_list, b_deny_list].concat(),
            },
        )),
        (
            Some(StakeAuthorizationPolicy::AllowList(allow_list)),
            Some(StakeAuthorizationPolicy::DenyList(_)),
        ) => Some(StakeAuthorizationPolicy::AllowList(allow_list)),
        (
            Some(StakeAuthorizationPolicy::DenyList(_)),
            Some(StakeAuthorizationPolicy::AllowList(allow_list)),
        ) => Some(StakeAuthorizationPolicy::AllowList(allow_list)),
        (Some(a), None) => Some(a),
        (None, Some(b)) => Some(b),
        (None, None) => None,
    }
}

fn combine_contract_execute_auths(
    a_auths: Vec<ContractExecutionSetting>,
    b_auths: Vec<ContractExecutionSetting>,
) -> AuthorizationType {
    // let new_auth_settings = [a_auths, b_auths]
    //     .concat()
    //     .into_iter()
    //     .fold(
    //         HashMap::<Addr, Vec<ContractExecutionSetting>>::new(),
    //         |mut all_settings, current_setting| {
    //             if let Some(existing_setting) = all_settings.get(&current_setting.contract_addr) {
    //                 let new_setting = existing_setting
    //                     .iter()
    //                     .flat_map(|existing| {
    //                         combine_contract_execution_settings(
    //                             existing.clone(),
    //                             current_setting.clone(),
    //                         )
    //                     })
    //                     .collect();
    //                 all_settings.insert(current_setting.contract_addr.clone(), new_setting);
    //                 all_settings
    //             } else {
    //                 all_settings
    //                     .insert(current_setting.contract_addr.clone(), vec![current_setting]);
    //                 all_settings
    //             }
    //         },
    //     )
    //     .values()
    //     .flatten()
    //     .cloned()
    //     .collect::<Vec<ContractExecutionSetting>>();

    // GrantType::ContractExecutionAuthorization(new_auth_settings)
    AuthorizationType::ContractExecutionAuthorization([a_auths, b_auths].concat())
}

// both settings are expected to have the same contract_addr otherwise there's nothing to
fn combine_contract_execution_settings(
    a: ContractExecutionSetting,
    b: ContractExecutionSetting,
) -> Vec<ContractExecutionSetting> {
    let (a_limits, a_amounts) = match a.limit.clone() {
        ContractExecutionAuthorizationLimit::CombinedLimit {
            calls_remaining,
            amounts,
        } => (calls_remaining, amounts),
        ContractExecutionAuthorizationLimit::MaxCallsLimit { remaining } => (remaining, vec![]),
        ContractExecutionAuthorizationLimit::MaxFundsLimit { amounts } => (u64::MIN, amounts),
    };

    let (b_limits, b_amounts) = match b.limit.clone() {
        ContractExecutionAuthorizationLimit::CombinedLimit {
            calls_remaining,
            amounts,
        } => (calls_remaining, amounts),
        ContractExecutionAuthorizationLimit::MaxCallsLimit { remaining } => (remaining, vec![]),
        ContractExecutionAuthorizationLimit::MaxFundsLimit { amounts } => (u64::MIN, amounts),
    };

    let limit = match (a_limits, a_amounts, b_limits, b_amounts) {
        (u64::MIN, a_amounts, u64::MIN, b_amounts) => {
            ContractExecutionAuthorizationLimit::MaxFundsLimit {
                amounts: [a_amounts, b_amounts].concat(),
            }
        }
        (a_limits, funds_a, b_limits, funds_b) if funds_a.is_empty() && funds_b.is_empty() => {
            ContractExecutionAuthorizationLimit::MaxCallsLimit {
                remaining: a_limits.saturating_add(b_limits),
            }
        }
        (a_limits, a_amounts, b_limits, b_amounts) => {
            ContractExecutionAuthorizationLimit::CombinedLimit {
                calls_remaining: a_limits.saturating_add(b_limits),
                amounts: [a_amounts, b_amounts].concat(),
            }
        }
    };

    let filters = match (a.filter.clone(), b.filter) {
        (ContractExecutionAuthorizationFilter::AllowAllMessagesFilter, _)
        | (_, ContractExecutionAuthorizationFilter::AllowAllMessagesFilter) => {
            vec![ContractExecutionAuthorizationFilter::AllowAllMessagesFilter]
        }
        (
            ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter { keys },
            ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter { keys: keys_b },
        ) => {
            vec![
                ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter {
                    keys: [keys, keys_b].concat(),
                },
            ]
        }
        (
            ContractExecutionAuthorizationFilter::AcceptedMessagesFilter { messages },
            ContractExecutionAuthorizationFilter::AcceptedMessagesFilter {
                messages: messages_b,
            },
        ) => {
            vec![
                ContractExecutionAuthorizationFilter::AcceptedMessagesFilter {
                    messages: [messages, messages_b].concat(),
                },
            ]
        }
        (
            ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter { keys },
            ContractExecutionAuthorizationFilter::AcceptedMessagesFilter { messages },
        )
        | (
            ContractExecutionAuthorizationFilter::AcceptedMessagesFilter { messages },
            ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter { keys },
        ) => {
            vec![
                ContractExecutionAuthorizationFilter::AcceptedMessageKeysFilter { keys },
                ContractExecutionAuthorizationFilter::AcceptedMessagesFilter { messages },
            ]
        }
    };

    filters
        .into_iter()
        .map(|filter| ContractExecutionSetting {
            contract_addr: a.contract_addr.clone(),
            limit: limit.clone(),
            filter,
        })
        .collect()
}
