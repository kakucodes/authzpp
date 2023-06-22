use cosmwasm_std::{Addr, BankMsg, CosmosMsg, Decimal};

use crate::{
    helpers::{
        filter_empty_coins, set_withdraw_rewards_address_msg, split_rewards, withdraw_rewards_msgs,
    },
    msg::{AllowedWithdrawlSettings, SimulateExecuteResponse},
    queries::{AllPendingRewards, PendingReward},
    ContractError,
};

/// creates the message that gets broadcast to claim the rewards to this contract
/// and then ensure that the withdraw address is set back to the grantee
/// this is all wrapped in one MsgExec to interface with the native Authz module
pub fn create_withdraw_rewards_exec_msg(
    delegator_addr: &Addr,
    contract_addr: &Addr,
    rewards: &[PendingReward],
) -> Result<CosmosMsg, ContractError> {
    let mut claim_rewards_msgs = vec![];

    // first set the withdraw rewards address to this contract
    claim_rewards_msgs.push(set_withdraw_rewards_address_msg(
        delegator_addr,
        contract_addr,
    )?);

    // claim all of the users rewards. these should now be sent into this contract
    claim_rewards_msgs.extend(withdraw_rewards_msgs(delegator_addr, rewards)?);

    // put the delegator's withdraw address back to them so they dont accidentally send us tokens
    claim_rewards_msgs.push(set_withdraw_rewards_address_msg(
        delegator_addr,
        delegator_addr,
    )?);

    // wrap the messages for claiming into a single exec message as these will get done via native Authz
    authzpp_utils::msg_gen::exec_msg(contract_addr, claim_rewards_msgs)
        .map_err(ContractError::EncodeError)
}

#[derive(PartialEq, Eq, Debug)]
pub struct RewardExecutionMsgs {
    pub msgs: Vec<CosmosMsg>,
    pub grantee: String,
}

/// Generates the withdraw rewards messages and the messages to disburse the funds to both
/// the granter and the fee wallet.
///
/// * `all_pending_rewards` - the rewards that are being withdrawn
/// * `grant_settings` - the settings for the grant including the maximum fee split
/// * `sender_addr` - the address of the wallet that initiated the withdraw. this must be the delegator or the grantee
/// * `contract_addr` - the address of the contract this function is running in
/// * `delegator_addr` - the address of the delegator that is withdrawing the rewards
/// * `percentage` - the percentage of the rewards to withdraw. If None, the max allowed fee will be used
pub fn generate_reward_withdrawl_msgs(
    AllPendingRewards {
        rewards,
        total: all_pending_rewards,
    }: AllPendingRewards,
    AllowedWithdrawlSettings {
        grantee,
        taxation_address,
        max_fee_percentage,
        ..
    }: AllowedWithdrawlSettings,
    sender_addr: &Addr,
    contract_addr: &Addr,
    delegator_addr: &Addr,
    percentage: Option<Decimal>,
) -> Result<RewardExecutionMsgs, ContractError> {
    // validate that the executor is either the granter or the grantee
    if sender_addr.ne(delegator_addr) && sender_addr.ne(&grantee) {
        return Err(ContractError::Unauthorized {});
    }

    // calculate how much the granter and grantee/withdraw address should get from the staking rewards
    let SimulateExecuteResponse {
        delegator_rewards,
        taxation_address_rewards,
    } = split_rewards(all_pending_rewards, max_fee_percentage, &percentage);

    // create the message to execute the rewards withdraw
    let withdraw_rewards_exec_msg =
        create_withdraw_rewards_exec_msg(delegator_addr, contract_addr, &rewards)?;

    let mut msgs = vec![withdraw_rewards_exec_msg];

    // send the share address their share of the rewards if there are any
    if filter_empty_coins(taxation_address_rewards.clone())
        .len()
        .gt(&0)
    {
        msgs.push(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
            to_address: taxation_address,
            amount: taxation_address_rewards,
        }));
    }

    // send the granter their share of the rewards
    if filter_empty_coins(delegator_rewards.clone()).len().gt(&0) {
        msgs.push(cosmwasm_std::CosmosMsg::Bank(BankMsg::Send {
            to_address: delegator_addr.to_string(),
            amount: delegator_rewards,
        }));
    }

    Ok(RewardExecutionMsgs { msgs, grantee })
}
