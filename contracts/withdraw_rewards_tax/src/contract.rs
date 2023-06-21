use crate::error::ContractError;
use crate::execute::{generate_reward_withdrawl_msgs, RewardExecutionMsgs};
use crate::helpers::{split_rewards, validate_grantee_address, validate_granter_address};
use crate::msg::{ExecuteMsg, ExecuteSettings, InstantiateMsg, QueryMsg};
use crate::queries::{self, query_active_grants_by_delegator};
use crate::queries::{query_active_grants_by_grantee, query_pending_rewards};
use crate::state::GRANTS;
use authzpp_utils::helpers::Expirable;
#[cfg(not(feature = "library"))]
use cosmwasm_std::entry_point;
use cosmwasm_std::{to_binary, Binary, Deps, DepsMut, Env, MessageInfo, Order, Response};

// version info for migration info
const CONTRACT_NAME: &str = "crates.io:authzpp-withdraw-rewards-tax-grant";
const CONTRACT_VERSION: &str = env!("CARGO_PKG_VERSION");

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn instantiate(
    deps: DepsMut,
    _env: Env,
    _info: MessageInfo,
    _msg: InstantiateMsg,
) -> Result<Response, ContractError> {
    cw2::set_contract_version(deps.storage, CONTRACT_NAME, CONTRACT_VERSION)?;

    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn migrate(_deps: DepsMut, _env: Env, _msg: InstantiateMsg) -> Result<Response, ContractError> {
    Ok(Response::default())
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn execute(
    deps: DepsMut,
    env: Env,
    info: MessageInfo,
    msg: ExecuteMsg,
) -> Result<Response, ContractError> {
    match msg {
        ExecuteMsg::Grant(grant_spec) => {
            let grantee_addr = validate_grantee_address(deps.api, &grant_spec.grantee)?;

            // validate that the withdraw share address is valid
            deps.api
                .addr_validate(&grant_spec.taxation_address)
                .map_err(|_| {
                    ContractError::InvalidWithdrawShareAddress(
                        grant_spec.taxation_address.to_string(),
                    )
                })?;

            GRANTS.save(deps.storage, &info.sender, &grant_spec)?;

            Ok(Response::default()
                .add_attribute("action", "grant")
                .add_attribute("granter", info.sender)
                .add_attribute("grantee", grantee_addr.to_string()))
        }
        ExecuteMsg::Revoke() => {
            // remove the grant from state
            GRANTS.remove(deps.storage, &info.sender);

            Ok(Response::default()
                .add_attribute("action", "revoke")
                .add_attribute("granter", info.sender))
        }
        ExecuteMsg::Execute(ExecuteSettings {
            delegator,
            percentage,
        }) => {
            let delegator_addr = validate_granter_address(deps.api, &delegator)?;

            // query the grant settings, this will error if there is no active/unexpired grant
            let grant_settings =
                match query_active_grants_by_delegator(deps.storage, &env.block, &delegator_addr) {
                    Ok(Some(grant)) => grant,
                    _ => {
                        return Err(ContractError::NoActiveGrant(delegator_addr.to_string()));
                    }
                }
                .allowed_withdrawls;

            // generate the messages to execute the withdrawl, both the MsgExec and the MsgSends
            let RewardExecutionMsgs { msgs, grantee } = generate_reward_withdrawl_msgs(
                query_pending_rewards(&deps.querier, &delegator_addr)?,
                grant_settings,
                &info.sender,
                &env.contract.address,
                &delegator_addr,
                percentage,
            )?;

            Ok(Response::default()
                .add_messages(msgs)
                .add_attribute("action", "execute_withdraw_rewards_split")
                .add_attribute("granter", delegator_addr)
                .add_attribute("grantee", grantee))
        }
        ExecuteMsg::PruneExpiredGrants() => {
            let mut expired_grants = vec![];

            // iterate through all the grants and check if they are expired
            for grant in GRANTS.range(deps.storage, None, None, Order::Ascending) {
                let (delegator_addr, grant_settings) = grant?;

                // if the grant is expired, add it to the list of expired grants
                if grant_settings.is_expired(&env.block) {
                    expired_grants.push(delegator_addr);
                }
            }

            // remove all the expired grants
            for expired_grant in expired_grants.clone() {
                GRANTS.remove(deps.storage, &expired_grant);
            }

            Ok(Response::default()
                .add_attribute("action", "prune_expired_grants")
                .add_attribute("num_expired_grants", expired_grants.len().to_string()))
        }
    }
}

#[cfg_attr(not(feature = "library"), entry_point)]
pub fn query(deps: Deps, env: Env, msg: QueryMsg) -> Result<Binary, ContractError> {
    match msg {
        QueryMsg::Version {} => to_binary(&queries::query_version()).map_err(ContractError::Std),
        QueryMsg::ActiveGrantsByDelegator(delegator) => {
            let delegator_addr = validate_granter_address(deps.api, &delegator)?;

            let grant =
                query_active_grants_by_delegator(deps.storage, &env.block, &delegator_addr)?;

            to_binary(&grant).map_err(ContractError::Std)
        }
        QueryMsg::ActiveGrantsByGrantee(grantee) => {
            let grantee = validate_grantee_address(deps.api, &grantee)?;

            // check all the grants to see if the grantee is the one being queried
            // and that the grant is active
            let grants = query_active_grants_by_grantee(deps.storage, &env.block, grantee);

            to_binary(&grants).map_err(ContractError::Std)
        }
        QueryMsg::SimulateExecute(ExecuteSettings {
            delegator,
            percentage: requested_percentage,
        }) => {
            let delegator_addr = validate_granter_address(deps.api, &delegator)?;

            // get the max fee percentage from the delegator's grant settings
            let max_fee_percentage = if let Ok(Some(grant)) =
                query_active_grants_by_delegator(deps.storage, &env.block, &delegator_addr)
            {
                grant.allowed_withdrawls.max_fee_percentage
            } else {
                return Err(ContractError::NoActiveGrant(delegator));
            };

            // get the split rewards
            let split_rewards = split_rewards(
                query_pending_rewards(&deps.querier, &delegator_addr)?.total,
                max_fee_percentage,
                &requested_percentage,
            );

            to_binary(&split_rewards).map_err(ContractError::Std)
        }
    }
}
