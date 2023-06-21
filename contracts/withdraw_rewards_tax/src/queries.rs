use crate::{
    helpers::{sum_coins, },
    msg::{ VersionResponse, GrantQueryResponse, }, state::GRANTS,
};
use authzpp_utils::helpers::Expirable;
use cosmwasm_std::{Addr, FullDelegation, QuerierWrapper, Storage, BlockInfo, StdResult, Order, Coin, };

use crate::ContractError;

pub fn query_version() -> VersionResponse {
    VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}

#[derive(Clone)]
pub struct AllPendingRewards {
    pub rewards: Vec<PendingReward>,
    pub total: Vec<Coin>,
}

#[derive(Clone)]
pub struct PendingReward {
    pub validator: String,
    pub amount: Vec<Coin>,
}

/// Queries the pending staking rewards for a given delegator
pub fn query_pending_rewards(
    querier: &QuerierWrapper,
    delegator_addr: &Addr,
) -> Result<AllPendingRewards, ContractError> {
    // gets all of the individual delegations for the delegator
    let rewards_query: Result<Vec<PendingReward>, ContractError> = querier
        .query_all_delegations(delegator_addr)?
        .into_iter()
        .map(
            // each delegation is queried for its pending rewards
            |delegation| match querier.query_delegation(delegator_addr, delegation.validator) {
                Ok(Some(FullDelegation {
                    validator,
                    accumulated_rewards,
                    ..
                })) => Ok(PendingReward {
                    validator,
                    amount: accumulated_rewards,
                }),
                _ => Err(ContractError::QueryPendingRewardsFailure),
            },
        )
        .collect();

    let rewards = rewards_query?;

    // sums the rewards
    let total = rewards.iter().fold(vec![], |mut acc, reward| {
        acc = sum_coins(acc, reward.amount.clone());
        acc
    });

    Ok(AllPendingRewards { rewards, total })
}

/// search for and return the grant settings for an abitrary granter
pub fn query_active_grants_by_delegator(storage: &dyn Storage, block: &BlockInfo, delegator_addr: &Addr ) -> StdResult<Option<GrantQueryResponse>> {
    // get the grant for the delegator from state 
    let grant_settings = GRANTS.load(storage, delegator_addr)?;

    Ok(Option::from(grant_settings)
        .filter(|grant| 
            // validate that the grant is still active and not expired
            grant.is_not_expired(block))
        .map(|allowed_withdrawls| GrantQueryResponse {
            delegator_addr: delegator_addr.clone(),
            allowed_withdrawls,
        }))
}

/// returns all of the grant settings that are for grants to the given grantee
pub fn query_active_grants_by_grantee(storage: &dyn Storage, block: &BlockInfo, grantee: Addr) -> Vec<GrantQueryResponse> {
    GRANTS
    // grab all of the grants
        .range(storage, None, None, Order::Ascending)
        // filter out the grants that are not for the requested grantee
        .filter_map(|item| {
            if let Ok((granter, allowed_withdrawls)) = item {
                // also ensure that the grant is active and unexpired
                if allowed_withdrawls.is_not_expired(block)
                    && allowed_withdrawls.grantee.eq(&grantee)
                {
                    return Some(GrantQueryResponse {
                        delegator_addr: granter,
                        allowed_withdrawls,
                    });
                }
            }
            None
        })       
        .collect::<Vec<_>>()
}
