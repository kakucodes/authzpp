use cosmwasm_std::{Addr, BankMsg, BlockInfo, CosmosMsg, Decimal, QuerierWrapper, Storage};

use crate::{
    helpers::{set_withdraw_rewards_address_msg, withdraw_rewards_msgs},
    msg::{AllowedWithdrawlSettings, SimulateExecuteResponse},
    queries::{query_active_grants_by_granter, query_split_rewards, PendingReward},
    ContractError,
};

/// creates the message that gets broadcast to claim the rewards to this contract
/// and then ensure that the withdraw address is set back to the grantee
/// this is all wrapped in one MsgExec to interface with the native Authz module
pub fn create_withdraw_rewards_exec_msg(
    granter: &Addr,
    grantee: &Addr,
    withdraw_address: &Addr,
    rewards: Vec<PendingReward>,
) -> Result<CosmosMsg, ContractError> {
    let mut claim_rewards_msgs = vec![];

    // first set the withdraw rewards address to this contract
    claim_rewards_msgs.push(set_withdraw_rewards_address_msg(granter, grantee)?);

    // claim all of the users rewards. these should now be sent into this contract
    claim_rewards_msgs.extend(withdraw_rewards_msgs(granter, rewards)?);

    // put the delegator's withdraw address back to them so they dont accidentally send us tokens
    claim_rewards_msgs.push(set_withdraw_rewards_address_msg(granter, withdraw_address)?);

    // wrap the messages for claiming into a single exec message as these will get done via native Authz
    authzpp_utils::msg_gen::exec_msg(grantee, claim_rewards_msgs)
        .map_err(ContractError::EncodeError)
}

pub struct RewardExecutionMsgs {
    pub msgs: Vec<CosmosMsg>,
    pub grantee: String,
}

/// Generates the withdraw rewards messages and the messages to disburse the funds to both
/// the granter and the fee wallet.
pub fn execute_rewards_withdraw(
    querier: &QuerierWrapper,
    storage: &mut dyn Storage,
    block: BlockInfo,
    sender: Addr,
    granter_addr: Addr,
    percentage: Option<Decimal>,
) -> Result<RewardExecutionMsgs, ContractError> {
    // query the grant settings, this will error if there is no active/unexpired grant
    let AllowedWithdrawlSettings {
        grantee,
        withdraw_fee_address,
        max_fee_percentage,
        ..
    } = match query_active_grants_by_granter(storage, block, granter_addr.clone()) {
        Ok(Some(grant)) => grant,
        _ => {
            return Err(ContractError::NoActiveGrant(granter_addr.to_string()));
        }
    }
    .allowed_withdrawls;

    // validate that the executor is either the granter or the grantee
    if sender.ne(&granter_addr) && sender.ne(&grantee) {
        return Err(ContractError::Unauthorized {});
    }

    // calculate how much the granter and grantee/withdraw address should get from the staking rewards
    let (
        SimulateExecuteResponse {
            granter_rewards,
            withdraw_address_rewards: grantee_rewards,
        },
        rewards,
    ) = query_split_rewards(querier, &granter_addr, max_fee_percentage, percentage)?;

    // create the message to execute the rewards withdraw
    let withdraw_rewards_exec_msg = create_withdraw_rewards_exec_msg(
        &granter_addr,
        &Addr::unchecked(&grantee),
        &Addr::unchecked(&withdraw_fee_address),
        rewards,
    )?;

    // send the share address their share of the rewards
    let withdraw_share_send_msg = BankMsg::Send {
        to_address: withdraw_fee_address,
        amount: grantee_rewards,
    };

    // send the granter their share of the rewards
    let granter_send_msg = BankMsg::Send {
        to_address: granter_addr.to_string(),
        amount: granter_rewards,
    };

    Ok(RewardExecutionMsgs {
        msgs: vec![
            withdraw_rewards_exec_msg,
            cosmwasm_std::CosmosMsg::Bank(withdraw_share_send_msg),
            cosmwasm_std::CosmosMsg::Bank(granter_send_msg),
        ],
        grantee,
    })
}
