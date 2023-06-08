use crate::{
    helpers::{sum_coins, partition_coins_by_percentage},
    msg::{ VersionResponse, GrantQueryResponse, SimulateExecuteResponse}, state::GRANTS,
};
use cosmwasm_std::{Addr, FullDelegation, QuerierWrapper, Storage, BlockInfo, StdResult, Order, Coin, Decimal};

use crate::ContractError;

pub fn query_version() -> VersionResponse {
    VersionResponse {
        version: env!("CARGO_PKG_VERSION").to_string(),
    }
}


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
    delegator: &Addr,
) -> Result<AllPendingRewards, ContractError> {
    // gets all of the individual delegations for the delegator
    let rewards_query: Result<Vec<PendingReward>, ContractError> = querier
        .query_all_delegations(delegator)?
        .into_iter()
        .map(
            // each delegation is queried for its pending rewards
            |delegation| match querier.query_delegation(delegator, delegation.validator) {
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
    let total = rewards.clone().into_iter().fold(vec![], |mut acc, reward| {
        acc = sum_coins(acc, reward.amount);
        acc
    });

    Ok(AllPendingRewards { rewards, total })
}

/// search for and return the grant settings for an abitrary granter
pub fn query_active_grants_by_granter(storage: &dyn Storage, block: BlockInfo, granter: Addr ) -> StdResult<Option<GrantQueryResponse>> {
    let grant_settings = GRANTS.load(storage, granter.clone())?;

    Ok(Option::from(grant_settings)
        .filter(|grant| 
            // validate that the grant is still active and not expired
            block.time <= grant.expiration)
        .map(|allowed_withdrawls| GrantQueryResponse {
            granter,
            allowed_withdrawls,
        }))
}

/// returns all of the grant settings that are for grants to the given grantee
pub fn query_active_grants_by_grantee(storage: &dyn Storage, block: BlockInfo, grantee: Addr) -> Vec<GrantQueryResponse> {
    GRANTS
    // grab all of the grants
        .range(storage, None, None, Order::Ascending)
        // filter out the grants that are not for the requested grantee
        .filter_map(|item| {
            if let Ok((granter, allowed_withdrawls)) = item {
                // also ensure that the grant is active and unexpired
                if block.time <= (allowed_withdrawls.expiration)
                    && allowed_withdrawls.grantee.eq(&grantee)
                {
                    return Some(GrantQueryResponse {
                        granter,
                        allowed_withdrawls,
                    });
                }
            }
            None
        })       
        .collect::<Vec<_>>()
}

pub fn query_split_rewards(querier: &QuerierWrapper, granter: &Addr,  
    max_percentage: Decimal, requested_percentage: Option<Decimal>) -> Result<(SimulateExecuteResponse, Vec<PendingReward>), ContractError> {
    let AllPendingRewards { total, rewards } =
                query_pending_rewards(querier, granter)?;

    // figure out what percentage of the rewards to send to the grantee
    let percentage_to_send = requested_percentage
        .unwrap_or(max_percentage)
        .min(max_percentage);

    // get the list of tokens that the granter and grantee should each recieve
    let (withdraw_address_rewards, granter_rewards) =
        partition_coins_by_percentage(percentage_to_send, total);

    Ok((SimulateExecuteResponse {
        granter_rewards,
        withdraw_address_rewards,
    }, rewards))
}