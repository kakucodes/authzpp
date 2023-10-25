use std::io::Cursor;

use crate::{
    helpers::{sum_coins, dec_coin_to_coin, filter_empty_coins, },
    msg::{ VersionResponse, GrantQueryResponse, }, state::GRANTS,
};
use authzpp_utils::helpers::Expirable;
use cosmos_sdk_proto::{cosmos::distribution::v1beta1::{ QueryDelegationTotalRewardsRequest, QueryDelegationTotalRewardsResponse, DelegationDelegatorReward, }, traits::Message};
use cosmwasm_std::{Addr, FullDelegation, QuerierWrapper, Storage, BlockInfo, StdResult, Order, Coin, Binary, QueryRequest, };

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

/// Queries the pending staking rewards and the total rewards for a given delegator via stargate queries
/// this is the cheapest/fastest way to get the data
pub fn query_total_pending_rewards_stargate(
    querier: &QuerierWrapper,
    delegator_addr: &Addr,
) -> Result<AllPendingRewards, ContractError> {

    let bin = QueryDelegationTotalRewardsRequest{ delegator_address: delegator_addr.to_string() }
    .encode_to_vec();

    let data = Binary::from(bin);

    let query = QueryRequest::Stargate {
        path: "/cosmos.staking.v1beta1.Query/DelegationTotalRewards".to_string(),
        data,
    };

    let bin: Binary = querier.query(&query)?;
    let QueryDelegationTotalRewardsResponse { rewards, total } = 
        QueryDelegationTotalRewardsResponse::decode(&mut Cursor::new(bin.to_vec()))
            .map_err(ContractError::Decode)?;


    Ok(AllPendingRewards { 
        total: total.iter().map(dec_coin_to_coin).collect::<Result<Vec<Coin>, ContractError>>()?,
        rewards: rewards.into_iter().map(|DelegationDelegatorReward {
            validator_address, reward }| Ok(PendingReward { 
                validator: validator_address, 
                amount: reward.iter().map(dec_coin_to_coin).collect::<Result<Vec<Coin>, ContractError>>()?
            })).collect::<Result<Vec<PendingReward>, ContractError>>()?,
        })
    
}

/// Queries the pending staking rewards for a given delegator
pub fn query_pending_rewards(
    querier: &QuerierWrapper,
    delegator_addr: &Addr,
) -> Result<AllPendingRewards, ContractError> {

    // first try to get the pending rewards via stargate query to save gas
    match query_total_pending_rewards_stargate(querier, delegator_addr) {
        Ok(rewards) => Ok(rewards),
        Err(_) => {
            // gets all of the individual delegations for the delegator since the stargate query failed
            let rewards_query: Result<Vec<PendingReward>, ContractError> = querier
                .query_all_delegations(delegator_addr)?
                .into_iter()
                
                .filter_map(
                    // each delegation is queried for its pending rewards
                    |delegation| match querier.query_delegation(delegator_addr, delegation.validator) {
                        Ok(Some(FullDelegation {
                            validator,
                            accumulated_rewards,
                            ..
                        })) if !accumulated_rewards.is_empty() => Some(Ok(PendingReward {
                            validator,
                            amount: accumulated_rewards,
                        })),
                        Ok(Some(FullDelegation {
                            
                            ..
                        })) => None,
                        _ => Some(Err(ContractError::QueryPendingRewardsFailure)),
                    },
                )
                .collect();

            let rewards = rewards_query?;

            // sums the rewards
            let total = filter_empty_coins(rewards.iter().fold(vec![], |mut acc, reward| {
                acc = sum_coins(acc, reward.amount.clone());
                acc
            }));

            Ok(AllPendingRewards { rewards, total })
        },
    }

    
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
