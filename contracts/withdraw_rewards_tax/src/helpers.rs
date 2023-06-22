use std::str::FromStr;

use crate::msg::{AllowedWithdrawlSettings, SimulateExecuteResponse};
use crate::queries::PendingReward;
use crate::ContractError;
use authzpp_utils::helpers::Expirable;
use cosmos_sdk_proto::cosmos::base::v1beta1::DecCoin;
use cosmos_sdk_proto::cosmos::distribution::v1beta1::MsgSetWithdrawAddress;
use cosmos_sdk_proto::traits::MessageExt;
use cosmos_sdk_proto::{
    cosmos::distribution::v1beta1::MsgWithdrawDelegatorReward, prost::EncodeError, Any,
};
use cosmwasm_std::{Addr, Api, BlockInfo, Coin, Decimal, Uint128};

pub fn validate_granter_address(api: &dyn Api, granter: &str) -> Result<Addr, ContractError> {
    api.addr_validate(granter)
        .map_err(|_| ContractError::InvalidGranterAddress(granter.to_string()))
}

pub fn validate_grantee_address(api: &dyn Api, grantee: &str) -> Result<Addr, ContractError> {
    api.addr_validate(grantee)
        .map_err(|_| ContractError::InvalidGranteeAddress(grantee.to_string()))
}

/// Combines two vectors of coins into just one where any overlapping denoms are added together
pub fn sum_coins(xs: Vec<Coin>, ys: Vec<Coin>) -> Vec<Coin> {
    let mut coins = xs;
    for y in ys {
        let mut found = false;
        for x in coins.iter_mut() {
            if x.denom == y.denom {
                x.amount += y.amount;
                found = true;
                break;
            }
        }
        if !found {
            coins.push(y);
        }
    }
    coins
}

/// Splits the given coins into two vectors based on the percentage given
pub fn partition_coins_by_percentage(
    percentage: Decimal,
    coins: Vec<Coin>,
) -> (Vec<Coin>, Vec<Coin>) {
    let mut percentage_coins = vec![];
    let mut remaining_coins = vec![];

    for Coin { amount, denom } in coins {
        let amount_to_send = amount * percentage;
        let amount_to_remain = amount - amount_to_send;

        percentage_coins.push(Coin {
            denom: denom.clone(),
            amount: amount_to_send,
        });
        remaining_coins.push(Coin {
            denom,
            amount: amount_to_remain,
        });
    }

    (percentage_coins, remaining_coins)
}

/// computes the rewards that should be sent to the granter and the withdraw address
pub fn split_rewards(
    total_rewards: Vec<Coin>,
    max_percentage: Decimal,
    requested_percentage: &Option<Decimal>,
) -> SimulateExecuteResponse {
    // figure out what percentage of the rewards to send to the grantee
    let percentage_to_send = requested_percentage
        .unwrap_or(max_percentage)
        .min(max_percentage);

    // get the list of tokens that the granter and grantee should each recieve
    let (withdraw_address_rewards, delegator_rewards) =
        partition_coins_by_percentage(percentage_to_send, total_rewards);

    SimulateExecuteResponse {
        delegator_rewards,
        taxation_address_rewards: withdraw_address_rewards,
    }
}

/// Generates a message for claiming rewards from a bunch of validators
pub fn withdraw_rewards_msgs(
    target_address: &Addr,
    pending_rewards: &[PendingReward],
) -> Result<Vec<Any>, ContractError> {
    let withdraw_rewards_msgs: Vec<Any> = pending_rewards
        .iter()
        .map(|PendingReward { validator, .. }| {
            MsgWithdrawDelegatorReward {
                validator_address: validator.to_string(),
                delegator_address: target_address.to_string(),
            }
            .to_any()
        })
        .collect::<Result<Vec<_>, EncodeError>>()?;

    Ok(withdraw_rewards_msgs)
}

/// Creates a MsgSetWithdrawAddress message for changing a wallet's delegation rewards withdrawal address
pub fn set_withdraw_rewards_address_msg(
    delegator_address: &Addr,
    target_withdraw_address: &Addr,
) -> Result<Any, ContractError> {
    let set_withdraw_address_msg = MsgSetWithdrawAddress {
        delegator_address: delegator_address.to_string(),
        withdraw_address: target_withdraw_address.to_string(),
    }
    .to_any()?;

    Ok(set_withdraw_address_msg)
}

impl Expirable for AllowedWithdrawlSettings {
    fn is_not_expired(&self, block: &BlockInfo) -> bool {
        block.time <= (self.expiration)
    }
    fn is_expired(&self, block: &BlockInfo) -> bool {
        block.time > (self.expiration)
    }
}

pub fn dec_coin_to_coin(dec_coin: &DecCoin) -> Result<Coin, ContractError> {
    Ok(Coin {
        denom: dec_coin.denom.clone(),
        amount: (Uint128::from_str(&dec_coin.amount)?.u128() / 1_000_000_000_000_000_000u128)
            .into(),
    })
}

pub fn filter_empty_coins(coins: Vec<Coin>) -> Vec<Coin> {
    coins
        .into_iter()
        .filter(|Coin { amount, .. }| !amount.is_zero())
        .collect()
}
